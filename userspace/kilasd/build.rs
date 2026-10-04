// SPDX-License-Identifier: GPL-3.0-only
use std::{env, path::PathBuf, process::Command};
fn run(cmd: &mut Command) {
    assert!(
        cmd.status().expect("native build tool missing").success(),
        "native build failed"
    );
}
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let source = "../../bootpatch/src/kila_boot.c";
    let target = env::var("TARGET").unwrap().replace('-', "_");
    let cc = env::var(format!("CC_{target}"))
        .or_else(|_| env::var("CC"))
        .unwrap_or("cc".into());
    let ar = env::var(format!("AR_{target}"))
        .or_else(|_| env::var("AR"))
        .unwrap_or("ar".into());
    run(Command::new(cc)
        .args([
            "-std=c11", "-O2", "-fPIC", "-Wall", "-Wextra", "-Werror", "-c", source, "-o",
        ])
        .arg(out.join("boot.o")));
    run(Command::new(ar)
        .arg("crs")
        .arg(out.join("libkilaboot.a"))
        .arg(out.join("boot.o")));
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=kilaboot");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rerun-if-changed={source}");
    println!("cargo:rerun-if-changed=../../bootpatch/src/kila_boot.h");
}
