// SPDX-License-Identifier: GPL-3.0-only
use crate::util::err;
use std::io::{self, Read, Write};
pub const MAX_FRAME: usize = 65536;
pub fn receive(r: &mut impl Read) -> io::Result<String> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    let n = u32::from_le_bytes(b) as usize;
    if n == 0 || n > MAX_FRAME {
        return Err(err("invalid frame length"));
    }
    let mut b = vec![0u8; n];
    r.read_exact(&mut b)?;
    String::from_utf8(b).map_err(|_| err("frame must be UTF-8"))
}
pub fn send(w: &mut impl Write, s: &str) -> io::Result<()> {
    if s.is_empty() || s.len() > MAX_FRAME {
        return Err(err("frame too large"));
    }
    w.write_all(&(s.len() as u32).to_le_bytes())?;
    w.write_all(s.as_bytes())
}
#[cfg(test)]
mod tests {
    #[test]
    fn framing() {
        let mut b = Vec::new();
        super::send(&mut b, "status").unwrap();
        assert_eq!(super::receive(&mut &b[..]).unwrap(), "status");
    }
    #[test]
    fn bounds() {
        assert!(super::receive(&mut &u32::MAX.to_le_bytes()[..]).is_err());
        assert!(super::receive(&mut &0u32.to_le_bytes()[..]).is_err());
    }
}
