// SPDX-License-Identifier: GPL-3.0-only
use crate::{sys, util::err};
use std::{
    fs::{File, OpenOptions},
    io,
    os::fd::AsRawFd,
};
pub const API: u16 = 1;
pub const REQUEST_ROOT: u16 = 9;
pub const IOCTL: u64 = 0xc2004b51;
#[derive(Clone, Debug)]
pub struct Version {
    pub api_min: u32,
    pub api_max: u32,
    pub version: u32,
    pub max_apps: u32,
    pub features: u64,
    pub release: String,
}
#[derive(Clone, Debug, Default)]
pub struct Profile {
    pub uid: u32,
    pub permission: u32,
    pub generation: u32,
    pub capabilities: u64,
    pub last_request: u64,
    pub last_grant: u64,
    pub remaining: u32,
}
pub struct Kernel {
    file: File,
}
pub fn u32_at(b: &[u8], n: usize) -> u32 {
    u32::from_le_bytes(b[n..n + 4].try_into().unwrap())
}
pub fn u64_at(b: &[u8], n: usize) -> u64 {
    u64::from_le_bytes(b[n..n + 8].try_into().unwrap())
}
pub fn app_uid(uid: u32) -> bool {
    uid <= 2147483647 && (10000..20000).contains(&(uid % 100000))
}
pub fn negotiate(min: u32, max: u32) -> io::Result<()> {
    if min <= API as u32 && max >= API as u32 {
        Ok(())
    } else {
        Err(err("incompatible KilaSU API"))
    }
}
impl Profile {
    pub fn encode(&self) -> [u8; 48] {
        let mut b = [0u8; 48];
        b[0..4].copy_from_slice(&self.uid.to_le_bytes());
        b[4..8].copy_from_slice(&self.permission.to_le_bytes());
        b[8..12].copy_from_slice(&self.generation.to_le_bytes());
        b[16..24].copy_from_slice(&self.capabilities.to_le_bytes());
        b
    }
    pub fn decode(b: &[u8]) -> Self {
        Self {
            uid: u32_at(b, 0),
            permission: u32_at(b, 4),
            generation: u32_at(b, 8),
            capabilities: u64_at(b, 16),
            last_request: u64_at(b, 24),
            last_grant: u64_at(b, 32),
            remaining: u32_at(b, 40),
        }
    }
}
impl Kernel {
    pub fn open() -> io::Result<Self> {
        let k = Self {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/kilasu")?,
        };
        let v = k.version()?;
        negotiate(v.api_min, v.api_max)?;
        Ok(k)
    }
    pub fn call(&self, cmd: u16, data: &[u8]) -> io::Result<Vec<u8>> {
        if data.len() > 496 {
            return Err(err("UAPI buffer too large"));
        }
        let mut b = [0u8; 512];
        b[..4].copy_from_slice(&0x4b494c41u32.to_le_bytes());
        b[4..6].copy_from_slice(&API.to_le_bytes());
        b[6..8].copy_from_slice(&cmd.to_le_bytes());
        b[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
        b[16..16 + data.len()].copy_from_slice(data);
        let r = unsafe { sys::ioctl(self.file.as_raw_fd(), IOCTL as _, b.as_mut_ptr()) };
        if r < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(b[16..16 + data.len()].to_vec())
    }
    pub fn version(&self) -> io::Result<Version> {
        let b = self.call(1, &[0u8; 88])?;
        Ok(Version {
            api_min: u32_at(&b, 0),
            api_max: u32_at(&b, 4),
            version: u32_at(&b, 8),
            max_apps: u32_at(&b, 12),
            features: u64_at(&b, 16),
            release: String::from_utf8_lossy(&b[24..])
                .trim_end_matches('\0')
                .to_owned(),
        })
    }
    pub fn manager(&self) -> io::Result<u32> {
        let b = self.call(3, &[0u8; 40])?;
        if u32_at(&b, 4) != 1 {
            return Err(err("Manager not enrolled"));
        }
        Ok(u32_at(&b, 0))
    }
    pub fn enroll(&self, uid: u32, hash: &str) -> io::Result<()> {
        let mut b = [0u8; 40];
        b[..4].copy_from_slice(&uid.to_le_bytes());
        b[4..8].copy_from_slice(&1u32.to_le_bytes());
        if hash.len() != 64 {
            return Err(err("bad APK hash"));
        }
        for i in 0..32 {
            b[8 + i] =
                u8::from_str_radix(&hash[2 * i..2 * i + 2], 16).map_err(|_| err("bad hash"))?;
        }
        self.call(4, &b)?;
        Ok(())
    }
    pub fn revoke_manager(&self) -> io::Result<()> {
        self.call(4, &[0u8; 40])?;
        Ok(())
    }
    pub fn set_profile(&self, p: &Profile) -> io::Result<()> {
        if !app_uid(p.uid) || p.permission > 2 {
            return Err(err("invalid profile"));
        }
        self.call(8, &p.encode())?;
        Ok(())
    }
    pub fn profiles(&self) -> io::Result<Vec<Profile>> {
        let mut out = Vec::new();
        for i in 0u32..1024 {
            let mut b = [0u8; 56];
            b[..4].copy_from_slice(&i.to_le_bytes());
            match self.call(5, &b) {
                Ok(b) => out.push(Profile::decode(&b[8..])),
                Err(e) if e.raw_os_error() == Some(2) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
    pub fn stage(&self, stage: u32) -> io::Result<()> {
        let mut b = [0u8; 16];
        b[..4].copy_from_slice(&stage.to_le_bytes());
        self.call(11, &b)?;
        Ok(())
    }
    pub fn status(&self) -> io::Result<Vec<u8>> {
        self.call(12, &[0u8; 16])
    }
    pub fn root(&self) -> io::Result<()> {
        self.call(REQUEST_ROOT, &[])?;
        Ok(())
    }
    pub fn issue(&self, uid: u32, pid: u32) -> io::Result<()> {
        let mut b = [0u8; 16];
        b[..4].copy_from_slice(&uid.to_le_bytes());
        b[4..8].copy_from_slice(&pid.to_le_bytes());
        self.call(13, &b)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn uid_bounds() {
        for uid in [0, 9999, 20000, 99001, u32::MAX] {
            assert!(!super::app_uid(uid));
        }
        for uid in [10000, 19999, 110000] {
            assert!(super::app_uid(uid));
        }
    }
    #[test]
    fn versions() {
        assert!(super::negotiate(1, 2).is_ok());
        assert!(super::negotiate(2, 4).is_err());
    }
    #[test]
    fn profile_bytes() {
        let p = super::Profile {
            uid: 10123,
            permission: 1,
            capabilities: 0x123456abcdef,
            ..Default::default()
        };
        let b = p.encode();
        assert_eq!(b.len(), 48);
        let q = super::Profile::decode(&b);
        assert_eq!(q.capabilities, p.capabilities);
        assert_eq!(q.uid, p.uid);
    }
}
