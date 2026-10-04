#!/sbin/sh
# SPDX-License-Identifier: GPL-3.0-only
say 'KilaSU Recovery Installer'
require_tools
device_info
prepare_data
[ ! -f "$STATE" ] || die 'This slot already has an installation record; uninstall before replacing the root kernel'
BUNDLE=/sdcard/KilaSU/KilaSU-patch.zip
[ -f "$BUNDLE" ] || die 'Export the Manager recovery bundle to /sdcard/KilaSU/KilaSU-patch.zip'
say 'Checking boot image...'
unzip -p "$BUNDLE" receipt.prop | head -c 8193 > "$WORK/receipt.prop"
[ "$(wc -c < "$WORK/receipt.prop")" -le 8192 ] || die 'Receipt exceeds size limit'
RECEIPT="$WORK/receipt.prop"
[ "$(prop "$RECEIPT" device)" = "$DEVICE" ] || die 'Patch was exported for a different device'
[ "$(prop "$RECEIPT" api)" = 1 ] || die 'Unsupported patch API'
SOURCE_HASH=$(prop "$RECEIPT" source_sha256)
PATCH_HASH=$(prop "$RECEIPT" patched_sha256)
valid_hash "$SOURCE_HASH" && valid_hash "$PATCH_HASH" || die 'Malformed image checksum'
[ "$(prop "$RECEIPT" source_size)" = "$PART_SIZE" ] || die 'Use an image dumped from this exact boot partition; sizes differ'
unzip -p "$BUNDLE" patched.img | head -c "$((PART_SIZE + 1))" > "$WORK/patched.img"
[ "$(wc -c < "$WORK/patched.img")" = "$PART_SIZE" ] || die 'Patched image size does not equal boot partition size'
[ "$(digest "$WORK/patched.img")" = "$PATCH_HASH" ] || die 'Patched image checksum mismatch'
[ "$(digest "$PART")" = "$SOURCE_HASH" ] || die 'Active boot partition differs from source image; nothing was written'
"$WORK/kila-boot" analyze "$WORK/patched.img" > "$WORK/image.prop" || die 'Boot parser rejected image'
[ "$(prop "$WORK/image.prop" arm64)" = 1 ] && [ "$(prop "$WORK/image.prop" kilasu)" = 1 ] || die 'Image lacks a supported KilaSU arm64 kernel'
IMAGE_VERSION=$(prop "$WORK/image.prop" header_version)
case "$IMAGE_VERSION" in 3|4) ;; *) die 'Only GKI boot headers 3 and 4 are installable';; esac
[ "$(prop "$WORK/image.prop" kernel_release)" = "$(prop "$RECEIPT" kernel_release)" ] || die 'Kernel release mismatch'
say 'Checking KilaSU compatibility...'
[ "$(prop "$RECEIPT" policy_in_rom)" = true ] && [ "$(prop "$RECEIPT" daemon_init_in_rom)" = true ] || die 'ROM SELinux/init integration must be provided by the payload builder'
if [ "$(prop "$RECEIPT" unsigned_export)" = true ]; then
 LOCK_STATE=$(getprop ro.boot.flash.locked)
 VB_STATE=$(getprop ro.boot.verifiedbootstate)
 [ "$LOCK_STATE" = 0 ] || [ "$VB_STATE" = orange ] || die 'Cannot confirm an unlocked bootloader for unsigned export'
fi
BACKUP="$STORE/backups/boot${SLOT:-_single}-$SOURCE_HASH.img"
if [ ! -f "$BACKUP" ]; then
 dd if="$PART" of="$BACKUP.tmp" bs=4096 || die 'Backup failed'
 sync
 [ "$(digest "$BACKUP.tmp")" = "$SOURCE_HASH" ] || die 'Backup verification failed'
 chmod 0600 "$BACKUP.tmp"
 mv "$BACKUP.tmp" "$BACKUP"
fi
[ "$(digest "$BACKUP")" = "$SOURCE_HASH" ] || die 'Existing backup is corrupt'
unzip -p "$ZIP" payload/kilasd > "$STORE/bin/kilasd.new"
unzip -p "$ZIP" payload/kila > "$STORE/bin/kila.new"
unzip -p "$ZIP" payload/manager.prop > "$STORE/manager.prop.new"
for binary in kilasd kila; do
 EXPECTED=$(prop "$WORK/package.prop" "${binary}_sha256")
 valid_hash "$EXPECTED" && [ "$(digest "$STORE/bin/$binary.new")" = "$EXPECTED" ] || die 'Userspace payload checksum mismatch'
 chmod 0755 "$STORE/bin/$binary.new"
 mv "$STORE/bin/$binary.new" "$STORE/bin/$binary"
done
PIN=$(prop "$STORE/manager.prop.new" apk_sha256)
valid_hash "$PIN" || die 'Invalid Manager identity pin'
chmod 0600 "$STORE/manager.prop.new"
mv "$STORE/manager.prop.new" "$STORE/manager.prop"
say 'Installing KilaSU...'
WRITE_OK=true
dd if="$WORK/patched.img" of="$PART" bs=4096 || WRITE_OK=false
sync
say 'Verifying...'
if [ "$WRITE_OK" != true ] || [ "$(digest "$PART")" != "$PATCH_HASH" ]; then
 say 'Verification failed. Restoring backup...'
 dd if="$BACKUP" of="$PART" bs=4096 || die 'Backup restore failed; do not reboot'
 sync
 [ "$(digest "$PART")" = "$SOURCE_HASH" ] || die 'Restore verification failed; do not reboot'
 die 'Installation rolled back after verification failure'
fi
printf 'device=%s\nslot=%s\npartition=%s\nsource_sha256=%s\npatched_sha256=%s\nbackup=%s\n' "$DEVICE" "$SLOT" "$PART" "$SOURCE_HASH" "$PATCH_HASH" "$BACKUP" > "$STATE.new"
chmod 0600 "$STATE.new"
mv "$STATE.new" "$STATE"
say 'Installation complete.'
say 'Backup retained. ROM init restores SELinux labels before starting kilasd.'
say 'Reboot manually when ready.'
