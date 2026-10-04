#!/sbin/sh
# SPDX-License-Identifier: GPL-3.0-only
say 'KilaSU Recovery Uninstaller'
require_tools
device_info
prepare_data
[ -f "$STATE" ] || die 'No KilaSU installation record for this slot'
[ "$(prop "$STATE" device)" = "$DEVICE" ] && [ "$(prop "$STATE" partition)" = "$PART" ] && [ "$(prop "$STATE" slot)" = "$SLOT" ] || die 'Installation record does not match this device and active slot'
BACKUP=$(prop "$STATE" backup)
case "$BACKUP" in "$STORE/backups/boot${SLOT:-_single}-"*.img) ;; *) die 'Backup path rejected';; esac
[ ! -L "$BACKUP" ] && [ -f "$BACKUP" ] || die 'Verified original boot backup is unavailable'
[ "$(wc -c < "$BACKUP")" = "$PART_SIZE" ] || die 'Backup size mismatch'
SOURCE_HASH=$(prop "$STATE" source_sha256)
PATCH_HASH=$(prop "$STATE" patched_sha256)
valid_hash "$SOURCE_HASH" && valid_hash "$PATCH_HASH" || die 'Invalid installation checksum'
[ "$(digest "$BACKUP")" = "$SOURCE_HASH" ] || die 'Backup checksum mismatch'
[ "$(digest "$PART")" = "$PATCH_HASH" ] || die 'Boot partition changed after KilaSU installation; refusing to overwrite it'
say 'Restoring original boot image...'
dd if="$BACKUP" of="$PART" bs=4096 || die 'Restore failed; do not reboot'
sync
say 'Verifying...'
[ "$(digest "$PART")" = "$SOURCE_HASH" ] || die 'Restore verification failed; do not reboot'
mv "$STATE" "$STATE.uninstalled"
say 'Uninstallation complete. Module data and backup were retained.'
say 'Reboot manually when ready.'
