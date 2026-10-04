// SPDX-License-Identifier: GPL-3.0-only
use std::{
    io::{self, Read, Write},
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
fn main() {
    if let Err(e) = run() {
        eprintln!("kila: {e}");
        std::process::exit(1);
    }
}
fn run() -> io::Result<()> {
    let mut a: Vec<_> = std::env::args().skip(1).collect();
    if std::env::args()
        .next()
        .and_then(|p| std::path::Path::new(&p).file_name().map(|n| n == "su"))
        .unwrap_or(false)
    {
        a.insert(0, "su".into());
    }
    if a.is_empty() {
        return Err(kilasd::util::err(
            "usage: kila version|status|apps|modules|module|log|su",
        ));
    }
    if a[0] == "version" {
        println!("KilaSU {} / UAPI {}", kilasd::VERSION, kilasd::kernel::API);
        return Ok(());
    }
    if a[0] == "su" {
        let k = kilasd::kernel::Kernel::open()?;
        let start = Instant::now();
        loop {
            if let Ok(mut s) = kilasd::server::connect() {
                let _ = kilasd::protocol::send(&mut s, "authorize-root");
                let _ = kilasd::protocol::receive(&mut s);
            }
            match k.root() {
                Ok(()) => break,
                Err(e)
                    if e.raw_os_error() == Some(13)
                        && start.elapsed() < Duration::from_secs(60) =>
                {
                    std::thread::sleep(Duration::from_millis(1000))
                }
                Err(e) => return Err(e),
            }
        }
        if unsafe { kilasd::sys::unshare(0x00020000) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let root = std::ffi::CString::new("/").unwrap();
        if unsafe {
            kilasd::sys::mount(
                std::ptr::null(),
                root.as_ptr(),
                std::ptr::null(),
                16384 | 0x80000,
                std::ptr::null(),
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        use std::os::unix::process::CommandExt;
        let mut cmd = Command::new("/system/bin/sh");
        if a.len() > 1 {
            if a.get(1).map(String::as_str) != Some("-c") || a.len() != 3 {
                return Err(kilasd::util::err("su accepts -c <command>"));
            }
            cmd.args(["-c", &a[2]]);
        }
        cmd.env_clear()
            .env("PATH", "/data/adb/kilasu/bin:/system/bin:/system/xbin")
            .env("HOME", "/data/adb/kilasu")
            .env("SHELL", "/system/bin/sh");
        return Err(cmd.exec());
    }
    let request = match a[0].as_str() {
        "status" | "apps" | "modules" | "diagnostics" => a[0].clone(),
        "log" => "logs".into(),
        "module" if a.get(1).map(String::as_str) == Some("list") => "modules".into(),
        "module" if a.get(1).map(String::as_str) == Some("install") => {
            let path = Path::new(
                a.get(2)
                    .ok_or_else(|| kilasd::util::err("missing ZIP path"))?,
            );
            let mut f = std::fs::File::open(path)?;
            let mut s = kilasd::server::connect()?;
            kilasd::protocol::send(&mut s, &format!("install\n{}", f.metadata()?.len()))?;
            let ready = kilasd::protocol::receive(&mut s)?;
            if !ready.contains("\"ready\":true") {
                return Err(kilasd::util::err(ready));
            }
            let mut buf = [0u8; 65536];
            loop {
                let n = f.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                s.write_all(&buf[..n])?;
            }
            println!("{}", kilasd::protocol::receive(&mut s)?);
            return Ok(());
        }
        "module" if a.len() == 3 => format!("module\n{}\n{}", a[1], a[2]),
        _ => return Err(kilasd::util::err("unknown command")),
    };
    let mut s = kilasd::server::connect()?;
    kilasd::protocol::send(&mut s, &request)?;
    println!("{}", kilasd::protocol::receive(&mut s)?);
    Ok(())
}
