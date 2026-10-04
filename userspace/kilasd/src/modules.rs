// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    config, sys,
    util::{atomic_write, err, json, secure_dir},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs, io,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const MAX_SIZE: usize = 256 * 1024 * 1024;
#[derive(Clone, Debug)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub version: String,
    pub version_code: u64,
    pub author: String,
    pub description: String,
    pub disabled: bool,
    pub remove: bool,
    pub pending: bool,
}
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
pub fn metadata(text: &str) -> io::Result<Module> {
    let c = config::parse(text)?;
    let get = |key: &str| {
        c.get(key)
            .cloned()
            .ok_or_else(|| err(format!("missing {key}")))
    };
    let id = get("id")?;
    if !valid_id(&id) {
        return Err(err("invalid module ID"));
    }
    let version_code = get("versionCode")?
        .parse()
        .map_err(|_| err("versionCode must be an unsigned integer"))?;
    if let Some(v) = c.get("minKilaApi") {
        let v: u32 = v.parse().map_err(|_| err("invalid minKilaApi"))?;
        if v > crate::kernel::API as u32 {
            return Err(err("module requires a newer KilaSU API"));
        }
    }
    Ok(Module {
        id,
        name: get("name")?,
        version: get("version")?,
        version_code,
        author: get("author")?,
        description: get("description")?,
        disabled: false,
        remove: false,
        pending: false,
    })
}
pub fn list(root: &Path) -> io::Result<Vec<Module>> {
    let mut result = BTreeMap::new();
    for dir in ["modules", "modules_update"] {
        let p = root.join(dir);
        if !p.exists() {
            continue;
        }
        for entry in fs::read_dir(p)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let p = entry.path();
            let mut m = metadata(&fs::read_to_string(p.join("module.prop"))?)?;
            if p.file_name().and_then(|x| x.to_str()) != Some(&m.id) {
                return Err(err("module directory ID mismatch"));
            }
            m.disabled = p.join("disable").exists();
            m.remove = p.join("remove").exists();
            m.pending = dir == "modules_update";
            result.insert(m.id.clone(), m);
        }
    }
    Ok(result.into_values().collect())
}
pub fn list_json(root: &Path) -> io::Result<String> {
    Ok(format!("[{}]",list(root)?.iter().map(|m|format!("{{\"id\":{},\"name\":{},\"version\":{},\"versionCode\":{},\"author\":{},\"description\":{},\"enabled\":{},\"remove\":{},\"rebootRequired\":{}}}",json(&m.id),json(&m.name),json(&m.version),m.version_code,json(&m.author),json(&m.description),!m.disabled,m.remove,m.pending||m.remove)).collect::<Vec<_>>().join(",")))
}
fn word(b: &[u8], n: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(n..n + 2)
            .ok_or_else(|| err("truncated ZIP"))?
            .try_into()
            .unwrap(),
    ))
}
fn dword(b: &[u8], n: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(n..n + 4)
            .ok_or_else(|| err("truncated ZIP"))?
            .try_into()
            .unwrap(),
    ))
}
pub struct Entry {
    pub name: String,
    pub bytes: Vec<u8>,
    pub directory: bool,
}
pub fn archive(b: &[u8]) -> io::Result<Vec<Entry>> {
    if b.len() > MAX_SIZE || b.len() < 22 {
        return Err(err("ZIP size outside bounds"));
    }
    let eocd = (b.len().saturating_sub(65557)..=b.len() - 22)
        .rev()
        .find(|&i| {
            b[i..i + 4] == [0x50, 0x4b, 5, 6]
                && word(b, i + 20)
                    .map(|n| i + 22 + n as usize == b.len())
                    .unwrap_or(false)
        })
        .ok_or_else(|| err("missing ZIP directory"))?;
    if word(b, eocd + 4)? != 0
        || word(b, eocd + 6)? != 0
        || word(b, eocd + 8)? != word(b, eocd + 10)?
    {
        return Err(err("multi-volume ZIP unsupported"));
    }
    let count = word(b, eocd + 10)? as usize;
    if count == 0 || count > 4096 {
        return Err(err("ZIP entry limit"));
    }
    let mut p = dword(b, eocd + 16)? as usize;
    let end = p
        .checked_add(dword(b, eocd + 12)? as usize)
        .ok_or_else(|| err("ZIP overflow"))?;
    if end != eocd {
        return Err(err("invalid central directory"));
    }
    let mut seen = BTreeSet::new();
    let mut ranges = Vec::new();
    let mut total = 0usize;
    let mut entries = Vec::new();
    for _ in 0..count {
        if dword(b, p)? != 0x02014b50 {
            return Err(err("bad directory entry"));
        }
        let flags = word(b, p + 8)?;
        let method = word(b, p + 10)?;
        let crc = dword(b, p + 16)?;
        let packed = dword(b, p + 20)? as usize;
        let size = dword(b, p + 24)? as usize;
        let n = word(b, p + 28)? as usize;
        let extra = word(b, p + 30)? as usize;
        let comment = word(b, p + 32)? as usize;
        let mode = dword(b, p + 38)? >> 16;
        let offset = dword(b, p + 42)? as usize;
        if flags & !0x808 != 0
            || !(method == 0 || method == 8)
            || mode & 0o170000 == 0o120000
            || word(b, p + 34)? != 0
        {
            return Err(err("encrypted, special, or unsupported ZIP entry"));
        }
        let name = std::str::from_utf8(
            b.get(p + 46..p + 46 + n)
                .ok_or_else(|| err("bad ZIP name"))?,
        )
        .map_err(|_| err("ZIP names must be UTF-8"))?
        .to_owned();
        if name.is_empty()
            || name.len() > 240
            || name.contains('\\')
            || name.contains('\0')
            || name.chars().any(|c| c.is_control())
            || name.starts_with('/')
            || name.contains(':')
            || name
                .trim_end_matches('/')
                .split('/')
                .any(|c| c.is_empty() || c == ".." || c == ".")
            || !Path::new(&name)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            || !seen.insert(name.trim_end_matches('/').to_owned())
        {
            return Err(err("unsafe or duplicate ZIP path"));
        }
        if dword(b, offset)? != 0x04034b50
            || word(b, offset + 8)? != method
            || word(b, offset + 6)? != flags
        {
            return Err(err("local header mismatch"));
        }
        let ln = word(b, offset + 26)? as usize;
        let le = word(b, offset + 28)? as usize;
        if b.get(offset + 30..offset + 30 + ln) != Some(name.as_bytes()) {
            return Err(err("local filename mismatch"));
        }
        let start = offset
            .checked_add(30 + ln + le)
            .ok_or_else(|| err("ZIP overflow"))?;
        let stop = start
            .checked_add(packed)
            .ok_or_else(|| err("ZIP overflow"))?;
        if stop > end || stop > dword(b, eocd + 16)? as usize {
            return Err(err("ZIP data crosses central directory"));
        }
        ranges.push((offset, stop));
        total = total
            .checked_add(size)
            .ok_or_else(|| err("ZIP expansion overflow"))?;
        if total > MAX_SIZE || size > 64 * 1024 * 1024 {
            return Err(err("ZIP expansion limit"));
        }
        let packed_bytes = b
            .get(start..stop)
            .ok_or_else(|| err("ZIP data truncated"))?;
        let mut bytes = vec![0u8; size];
        if method == 0 {
            if packed != size {
                return Err(err("stored ZIP size mismatch"));
            }
            bytes.copy_from_slice(packed_bytes);
        } else if unsafe {
            sys::kila_inflate_raw(packed_bytes.as_ptr(), packed, bytes.as_mut_ptr(), size)
        } != 0
        {
            return Err(err("invalid deflate stream"));
        }
        if unsafe { sys::kila_crc32(bytes.as_ptr(), bytes.len()) } != crc {
            return Err(err("ZIP checksum mismatch"));
        }
        let directory = name.ends_with('/');
        if directory && !bytes.is_empty() {
            return Err(err("directory has payload"));
        }
        entries.push(Entry {
            name,
            bytes,
            directory,
        });
        p = p
            .checked_add(46 + n + extra + comment)
            .ok_or_else(|| err("ZIP directory overflow"))?;
        if p > end {
            return Err(err("ZIP directory truncated"));
        }
    }
    if p != end {
        return Err(err("extra ZIP directory data"));
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|r| r[0].1 > r[1].0) {
        return Err(err("overlapping ZIP data"));
    }
    Ok(entries)
}
pub fn run_script(root: &Path, dir: &Path, name: &str, seconds: u64) -> io::Result<()> {
    let script = dir.join(name);
    if !script.is_file() {
        return Ok(());
    }
    let logs = root.join("logs");
    secure_dir(&logs, 0o700)?;
    let log = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(logs.join("modules.log"))?;
    let mut child = Command::new("/system/bin/sh")
        .arg(&script)
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/data/adb/kilasu/bin:/system/bin:/system/xbin")
        .env("MODPATH", dir)
        .env("KILASU", "true")
        .env("KILASU_API", "1")
        .env("BOOTMODE", "true")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    let start = Instant::now();
    loop {
        if let Some(s) = child.try_wait()? {
            return if s.success() {
                Ok(())
            } else {
                Err(err(format!("{name} failed: {s}")))
            };
        }
        if start.elapsed() > Duration::from_secs(seconds) {
            child.kill()?;
            let _ = child.wait();
            return Err(err(format!("{name} timed out")));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
pub fn install(root: &Path, zip: &Path) -> io::Result<String> {
    sys::root()?;
    let f = fs::File::open(zip)?;
    if !f.metadata()?.is_file() || f.metadata()?.len() > MAX_SIZE as u64 {
        return Err(err("invalid module ZIP"));
    }
    use std::io::Read;
    let mut b = Vec::new();
    f.take(MAX_SIZE as u64 + 1).read_to_end(&mut b)?;
    let entries = archive(&b)?;
    let prop = entries
        .iter()
        .find(|e| e.name == "module.prop")
        .ok_or_else(|| err("module.prop must be at ZIP root"))?;
    let m =
        metadata(std::str::from_utf8(&prop.bytes).map_err(|_| err("module.prop must be UTF-8"))?)?;
    let stage = root
        .join("staging")
        .join(format!("{}.{}", m.id, std::process::id()));
    if stage.exists() {
        return Err(err("staging transaction already exists"));
    }
    secure_dir(&stage, 0o700)?;
    let result = (|| {
        for e in &entries {
            let dest = stage.join(&e.name);
            if e.directory {
                secure_dir(&dest, 0o755)?;
            } else {
                if let Some(p) = dest.parent() {
                    fs::create_dir_all(p)?;
                }
                atomic_write(
                    &dest,
                    &e.bytes,
                    if e.name.ends_with(".sh") {
                        0o755
                    } else {
                        0o644
                    },
                )?;
            }
        }
        run_script(root, &stage, "customize.sh", 120)?;
        let customized = metadata(&fs::read_to_string(stage.join("module.prop"))?)?;
        if customized.id != m.id {
            return Err(err("installer changed the module ID"));
        }
        // Update staging never replaces an already queued transaction silently.
        let pending = root.join("modules_update");
        secure_dir(&pending, 0o700)?;
        let dest = pending.join(&m.id);
        if dest.exists() {
            return Err(err("an update for this module is already queued"));
        }
        fs::rename(&stage, &dest)?;
        fs::File::open(&pending)?.sync_all()?;
        Ok(format!("{{\"id\":{},\"rebootRequired\":true,\"log\":\"Extracted, verified, installer completed, update queued\"}}",json(&m.id)))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(stage);
    }
    result
}
pub fn change(root: &Path, id: &str, op: &str) -> io::Result<()> {
    sys::root()?;
    if !valid_id(id) {
        return Err(err("invalid module ID"));
    }
    let dir = if root.join("modules_update").join(id).is_dir() {
        root.join("modules_update").join(id)
    } else {
        root.join("modules").join(id)
    };
    if !dir.is_dir() {
        return Err(err("module not found"));
    }
    match op {
        "enable" => {
            let p = dir.join("disable");
            if p.exists() {
                fs::remove_file(p)?;
            }
        }
        "disable" => atomic_write(&dir.join("disable"), b"", 0o600)?,
        "remove" => atomic_write(&dir.join("remove"), b"", 0o600)?,
        _ => return Err(err("invalid module action")),
    }
    Ok(())
}
pub fn activate(root: &Path) -> io::Result<()> {
    let mods = root.join("modules");
    secure_dir(&mods, 0o700)?;
    let updates = root.join("modules_update");
    secure_dir(&updates, 0o700)?;
    for e in fs::read_dir(&updates)? {
        let e = e?;
        if !e.file_type()?.is_dir() {
            continue;
        }
        let id = e.file_name();
        let dst = mods.join(&id);
        let backup = root.join("rollback").join(&id);
        secure_dir(&root.join("rollback"), 0o700)?;
        if backup.exists() {
            return Err(err(
                "unfinished rollback transaction; diagnose before booting modules",
            ));
        }
        if dst.exists() {
            fs::rename(&dst, &backup)?;
        }
        if let Err(e2) = fs::rename(e.path(), &dst) {
            if backup.exists() {
                let _ = fs::rename(&backup, &dst);
            }
            return Err(e2);
        }
        if backup.exists() {
            fs::remove_dir_all(backup)?;
        }
    }
    for m in list(root)? {
        if m.remove {
            let d = mods.join(&m.id);
            run_script(root, &d, "uninstall.sh", 60)?;
            fs::remove_dir_all(d)?;
        }
    }
    Ok(())
}
fn files(dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        let t = e.file_type()?;
        if t.is_symlink() {
            return Err(err("module mount symlink denied"));
        }
        if t.is_dir() {
            out.extend(files(&e.path())?);
        } else if t.is_file() {
            out.push(e.path());
        }
    }
    out.sort();
    Ok(out)
}
struct BindMount {
    source: PathBuf,
    target: PathBuf,
}
fn mount_plan(root: &Path, system: &Path) -> io::Result<Vec<BindMount>> {
    let mut owners = BTreeMap::new();
    let mut plan = Vec::new();
    let system = system.canonicalize()?;
    for m in list(root)? {
        if m.disabled || m.remove || m.pending {
            continue;
        }
        let source = root.join("modules").join(&m.id).join("system");
        if !source.exists() {
            continue;
        }
        if !fs::symlink_metadata(&source)?.file_type().is_dir() {
            return Err(err("module system directory must not be a symlink"));
        }
        for file in files(&source)? {
            let rel = file
                .strip_prefix(&source)
                .map_err(|_| err("mount path invalid"))?;
            let dst = system.join(rel);
            if !fs::symlink_metadata(&dst)?.file_type().is_file()
                || !dst.canonicalize()?.starts_with(&system)
            {
                return Err(err(format!(
                    "overlay requires existing regular target: {}",
                    dst.display()
                )));
            }
            if let Some(id) = owners.insert(dst.clone(), m.id.clone()) {
                return Err(err(format!("module conflict: {} and {}", id, m.id)));
            }
            plan.push(BindMount {
                source: file,
                target: dst,
            });
        }
    }
    Ok(plan)
}
fn cpath(path: &Path) -> io::Result<CString> {
    CString::new(path.to_str().ok_or_else(|| err("mount path UTF-8"))?)
        .map_err(|_| err("invalid mount path"))
}
fn copy_label(source: &CString, target: &CString) -> io::Result<()> {
    let name = b"security.selinux\0";
    let mut label = [0u8; 4096];
    let n = unsafe {
        sys::getxattr(
            target.as_ptr(),
            name.as_ptr().cast(),
            label.as_mut_ptr().cast(),
            label.len(),
        )
    };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }
    if n == 0 || n as usize > label.len() {
        return Err(err("invalid target SELinux label"));
    }
    if unsafe {
        sys::setxattr(
            source.as_ptr(),
            name.as_ptr().cast(),
            label.as_ptr().cast(),
            n as usize,
            0,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
pub fn mount_system(root: &Path) -> io::Result<String> {
    sys::root()?;
    // Validate the complete graph and label all sources before touching mounts.
    let plan = mount_plan(root, Path::new("/system"))?;
    let mut ready = Vec::new();
    for m in &plan {
        let source = cpath(&m.source)?;
        let target = cpath(&m.target)?;
        copy_label(&source, &target)?;
        ready.push((source, target));
    }
    let mut mounted: Vec<&CString> = Vec::new();
    for (src, dst) in &ready {
        if unsafe {
            sys::mount(
                src.as_ptr(),
                dst.as_ptr(),
                std::ptr::null(),
                4096,
                std::ptr::null(),
            )
        } != 0
        {
            let cause = io::Error::last_os_error();
            let mut rollback_failed = false;
            for target in mounted.iter().rev() {
                if unsafe { sys::umount2(target.as_ptr(), 2) } != 0 {
                    rollback_failed = true;
                }
            }
            return Err(err(format!(
                "module bind mount failed: {cause}; rollback {}",
                if rollback_failed {
                    "incomplete; inspect mount state"
                } else {
                    "complete"
                }
            )));
        }
        mounted.push(dst);
    }
    Ok(format!("{} file bind mounts", mounted.len()))
}
#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let p = std::env::temp_dir().join(format!(
                "kilasu-module-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn module(&self, id: &str) {
            let p = self.0.join("modules").join(id);
            std::fs::create_dir_all(p.join("system/etc")).unwrap();
            std::fs::write(
                p.join("module.prop"),
                format!("id={id}\nname=Test\nversion=1\nversionCode=1\nauthor=A\ndescription=D"),
            )
            .unwrap();
            std::fs::write(p.join("system/etc/sample"), "new").unwrap();
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn props() {
        let m=super::metadata("id=hello\nname=Hello\nversion=1.0\nversionCode=1\nauthor=A\ndescription=D\nminKilaApi=1").unwrap();
        assert_eq!(m.id, "hello");
    }
    #[test]
    fn reject() {
        assert!(!super::valid_id("../bad"));
        assert!(super::metadata("id=x\nversionCode=-1").is_err());
        assert!(super::archive(&[0; 22]).is_err());
    }
    #[test]
    fn standard_zip_fixtures_validate_content_and_reject_unsafe_paths() {
        let good = include_bytes!("../../../tests/fixtures/modules/valid.zip");
        let entries = super::archive(good).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].bytes, b"replacement");
        for bad in [
            include_bytes!("../../../tests/fixtures/modules/traversal.zip").as_slice(),
            include_bytes!("../../../tests/fixtures/modules/duplicate.zip").as_slice(),
            include_bytes!("../../../tests/fixtures/modules/symlink.zip").as_slice(),
        ] {
            assert!(super::archive(bad).is_err());
        }
        let mut corrupt = good.to_vec();
        corrupt["module.prop".len() + 31] ^= 0x55;
        assert!(super::archive(&corrupt).is_err());
    }
    #[test]
    fn mount_preflight_rejects_conflicts_and_does_not_follow_system_symlinks() {
        let temp = Temp::new();
        let system = temp.0.join("system");
        std::fs::create_dir_all(system.join("etc")).unwrap();
        std::fs::write(system.join("etc/sample"), "original").unwrap();
        temp.module("a");
        assert_eq!(super::mount_plan(&temp.0, &system).unwrap().len(), 1);
        temp.module("b");
        assert!(super::mount_plan(&temp.0, &system).is_err());
        std::fs::write(temp.0.join("modules/b/disable"), "").unwrap();
        assert_eq!(super::mount_plan(&temp.0, &system).unwrap().len(), 1);
        std::fs::remove_file(system.join("etc/sample")).unwrap();
        let outside = temp.0.join("outside");
        std::fs::write(&outside, "outside").unwrap();
        std::os::unix::fs::symlink(outside, system.join("etc/sample")).unwrap();
        assert!(super::mount_plan(&temp.0, &system).is_err());
    }
}
