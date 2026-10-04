# Module engine

Modules are trusted root code installed explicitly through Manager or an
already authorized CLI administrator. KilaSU does not download or auto-update
modules from unverified feeds.

ZIP root must contain module.prop with id, name, version, unsigned versionCode,
author and description. Optional minKilaApi prevents loading a newer API
requirement. ID accepts at most 64 alphanumeric, underscore or hyphen characters.
Duplicate property keys fail validation.

```properties
id=my_module
name=My Module
version=1.0
versionCode=1
author=Your Name
description=Describe actual changes
minKilaApi=1
```

Archive validation checks central/local headers, UTF-8 paths, duplicates,
traversal, symlinks, supported compression, expansion bounds and CRC. Encrypted,
multi-volume and ZIP64 archives are unsupported. Limits: 256 MiB ZIP/total
expanded data, 64 MiB per entry and 4096 entries. Stored and raw DEFLATE work.
Extraction does not delegate unsafe paths to an external unzip executable.

Install runs optional customize.sh with MODPATH/KILASU/KILASU_API and controlled
PATH, then atomically queues a directory under modules_update. The old active
directory is retained during rename and restored on rename failure. Unfinished
power-loss transactions are detected before activating modules. Updates become
active during post-fs-data; enable/disable/remove use persistent markers and
take effect at boot. Removal runs uninstall.sh before deleting module data.

Supported scripts: post-fs-data.sh, service.sh, boot-completed.sh, uninstall.sh.
They run with the daemon's enforcing domain, logs and a bounded execution time.
Module-provided scripts are privileged and must avoid printing secrets themselves.

The initial systemless engine bind-mounts existing regular /system files. It
rejects conflicting ownership and non-existing targets.
Mount preflight checks every target before binding any file and rejects symlinks
that could redirect a mount outside the system tree. The engine copies each
target's SELinux label to its replacement; the ROM policy must explicitly allow
the relevant relabel operations. A failed bind rolls back earlier binds in
reverse order and reports any rollback failure.

It does not implement
OverlayFS directory merges, .replace semantics, new system paths, vendor/product
remaps, Magisk installer helper functions or per-app mount filtering. Such
modules fail clearly rather than pretending to mount successfully.

Use examples/module as a small lifecycle-only module. Root/mount behavior must
be validated on a real source-integrated enforcing device before distribution.
