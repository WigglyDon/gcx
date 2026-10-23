# Direct Dolphin integration

## Checked native save reads, 2026-10-22

ABI7 closes an additional coherence gap found after the ABI6 candidate: the native
raw-card constructor treated a failed open as absence and ignored the full-read
result; SRAM initialization fell back to defaults on read errors. Disk generation
hashes alone could therefore match while newly initialized native RAM contained
fallback bytes. A later forced write could replace existing data. The earlier
ABI6 candidates remain preserved as source-bound development artifacts; they are
not acceptance of this read-failure case.

An optional native image-read callback now delegates existing card/SRAM reads to
Rust. Rust distinguishes genuine absence from errors, owns path/canonical-profile
checks and verifies exact length/hash against the admitted generation. Supported
raw sizes are the GameCube powers of two from512KiB through16MiB, with68-byte SRAM;
unsupported/truncated files are retained, never resized or reformatted. Missing
read observations reject unexpected later file creation until a new admission.
The donor raw-card formatter runs only for genuine absence. Checked read failure
creates only an inert in-memory device, with no formatter or flush thread, and
marks that runtime for recovery. All profile persistence stays blocked through
shutdown, and the Rust host's paused-start protocol prevents guest execution.
SRAM's initialized
byte buffer avoids passing uninitialized native storage to Rust. Standalone
upstream file-read behavior remains the fallback when no hook is installed.

The existing native recovery getter is generalized as
`gcx_engine_recovery_required`; ABI7 requires the saved-image reader before init.
The Rust app uses paused boot for fresh starts as well as restore, then resumes a
healthy ready runtime. This is the same emulated boot/DSP-HLE/native backend route,
with an acknowledgment barrier before execution. Read causes remain visible and
retained. No new firmware, frontend, profile, shared framework or second runtime.

36 focused Rust tests pass. New read cases cover exact images, changed/truncated
bytes, bad SRAM length, missing→unexpected creation and dangling links without
replacing files. Actual native checkpoint tests now pass in seven fresh processes,
including failed card/SRAM read acknowledgments: no guest execution, no resume or
persistence, unchanged ordinary generations through teardown, and recovery after
fresh checked init with the fault removed. State/card-protocol/event/newer-state/
truncation/card-image cases pass again. Same-process A→B→A→B exact scene/card
isolation and the original CPU/pad/card failure/retry/teardown regressions pass.
These are original Null-video/audio fixtures, not gameplay or Android acceptance.
Host/ARM64 native builds and warnings-denied Rust checks pass. Hardware remains
unrun; all working/candidate APKs and actual card backups stay protected.

## Library and launcher follow-up, 2026-10-22

Latest prepared Rust UI/launch implementation:
`1a00fdcabd03ec488176661d9f0e363d0f72b251`. Private APK SHA256
`8764ee5d140980db9b4e4db91611e000a03b7738d77dbd43b333559ad56fedb9`
(15,640,199 bytes), matching Rust image
`76daff8293cf363f64537aca76dda42babca62662067c32e7517f97e60fdd7c7`
retained with corresponding source. ABI6 native image/BuildID and resources remain
unchanged from the checkpoint candidate below; no native rebuild was needed.
The library-only5d71213/407ee928 and checkpoint-only0e534b7/ce092d5e candidates
also remain preserved independently alongside working ABI5 builds and actual saves.

Original Rust builds the scrollable saved-game list, validates opaque GameIds,
returns to the current runtime and uses coherent checkpoint/join for another game.
A live recovery error keeps its RAM until explicit Stop/restart. The same path
handles canonical `machinerhythm-gcx://play/<GameId>` links, with cold malformed or
unknown links refusing fallback boot and pending game requests coalescing. The
singleTask Java activity forwards new intents mechanically; it supplies no game,
profile, core or path policy. Pin requests explicitly target this activity with
only an ID link and local label. No private game artwork is copied. Accepted
requests and later own-app OS pin observations are stored separately. Slow Binder
queries run on the Rust worker while paused/unloaded; no service or legacy install
broadcast is added. Unpin observations cannot remove games/saves. Physical UI,
launcher pin/confirmation/taps, switching and Android checkpoint resume are UNRUN.

All34 focused host tests, host/all-target native-feature and ARM64 warnings-denied
Clippy, formatting/diff/TOML/source checks pass. Actual mechanical Java compilation
and packaged JNI definitions match; Android omits the host-only fixture exports.
Native ABI/export/dependency closure, unchanged signer, zero permissions,
resource/native bytes and16KiB ELF/ZIP alignment checks pass. All65 preserved
corresponding-source checksums pass, with92 public source inputs. No phone queries,
lease polling, install or public binary release. Fold/siblings remain untouched.
The accepted picture and prepared API31 navigation policy are unchanged; the
Home-handle cause/fix remains unproven. Existing gameplay/card evidence stays bound
to the actual earlier device builds, never inferred from this package success.

## Rust-owned checkpoint candidate, 2026-10-22

New off-device work uses native ABI6 and private state ABI1 over the same pinned
Dolphin source. Prior ABI5 APK/source/symbol pairs, the accepted Wind Waker
candidate and all private card backups remain protected. This candidate has not
run on Android; earlier gameplay/audio/ordinary-save evidence does not qualify it.

Prepared checkpoint source: `0e534b7e087a0389ba744c71af620064f0ed298f`.
Private APK SHA256 `ce092d5e4dfee6c8a37872f7b6084e0b9ecc4cffb0aadb77139cb7a6ba0c8a80`
(15,591,047 bytes). Package/defined-export/dependency/signature/resource checks and
16 KiB ELF/ZIP alignment pass; unchanged separate signer and zero permissions.
All 65 preserved corresponding-source checksums pass, with 92 public source
inputs in the receipt. Matching ARM64 native image SHA256
`883d49b9cfc75629dc372aa916b986ce0c11a3139b374176be4042fbbddf9e74`,
BuildID `c337dd1497d506d1ba8d5e38d66ae0d4c82b6904`, and Rust image
`ef8d2cdca79f656f1a65acb35a558671b76737d71be1d790284022ff6545163d`
are retained. Maintained patch SHA256
`da8e8ee045be36030365435d4ae8223a511d4ba85e94d1ab72e06c014863be7f`.
The prepared API31 navigation policy is included; its Home-handle fix remains
unproven. Both older ABI5 candidates and actual newer card backups remain intact.

The targeted GPL changes add synchronous paused capture/load, paused boot and a
private serialization scope. The scope always preserves card command, status,
address, staged programming bytes and interrupt state, validates card/device
identity, and never restores card-image bytes. Its cookie/format is separate from
upstream state files; movie-active payloads are refused. CoreTiming events remain
native. Card completion callbacks complete interrupts/status rather than repeat
the original write. Rust validates the bound ordinary files before any guest
execution and again immediately before load. Failed partial deserialization
blocks all card/SRAM callbacks and resume through joined teardown. SRAM now uses
the same Rust atomic writer, including native shutdown, rather than a separate
truncating path. The upstream non-gcx SRAM path remains available.

Original Rust `checkpoint.rs` publishes an immutable content-addressed envelope,
with exact game/profile/media/disc, linked engine bytes, selected resources/config,
state ABI, payload length/hash and complete durable save-generation metadata.
Media is streamed from the selected descriptor without copying the image. The
256MiB payload ceiling is a gcx allocation bound; the actual original host fixture
payload is90,108,466bytes. This is not the PSX64MiB limit. Readback must pass before
atomic session-reference publication. Only a known previous successful generation
may be retired after acknowledgment; rejected files and errors remain retained.

`storage.rs` journals a monotonically increasing generation before changing card
or SRAM, then atomically writes/syncs the image and publishes clean metadata.
Interrupted intent is reconciled against actual files before a new native boot;
no rollback occurs. Existing ABI5 card/SRAM files are admitted before startup.
While native RAM is live, a callback cannot adopt and overwrite unexpected newer
files, including during pending-write retry. Even A→B→A identical image bytes
invalidate the earlier generation. The existing Android-proven ProfileLock stays
held through capture, publication, native join and final metadata. Successful
checkpoint departure rechecks the generation after teardown.

The Rust player captures on explicit Pause, focus/background/window departure,
selected-game departure and Activity destruction. It starts a restored runtime
paused, waits for startup/surface readiness, revalidates, loads, and only then
permits input/execution or displays the paused menu according to the latest
durable SavedSession intent. A failed or
incompatible restore retains the checkpoint/current saves and offers explicit
restart. Explicit Stop remains an ordinary-save unload; Open cold boots using
current saves. After durable explicit Stop, a known successful scene file may be
retired while the profile remains locked; rejected/unacknowledged files stay.
Process death before a successful checkpoint is not claimed safe
scene recovery: absent/stale state requires explicit restart. No periodic capture,
background game execution, service, second resident game or launcher shortcut is
added by this slice.

Focused host results:32 Rust tests pass. The actual native fixture, across five
fresh processes, restored CPU/register state and a pending completion event,
continued a partly staged card program into the expected11/22/33/44 bytes, and
rejected a newer save before startup. Truncated and card-image-restoring native
payloads failed closed, with card/SRAM unchanged through teardown and checkpoint
files retained. The second pass uses the same Rust synchronous capture/publish
callback as the Android app. Host/ARM64 native builds and warnings-denied Rust
checks pass. A subsequent optimized host-only fixture also passes same-process
A→B→A→B exact CPU/register checkpoint reentry using two distinct original programs,
GameIds, profiles and native card markers. Each resume checks the matching paused
scene before execution; other-profile card bytes stay unchanged. The real host
writer locks exclude another open through teardown and release afterward. This
adds native reentry evidence, not gameplay or Android continuity acceptance. Null video/audio are deliberate fixture choices: real Vulkan/audio
state, Android interruption timing, process-restart gameplay, A→B→A gameplay,
update retention, shortcuts and Fold remain unqualified. No phone queries/actions
are authorized while the owner asks for off-device continuation.

Reproduce using a NEW private fixture directory, never an owner profile:

```sh
cargo xtask source-check
cargo xtask build-host
GCX_ENGINE_LIB_DIR="$PWD/build/host/gcx" cargo build -p gcx --features native-engine --bin gcx-checkpoint-probe
LD_LIBRARY_PATH="$PWD/build/host/gcx" timeout 180 target/debug/gcx-checkpoint-probe NEW_PRIVATE_FIXTURE .local/source/dolphin/Data/Sys
# Separate bounded same-process reentry coverage (original fixtures only):
GCX_ENGINE_LIB_DIR="$PWD/build/host/gcx" cargo build --release -p gcx --features native-engine --bin gcx-checkpoint-probe
LD_LIBRARY_PATH="$PWD/build/host/gcx" timeout 90 target/release/gcx-checkpoint-probe ANOTHER_NEW_PRIVATE_FIXTURE .local/source/dolphin/Data/Sys --reentry
```

For a copied, offline private profile, a read-only host inspection mode uses the
same Rust envelope/generation validator without native boot or restore:

```sh
LD_LIBRARY_PATH="$PWD/build/host/gcx" target/release/gcx-checkpoint-probe --verify COPIED_PRIVATE_PROFILE CHECKPOINT_ID TRUSTED_BINDING_JSON
```

The expected Binding JSON must come from independent trusted source/media/profile
receipts, not merely from the checkpoint header being checked. It must match the
Android engine/resource/disc identities when inspecting an Android copy; the host
engine identity is not interchangeable. The tool rechecks current files after
payload verification and does not mutate the profile. Actual original-fixture
inspection passed, wrong engine binding was refused, and all file content hashes
remained unchanged. This is not a restore operation or authority to inspect a live
owner profile. Device test preparation remains private and requires a fresh window.

The host JIT test needs ordinary native execution; the restricted agent sandbox
traps it. No device/game/firmware input is used. `native/host_fixture.cpp` exposes
only CPU/card fixture operations and is excluded from Android. Application policy
and container/generation logic are Rust; native changes retain GPL provenance.
RustCrypto SHA-2560.10.9 and its exact transitive graph are pinned in Cargo.lock;
the packaging tool includes their supplied licenses and corresponding sources.


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

The Rust host adds Resume and Stop game actions to the existing
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
AudioTrack, card or DVD workers. The subsequent ABI 5 candidate also passed
Pause/Resume at Wind Waker's file menu.

One earlier intro run crashed on the Video thread while releasing an Android
window in Vulkan surface replacement. Exact crash frames and matching symbols
are retained privately; later successful Stop/reload does not clear this defect.
The owner also reported missing landscape presentation and requested adaptation
of the public PSX layout. The prepared repairs have now had a bounded Pixel 4 check.

The landscape candidate uses ABI 5: a native surface-owner deleter requests
Rust JVM attachment before releasing its Android window. Rust keeps android_main
attached too; the JNI crate detaches newly attached native workers at thread exit.
The Android 13 [BLAST callback owner](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-13.0.0_r8/core/jni/android_graphics_BLASTBufferQueue.cpp)
uses GetEnv without attaching in its callback destructor. This matches the
captured stack and motivates the repair. One actual Pixel 4 Home/foreground return
now passed with the same process and no observed crash; repeated transitions and
rotation remain unqualified. `cargo xtask check-render-surface` checks the real patched owner with
an isolated native-window stub, moving its last shared reference to a native
worker and verifying preparation before balanced release. It does not simulate
Android JNI or certify the framework fix.

The same candidate defaults to sensor landscape, with shared Rust density-aware
side rails adapted from the reviewed public PSX geometry. All 18 GameCube input
channels remain reachable; trigger travel and clicks are independent. Native
drawing uses original GameCube color/label cues and the engine's reported aspect,
with the complete picture high and clear of controls. An original placeholder
preview is generated by `cargo run -p gcx --example layout_preview`; it contains
no game assets. Pixel 4 landscape/colors were visually inspected and the owner
reports the controls work. This is broad feedback, not an independently measured
control matrix. System bars overlap the upper triggers and bottom picture;
insets/cutouts, rotation and Fold layout remain due. No snapshot/shortcut
implementation is added.

Prepared implementation `162d6b4e5995ad398384ff5ab4c944675e348a3d` produced
private APK SHA256 `be16792ac7cb8c10830c421fa66bf6ce6c3328acaca8dea6c9302216f3a848dd`.
ARM64 link, defined exports/dependency closure, signature, 16 KiB ELF/ZIP alignment,
actual resource/native bytes, zero requested permissions and the compiled
sensor-landscape manifest passed. The signer is unchanged. Its corresponding
source Git bundles/shallow boundaries/vendor/tool inputs are preserved; all 65
candidate checksums pass. Native BuildID is
`50270b219102675907804c0c771fd7983bb2f869`; matching symbols are retained.
Sixteen Rust tests, host/all-target and ARM64 warnings-denied Clippy, formatting
and the native surface-owner fixture pass. In the owner-authorized window ending
2026-10-22 18:29:55 UTC this candidate installed with the same signer over backed-up
gcx data. Cold launch stayed unloaded until Open. Wind Waker loaded its existing
quest; Pause/Resume and one Home/hot-return passed. The owner reported the layout
works and confirmed an in-game save finished. The observed world retained its
one-rupee marker. Normal Stop joined native workers, published clean metadata and
left no CPU/Video/AudioTrack/card/DVD threads. Before-update and final private card
backups validate and contain identical card bytes. No cold reload after this
candidate's final Stop was attempted. The successful ordinary progress round-trip above used
implementation `27931622c90203348947006c6a1d2cffc911cffa`, APK `05902ff6…`.
No new audio measurement was made. The final already-unloaded app was retired,
screen-off and keyguard state verified, and the exclusive lease released without
pending restoration. Fold, sibling projects, signer and earlier recovery data
were untouched. This is a bounded device result, not general lifecycle acceptance.

The next bounded continuation adds app-local immersive play and a single Rust
`PlayerView` geometry owner. Main-thread measurements include current window,
decor surface, content bounds, input origin, density and stable bar/cutout/mandatory
gesture clearance. Intersecting before mapping avoids double subtraction on
already-inset surfaces. Drawn controls, game picture and window-local native input
use the same geometry; changed/invalid/rotation-mismatched geometry releases input.
Stable bar space remains reserved during immersive play to protect transient
swipe reveal. API30+ uses the platform insets controller; API28/29 retains supported
immersive-sticky flags. Pause/dialog/picker navigation restores visible bars.
Activity identity and epoch checks retire old callbacks. Twenty host tests and
warnings-denied host/ARM64 Clippy pass. The native ABI5 engine and its symbols are
unchanged. Implementation `52e240d08f0efc0d902d80e478e8dbe31042668b` produced
private APK SHA256 `053ad7f653c062c9107206ed04f5f3fd627c997f34aa78265a276fe24b04a062`
(15,409,094 bytes). Package ABI/defined JNI exports, native dependency closure,
16 KiB ELF/ZIP alignment, unchanged signer, resource/native bytes and all 65
corresponding-source checksums pass. The exact Rust symbol image is retained.
Its Pixel4 window ended early at2026-10-22 18:54:56UTC on a concrete picture-size
usability failure. Hidden bars, the world/one-rupee marker, aligned Pause target,
visible menu navigation and normal joined Stop were observed; owner feedback
rejected the reduced picture. The actual final card backup matches protected
saved-progress bytes. Four captures were inspected; no crash or pending recovery.
The already-unloaded app was retired, screen-off/keyguard verified and lease
released before repair. Repeated Home/return, deliberate bar reveal, reverse
landscape and post-Stop cold reload were not continued on that candidate.

The correction selects the platform's title-free NoActionBar Activity theme before
window creation; the default application chrome had reduced the measured content
inside the taken native window. Rust also removes the dialog's empty title area.
Geometry still honors actual measured safe bounds and game aspect. Twenty-one
host tests, including content-height reclamation without repeated clearance,
and host/ARM64 warnings-denied Clippy pass. Implementation
`74b2ac4b938486ea6f1937b9f1b41c4ca6065096` produced private APK SHA256
`5aa4ee8082ee3682c78e1f029b00fd029277c7039c5805df2019a161018ecc69`;
package, unchanged signer, ABI/export/dependency, resource/native byte, 16KiB
alignment and all65 corresponding-source checksums pass. The actual compiled
manifest's theme ID resolves to platform Theme.Material.NoActionBar. Its actual
Pixel4 check ended at2026-10-22 19:06:29UTC: bars hid, existing world/one-rupee marker
rendered, aligned Pause and title-free dialog worked, and normal Stop joined with
clean metadata/no emulation workers. The owner rejected the inset picture and
clarified edge-to-edge like PSX. Save completion was confirmed; final actual card
backup matched protected progress, app retirement followed joinedStop, and
screen-off/keyguard/lease release were verified. No remaining lifecycle or
post-Stop cold-reload acceptance was attempted after the sizing rejection.

The new correction adapts the reviewed public PSX picture/control viewport split:
the picture uses the full taken drawable, including gesture-reserved pixels,
while controls retain their safe geometry. A centered clear-picture calculation
maximizes Dolphin's reported aspect around actual touch rectangles instead of
reserving entire side rails. Transient OS bars can overlay the stable picture;
they do not shrink it. Input and drawing still share one measured mapping. MIT
source/revision/coupling are in docs/PSX_TRANSFER.md; no fixed PS1 aspect or sibling
data/core is copied. Twenty-three host tests and host/ARM64 warnings-denied Clippy
pass, including full-height1.25 fixture, variable-aspect/non-overlap and bar-reveal
stability. The actual geometry placeholder was generated/opened, without game
artwork. Implementation `58335ffe726178d6346f69305af1772c32a3eb1e` produced
private APK SHA256 `f4f60fa0dfecaa13e1bf1d9852ccafb81f5f7d6b26b343c556025abb5eec178a`
(15,409,094bytes). Package/signature/ABI/defined JNI exports/dependency closure,
resource/native bytes, 16KiB ELF/ZIP alignment and all65 corresponding-source
checksums pass. Matching Rust image is retained; native engine remains unchanged.
The subsequent fresh Pixel4 window ended with verified return at2026-10-22
23:28:04UTC. The actual installed APK hash matches this candidate. The owner
accepted picture size and reported the other landscape/control/sound check good,
apart from a white Home gesture handle stuck across the picture centre after
bar reveal. That artifact is captured before and after cold relaunch. Scoped
app/navigation window geometry is retained; its cause is not established. Two
extra Home/return cycles were skipped. Pause/Resume and menu navigation worked.
The owner corrected an initial Save-finished response: the two-rupee play was
unsaved/disposable. No new in-game save is earned. Normal Stop joined, clean
metadata/no emulation workers were verified, and only then was the unloaded app
retired. A confirmed new process stayed unloaded until explicit Open. The earlier
one-rupee quest loaded and the owner/capture confirmed it. Final normal Stop again
joined with clean metadata; actual final card/SRAM backups were retained
separately from earlier backups, with no rollback of the changed final card.
Eight minimized captures were opened. Physical screenOFF/nonsecure keyguard,
client retirement, foreground exit0 and lease release were verified; no pending
recovery. Fold and siblings were untouched. This earns current-build existing-card
cold reload, not a new-save round trip or automatic scene resume.

The next bounded Rust app-window candidate follows public PSX's API31+
BEHAVIOR_DEFAULT choice, keeping navigation gestures active while bars are hidden;
API30 retains transient-bars behavior. Shared geometry, native ABI5 and engine
objects remain unchanged. This policy adjustment is not a proven cause/fix of the
handle artifact and needs a new authorized device window after package checks.
Implementation `0176fa894e095d0bf230e3fa0a36c12c2869ccc2` produces private APK
SHA256 `50a7430407b54d095e06ba1b8483894f2b595c515982483fef1fb1f5ed698231`
(15,409,094bytes). The23 existing host tests, host/all-target and ARM64 warnings-denied
Clippy, formatting/diff, actual package ABI/defined exports/dependency closure,
signature/resource-native bytes/16KiB alignment and all65 corresponding-source
checksums pass. Matching Rust image SHA256
`d0e0813c336edc0a9f1e5781d112a2d7c91e370c93e69f022dea0f34f8f6fcd2`
is retained. No native rebuild and no device test of this candidate yet.

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
  excluded from this initial route. Android linking and the bounded Pixel 4
  gameplay/ordinary-save/lifecycle paths above passed.
- `InputCommon/ControllerInterface/Touch/InputOverrider.*`: 18 GameCube
  channels include independent L/R analog and digital signals. The adapter
  applies a complete Rust-owned pad under the controller's state mutex, avoiding racing
  the override array with native polling. Native overrides are removed during
  teardown. Rust touch tests cover simultaneous pointers, partial travel, release
  and bounded stick motion. Android interruption/controls proof remains due.
- `Core/HW/EXI/EXI_DeviceMemoryCard.cpp:DoState`: normal states omit card
  contents and command/protocol state; movie states serialize both, and the stored flag controls loading.
  Both paths must be considered before accepting checkpoints from outside gcx.
- `GCMemcardRaw.cpp`: a 15-second flush worker writes raw bytes; destruction
  triggers a final write and joins. Upstream lacks the durable-flush acknowledgment
  added by the maintained raw-card patch below. `GCMemcardDirectory.cpp` similarly
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

The isolated host checks alone do not prove Android filesystem behavior or real
game saving; the separately recorded Pixel 4 ordinary-save/Stop/cold-reload and
live lock observations above provide their bounded device evidence. The
GCI implementation's early dirty clearing and direct truncation are not selected.
Ordinary saves and machine checkpoints remain separate. The earlier ABI5 candidate
exposes no snapshot API; the new ABI6 candidate above adds a private format.
Movie states can restore card bytes/metadata and normal states omit card protocol;
restored CoreTiming events and pending transactions still require qualification.
Future restore must verify profile generation and exact core/resource/media binding
before touching native state. A two-process Android test of the same Rust
`ProfileLock` passed in gcx's private directory through stock debug `run-as`:
contender refused, then acquisition succeeded after release. This is primitive/
path evidence. The later live-game observer proved exclusion across native join
and acquisition after durable Stop publication, as recorded above.

SavedSession records a durable incomplete-runtime marker before native startup.
Clean card acknowledgment and joined teardown clear it; a later launch retains
an interrupted runtime's error and requires explicit restart with current saves.
This prevents an automatic startup loop after a future crash. The added metadata
recovery test and actual Pixel 4 interrupted-error/explicit-restart scenario pass.
No machine state is
restored or synthesized by that marker.

### Off-device checkpoint audit, 2026-10-22

Inspected the actual Dolphin pin `cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b`
with maintained patch SHA256
`92735cf12f0827b71783eab6280ac6221bd30b2ff387f0a3305220e303d93b0f`.
This is source inspection, not a state round-trip test. It leaves ABI5 and the
preserved full-drawable APK unchanged. No state capture/load API is exposed.
Paths below are relative to Dolphin's `Source/Core`.

| Actual owner | Observed behavior and consequence |
| --- | --- |
| `Core/HW/EXI/EXI_DeviceMemoryCard.cpp:DoState` | Normal movie-inactive capture writes a false flag and omits the interrupt flags, command, status, position, 128-byte programming buffer, address and card image. Load reads the flag from the payload, so disabling movies in the host does not prevent an externally supplied true flag restoring card bytes. |
| Same file: `SetCS`, `DMAWrite`, `CmdDone`, `TransferComplete` | Erase/page-program writes happen synchronously at deselection; DMA writes happen before scheduling completion. The two delayed completion handlers change status/interrupts, not card bytes. Restored timing callbacks alone do not repeat those writes. However, omitted staged command bytes and restored guest execution can still cause incoherent or stale writes. A CPU pause plus raw-image flush does not finish or serialize a partially received command. |
| `Core/CoreTiming.cpp:DoState`; `Core/State.cpp:DoState` | Timing serializes pending events by name before hardware restores. Card events are registered during boot. Queue/protocol coherence is required; do not discard events as a substitute for capturing the device. |
| `Core/HW/GCMemcard/GCMemcardRaw.cpp:DoState`, `CommitSnapshot` | The movie path replaces the raw image/size/slot in RAM without marking it dirty. Our explicit forced barrier can subsequently commit those older bytes regardless of dirty status. Reject incompatible/stale state before native mutation; never fix it by copying an older card onto disk. |
| `Core/HW/EXI/EXI_DeviceIPL.cpp:DoState`; `EXI.cpp:Shutdown`; `EXI_Channel.cpp:DoState` | Even a normal state restores SRAM and channel/header/device identity. SRAM is written separately at shutdown through upstream IOFile, outside the Rust card barrier. The GCI-folder header-change path removes/reconnects a card to avoid stale writes; that safeguard does not apply to our raw-card mode. Console/card identity and durable SRAM handling need qualification too. |
| `Core/State.cpp:SaveAs`, `CompressAndDumpState` | Save schedules CPU capture and a compression/file worker and returns void. Worker completion/flush hooks do not return durable success. The file path reports write/close errors to OSD, can proceed to publication after an earlier error, and uses close/rename without file or directory sync in this path. It cannot serve as gcx's checkpoint commit acknowledgment. |
| `Core/State.cpp:LoadAs`, `LoadAsFromCore`, `ValidateHeaders` | Load returns void; the after-load callback also runs after missing/failed input. Deserialization can partially mutate the machine and attempts an undo on failure. State version192 is checked, but an exact revision string is deliberately not required. Neither the callback nor the six-byte game ID proves successful compatible gcx restore. |

The next native checkpoint work must address the actual card protocol owner:
capture/restore protocol independently of optional card bytes, or prove a complete
guest transaction boundary. Do not enable movie mode as a shortcut. Any changed
serialization needs a distinct gcx state ABI and rejection of prior payloads.
Use a completion-bearing native capture/load operation; Rust should own bounded
payload validation, durable publication and the checkpoint/card generation gate.
The already-held profile lock must cover capture, publication, validation, load
and teardown. Compare current durable card generation/digest before native load,
not after it; bind SRAM identity, media/disc, resources and exact engine/patch ABI.
On mismatch or load failure, retain the checkpoint and error and offer explicit
restart with current ordinary saves. A partial load must remain paused and must
not flush mutated card/SRAM over current progress during recovery teardown.

Before automatic resume, use isolated original fixtures to interrupt a page
program, retain pending completion events, reject a deliberately newer card,
exercise a movie-card payload, and inject capture/publication/load failures.
No such machine-state fixture or hardware resume result is claimed by this audit.

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
Java trampoline to DEX. The staged and actual APK native/resource byte audits
passed for the retained candidates. Do not copy the entire donor Sys directory into a shipping package by
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

The retained candidates earned the scoped Pixel 4 installation/Vulkan/audio,
ordinary-save/reload and lifecycle evidence at the top of this document. The new
fullscreen/inset continuation still needs a fresh exclusive Pixel 4 window after
packaging/source checks. A-to-B-to-A, shortcuts, folds and measured speed remain
separate acceptance checks.
