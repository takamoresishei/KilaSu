#!/sbin/sh
# SPDX-License-Identifier: GPL-3.0-only
set -eu
say() { printf 'ui_print %s\nui_print\n' "$*" >&"$OUTFD"; }
die() { say "ERROR: $*"; exit 1; }
prop() {
 awk -v wanted="$2" 'index($0,"=") && substr($0,1,index($0,"=")-1)==wanted { n++; value=substr($0,index($0,"=")+1) } END { if(n!=1)exit 1; print value }' "$1"
}
digest() { sha256sum "$1" | awk '{print $1}'; }
valid_hash() { [ "${#1}" = 64 ] && ! printf '%s' "$1" | LC_ALL=C grep -q '[^0-9a-f]'; }
require_tools() { for tool in dd blockdev readlink sha256sum unzip awk getprop; do command -v "$tool" >/dev/null 2>&1 || die "Recovery tool missing: $tool"; done; }
device_info() {
 DEVICE=$(getprop ro.product.device)
 [ -n "$DEVICE" ] || die 'Cannot determine device identity'
 case "$DEVICE" in *[!A-Za-z0-9_.-]*) die 'Invalid device identity';; esac
 SLOT=$(getprop ro.boot.slot_suffix)
 if [ -z "$SLOT" ]; then
  RAW_SLOT=$(getprop ro.boot.slot)
  case "$RAW_SLOT" in a|b) SLOT="_$RAW_SLOT";; '') SLOT='';; *) die 'Unrecognized boot slot';; esac
 fi
 case "$SLOT" in _a|_b|'') ;; *) die 'Invalid slot suffix';; esac
 AB=false
 if [ -e /dev/block/by-name/boot_a ] || [ -e /dev/block/bootdevice/by-name/boot_a ]; then AB=true; fi
 if [ "$AB" = true ] && [ -z "$SLOT" ]; then die 'A/B boot partitions exist but active slot is unknown'; fi
 if [ "$AB" = false ] && [ -n "$SLOT" ]; then die 'Slot property and partition layout disagree'; fi
 PART=''
 for candidate in "/dev/block/by-name/boot$SLOT" "/dev/block/bootdevice/by-name/boot$SLOT"; do
  if [ -e "$candidate" ]; then
   resolved=$(readlink -f "$candidate")
   [ -b "$resolved" ] || die 'Boot target is not a block device'
   if [ -n "$PART" ] && [ "$PART" != "$resolved" ]; then die 'Ambiguous boot partition mappings'; fi
   PART=$resolved
  fi
 done
 [ -n "$PART" ] || die 'Exact boot partition was not found'
 PART_SIZE=$(blockdev --getsize64 "$PART")
 case "$PART_SIZE" in ''|*[!0-9]*) die 'Cannot determine boot partition size';; esac
 [ "$PART_SIZE" -gt 4096 ] && [ "$PART_SIZE" -le 536870912 ] || die 'Boot partition size outside supported limits'
 say "Device: $DEVICE"
 say "Kernel: $(uname -r) (recovery runtime)"
 say "Boot Slot: ${SLOT:-non-A/B}"
 say "Boot Partition: $PART"
}
prepare_data() {
 [ -d /data/adb ] || die '/data/adb unavailable; mount/decrypt data in recovery first'
 STORE=/data/adb/kilasu
 [ ! -L "$STORE" ] || die 'State directory cannot be a symlink'
 mkdir -p "$STORE/bin" "$STORE/backups" "$STORE/logs"
 chmod 0700 "$STORE" "$STORE/bin" "$STORE/backups" "$STORE/logs"
 STATE="$STORE/installed${SLOT:-_single}.prop"
}
