#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""A stable tag is permitted only with a matching, recorded device validation."""
import json
from pathlib import Path
import re
import sys
ROOT=Path(__file__).resolve().parents[1]
def check(tag):
    if not re.fullmatch(r'v\d+\.\d+\.\d+',tag):
        raise ValueError('stable release tags use vX.Y.Z')
    cargo=(ROOT/'userspace/kilasd/Cargo.toml').read_text()
    version=re.search(r'^version = "([^"]+)"',cargo,re.M).group(1)
    if tag != 'v'+version:
        raise ValueError('bump component versions before tagging a release')
    report=json.loads((ROOT/'docs/validation.json').read_text())
    if report['version'] != version or not report['devices']:
        raise ValueError('a stable release requires a real enforcing device validation report')
    required=['default_deny','allow_once','revoke','daemon','modules','recovery_roundtrip','selinux_enforcing']
    for device in report['devices']:
        if any(device['checks'].get(key) is not True for key in required):
            raise ValueError('device validation is incomplete')
    return report
if __name__=='__main__':
    try: check(sys.argv[1])
    except (ValueError,KeyError,OSError) as e: sys.exit(f'Release gate: {e}')
