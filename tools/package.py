#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Build deterministic recovery packages with identity-pinned Manager metadata."""
import argparse
import hashlib
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parents[1]
def digest(file):
    h = hashlib.sha256()
    with file.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()

def package(version, apk, daemon_dir, validator, output):
    if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?", version):
        raise ValueError("version must be vX.Y.Z with an optional prerelease suffix")
    apk, daemon_dir, validator, output = map(Path, (apk, daemon_dir, validator, output))
    files = [apk, daemon_dir / "kilasd", daemon_dir / "kila", validator]
    if any(not p.is_file() or p.stat().st_size == 0 for p in files):
        raise ValueError("Manager, daemon, CLI and static arm64 validator must exist")
    output.mkdir(parents=True, exist_ok=True)
    entries = {
        "META-INF/com/google/android/update-binary": (ROOT / "recovery/common/update-binary", 0o755),
        "lib/common.sh": (ROOT / "recovery/common/common.sh", 0o644),
        "bin/kila-boot": (validator, 0o755),
        "LICENSE": (ROOT / "LICENSE", 0o644),
    }
    for action, label in [("install", "Installer"), ("uninstall", "Uninstaller")]:
        payload = dict(entries)
        payload["lib/action.sh"] = (ROOT / f"recovery/{'installer/install' if action == 'install' else 'uninstaller/uninstall'}.sh", 0o644)
        prop = f"action={action}\nversion={version}\nkilasd_sha256={digest(daemon_dir / 'kilasd')}\nkila_sha256={digest(daemon_dir / 'kila')}\nvalidator_sha256={digest(validator)}\n"
        data = {name: (path.read_bytes(), mode) for name, (path, mode) in payload.items()}
        data["lib/package.prop"] = (prop.encode(), 0o644)
        data["META-INF/com/google/android/updater-script"] = (b"# KilaSU custom update-binary\n", 0o644)
        if action == "install":
            for binary in ["kilasd", "kila"]:
                data[f"payload/{binary}"] = ((daemon_dir / binary).read_bytes(), 0o755)
            data["payload/manager.prop"] = (f"apk_sha256={digest(apk)}\nversion={version}\n".encode(), 0o600)
        target = output / f"KilaSU-{label}-{version}.zip"
        with zipfile.ZipFile(target, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for name, (blob, mode) in sorted(data.items()):
                info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
                info.create_system = 3
                info.external_attr = (0o100000 | mode) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(info, blob)
    apk_name = output / f"KilaSU-Manager-{version}.apk"
    apk_name.write_bytes(apk.read_bytes())
    # Short filename retained as the documented release alias.
    (output / f"KilaSU-{version}.apk").write_bytes(apk.read_bytes())
    checksums = "".join(f"{digest(p)}  {p.name}\n" for p in sorted(output.iterdir()) if p.suffix in {".apk", ".zip"})
    (output / "checksums.txt").write_text(checksums)
    return output

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    for name in ["version", "apk", "daemon-dir", "validator", "output"]:
        parser.add_argument("--" + name, required=True)
    a = parser.parse_args()
    package(a.version, a.apk, a.daemon_dir, a.validator, a.output)
