// SPDX-License-Identifier: GPL-3.0-only
use std::{io, path::Path};
fn main() {
    unsafe {
        kilasd::sys::umask(0o077);
    }
    if let Err(e) = run() {
        eprintln!("kilasd: {e}");
        std::process::exit(1);
    }
}
fn run() -> io::Result<()> {
    let a: Vec<_> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("version") => {
            println!("kilasd {} API {}", kilasd::VERSION, kilasd::kernel::API);
            Ok(())
        }
        Some("daemon") => kilasd::server::run(Path::new(kilasd::ROOT)),
        Some("lifecycle") => kilasd::lifecycle::stage(
            Path::new(kilasd::ROOT),
            a.get(2).ok_or_else(|| kilasd::util::err("missing stage"))?,
        ),
        _ => Err(kilasd::util::err(
            "usage: kilasd version | daemon | lifecycle <stage>",
        )),
    }
}
