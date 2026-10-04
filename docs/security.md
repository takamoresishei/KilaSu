# Privileged trust model

Trust anchors are the source kernel/ROM, the release signing key, the exact
Manager APK pinned by recovery, root-owned state, and explicitly installed root
modules. An existing root administrator can change this system; protection
against an already compromised kernel/root administrator is not claimed.

Kernel rules: application UID range validation; current real/effective UID
agreement; initial user namespace; single-threaded root clients; exact command
sizes; supported API/magic; bounded tables; admin-only enrollment/profile/ticket
mutation; default-deny profiles; one-time task tickets; serialized credential
commit/revoke; fixed SELinux domain. A transferred /dev/kilasu descriptor does
not transfer caller privileges. Tickets hold task references, preventing PID
reuse from converting a prior ticket into another process's access.

Daemon rules: kernel-provided socket peers; unique package/UID mapping; exact
APK hash verification on administrative calls and on every approved root-ticket request;
root-owned pin/database; atomic writes; install size limits; bounded workers;
safe module paths; CRC checks; controlled script environment/timeouts. Shared
UID packages, split APK packages and unpinned app updates fail closed.

Manager and CLI verify that the connected socket peer is UID 0 before sending
requests. An ordinary app that binds the abstract socket name cannot impersonate
the daemon. APK hashing streams through a fixed 4 KiB buffer and rejects a file
that changes during verification; unapproved callers are rejected before APK
hashing. Both sides apply bounded socket timeouts.

APK pins are hashes of signed complete artifacts, rather than a Java-provided
claim about their signer. The Android package manager validates the APK install;
the daemon additionally compares actual installed bytes with the recovery pin.
Manager updates require a matching pin update through the trusted release path.

Recovery rules: exact device/slot/partition, full source checksum, image bounds,
mandatory verified backup, same-size write, readback and conservative removal.
Unsigned output requires explicit Manager export and verified unlocked state.
No Manager partition flashing or automatic reboot exists.

Logs omit commands, environment secrets and private app content. Kernel audit
records numeric UID, command, result and boot time. Third-party module scripts
are trusted code and control their own output. The project does not implement
bank, anti-cheat, DRM or integrity-check evasion.

Known validation gaps are recorded rather than hidden: runtime root, source ROM
policy compilation, enforcing mounts, hardware recovery and startup ordering.
Passing host tests or compiling a driver does not close those gaps.
