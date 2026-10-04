# Building and signing

Host: Rust 1.75+, Cargo, C11 compiler, zlib headers/library, Python 3.11+, make.
Android: Java 17, Android SDK platform/build-tools 35, NDK 27.2.12479018,
CMake 3.22.1, Gradle wrapper 8.11.1. Manager uses Kotlin 2.1.20, Compose and Haze
1.6.10. Dependencies are version-pinned; the Rust workspace has no crates.io
dependencies and can run host tests offline.

```sh
./scripts/build.sh host
cd manager && ./gradlew assembleDebug testDebugUnitTest lintDebug
```

For daemon cross-build, install Rust target aarch64-linux-android and set
CC_aarch64_linux_android, AR_aarch64_linux_android and
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER to the NDK API31 clang/llvm-ar paths.
Then run scripts/build.sh daemon. The NDK's zlib and libc satisfy native bindings.

For recovery, build on an arm64 Linux host:

```sh
gcc -std=c11 -O2 -static -Wl,-z,max-page-size=16384 \
 bootpatch/src/main.c bootpatch/src/kila_boot.c -lz -o kila-boot
./scripts/package.sh --version v0.1.0 --apk app.apk \
 --daemon-dir target/aarch64-linux-android/release \
 --validator kila-boot --output dist
```

Production APKs require a persistent private keystore. Generate it outside the
repository with keytool and retain it securely. Never commit keys/passwords.
The release workflow requires GitHub secrets KILASU_KEYSTORE_BASE64,
KILASU_STORE_PASSWORD, KILASU_KEY_PASSWORD and KILASU_KEY_ALIAS. Manager Gradle
reads these into signingConfigs.release; debug keys are not stable release keys.
The private key is decoded only into the runner's temporary directory.

Before tagging vX.Y.Z, update Cargo/Manager/kernel versions and CHANGELOG, run
all checks, and record real device checks in docs/validation.json. The release
gate rejects mismatched or unvalidated stable tags. Successful signed releases
publish APK aliases, Installer/Uninstaller ZIPs and checksums.txt with release
notes. No signing secret was supplied with the project request, so none is
fabricated or committed.

## Kernel authorization tests

On a development kernel enable `CONFIG_KUNIT=y`, `CONFIG_COMPILE_TEST=y` and
`CONFIG_KILASU_KUNIT_TEST=y`. The `kilasu-allowlist` KUnit suite exercises the
real allowlist functions: default denial, allow/revoke, once exhaustion and
invalid profiles. It does not grant credentials. The ACK CI compiles the suite;
running it requires booting that test kernel and inspecting its KTAP output.
Keep `CONFIG_KILASU_KUNIT_TEST=n` in distributed device kernels.
