# ntx native adapters

GameCube continues through the direct Dolphin adapter; it is not libretro.
Dolphin is pinned at `cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b` with the
reviewed [maintained patch](patches/0001-direct-adapter.patch), dependency
gitlinks and selected resources. Adapter symbols/library are `ntx_engine`.
ABI8/stateABI1 retain their structural contracts; the renamed binary and resource
hashes must be measured after a new build. Old snapshots never inherit them.

The original patch is retained under `patches/history/`. Detailed prior
[native evidence and boundary audits](../docs/history/GCX_NATIVE.md) retain their
original build limits. None establishes the new package's device behavior.

Read [operations](../docs/OPERATIONS.md) before growing builds. Current commands
use fresh project-owned output directories after relocation, preserving the old
CMake caches and binaries. Use the existing licensed SDK, platform android-37.0,
build-tools 37.0.0, NDK 30.0.16248370 and Rust 1.96.1. Bind `ANDROID_HOME` and
`ANDROID_NDK_HOME` to their actual available locations; no old absolute prefix.

```sh
cargo xtask source-check
cargo xtask check-render-surface
# Only after current capacity/concurrency preparation:
cargo xtask configure-host
cargo xtask build-host
NTX_ENGINE_LIB_DIR="$PWD/build/ntx-host/ntx" cargo build -p ntx --features native-engine
# Android is a separate admitted build phase:
cargo xtask configure-android
cargo xtask build-android
cargo xtask prepare-android
cargo xtask check-signer
cargo xtask package-android
```

The build uses one native job. `package-android` requires the new engine, checks
native exports/JNI, ELF/16 KiB alignment, notices and manifest, and preserves
unstripped symbols. The existing signer must match its independent recovery pair
and retained installed-candidate certificate. No key-generation fallback exists. Independent source reproducers can instead
use `cargo xtask package-android-unsigned`, which requires no owner signing
material and makes no signature or installation claim.
Legacy alias/certificate identity is immutable provenance. Canonical private
signer filename is ntx-development.p12; originals and recovery stay retained.

After committing reviewed source and building, `cargo xtask preserve-candidate`
binds the private APK to corresponding source; it refuses changed source.
[Reconstruction](CORRESPONDING_SOURCE.txt), [resource audit](RESOURCE_AUDIT.md)
and [distribution](../docs/DISTRIBUTION.md) remain required. Source reproduction
and private development signing are separate from public binary release.

N64 source pin and inspected interfaces are recorded in
[mupen/source-lock.json](mupen/source-lock.json) and [integration notes](mupen/README.md).
Core packaging, Rust libretro execution, GLES/audio lifecycle and full selected
component audit remain unfinished. No N64 playable or resume claim is made.
