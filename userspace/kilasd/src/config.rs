// SPDX-License-Identifier: GPL-3.0-only
use crate::util::err;
use std::{collections::BTreeMap, io};
pub fn parse(text: &str) -> io::Result<BTreeMap<String, String>> {
    if text.len() > 65536 {
        return Err(err("config too large"));
    }
    let mut map = BTreeMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| err("expected key=value"))?;
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.')
            || value.len() > 4096
            || value.chars().any(|c| c.is_control())
            || map.insert(key.to_owned(), value.to_owned()).is_some()
        {
            return Err(err("invalid or duplicate config key"));
        }
    }
    Ok(map)
}
#[cfg(test)]
mod tests {
    #[test]
    fn parser() {
        assert_eq!(
            super::parse("# comment\nid=x\nversion=1").unwrap()["id"],
            "x"
        );
    }
    #[test]
    fn duplicates() {
        assert!(super::parse("id=a\nid=b").is_err());
        assert!(super::parse("../=x").is_err());
    }
}
