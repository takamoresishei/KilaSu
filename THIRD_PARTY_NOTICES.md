# Third-party notices and architectural references

KilaSU's authorization core, UAPI, Rust daemon, module engine, boot-container
implementation and Compose screens are independently written. No KernelSU,
ReSukiSU, SukiSU, APatch or KOWSU kernel/manager implementation is vendored or
renamed here. References inform the separation of privileged layers and the
need for explicit policy, attribution and lifecycle integration.

| Material | Use | License / notice |
|---|---|---|
| Linux / Android Common Kernel | Build environment and kernel interfaces | GPL-2.0; its original tree retains all notices |
| Android UAPI headers | OS syscall interfaces | Their original SPDX licenses and syscall exceptions |
| Gradle 8.11.1 wrapper | `manager/gradlew` and wrapper JAR | Apache-2.0; Gradle and contributors; original wrapper notices preserved |
| AndroidX / Jetpack Compose / Material 3 | Android Manager dependencies | Apache-2.0; Android Open Source Project |
| Haze 1.6.10 | Real backdrop capture and blur | Apache-2.0; Christopher Banes and Haze contributors |
| Rust standard library | Daemon runtime | Rust project licenses, MIT / Apache-2.0 |
| zlib | Gzip and raw DEFLATE validation | zlib license; Jean-loup Gailly, Mark Adler and contributors |

License texts are included in `LICENSES/`, embedded in Manager assets, and
carried in the recovery archives. Rust's MIT notice and the zlib notice are
preserved alongside Apache-2.0. Dependency artifacts
retain their embedded notices. SHA-256 uses the public FIPS 180-4 algorithm; its
implementation here is original. Android boot layouts follow the published
[AOSP header specification](https://source.android.com/docs/core/architecture/bootloader/boot-image-header).

Architecture references:

- [KernelSU](https://github.com/tiann/KernelSU): kernel/userspace/manager split,
  explicit authorization and enforcing SELinux credential handling.
- [ReSukiSU](https://github.com/ReSukiSU/ReSukiSU) and
  [SukiSU](https://github.com/SukiSU-Ultra/SukiSU-Ultra): source integration and
  independently attributable extensions to upstream components.
- [APatch](https://github.com/bmax121/APatch): distinction between actual kernel
  patching and attaching a userspace program to an unchanged kernel.
- [KOWSU](https://github.com/KOWX712/KernelSU): comparison reference named in the
  project brief; no code is incorporated.

Any future upstream incorporation must record the exact source revision, retain
copyright and license text, and provide corresponding source and build inputs.
Do not relabel copied GPL-2.0-only kernel code as GPL-3.0-only Manager code.
