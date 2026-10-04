# Changelog

## 0.1.0 — Development source implementation

- Independent `/dev/kilasu` core with bounded, versioned UAPI and default-deny
  UID authorization; short-lived task-bound daemon tickets precede grants.
- APK identity enrollment, persisted app grants, one-time access, capabilities,
  boot-relative request/use times and bounded audit records.
- Modular Rust daemon/CLI, authenticated Unix socket protocol, validated module
  ZIP extraction, staged updates, lifecycle hooks and existing-file bind mounts.
- C GKI boot parser/repacker reused by Manager JNI and recovery validation.
- Compose Manager with captured backdrop blur, appearance settings, boot export,
  superuser controls, module controls, request dialogs and diagnostics.
- Slot-specific recovery install/restore, original-image backups and readback.
- Target-specific kernel integration, host tests, Android builds, compile matrix,
  emulator screenshot workflow and gated signed release pipeline.

Runtime GKI root, ROM policy/init, real recovery writes and module mounts remain
unvalidated. This version is not a stable device release.
