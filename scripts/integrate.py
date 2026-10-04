#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Idempotent source integration; refuses to overwrite unknown directories."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MARK = "# KilaSU managed integration"

def version(tree):
    content = (tree / "Makefile").read_text()
    def number(key):
        match = re.search(rf"^{key}\s*=\s*(\d+)\s*$", content, re.M)
        if not match:
            raise ValueError(f"missing kernel {key}")
        return int(match.group(1))
    result = (number("VERSION"), number("PATCHLEVEL"), number("SUBLEVEL"))
    if result[:2] not in {(5, 10), (5, 15), (6, 1)}:
        raise ValueError(f"kernel {result} is outside the ACK target matrix")
    return result

def atomic(path, text):
    fd, name = tempfile.mkstemp(dir=path.parent, prefix=".kila-")
    try:
        with os.fdopen(fd, "w") as output:
            output.write(text)
            output.flush()
            os.fsync(output.fileno())
        os.chmod(name, path.stat().st_mode & 0o777)
        os.replace(name, path)
    finally:
        if os.path.exists(name):
            os.unlink(name)

def integrate(tree):
    tree = Path(tree).resolve(strict=True)
    release = version(tree)
    drivers = tree / "drivers"
    targets = {
        drivers / "Makefile": 'obj-$(CONFIG_KILASU) += kilasu/kernel/',
        drivers / "Kconfig": 'source "drivers/kilasu/kernel/Kconfig"',
    }
    original = {}
    for path, entry in targets.items():
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"unsafe integration target: {path}")
        text = path.read_text()
        if "KilaSU" in text and f"{MARK}\n{entry}" not in text:
            raise ValueError("unrecognized prior KilaSU integration; resolve manually")
        original[path] = text
    destination = drivers / "kilasu"
    marker = destination / "integration.json"
    if destination.exists() and (destination.is_symlink() or not marker.is_file()):
        raise ValueError("drivers/kilasu is not owned by this integration script")
    temporary = Path(tempfile.mkdtemp(prefix=".kila-source-", dir=drivers))
    backup = drivers / ".kilasu-previous"
    if backup.exists():
        shutil.rmtree(temporary)
        raise ValueError("unfinished previous transaction: recover .kilasu-previous first")
    try:
        for folder in ["kernel", "uapi"]:
            shutil.copytree(ROOT / folder, temporary / folder, ignore=shutil.ignore_patterns("*.o", "*.ko", "*.mod*", ".*.cmd", "Module.symvers", "modules.order"))
        shutil.copy(ROOT / "LICENSE", temporary / "LICENSE")
        report = {"kernel": ".".join(map(str, release)), "api": 1,
                  "source": str(ROOT), "selinux_policy_required": True,
                  "grant_enabled_only_with_rom_policy": True}
        (temporary / "integration.json").write_text(json.dumps(report, indent=2) + "\n")
        if destination.exists():
            os.replace(destination, backup)
        os.replace(temporary, destination)
        for path, entry in targets.items():
            text = original[path]
            block = f"{MARK}\n{entry}\n"
            if block not in text:
                atomic(path, text.rstrip() + "\n\n" + block)
        if backup.exists():
            shutil.rmtree(backup)
        print(json.dumps(report, indent=2))
        print("Enable CONFIG_KILASU=y and integrate ROM policy/init. No kernel was built or flashed.")
        return report
    except Exception:
        for path, text in original.items():
            if path.read_text() != text:
                atomic(path, text)
        if backup.exists():
            if destination.exists():
                shutil.rmtree(destination)
            os.replace(backup, destination)
        elif destination.exists() and marker.is_file():
            shutil.rmtree(destination)
        raise
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("kernel_tree")
    args = parser.parse_args()
    try:
        integrate(args.kernel_tree)
    except (OSError, ValueError) as error:
        parser.exit(1, f"KilaSU integration failed: {error}\n")
