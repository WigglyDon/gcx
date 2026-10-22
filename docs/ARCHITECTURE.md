# Architecture contracts

Status: implementation in progress; these remain acceptance contracts.
Use the smallest boundaries justified by the executable player. Do not freeze
an invented engine API or crate graph before inspecting Dolphin's actual owners.

## Product, platform and emulator

Shared Rust owns the local collection, durable identities, launch requests,
pause intent, input policy, UI geometry and persistence coordination. Platform
adapters own document grants, lifecycle, native windows, audio-device delivery,
shortcuts and accessible platform semantics. The borrowed engine owns console
execution, timing, its CPU/GPU/DSP/device state and legitimate internal threads.

Inspect Dolphin's Android native integration, Core/Host/Boot/State entrypoints,
controller input override boundary, renderer surface lifecycle and completion
callbacks. A direct maintained C ABI is the first integration approach to evaluate.
Libretro is not required. Retain C++ where valuable; write new substantive logic
in Rust and make small C++/build/ABI changes when that is the coherent solution.
Dolphin's existing managed frontend is a reference, not our application layer.

Do not drive a CPU/render worker from the UI thread arbitrarily. One session can
own multiple donor threads without being multiple user game sessions. Pause,
quiesce, flush, serialize, resume and shutdown must acknowledge actual completion.
A C callback pointer cannot outlive its session; no unwinding across the ABI.
Validate handles, input sizes and errors. Package checks must verify actual defined
C exports, not only successful linking or wrapper declarations.

Candidate Rust responsibilities: gcx-player, a small engine adapter, gcx-android,
and xtask. Names are not mandatory crates. Reuse upstream CMake/NDK and existing
Rust bindings. No reimplementation of third-party build tools for language purity.

The current workspace uses `gcx` for input, identity, locking and atomic storage,
`gcx-android` for the player/platform policy, and `xtask` for reproduction. A small
GPL native adapter calls the inspected Dolphin owners. An original CPU fixture
has passed on Linux, including native pad readback and card commit. A secondary
Android device passed the private-directory lock contention fixture, chooser and
scoped RVZ selection. Wind Waker renders its title, intro and file menu, responds
to Start/Control Stick/A and has owner-confirmed audible sound. The game-created
ordinary card file is privately backed up. The owner has played into the interactive world, saved in-game and loaded the
one-rupee marker after normal Stop and a confirmed cold process. Android lock
contention samples span native teardown; acquisition succeeds after joined workers
and metadata publication. A native Android surface-replacement crash remains open.
The Android-only analytics metadata provider was invoked while empty even with
reporting disabled. A guard at that owner passes the host regression of the actual
Android branch and the secondary Android device's boot test.

Before starting native execution, SavedSession durably marks that runtime as
incomplete. A clean card barrier followed by joined teardown clears the marker.
After interruption, retain the error and require explicit restart with current
saves. Departure stays paused across the barrier and teardown, and the native
writer skips a redundant unchanged write after its successful acknowledgment.
The host failure/retry fixtures and actual Pixel4 interrupted-error recovery and
normal joined Stop provide separate evidence; fault injection remains isolated.

The Rust pause control opens a small Resume / Stop game dialog. Stop explicitly
warns that play since the game's own last save is lost; it creates no snapshot.
Its owner pauses/quiesces and neutralizes, acknowledges the native card barrier,
joins native workers, publishes clean or recovery metadata while retaining the
profile lock, then releases ownership. Barrier failure retains the paused runtime
for Stop retry. Menu generations and runtime epochs reject stale actions.
A backward-compatible saved `stopped` flag prevents unintended boot after Stop,
including in a fresh process; Open retains the same GameId/profile/document grant.
Focused host coverage includes failed barrier/retry, duplicate/stale Stop, metadata
failure and lock ordering. Pixel 4 normal Stop passed twice, with visible unload, clean metadata and no
automatic boot in a cold process. The ordinary progress reload passed on2793162;
the ABI5 candidate162d6b4 subsequently passed Pause/Resume and one Home/return.
The complete input matrix remains unqualified.

Rust owns a single measured presentation/input geometry. Android supplies current
window, taken decor surface and content rectangles, density, stable system-bar,
cutout and mandatory-gesture clearance. Control intersections occur in screen
coordinates before mapping to buffer coordinates, so already inset surfaces are
not inset twice. Native motion maps to those same presented targets. The picture
independently uses the complete taken drawable/window intersection, including
pixels reserved only for gestures. PSX's reviewed public clear-picture policy is
adapted to maximize the complete picture around actual control targets using
Dolphin's reported aspect. It does not reserve entire side rails or shrink for
transient bars. The OS can overlay its temporarily revealed bars on that stable
picture; controls retain clearance. A changed/invalid measurement or mismatched
rotation releases held input; the game is retained.
App-local API30+ WindowInsetsController (API28/29 immersive-sticky fallback) hides
bars only while playing; menus/picker restore ordinary navigation. Main-thread
callbacks are checked against the bound Activity; posted work also checks its
epoch. Candidate52e240d passed hidden bars, world rendering and the aligned Pause
target on Pixel4; its picture was too small for owner acceptance. The pass ended
with normal joined Stop, matching protected card backups and verified return.
The title-free74b2ac4 candidate reclaimed space and passed aligned Pause/Stop,
but the owner rejected its inset picture and explicitly required edge-to-edge
like PSX. That clarification governs the independent picture bounds above.
The new viewport and remaining bar, rotation, repeated-return and fresh-reload
checks need a new authorized window. The title-free window/menu are retained.

## Stable identities, ephemeral execution

GameId is a local opaque stable entry ID. Content fingerprints validate files,
including region/revision/disc membership; they do not replace that identity.
SaveProfile identifies attached card storage, region/format, slot topology and
applicable persistent console settings. SavedSession binds GameId, SaveProfile,
selected media, compatible checkpoint and pause intent. LiveRuntime is ephemeral.
Never embed PIDs, Android task IDs, worker slots or graphics handles in save IDs
or shortcut targets.

One validated OpenGame(GameId) serves collection taps, initial launch, warm
intents and eventual iOS entrypoints. Check target availability first. Same-game
return reuses the current runtime and pause state. Different-game requests are
serialized and repeated requests coalesced. Commit the departing session safely
before destroying its only usable live state. A failed save exposes recovery or
an explicit discard decision, not silent progress loss. Invalid IDs do not become
arbitrary filesystem, core-library or shell paths.

The minimum state machine distinguishes not-loaded, restoring, running,
intentionally-paused, interrupted, saving, checkpointed/unloaded and failed.
More than one saved session is ordinary behavior; one live runtime is initial
policy. A future paused-resident optimization must not change durable identities.

## GameCube storage, not a PS1 card copy or Xbox disk overlay

Dolphin's inspected implementation has raw-memory-card and GCI-directory modes,
region-dependent paths and card identity reflected in SRAM [S4]. Choose one primary
mode explicitly after inspecting flush/state behavior. Do not mix modes silently,
reuse PS1 card sizes or substitute arbitrary card bytes after restoring RAM.

The initial implementation selects native raw card A and no card B, preserving
Dolphin's GameCube format/capacity handling. A maintained native whole-image write
hook invokes Rust's atomic candidate write, file sync, rename and directory sync.
The raw worker retains dirty status on failure. A synchronous barrier runs under
the native CPU guard and serializes with that worker. GCI-directory mode is not
selected: its inspected write path clears dirty status before confirming a write
and truncates the target. Native movie states may restore card bytes and normal
states omit card protocol/data; pending native events also need qualification.
The adapter exposes no snapshot restore. A card barrier alone does not establish
a coherent machine checkpoint or Android durability.

Prefer isolated per-game save profiles for safe A -> B -> A continuity. Keep
multi-disc games together. Shared cards can enable cross-title interactions, but
an independently paused machine can retain stale card/cache state; shared profiles
need an explicit design and exclusive writer protocol. No accidental use of the
user's existing Dolphin global directory or a sibling app's storage.

Ordinary card durability and emulator checkpoint durability are separate commits.
Inspect whether the chosen state path includes card bytes, directory contents,
SRAM/card identity, cached file metadata or in-flight EXI writes. Do not assume
State::Save completion means every associated card write is durable on disk.
Implement proper flush/quiesce operations if necessary; do not poll files and guess.

A checkpoint record must identify schema, exact modified core/state ABI, boot-route
and relevant resource identities, game/discs/active disc, controller topology,
SaveProfile, accepted card generation/digests, machine payload and pause intent.
Capture at a coherent native boundary. Complete required writes, sync payloads
and metadata, then publish an atomic commit record. Keep a last-known-good
checkpoint until its replacement is verified. Bound snapshot size and worker
queues from actual observations; do not carry PS1/PS2 numeric limits over.

On restore, validate completeness, compatibility and current durable card state
before entering the core. If current card generation differs, preserve both and
refuse automatic stale restore. Offer explicit cold restart with current progress;
an intentional historical restore must be a clearly described separate action,
not implicit rollback. Even equal disk hashes are insufficient if native restore
silently replaces card contents: test actual behavior with an isolated newer save.

Do not splice a newer card into older machine state without a verified native
semantics model. Suspend dirty/native writers during snapshot operations; one
OS-backed profile lock survives through engine and storage-worker teardown.
Distinguish lock contention from I/O/API errors. Validate locking on Android itself,
not only on a Linux host. Atomic rename alone does not exclude another writer.

Keep region/encoding, card capacity, corruption and unsupported imports explicit.
Never format, truncate, change card size or migrate the user's only copy on error.
Use private recoverable backups before app/storage updates. Export/import should
preserve ordinary card saves independently of version-bound machine checkpoints.

## Native picture, audio and controls

Evaluate native Vulkan and GLES paths against actual Fold capabilities, retaining
Dolphin's working backend behavior. No required backend is chosen solely because
PSX uses it. One logical owner controls each context/window/command queue. Surface
replacement is not permission to reset a game. Handle zero/tiny startup dimensions,
late buffers, camera insets tagged to their coordinate space and all postures.
Avoid CPU frame readbacks or managed per-frame copies. Restore shared graphics
state correctly if our UI shares the presentation path.

Use one real audio-delivery path, not donor audio plus a second host output.
Respect reported sample rate/cadence, bound latency and separate game FPS from
emulation speed and presentation rate. Do not conceal underperformance through
frame skipping or guest underclocking without an explicit qualified product choice.
Release held input on all focus/surface/controller interruptions. Partial analog
L/R and full-click L/R are distinct, as are the Control and C Sticks. Calibrate
dead zones/gates without creating unreachable diagonals or unintended full clicks.

## Home Screen and later platforms

Android pin requests contain only a validated game ID, explicit app destination,
label and local icon. Record confirmed pinning separately from accepted requests.
Handle cold and warm intents and repeated taps without a duplicate core. Unpinning
never deletes games/saves. Recovery of a restored stale shortcut must not bind to
another game. Retain the single-runtime launch behavior until a measured extension.

Proposed scheme: machinerhythm-gcx://play/<GameId>. Future iOS URL/App Intent
adapters can forward the same operation; user-created Shortcuts are not silently
installed extra apps. Ordinary iOS performance and store suitability are unresolved.
No debugger, special JIT entitlement or resident worker is assumed by the product.

Multiple paused resident processes are future optimization, not the baseline.
Declare bounded Android components only if justified; each worker gets one core
and exclusive storage. Android may reclaim them. Checkpoint recovery must work
without residency. No fake foreground audio, silent service or security weakening.
