# Installation

0.1.0 is for source integration and device validation. Use the target device's
compatible kernel and a ROM containing reviewed KilaSU policy/init integration.
Android/GKI versions do not make an arbitrary prebuilt kernel compatible.

Install `/system/bin/kilasd` and `/system/bin/kila` in the ROM, expose the CLI as
`/system/xbin/su` when desired, and import the lifecycle rc. Recovery provisions
the mutable daemon/CLI and Manager APK identity pin under `/data/adb/kilasu`.

Build the kernel with CONFIG_KILASU=y and the built-in SELinux adapter. Dump the
complete original boot partition from the active slot. Its size must match the
partition, because recovery verifies the entire partition checksum. Create a
device payload with `tools/make_payload.py`; its source hash must match the
kernel blob in that original image.

Install the corresponding signed Manager APK. Select original boot.img and the
payload, inspect analysis, patch and export the recovery bundle. Place it at
`/sdcard/KilaSU/KilaSU-patch.zip`. In TWRP, OrangeFox, PBRP or a compatible recovery,
mount/decrypt data and flash the matching official Installer ZIP. Installation
is restricted to the detected active slot. Read its backup/verification result.
No automatic reboot is performed.

After a successful manual reboot, Manager should separately report kernel, API,
policy readiness and daemon connectivity. Test an unapproved `su -c id` first:
it must fail or wait for Manager approval. Then test allow once, permanent allow,
revoke and enforcing status. Record these results before a stable release.
