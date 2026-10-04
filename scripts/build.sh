#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"
case ${1:-host} in
 host)
  cargo fmt --check
  cargo clippy --locked --all-targets -- -D warnings
  cargo test --locked
  python3 -m unittest discover -s tests -v
  ;;
 daemon) cargo build --locked --release --target "${KILASU_TARGET:-aarch64-linux-android}" ;;
 manager) cd manager; ./gradlew assembleDebug testDebugUnitTest lintDebug ;;
 kernel)
  : "${KERNEL_TREE:?Set KERNEL_TREE to an ACK source tree}"
  "$root/scripts/integrate.sh" "$KERNEL_TREE"
  out=${KERNEL_OUT:-$KERNEL_TREE/out-kilasu}
  make -C "$KERNEL_TREE" O="$out" ARCH=arm64 LLVM=1 gki_defconfig
  "$KERNEL_TREE/scripts/config" --file "$out/.config" --enable KILASU --enable KILASU_SELINUX_INTERNAL
  make -C "$KERNEL_TREE" O="$out" ARCH=arm64 LLVM=1 olddefconfig
  make -C "$KERNEL_TREE" O="$out" ARCH=arm64 LLVM=1 -j"${JOBS:-$(nproc)}" Image
  ;;
 *) printf 'usage: build.sh host|daemon|manager|kernel\n' >&2; exit 2;;
esac
