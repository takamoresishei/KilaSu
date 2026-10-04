// SPDX-License-Identifier: GPL-3.0-only
use std::{
    ffi::{c_char, c_int, c_ulong, c_void},
    io,
};
#[repr(C)]
pub struct Peer {
    pub pid: c_int,
    pub uid: u32,
    pub gid: u32,
}
extern "C" {
    pub fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int;
    pub fn bind(fd: c_int, address: *const c_void, length: u32) -> c_int;
    pub fn listen(fd: c_int, backlog: c_int) -> c_int;
    pub fn close(fd: c_int) -> c_int;
    pub fn flock(fd: c_int, operation: c_int) -> c_int;
    pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    pub fn getuid() -> u32;
    pub fn geteuid() -> u32;
    pub fn umask(mask: u32) -> u32;
    pub fn getsockopt(
        fd: c_int,
        level: c_int,
        name: c_int,
        value: *mut c_void,
        len: *mut u32,
    ) -> c_int;
    pub fn mount(
        src: *const c_char,
        dst: *const c_char,
        kind: *const c_char,
        flags: c_ulong,
        data: *const c_void,
    ) -> c_int;
    pub fn unshare(flags: c_int) -> c_int;
    pub fn kila_inflate_raw(src: *const u8, len: usize, dst: *mut u8, size: usize) -> c_int;
    pub fn kila_crc32(data: *const u8, len: usize) -> u32;
    pub fn kila_sha256(data: *const u8, len: usize, dst: *mut u8);
}
pub fn peer(fd: c_int) -> io::Result<Peer> {
    let mut p = Peer {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<Peer>() as u32;
    let ret = unsafe { getsockopt(fd, 1, 17, &mut p as *mut _ as *mut c_void, &mut len) };
    if ret != 0 || len as usize != std::mem::size_of::<Peer>() {
        return Err(io::Error::last_os_error());
    }
    Ok(p)
}
pub fn root() -> io::Result<()> {
    if unsafe { geteuid() } == 0 {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "root daemon context required",
        ))
    }
}
