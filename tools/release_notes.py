#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
import sys
tag=sys.argv[1]
print(f'''# KilaSU {tag}

## What's New

See [CHANGELOG](https://github.com/takamoresishei/KilaSu/blob/{tag}/CHANGELOG.md).

## Kernel Support

ACK arm64 source targets: android12-5.10, android13-5.15, android14-6.1.
The [validation report](https://github.com/takamoresishei/KilaSu/blob/{tag}/docs/validation.json)
identifies tested devices. A passing compile is not device certification.

## Installation

Integrate enforcing policy/init into the target ROM. Build a compatible source
kernel, patch/export boot.img in Manager, export KilaSU-patch.zip, then flash
the matching official Installer ZIP in recovery. Manager does not flash.
See [installation](https://github.com/takamoresishei/KilaSu/blob/{tag}/docs/installation.md).

## Downloads

- KilaSU-Manager-{tag}.apk (KilaSU-{tag}.apk is an identical alias)
- KilaSU-Installer-{tag}.zip
- KilaSU-Uninstaller-{tag}.zip

## Checksums

Download checksums.txt and run `sha256sum -c checksums.txt` in the downloads directory.

## Known Issues

See [troubleshooting](https://github.com/takamoresishei/KilaSu/blob/{tag}/docs/troubleshooting.md).
No universal stock-kernel binary patcher, directory overlays, split Manager APKs,
or arbitrary SELinux domain selection are offered by API v1.
''')
