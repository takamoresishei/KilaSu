# Security policy

KilaSU controls privileged credentials. The development branch has no validated
production support guarantee. Report vulnerabilities privately through this
repository's GitHub security advisory feature when available; avoid public
exploit details before coordinated fixes. Do not send private data in an issue.

Useful reports identify the affected commit/API, caller UID/domain, necessary
prior permissions, reproducible behavior, and a proposed minimal fix. Do not
include passwords, APK signing keys, root secrets, bank credentials or user data.

The trust boundary is detailed in [docs/security.md](docs/security.md). Signed
Manager code, root-approved modules, the target kernel/ROM and an already
privileged root administrator are trusted. Arbitrary applications are not.

Stable releases require regression tests and a recorded enforcing device report.
Users can disable a module in recovery and restore the exact original boot
partition through the slot-specific Uninstaller when its record still matches.
