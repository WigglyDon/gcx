# Direct Dolphin integration

The off-device Rust CPU/lifecycle probe has passed with normal process exit on
Linux, both with null audio and an intentionally unavailable audio backend. It
boots an original 20-byte PowerPC program in a correctly padded DOL, observes its
RAM write through the actual engine, checks native GameCube pad values, commits
an ordinary raw-card image, neutralizes input and completes teardown before
releasing its profile lock. Null video/audio are intentional; this is not visible
gameplay, audible sound or an in-game save/reload result.

The Android candidate has a Rust-owned document picker, host loop, native control
geometry/input and recovery policy. Its small Java trampoline only forwards
framework events. Native Vulkan and Cubeb are selected; the ARM64 engine build is
linked successfully. The private development APK passes ABI/export, 16 KiB ELF/APK
alignment, signature, manifest, native-library and resource byte checks. Device
installation and compatible same-signer updates passed on the secondary stock
Pixel 4 (Android 13/API 33, 4 KiB pages). The Rust-built chooser is visible in its
own dialog window; the NativeActivity root hierarchy accepted views without
painting them. The Java trampoline explicitly associates the native library
with its class loader and forwards destruction for Rust-owned dialog retirement.
Scoped Wind Waker RVZ selection now reaches its rendered title, intro and file
menu through native Vulkan. Start, Control Stick and A work; the owner confirmed
audible sound. The game reported ordinary Slot A file creation; its 16 MiB native
raw card with a 12-block game directory entry is privately backed up. The owner subsequently played into the world and saved in-game. Normal Stop and
a confirmed cold-process game-menu load retained the visible one-rupee marker.
Fold results and the full control matrix remain unqualified.
Private diagnostics/data and each
tested APK/source pair are retained; each Pixel 4 window returned screen-off and
released before off-device work.

The CPU-thread startup path constructs `DolphinAnalytics`, whose Android metadata
builder called an empty `std::function` normally installed by Dolphin's managed
frontend. Reporting being disabled does not skip that builder. A focused host
regression using the actual Android analytics translation-unit branch reproduced
`bad_function_call`; guarding the optional provider at its owner lets the same
Rust CPU/input/card/teardown fixture finish successfully. The same repair now
passes actual Pixel 4 boot through the title and game menus. The host regression
alone is not Android execution evidence.

A second process is refused by the real live game's SaveProfile writer lock.
Android's root Back action backgrounds the activity and retains the paused runtime
and lock. Acquisition succeeds after the gcx process is explicitly stopped; that
does not establish exclusion through joined native teardown. The interrupted
runtime marker remains and requires explicit restart with current card data at
the next launch. No machine snapshot or automatic resume is involved.

The next candidate adds Rust-owned Resume and Stop game actions to the existing
dialog. Stop warns that unsaved play is lost; committed ordinary card data remains.
The profile/card binding outlives the pause/barrier, joined native shutdown and
metadata publication. A failed barrier keeps the paused runtime available for
retry. Later failure records interrupted recovery. A durable `stopped` flag keeps
the game unloaded until Open, preserving its identities and document access.
Four focused stop tests pass, including actual profile-lock exclusion during join
and publication. Pixel 4 normal Stop passed twice. An actual competing Android
writer observed 313 busy samples spanning the native shutdown interval and
acquired only after join; a subsequent acquisition passed. The cold process stayed
unloaded until explicit Open. Final app-scoped inspection found no CPU, Video,
AudioTrack, card or DVD workers. Resume itself remains unqualified.

One earlier intro run crashed on the Video thread while releasing an Android
window in Vulkan surface replacement. Exact crash frames and matching symbols
are retained privately; later successful Stop/reload does not clear this defect.
The owner also reported missing landscape presentation and requested adaptation
of the public PSX layout. Repairs are prepared off-device before further custody.

The next source candidate uses ABI 5: a native surface-owner deleter requests
Rust JVM attachment before releasing its Android window. Rust keeps android_main
attached too; the JNI crate detaches newly attached native workers at thread exit.
The Android 13 [BLAST callback owner](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-13.0.0_r8/core/jni/android_graphics_BLASTBufferQueue.cpp)
uses GetEnv without attaching in its callback destructor. This matches the
captured stack and motivates the repair; actual device crash recovery remains
unverified. `cargo xtask check-render-surface` checks the real patched owner with
an isolated native-window stub, moving its last shared reference to a native
worker and verifying preparation before balanced release. It does not simulate
Android JNI or certify the framework fix.

The same candidate defaults to sensor landscape, with shared Rust density-aware
side rails adapted from the reviewed public PSX geometry. All 18 GameCube input
channels remain reachable; trigger travel and clicks are independent. Native
drawing uses original GameCube color/label cues and the engine's reported aspect,
with the complete picture high and clear of controls. An original placeholder
preview is generated by `cargo run -p gcx --example layout_preview`; it contains
no game assets. Device layout, rotation, insets/cutouts and input usability still
require a fresh exclusive window. No snapshot/shortcut implementation is added.

## Source and reproduction

Official source: `https://github.com/dolphin-emu/dolphin.git`, revision
`cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b`, root tree
`be54b87e1784a9a26bf406a5cc222c642fb13db0`. Resources are bound by its
`Data/Sys` tree `899ac7b90a5bf93cfd0c1208a920415fe49b7ba0`.

After checking current host capacity and competing work, create exactly one
checkout at `.local/source/dolphin` using `git init`, add the official origin,
fetch the exact revision with `--depth 1`, and check it out detached. Apply
`native/patches/0001-direct-adapter.patch` with `git apply`. Do not substitute
today's upstream HEAD. Then run:

```sh
cargo xtask fetch-dependencies
cargo xtask source-check
cargo xtask configure-host
cargo xtask build-host
GCX_ENGINE_LIB_DIR="$PWD/build/host/gcx" cargo build -p gcx --features native-engine
LD_LIBRARY_PATH="$PWD/build/host/gcx" target/debug/gcx-probe \
  "$PWD/.local/probe-unique" "$PWD/.local/source/dolphin/Data/Sys"
```

Use a new probe directory each time. `--unavailable-audio` additionally exercises
failed native audio initialization and clean process exit in the host configuration.
`--reentry` boots isolated A/B/A profiles in one process and checks a marker in
A's ordinary raw card survives without appearing in B. `--card-retry` blocks only
a new fixture's candidate path with a directory, checks failure reporting and
retention of its prior card/RAM, removes that obstruction and verifies retry.
Both checks passed. Neither is an in-game save or machine-resume test.
`cargo xtask check-android-analytics` repeats the absent-provider regression after
the normal host engine and Rust probe are built. It compiles only the real
analytics translation unit with its Android branch, using preincluded host window
declarations, and links it ahead of the normal core archive into an isolated
fixture library. The ordinary host build is unchanged. Compile/link/probe logs
remain in a new private `.local/android-analytics-*` directory. This bounded host
branch test needs normal native execution; it does not emulate Android or ARM64.
The restricted runner trapped during native setup; the successful probes used
normal host execution. Failures and profiles are retained. Do not
point the probe at any existing saves. The source check binds the complete root
gitlink list in `dolphin-gitlinks.tsv`, verifies each selected dependency in
`dependencies.txt`, and rejects unreviewed tracked engine/dependency edits.
Excluded gitlinks remain pinned in the root graph but are not build inputs.

The host configuration bundles the donor libraries instead of automatically
selecting system copies. Native system dependencies and the compiler/sysroot
remain platform inputs, not a claim of bit-identical builds on arbitrary hosts.
Initial host tools: Rust 1.96.1, Clang 22.1.8, CMake 4.3.0 and Ninja. Compilation
uses one job without LTO or debug information. No source archive is needed.

## Inspected ownership

- `Core/Core.h` and `Core.cpp`: Boot starts donor execution threads; the Rust
  host pumps `HostDispatchJobs`. `CPUThreadGuard` pauses/locks CPU, DSP and FIFO
  before temporary host access. `Stop` and `Shutdown` precede UI/controller
  teardown. Callbacks remain installed until those workers have retired.
- Android `jni/MainAndroid.cpp`: uses `WindowSystemInfo::Android`, native
  `ANativeWindow` surfaces and `Presenter::ChangeSurface`. It is a reference,
  not gcx's frontend. Its JNI helpers also serve core file access, controller
  discovery, USB adapters and OpenSL audio configuration. The maintained Android
  graph excludes that JNI frontend and uses Rust document callbacks, emulated pad
  overrides and native Cubeb. Hardware USB pads/custom drivers/microphone/Wii are
  excluded from this initial route. Android linking passed; execution is unrun.
- `InputCommon/ControllerInterface/Touch/InputOverrider.*`: 18 GameCube
  channels include independent L/R analog and digital signals. The adapter
  applies a complete Rust-owned pad under the controller's state mutex, avoiding racing
  the override array with native polling. Native overrides are removed during
  teardown. Rust touch tests cover simultaneous pointers, partial travel, release
  and bounded stick motion. Android interruption/controls proof remains due.
- `Core/HW/EXI/EXI_DeviceMemoryCard.cpp:DoState`: normal states omit card
  contents; movie states serialize them, and the stored flag controls loading.
  Both paths must be considered before accepting checkpoints from outside gcx.
- `GCMemcardRaw.cpp`: a 15-second flush worker writes raw bytes; destruction
  triggers a final write and joins. The write path does not expose a successful
  durable-flush acknowledgment to this host. `GCMemcardDirectory.cpp` similarly
  has delayed GCI flushing, final destructor flushing, cached metadata and state
  serialization. Neither `State::SaveAs` nor a file timestamp proves card
  durability. No snapshot API is exposed by this adapter yet.

The initial mode is native raw card A, with no card B. The maintained MemoryCard
write hook hands a complete native image to Rust's atomic candidate writer. Rust
checks the owned profile path, writes/syncs, renames and syncs the parent directory;
the native worker retains dirty status on failure. `FlushToStorage` serializes with
the worker and snapshots under its data mutex; the adapter calls its barrier under
a CPU thread guard. Explicit restart/game change retains the old runtime if that
barrier fails. Departure stays paused from the barrier through native teardown.
The worker skips a redundant unchanged write after an acknowledged image; dirty
testing and acknowledgment are serialized under its commit mutex. The host
fixture blocks candidate writes during joined teardown to prove that behavior.
Explicit barriers still force the requested durable write and report failures.
An isolated Rust fault test verifies a failed candidate leaves the prior card
untouched.

These checks do not prove Android filesystem behavior or real game saving. The
GCI implementation's early dirty clearing and direct truncation are not selected.
Ordinary saves and machine checkpoints remain separate: no snapshot API is exposed.
Movie states can restore card bytes/metadata and normal states omit card protocol;
restored CoreTiming events and pending transactions still require qualification.
Future restore must verify profile generation and exact core/resource/media binding
before touching native state. A two-process Android test of the same Rust
`ProfileLock` passed in gcx's private directory through stock debug `run-as`:
contender refused, then acquisition succeeded after release. This is primitive/
path evidence; exclusion through native-game teardown still needs qualification.

SavedSession records a durable incomplete-runtime marker before native startup.
Clean card acknowledgment and joined teardown clear it; a later launch retains
an interrupted runtime's error and requires explicit restart with current saves.
This prevents an automatic startup loop after a future crash. The added metadata
recovery test passes; the device scenario remains unrun. No machine state is
restored or synthesized by that marker.

## Resource and license handling

`bridge.cpp` adapts Dolphin's Host definitions and is GPL-2.0-or-later with
upstream attribution. `COPYING` is retained verbatim. Rust host/input/build code
is original MIT work. The combined program retains all applicable obligations;
this ABI is not license separation. Keep the upstream revision, patches, selected
dependency sources/licenses, compiler inputs and resource inventory with each
candidate. Public binary distribution remains separately gated.

The observed replacement GC fonts identify Droid Sans, Google copyright and
Apache-2.0 in `Data/Sys/GC/font-licenses.txt`, with the generator at
`docs/gc-font-tool.cpp`. DSP replacement sources are under `docs/DSP/free_dsp_rom`;
the first player route uses DSP HLE. `cargo xtask prepare-android` stages the reviewed
allowlist, source/dependency notices, resource hashes, and compiles the mechanical
Java trampoline to DEX. This passed; actual APK resources remain to audit. Do not copy the entire donor Sys directory into a shipping package by
default, or bundle original IPL/game/startup assets.

## Android reproduction

Use the existing licensed SDK without changing shared caches. Bind `ANDROID_HOME`
to build-tools 37.0.0/platform android-37.0 and `ANDROID_NDK_HOME` to NDK
30.0.16248370. The configuration selects arm64-v8a, API 28 and libc++_shared.
The installed Java module compiler is used directly; no Gradle frontend or managed
application layer is imported.

```sh
cargo xtask configure-android
cargo xtask build-android
cargo xtask prepare-android
cargo xtask package-android
```

Native compilation uses one job. The package command requires the built engine;
it builds the Rust library, checks ARM64/16 KiB segments and entry points, stages
runtime libraries, aligns and signs with a separate private gcx development key,
and verifies the resulting signature/manifest/assets. The signing identity and
backup stay under `.local`, outside public source. Do not delete an existing signer
or replace it after a partial failure. Packaging passed. Packaged libraries are
stripped copies; the exact unstripped engine is copied into private hash-addressed
symbol storage before packaging so later links cannot replace its debug evidence. The
development APK is about 15 MiB; this is not a performance or runtime memory claim.

After committing reviewed source and rebuilding the package, run
`cargo xtask preserve-candidate`. It refuses changed source and preserves the APK,
tool/source inventories, every selected Git source bundle, shallow boundaries,
locked Cargo vendor archive and reconstruction instructions under one private
APK-digest directory. Each exported repository is checked in a temporary bare
repository with its shallow boundary. Public binary distribution remains gated.
See [source reconstruction](CORRESPONDING_SOURCE.txt).

The `gcx-lock-probe` Rust binary uses the product's actual `ProfileLock` and offers
bounded `hold`, `expect-busy` and `expect-free` modes. Its ARM64 build is prepared
for stock-device qualification. The isolated app-owned two-process fixture passed
on Pixel 4; the later live-game observer also proved exclusion across joined
native teardown and acquisition afterward.

This private development manifest is debuggable so the owner can recover and
verify gcx's own private data with stock `run-as` before an update. It disables
automatic Android backup and requests no permissions. A release manifest and
public binary remain separately gated; this is not a store-ready artifact.

A later, fresh exclusive phone window is required for installation, actual Vulkan
picture, audible sound, touch/interruptions and an in-game save/reload. Resume,
A-to-B-to-A, shortcuts, updates, folds and speed are separate acceptance checks.
