// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    config,
    kernel::{self, Kernel},
    util::err,
};
use std::{
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::Command,
};
pub const PACKAGE: &str = "io.github.kilasu.manager";
#[derive(Clone, Debug)]
pub struct App {
    pub package: String,
    pub uid: u32,
}
pub fn apps() -> io::Result<Vec<App>> {
    let text = fs::read_to_string("/data/system/packages.list")?;
    if text.len() > 4 * 1024 * 1024 {
        return Err(err("package database too large"));
    }
    let mut out = Vec::new();
    for line in text.lines() {
        let mut cols = line.split_whitespace();
        if let (Some(package), Some(uid)) = (cols.next(), cols.next()) {
            let uid = uid.parse().map_err(|_| err("invalid package UID"))?;
            if kernel::app_uid(uid) {
                out.push(App {
                    package: package.into(),
                    uid,
                });
            }
        }
    }
    Ok(out)
}
pub fn unique_package(uid: u32) -> io::Result<String> {
    let a: Vec<_> = apps()?.into_iter().filter(|a| a.uid == uid).collect();
    if a.len() != 1 {
        return Err(err("shared or unknown UID denied"));
    }
    Ok(a[0].package.clone())
}
fn apk_path(package: &str) -> io::Result<PathBuf> {
    if !package
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.')
    {
        return Err(err("invalid package name"));
    }
    let result = Command::new("/system/bin/cmd")
        .args(["package", "path", package])
        .output()?;
    if !result.status.success() {
        return Err(err("package path failed"));
    }
    let s = String::from_utf8(result.stdout).map_err(|_| err("invalid package path"))?;
    let paths: Vec<_> = s
        .lines()
        .filter_map(|l| l.strip_prefix("package:"))
        .collect();
    if paths.len() != 1 {
        return Err(err(
            "split APK identity unsupported; a universal APK is required",
        ));
    }
    let p = fs::canonicalize(paths[0])?;
    if !p.starts_with("/data/app/") {
        return Err(err("APK outside package install directory"));
    }
    Ok(p)
}
pub fn app_hash(package: &str) -> io::Result<String> {
    let p = apk_path(package)?;
    let f = fs::File::open(p)?;
    let m = f.metadata()?;
    if !m.is_file() || m.len() > 256 * 1024 * 1024 || m.mode() & 0o022 != 0 {
        return Err(err("unsafe APK file"));
    }
    use std::os::fd::AsRawFd;
    let mut digest = [0u8; 32];
    let mut length = 0u64;
    if unsafe {
        crate::sys::kila_sha256_fd(
            f.as_raw_fd(),
            256 * 1024 * 1024,
            digest.as_mut_ptr(),
            &mut length,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    let after = f.metadata()?;
    if length != m.len()
        || after.len() != m.len()
        || after.mtime() != m.mtime()
        || after.mtime_nsec() != m.mtime_nsec()
        || after.ctime() != m.ctime()
        || after.ctime_nsec() != m.ctime_nsec()
    {
        return Err(err("APK changed during verification"));
    }
    Ok(crate::util::hex_hash(&digest))
}
pub fn manager_hash(root: &Path) -> io::Result<String> {
    let p = root.join("manager.prop");
    let m = fs::symlink_metadata(&p)?;
    if !m.is_file() || m.file_type().is_symlink() || m.uid() != 0 || m.mode() & 0o077 != 0 {
        return Err(err("unsafe manager pin"));
    }
    let c = config::parse(&fs::read_to_string(p)?)?;
    let pin = c
        .get("apk_sha256")
        .ok_or_else(|| err("missing Manager pin"))?;
    if pin.len() != 64 || !pin.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(err("invalid Manager pin"));
    }
    Ok(pin.to_ascii_lowercase())
}
pub fn verify_manager(root: &Path, uid: u32) -> io::Result<()> {
    if unique_package(uid)? != PACKAGE {
        return Err(err("caller is not KilaSU Manager"));
    }
    if app_hash(PACKAGE)? != manager_hash(root)? {
        return Err(err("Manager APK pin mismatch"));
    }
    Ok(())
}
pub fn enroll(root: &Path, k: &Kernel) -> io::Result<()> {
    let a = apps()?
        .into_iter()
        .find(|a| a.package == PACKAGE)
        .ok_or_else(|| err("Manager not installed"))?;
    verify_manager(root, a.uid)?;
    k.enroll(a.uid, &manager_hash(root)?)
}
