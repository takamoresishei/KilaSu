// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    identity,
    kernel::{u32_at, u64_at, Kernel, Profile},
    lifecycle, modules, protocol, sys,
    util::{atomic_write, err, json, lock, secure_dir, sha256},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            fs::OpenOptionsExt,
            net::{UnixListener, UnixStream},
        },
    },
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
pub const SOCKET: &str = "kilasu.control.v1";
#[repr(C)]
struct Address {
    family: u16,
    path: [u8; 108],
}
pub fn listener() -> io::Result<UnixListener> {
    let fd = unsafe { sys::socket(1, 1 | 0x80000, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut a = Address {
        family: 1,
        path: [0; 108],
    };
    a.path[1..1 + SOCKET.len()].copy_from_slice(SOCKET.as_bytes());
    let r = unsafe { sys::bind(fd, &a as *const _ as *const _, (3 + SOCKET.len()) as u32) };
    if r != 0 || unsafe { sys::listen(fd, 16) } != 0 {
        let e = io::Error::last_os_error();
        unsafe {
            sys::close(fd);
        }
        return Err(e);
    }
    Ok(unsafe { UnixListener::from_raw_fd(fd) })
}
pub fn connect() -> io::Result<UnixStream> {
    extern "C" {
        fn connect(fd: i32, addr: *const std::ffi::c_void, len: u32) -> i32;
    }
    let fd = unsafe { sys::socket(1, 1 | 0x80000, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut a = Address {
        family: 1,
        path: [0; 108],
    };
    a.path[1..1 + SOCKET.len()].copy_from_slice(SOCKET.as_bytes());
    if unsafe { connect(fd, &a as *const _ as *const _, (3 + SOCKET.len()) as u32) } != 0 {
        let e = io::Error::last_os_error();
        unsafe {
            sys::close(fd);
        }
        return Err(e);
    }
    let stream = unsafe { UnixStream::from_raw_fd(fd) };
    if sys::peer(stream.as_raw_fd())?.uid != 0 {
        return Err(err("control socket is not owned by the root daemon"));
    }
    stream.set_read_timeout(Some(Duration::from_secs(180)))?;
    stream.set_write_timeout(Some(Duration::from_secs(180)))?;
    Ok(stream)
}
fn validate(root: &Path, uid: u32) -> io::Result<()> {
    if uid == 0 {
        Ok(())
    } else {
        identity::verify_manager(root, uid)
    }
}
pub fn status(root: &Path) -> io::Result<String> {
    let k = Kernel::open()?;
    let v = k.version()?;
    let s = k.status()?;
    Ok(format!("{{\"kernel\":{},\"api\":{},\"kernelVersion\":{},\"features\":{},\"daemon\":{},\"stage\":{},\"rootApps\":{},\"modules\":{},\"selinux\":{},\"mountStatus\":{},\"android\":{}}}",json(&v.release),v.api_max,v.version,v.features,json(crate::VERSION),u32_at(&s,0),k.profiles()?.iter().filter(|p|p.permission!=0).count(),modules::list(root)?.len(),json(&lifecycle::enforce()),json(&fs::read_to_string(root.join("mount-status")).unwrap_or("Not mounted".into())),json(&lifecycle::property("ro.build.version.release"))))
}
fn apps() -> io::Result<String> {
    let profiles = Kernel::open()?.profiles()?;
    Ok(format!("[{}]",identity::apps()?.iter().map(|a|{let p=profiles.iter().find(|p|p.uid==a.uid).cloned().unwrap_or_default();format!("{{\"package\":{},\"uid\":{},\"permission\":{},\"capabilities\":{},\"lastRequest\":{},\"lastGrant\":{}}}",json(&a.package),a.uid,p.permission,p.capabilities,p.last_request,p.last_grant)}).collect::<Vec<_>>().join(",")))
}
pub fn restore_permissions(root: &Path) -> io::Result<()> {
    let p = root.join("permissions.db");
    if !p.exists() {
        return Ok(());
    }
    let text = fs::read_to_string(p)?;
    if text.len() > 512 * 1024 {
        return Err(err("permission database too large"));
    }
    let k = Kernel::open()?;
    for line in text.lines() {
        let cols: Vec<_> = line.split('\t').collect();
        if cols.len() != 5 {
            return Err(err("corrupt permission database"));
        }
        let uid: u32 = cols[0].parse().map_err(|_| err("corrupt UID"))?;
        let permission: u32 = cols[3].parse().map_err(|_| err("corrupt permission"))?;
        let caps: u64 = cols[4].parse().map_err(|_| err("corrupt capabilities"))?;
        if permission > 1 {
            return Err(err("once grants cannot persist"));
        }
        let matching = identity::unique_package(uid).ok().as_deref() == Some(cols[1])
            && identity::app_hash(cols[1]).ok().as_deref() == Some(cols[2]);
        k.set_profile(&Profile {
            uid,
            permission: if matching { permission } else { 0 },
            capabilities: caps,
            ..Default::default()
        })?;
    }
    Ok(())
}
fn permission(root: &Path, args: &[&str]) -> io::Result<String> {
    if args.len() != 4 {
        return Err(err("permission expects UID, access, capabilities"));
    }
    let uid: u32 = args[1].parse().map_err(|_| err("invalid UID"))?;
    let permission: u32 = args[2].parse().map_err(|_| err("invalid access"))?;
    let caps: u64 = args[3].parse().map_err(|_| err("invalid capabilities"))?;
    if !crate::kernel::app_uid(uid) || permission > 2 || caps >> 41 != 0 {
        return Err(err("invalid app profile"));
    }
    let package = identity::unique_package(uid)?;
    let hash = identity::app_hash(&package)?;
    let _lock = lock(root, "permissions.lock")?;
    let path = root.join("permissions.db");
    let old = fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = old
        .lines()
        .filter(|l| l.split('\t').next() != Some(args[1]))
        .map(str::to_owned)
        .collect();
    if lines.len() >= 1024 {
        return Err(err("allowlist full"));
    }
    lines.push(format!(
        "{uid}\t{package}\t{hash}\t{}\t{caps}",
        if permission == 2 { 0 } else { permission }
    ));
    let new = lines.join("\n") + "\n";
    let kernel = Kernel::open()?;
    atomic_write(&path, new.as_bytes(), 0o600)?;
    if let Err(e) = kernel.set_profile(&Profile {
        uid,
        permission,
        capabilities: caps,
        ..Default::default()
    }) {
        atomic_write(&path, old.as_bytes(), 0o600)?;
        return Err(e);
    }
    Ok("{\"updated\":true}".into())
}
fn audit(args: &[&str]) -> io::Result<String> {
    let k = Kernel::open()?;
    let mut seq: u64 = args
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|_| err("invalid sequence"))?;
    let mut out = Vec::new();
    for _ in 0..128 {
        let mut b = [0u8; 32];
        b[..8].copy_from_slice(&seq.to_le_bytes());
        match k.call(10, &b) {
            Ok(b) => {
                seq = u64_at(&b, 0) + 1;
                out.push(format!(
                    "{{\"sequence\":{},\"time\":{},\"uid\":{},\"result\":{},\"command\":{}}}",
                    seq - 1,
                    u64_at(&b, 8),
                    u32_at(&b, 16),
                    u32_at(&b, 20) as i32,
                    u32_at(&b, 24)
                ));
            }
            Err(e) if e.raw_os_error() == Some(2) => break,
            Err(e) => return Err(e),
        }
    }
    Ok(format!("[{}]", out.join(",")))
}
fn import(root: &Path, args: &[&str], s: &mut UnixStream) -> io::Result<String> {
    let size: usize = args
        .get(1)
        .ok_or_else(|| err("missing upload size"))?
        .parse()
        .map_err(|_| err("invalid upload size"))?;
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err(err("module upload limit"));
    }
    let _lock = lock(root, "modules.lock")?;
    let dir = root.join("staging");
    secure_dir(&dir, 0o700)?;
    let tmp = dir.join(format!(
        "upload-{}-{}.zip",
        std::process::id(),
        sys::peer(s.as_raw_fd())?.pid
    ));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
    let result = (|| {
        protocol::send(s, "{\"ready\":true}")?;
        let mut remaining = size;
        let mut buf = [0u8; 65536];
        while remaining > 0 {
            let n = remaining.min(buf.len());
            s.read_exact(&mut buf[..n])?;
            f.write_all(&buf[..n])?;
            remaining -= n;
        }
        f.sync_all()?;
        modules::install(root, &tmp)
    })();
    let _ = fs::remove_file(tmp);
    result
}
fn dispatch(root: &Path, request: &str, s: &mut UnixStream) -> io::Result<String> {
    if request.contains('\0') || request.len() > 4096 {
        return Err(err("invalid request"));
    }
    let a: Vec<_> = request.split('\n').collect();
    match a[0] {
        "status" | "diagnostics" => status(root),
        "apps" => apps(),
        "modules" => modules::list_json(root),
        "permission" | "profile" => permission(root, &a),
        "audit" => audit(&a),
        "install" => import(root, &a, s),
        "module" => {
            if a.len() != 3 {
                return Err(err("module expects action and ID"));
            }
            let _lock = lock(root, "modules.lock")?;
            modules::change(root, a[2], a[1])?;
            Ok("{\"rebootRequired\":true}".into())
        }
        "logs" => {
            let text = fs::read_to_string(root.join("logs/daemon.log")).unwrap_or_default();
            let start = text.len().saturating_sub(12000);
            let start = (start..text.len())
                .find(|&i| text.is_char_boundary(i))
                .unwrap_or(text.len());
            Ok(json(&text[start..]))
        }
        _ => Err(err("unknown command")),
    }
}
fn authorize_root(root: &Path, peer: &sys::Peer) -> io::Result<String> {
    let uid = peer.uid;
    if !crate::kernel::app_uid(uid) {
        return Err(err("non-application caller denied"));
    }
    let _lock = lock(root, "permissions.lock")?;
    let text = fs::read_to_string(root.join("permissions.db"))?;
    let uid_text = uid.to_string();
    if text.len() > 512 * 1024
        || !text
            .lines()
            .any(|l| l.split('\t').next() == Some(uid_text.as_str()))
    {
        return Err(err("application identity is not approved"));
    }
    let package = identity::unique_package(uid)?;
    let hash = identity::app_hash(&package)?;
    let matches = text.lines().any(|l| {
        let c: Vec<_> = l.split('\t').collect();
        c.len() == 5 && c[0] == uid_text && c[1] == package && c[2] == hash
    });
    if !matches {
        return Err(err("application identity is not approved"));
    }
    let k = Kernel::open()?;
    let p = k
        .profiles()?
        .into_iter()
        .find(|p| p.uid == uid)
        .ok_or_else(|| err("application denied"))?;
    if p.permission == 0 || (p.permission == 2 && p.remaining == 0) {
        return Err(err("application denied"));
    }
    k.issue(uid, peer.pid as u32)?;
    Ok("{\"authorized\":true}".into())
}
fn handle(root: &Path, mut s: UnixStream) -> io::Result<()> {
    s.set_read_timeout(Some(Duration::from_secs(30)))?;
    s.set_write_timeout(Some(Duration::from_secs(30)))?;
    let peer = sys::peer(s.as_raw_fd())?;
    let result = (|| {
        let request = protocol::receive(&mut s)?;
        if request == "authorize-root" {
            return authorize_root(root, &peer);
        }
        validate(root, peer.uid)?;
        dispatch(root, &request, &mut s)
    })();
    let reply = match result {
        Ok(data) => format!("{{\"ok\":true,\"data\":{data}}}"),
        Err(e) => format!("{{\"ok\":false,\"error\":{}}}", json(&e.to_string())),
    };
    protocol::send(&mut s, &reply)
}
pub fn run(root: &Path) -> io::Result<()> {
    lifecycle::initialize(root)?;
    let _daemon_lock = lock(root, "daemon.lock")?;
    let k = Kernel::open()?;
    let _ = k.revoke_manager();
    if let Err(e) = identity::enroll(root, &k) {
        log(root, &format!("Manager enrollment deferred: {e}"));
    }
    restore_permissions(root)?;
    let listener = listener()?;
    let running = Arc::new(AtomicUsize::new(0));
    // Package identity is rechecked per administrative request and on package-map changes.
    let monitor_root = root.to_owned();
    std::thread::spawn(move || {
        let mut previous = String::new();
        loop {
            let map = fs::read("/data/system/packages.list").unwrap_or_default();
            let current = sha256(&map);
            if current != previous {
                if let Ok(k) = Kernel::open() {
                    let _ = k.revoke_manager();
                    let _ = identity::enroll(&monitor_root, &k);
                    let _ = restore_permissions(&monitor_root);
                }
                previous = current;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
    for s in listener.incoming() {
        let s = s?;
        if running.load(Ordering::Acquire) >= 16 {
            continue;
        }
        running.fetch_add(1, Ordering::AcqRel);
        let root = root.to_owned();
        let n = running.clone();
        std::thread::spawn(move || {
            let _ = handle(&root, s);
            n.fetch_sub(1, Ordering::AcqRel);
        });
    }
    Ok(())
}
pub fn log(root: &Path, msg: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(root.join("logs/daemon.log"))
    {
        let _ = writeln!(f, "Daemon: {msg}");
    }
}
