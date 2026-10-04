# Development evidence

| Stage | Implemented source | Evidence / remaining validation |
|---|---|---|
| 1 Repository/build | Monorepo, licenses, Gradle/Cargo/CMake, scripts | Local host tools and CI definitions |
| 2 Kernel core | Own driver/core, UID checks, credential commit | External Linux compile smoke passed; ACK matrix CI |
| 3 UAPI | Fixed 512-byte protocol, commands, features | C ABI assertions and Rust serialization tests |
| 4 Version client | Rust/native version reads | Real device ioctl still required |
| 5 Daemon | Root init service, socket/kernel integration | Host parser/protocol tests; runtime pending |
| 6 Manager detection | JNI version/features and actual RPC state | Android build/emulator CI; no synthetic statuses |
| 7 Authorization | UID/profile plus live identity/task tickets | Must test allow/once/revoke on actual GKI |
| 8 Lifecycle | Early, post-fs-data, service, boot-completed | ROM init ordering/device run required |
| 9 Modules | Safe ZIP, staged update, scripts, file mounts | ZIP tests; enforcing mounts/reboot pending |
| 10 Boot patcher | Shared C parser/repack, JNI/export | Host repack/preservation/rejection tests passed |
| 11 Recovery | Slot, full hashes, backup/write/readback/restore | Static validator/shell CI; no hardware flash performed |
| 12 Profiles | Capability mask, grant type, boot-time history | Environment/per-module visibility/namespace selection deferred |
| 13 SELinux | Fixed-domain adapter, source policy/labels | Target-ROM compiler/neverallow/AVC validation still required |
| 14 Glass UI | Captured blur, tint/reflection, preferences, launch | Actual emulator screenshot workflow |
| 15 Release/tests/docs | Host tests, CI matrix, signing/release gate | Signing secrets and genuine device report required for stable tag |

These are separate implementation and runtime-validation states. Source files,
passing parser tests or compile success do not mean all fifteen stages have
been certified on a phone. The stable release gate remains closed until the
runtime checks are supplied and pass.
