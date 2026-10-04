#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Collect only successful same-commit CI outputs; never publish a release key."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import zipfile

from package import package

def api(path):
    return json.loads(subprocess.check_output(["gh", "api", path], text=True))

def collect(repo, sha, workspace, include_manager=False):
    expected = {"Daemon": ("daemon-arm64", {"kila", "kilasd"}),
                "Recovery": ("recovery-validator", {"kila-boot"}),
                "Kernel": (None, set())}
    if include_manager:
        expected["Manager"] = ("manager-debug", {"app-debug.apk"})
    completed = {}
    deadline = time.monotonic() + 600
    while set(expected) != set(completed):
        runs = api(f"repos/{repo}/actions/runs?head_sha={sha}&per_page=30")["workflow_runs"]
        for name, (artifact_name, filenames) in expected.items():
            if name in completed:
                continue
            candidates = [r for r in runs if r["name"] == name and r["event"] == "push"
                          and r["head_branch"] == "main" and r["head_sha"] == sha
                          and r["path"] == f".github/workflows/build-{'kernel-test' if name == 'Kernel' else name.lower()}.yml"]
            if not candidates:
                continue
            run = max(candidates, key=lambda r: (r["run_number"], r["run_attempt"]))
            if run["status"] != "completed":
                continue
            if run["conclusion"] != "success":
                raise RuntimeError(f"{name} did not pass for commit {sha}")
            if artifact_name:
                artifacts = api(f"repos/{repo}/actions/runs/{run['id']}/artifacts")["artifacts"]
                matches = [a for a in artifacts if a["name"] == artifact_name and not a["expired"]]
                if len(matches) != 1:
                    raise RuntimeError(f"Missing or ambiguous {artifact_name}")
                artifact = matches[0]
                archive = workspace / f"{name}.zip"
                with archive.open("wb") as output:
                    subprocess.run(["gh", "api", f"repos/{repo}/actions/artifacts/{artifact['id']}/zip"],
                                   stdout=output, check=True)
                if archive.stat().st_size > 32 * 1024 * 1024:
                    raise RuntimeError("CI artifact exceeds limit")
                digest = artifact.get("digest")
                if digest and digest != "sha256:" + hashlib.sha256(archive.read_bytes()).hexdigest():
                    raise RuntimeError("CI artifact checksum mismatch")
                target = workspace / name.lower()
                target.mkdir(exist_ok=True)
                with zipfile.ZipFile(archive) as z:
                    if set(z.namelist()) != filenames or len(z.namelist()) != len(filenames):
                        raise RuntimeError("Unexpected files in CI artifact")
                    for entry in z.infolist():
                        limit = (128 if name == "Manager" else 16) * 1024 * 1024
                        if entry.file_size > limit:
                            raise RuntimeError("CI binary exceeds limit")
                        (target / entry.filename).write_bytes(z.read(entry))
            completed[name] = {"id": run["id"], "url": run["html_url"], "sha": sha}
            print(f"Verified {name} at {sha}", flush=True)
        if set(expected) != set(completed):
            if time.monotonic() >= deadline:
                raise RuntimeError("Timed out waiting for same-commit CI outputs")
            time.sleep(10)
    return completed

if __name__ == "__main__":
    import tomllib
    version = tomllib.loads(Path("userspace/kilasd/Cargo.toml").read_text())["package"]["version"]
    sha = os.environ["GITHUB_SHA"]
    workspace = Path(os.environ["RUNNER_TEMP"]) / "kilasu-ci-package"
    workspace.mkdir(exist_ok=True)
    collect(os.environ["GITHUB_REPOSITORY"], sha, workspace)
    package(f"v{version}-dev.{sha[:12]}", "artifacts/manager/app-debug.apk", workspace / "daemon",
            workspace / "recovery/kila-boot", "dist")
    Path("dist/DEVELOPMENT.txt").write_text(
        f"KilaSU development build\nCommit: {sha}\n"
        "Manager uses the CI debug key; install the exact APK in this package.\n"
        "Recovery pins its signed APK bytes. Rebuilding changes the pin.\n"
        "Source-ROM SELinux/init integration is required. Device root and flashing\n"
        "have not been certified. This package is not a stable release.\n")
