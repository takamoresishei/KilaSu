# Boot patching

The native library analyzes ANDROID! containers, their declared header/page
sizes, bounded kernel/ramdisk ranges, kernel release, arm64 magic, signatures
and SHA-256. Header 3/4 patching supports raw Image or a single bounded Image.gz.
Headers 0–2 can be analyzed but cannot be repacked by this version.

Create a payload after implementing the ROM policy/init integration:

```sh
gcc -std=c11 bootpatch/src/main.c bootpatch/src/kila_boot.c -lz -o kila-boot
python3 tools/make_payload.py --source original-boot.img --image Image \
  --device YOUR_DEVICE_CODENAME --validator ./kila-boot \
  --output KilaSU-device-payload.zip --rom-integrated
```

The ZIP contains only Image and payload.prop. The manifest supplies device,
API, exact source-kernel hash and replacement-kernel hash. These checks protect
against accidentally mixing images; a user-supplied payload is not an official
device certification or cryptographic release signature.

The replacement must contain KilaSU and preserve the original kernel release.
Matching release strings alone do not prove vendor symbol/ABI compatibility;
the payload builder must preserve the device's configuration and vendor ABI.
The native repacker retains the original header/cmdline and ramdisk, replaces
the kernel, aligns sections, pads to the original image size, reparses output
and verifies the ramdisk did not change.

Existing AVB footer or GKI boot signature requires explicit unsigned export for
an unlocked bootloader. Invalid signatures are removed only in that mode; the
recovery installer independently checks unlocked state. Device signing can be
performed externally with the proper device keys. No AVB security bypass or
automatic bootloader unlock is implemented.

Export the image for review and/or the recovery bundle containing patched.img
and a checksum receipt. A boot image is not bundled with the repository because
no genuine device source image was provided. The Manager has no flash operation.
