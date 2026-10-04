#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Publish an explicit development request after all same-source workflows pass."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib

from ci_package import api, collect
from package import package, digest

ROOT = Path(__file__).resolve().parents[1]


def validate_request(request, version, sha):
    if set(request) != {"tag"}:
        raise ValueError("prerelease request must contain exactly one tag")
    tag = request["tag"]
    if not isinstance(tag, str) or not re.fullmatch(
            rf"v{re.escape(version)}-(?:alpha|beta|rc)\.[1-9][0-9]*", tag):
        raise ValueError("tag must match the component version and alpha/beta/rc sequence")
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("a full source commit is required")
    return tag


def write_metadata(output, tag, sha, runs):
    (output / "DEVELOPMENT.txt").write_text(
        f"KilaSU {tag} — DEVELOPMENT PRERELEASE\nCommit: {sha}\n"
        "Manager uses a CI debug signing key, not the persistent stable release key.\n"
        "Install this exact APK to match the recovery package's signed APK pin.\n"
        "This package requires a source-integrated ROM with KilaSU SELinux/init rules\n"
        "and a device-specific CONFIG_KILASU kernel payload. It cannot root stock ROMs.\n"
        "Device root, enforcing policy, lifecycle, mounts and recovery flashing\n"
        "have not been validated on physical hardware. Read the release notes.\n")
    (output / "build-info.json").write_text(json.dumps({
        "tag": tag, "source_commit": sha, "api": 1,
        "manager_signing": "ci-debug", "builds": runs,
        "device_validation": [],
    }, indent=2) + "\n")
    (output / "checksums.txt").write_text("".join(
        f"{digest(p)}  {p.name}\n" for p in sorted(output.iterdir())
        if p.is_file() and p.name != "checksums.txt"))


def notes(tag, repo, sha, runs):
    base = f"https://github.com/{repo}"
    evidence = "\n".join(f"- [{name}]({run['url']}): passed" for name, run in sorted(runs.items()))
    return f"""# KilaSU {tag}

## What's New

First development prerelease of the independent KilaSU kernel core, versioned
UAPI, Rust kilasd/CLI, UID allowlist and process tickets, module lifecycle,
boot-container patcher, recovery packages, and Compose Manager with real backdrop
glass blur. See [CHANGELOG]({base}/blob/{tag}/CHANGELOG.md).

Source: [{sha}]({base}/commit/{sha}). All binaries below were built from this
exact revision; the recovery package pins the exact included signed Manager APK.

## Kernel Support

arm64 ACK source compile targets: android12-5.10, android13-5.15, android14-6.1.
The core, enforcing credential adapter and KUnit suite compile on all three.
Android Manager requires Android 12 / API 31 or newer.
These are compilation targets, **not certified device or stock-ROM support**.

## Installation

This is for source-integrated development devices. Integrate ROM SELinux/init
and ueventd rules, compile the exact device kernel with CONFIG_KILASU=y, create
its pinned kernel payload, patch/export boot.img and KilaSU-patch.zip in Manager,
then use the matching Installer ZIP in recovery. Manager only patches/exports.
Follow [installation]({base}/blob/{tag}/docs/installation.md) and
[recovery installation]({base}/blob/{tag}/docs/recovery-installation.md).

The release does not include a generic patched boot image or kernel payload:
those must match the device, source kernel, partition size and vendor ABI.

## Downloads

- KilaSU-Manager-{tag}.apk — actual built Manager, signed with the CI debug key.
- KilaSU-{tag}.apk — identical short-name APK alias.
- KilaSU-Installer-{tag}.zip — arm64 daemon/CLI and static recovery validator.
- KilaSU-Uninstaller-{tag}.zip — validated backup restoration workflow.
- build-info.json — source and successful CI run links.
- DEVELOPMENT.txt — package prerequisites and development status.
- checksums.txt — SHA-256 for every attached package and metadata file.

The CI debug signing identity is specific to this build and may change in a
future prerelease; a persistent production signing key is not provisioned.

## Checksums

Download the listed assets and run `sha256sum -c checksums.txt` in that directory.
The two APK hashes are equal. Do not combine an APK from another build with the
Installer's identity pin.

## Validation

{evidence}

Rust/host tests, Manager unit tests/lint, disconnected-state emulator test, ACK
compile matrix, arm64 daemon cross-build and static recovery build pass. KUnit
is compiled; it has not been executed in a booted kernel. See the
[validation report]({base}/blob/{tag}/docs/validation.json).

## Known Issues

- Root grant/revoke, enforcing ROM policy, lifecycle, physical module mounts and
  recovery write/restore have not been validated on a physical device.
- No universal stock-kernel binary injector or generic live OEM policy patcher.
- Boot header 3/4 patching needs a precompiled compatible KilaSU kernel payload;
  legacy headers 0–2 are analysis-only. No direct Manager flashing.
- Module mounts support existing /system file targets; directory overlays and
  new files are not implemented. Conflicting file targets are rejected.
- Universal single APKs are required for identity-pinned root authorization;
  split APKs and shared UIDs are rejected.
- Advanced environment, UID remapping and module-visibility app profiles remain
  future work. API v1 currently enforces access and capability settings.
- Stable release publication remains gated on persistent signing and genuine
  SELinux Enforcing device validation.

License notices and corresponding project source remain available at this tag.
"""


def main():
    repo, sha = os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_SHA"]
    version = tomllib.loads((ROOT / "userspace/kilasd/Cargo.toml").read_text())["package"]["version"]
    request = json.loads((ROOT / ".github/prerelease.json").read_text())
    tag = validate_request(request, version, sha)
    # Existing releases/tags are immutable inputs to this publisher.
    releases = api(f"repos/{repo}/releases?per_page=100")
    tags = api(f"repos/{repo}/tags?per_page=100")
    if any(r["tag_name"] == tag for r in releases) or any(t["name"] == tag for t in tags):
        raise RuntimeError("prerelease tag already exists; request a new sequence")
    with tempfile.TemporaryDirectory(prefix="kilasu-prerelease-", dir=os.environ["RUNNER_TEMP"]) as temp:
        workspace = Path(temp)
        runs = collect(repo, sha, workspace, include_manager=True)
        output = ROOT / "dist/prerelease"
        if output.exists() and any(output.iterdir()):
            raise RuntimeError("prerelease output directory must be empty")
        package(tag, workspace / "manager/app-debug.apk", workspace / "daemon",
                workspace / "recovery/kila-boot", output)
        write_metadata(output, tag, sha, runs)
        note_file = workspace / "release-notes.md"
        note_file.write_text(notes(tag, repo, sha, runs))
        assets = [str(p) for p in sorted(output.iterdir()) if p.is_file()]
        subprocess.run(["gh", "release", "create", tag, *assets, "--repo", repo,
                        "--target", sha, "--prerelease", "--latest=false",
                        "--title", f"KilaSU {tag} — Development Prerelease",
                        "--notes-file", str(note_file)], check=True)
        release = api(f"repos/{repo}/releases/tags/{tag}")
        attached = {a["name"]: a for a in release["assets"]}
        expected = {p.name: p for p in output.iterdir() if p.is_file()}
        if release["draft"] or not release["prerelease"] or set(attached) != set(expected):
            raise RuntimeError("published release metadata/asset list does not match")
        for name, path in expected.items():
            asset = attached[name]
            if asset["size"] != path.stat().st_size or asset.get("digest") != "sha256:" + digest(path):
                raise RuntimeError(f"uploaded asset validation failed: {name}")
        print(f"Verified published prerelease: {release['html_url']}", flush=True)


if __name__ == "__main__":
    main()
