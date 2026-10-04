#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Create an explicit source-kernel payload, without inventing a device image."""
import argparse
import hashlib
from pathlib import Path
import re
import subprocess
import zipfile

def make(source, image, device, validator, output):
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", device):
        raise ValueError("invalid device codename")
    info = dict(line.split("=", 1) for line in subprocess.check_output([validator, "analyze", source], text=True).splitlines())
    content = Path(image).read_bytes()
    if len(content) > 128 * 1024 * 1024:
        raise ValueError("kernel payload exceeds size limit")
    prop = (f"device={device}\napi=1\nsource_kernel_sha256={info['kernel_sha256']}\n"
            f"kernel_sha256={hashlib.sha256(content).hexdigest()}\npolicy_in_rom=true\ndaemon_init_in_rom=true\n")
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("Image", content)
        z.writestr("payload.prop", prop)

if __name__ == "__main__":
    p = argparse.ArgumentParser(description="Requires an already source-integrated ROM policy and init service.")
    for name in ["source", "image", "device", "validator", "output"]:
        p.add_argument("--" + name, required=True)
    p.add_argument("--rom-integrated", action="store_true", required=True,
                   help="confirm this ROM contains KilaSU SELinux/init integration")
    a = p.parse_args()
    make(a.source, a.image, a.device, a.validator, a.output)
