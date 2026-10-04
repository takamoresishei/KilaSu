# Application profiles

A profile binds an Android application UID to deny, permanent allow or allow
once, plus a 64-bit Linux capability mask. Kernel validates the Android UID
range, capability range, permission values and reserved/unsupported flags.
Profile mutations occur through the privileged, identity-verifying daemon.

Manager lists package and UID, current access, the last request and grant during
this boot. It supports revoke and editing the capability mask. Request dialogs
offer deny, allow once and allow permanently. An approval is written atomically
with its package/APK hash; failed kernel writes restore the previous database.
Once grants are consumed by the kernel only after an authorized commit and
are not persisted as allowed across reboot.

API v1 uses a private mount namespace for su execution. A mask excluding
CAP_SYS_ADMIN cannot perform that namespace setup; its command fails explicitly.
Environment is sanitized by the CLI. Arbitrary UID/GID remaps, custom SELinux
domains, user-defined environment maps, namespace selection and per-module
visibility are not supported in v1 and have no dummy Manager controls.

Revocation denies future requests; it cannot undo credentials already granted
to a running process. UID grants alone never suffice: each new su task also
requires a fresh identity-verified daemon ticket bound to that task. Application
updates change the pinned APK hash and require approval again.
