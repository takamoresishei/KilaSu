# Recovery installation and removal

Packages use the update-binary interface and include a static arm64 validator.
Required recovery tools: sh, unzip, sha256sum, blockdev, readlink, dd, awk,
getprop, head, wc and sync. Data must be mounted/decrypted; backup creation is
mandatory in this version.

The installer derives active slot from boot properties and checks actual
by-name layout. Missing/invalid slot information, conflicting block mappings,
wrong device, wrong size, mismatched original checksum or invalid boot parser
results stop installation before a partition write. A/B writes use only the
detected active boot slot. init_boot/vendor_boot are not guessed or modified.

The patch bundle must be `/sdcard/KilaSU/KilaSU-patch.zip`. The receipt binds the
device, source image, full source size, patched checksum, API and target release.
The full original partition is backed up under `/data/adb/kilasu/backups` and
its hash is verified. The installer writes the same-size output and verifies
the entire partition. Failed writes/readback attempt to restore the original
verified backup. A restore failure is printed explicitly.

An installation record binds device, slot, resolved partition, original hash,
patched hash and backup path. The Uninstaller checks every field and refuses
to overwrite a partition changed by another installation or firmware update.
It restores the exact original image, verifies it and retains module data and
backup. No package automatically reboots.

For updates, uninstall/restore the original boot first, then prepare a new
device payload and install the matching APK/Installer. In-place root-kernel
updates and dual-slot transactions are not implemented by 0.1.0. Recovery flash
behavior still needs validation on each supported recovery/device combination.
