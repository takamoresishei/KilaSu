# Contributing

Open a focused issue or pull request against `main`. State the trigger, observed
behavior, expected behavior, device/kernel family and reproducible steps. Strip
tokens, private application data and complete device identifiers from reports.

Run `./scripts/build.sh host` and Manager checks when changing its sources. For
kernel changes, all three ACK compile jobs must pass. Provide actual enforcing
device evidence for credential, lifecycle, mount or recovery changes. Compile
success alone cannot establish boot or root behavior.

Keep authorization checks in the kernel/daemon boundary. Add a protocol version
or negotiated feature before extending wire behavior. Never return invented
backend statuses. Unsupported operations should fail with a useful error.

Preserve the component licenses and third-party notices. Contributions must be
yours to license under the destination component's SPDX license. Submit small,
reviewable changes and explain validation, compatibility and any residual risk.
