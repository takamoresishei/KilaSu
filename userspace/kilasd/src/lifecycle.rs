// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    kernel::Kernel,
    modules, sys,
    util::{atomic_write, err, lock, secure_dir},
};
use std::{fs, io, path::Path, process::Command};
pub fn initialize(root: &Path) -> io::Result<()> {
    sys::root()?;
    secure_dir(root, 0o700)?;
    for d in ["modules", "modules_update", "logs", "staging", "rollback"] {
        secure_dir(&root.join(d), 0o700)?;
    }
    Ok(())
}
pub fn stage(root: &Path, name: &str) -> io::Result<()> {
    if name == "early-init" {
        sys::root()?;
        return Kernel::open()?.stage(0);
    }
    initialize(root)?;
    let _lock = lock(root, "modules.lock")?;
    let (number, script) = match name {
        "early-init" => (0, None),
        "post-fs-data" => (1, Some("post-fs-data.sh")),
        "service" => (2, Some("service.sh")),
        "boot-completed" => (3, Some("boot-completed.sh")),
        _ => return Err(err("unknown boot stage")),
    };
    let k = Kernel::open()?;
    // A stage is recorded only after successful processing, so failures are visible.
    if name == "post-fs-data" {
        modules::activate(root)?;
        let mounts = modules::mount_system(root)?;
        atomic_write(&root.join("mount-status"), mounts.as_bytes(), 0o600)?;
    }
    if let Some(script) = script {
        for m in modules::list(root)? {
            if !m.disabled && !m.remove && !m.pending {
                modules::run_script(root, &root.join("modules").join(&m.id), script, 120)?;
            }
        }
    }
    k.stage(number)?;
    if name == "post-fs-data" {
        let status = Command::new("/system/bin/setprop")
            .args(["sys.kilasu.postfs.ready", "1"])
            .status()?;
        if !status.success() {
            return Err(err("cannot report post-fs-data completion to init"));
        }
    }
    atomic_write(&root.join("boot-stage"), name.as_bytes(), 0o600)
}
pub fn property(key: &str) -> String {
    Command::new("/system/bin/getprop")
        .arg(key)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default()
}
pub fn enforce() -> String {
    fs::read_to_string("/sys/fs/selinux/enforce")
        .map(|s| {
            if s.trim() == "1" {
                "Enforcing".into()
            } else {
                "Permissive".into()
            }
        })
        .unwrap_or("Unavailable".into())
}
