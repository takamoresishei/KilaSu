// SPDX-License-Identifier: GPL-3.0-only
pub mod config;
pub mod identity;
pub mod kernel;
pub mod lifecycle;
pub mod modules;
pub mod protocol;
pub mod server;
pub mod sys;
pub mod util;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ROOT: &str = "/data/adb/kilasu";
