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
picture; controls retain clearance. A changed effective/invalid mapping or mismatched
rotation releases held input; the game is retained.
The first Fold world pass exposed a tall deck narrowing the picture and a
following stick drawing over adjacent controls. The next Rust-only presentation
candidate reduces the default dock from260 to172 points: D-pad beside Control
Stick, C Stick below-left of the face buttons, independent analog/click shoulders
and separate Start/Pause. All digital targets remain at least48 points; docked
sticks are104 points. Synthetic Fold-size fixtures keep a full-width picture at
the supplied engine aspect without covering targets. Drawing confines each
following well to its generous hit rectangle; input following and native sampling
stay unchanged. Existing saved placements/preferences are not erased.

Gameplay contacts and admitted menu/library clicks share the saved haptics
preference. Modal menus use their own focused Dialog window; the Activity decor
is unfocused while a Dialog is up. Android feedback return values are recorded
once per outcome per Activity (accepted, declined, no focus, JNI error), without
forcing system settings or adding work to pad polling. Acceptance of a request
does not establish felt vibration. Physical consistency still needs qualification.
Window measurement/input diagnostics may not exhaust the evidence needed for a
new presented frame. The shared48-sample Activity budget reserves12 slots for
presentation-ready/unavailable reports; optional callbacks use an atomic
reserve-preserving take. The overall192-event bound remains unchanged. The
per-focus measurement counter does not reopen that shared budget. Presentation
exhaustion has an explicit marker, and native test input still refuses absent,
stale or capped geometry. This repairs observed diagnostic starvation, not the
underlying original Fold process-loss incident.
App-local API30+ WindowInsetsController (API28/29 immersive-sticky fallback) hides
bars only while playing; menus/picker restore ordinary navigation. Main-thread
callbacks are checked against the bound Activity; posted work also checks its
epoch. Candidate52e240d passed hidden bars, world rendering and the aligned Pause
target on Pixel4; its picture was too small for owner acceptance. The pass ended
with normal joined Stop, matching protected card backups and verified return.
The title-free74b2ac4 candidate reclaimed space and passed aligned Pause/Stop,
but the owner rejected its inset picture and explicitly required edge-to-edge
like PSX. That clarification governs the independent picture bounds above.
The58335ff/f4f60fa0 Pixel4 pass earned owner acceptance of picture size, broad
other-landscape/control/sound feedback, aligned menu actions and normal Stop with
a confirmed fresh-process load of the earlier one-rupee card save. The owner
corrected an initial Save-finished response: the two-rupee play was unsaved and
disposable. No new in-game-save result transfers to this build. A white system
Home handle remained across the picture centre after reveal and cold relaunch;
the two extra Home/return cycles were skipped. Actual final card/SRAM backups were
retained and the phone returned screen-off/released. Its changed final card was
preserved separately; no older backup was restored.

The next bounded policy candidate follows public PSX's API31+ BEHAVIOR_DEFAULT,
which permits navigation gestures while bars are hidden; API30 retains transient
bars. This is an app-local candidate change, not proof of the handle's cause or
repair. Shared picture/control geometry and title-free window/menu are retained.
The remaining artifact and repeated-return checks need a new authorized window.

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

The Rust library dialog now lists imported entries by stable GameId, with scrollable
native views built from Rust and bounded JNI reference frames. Rows carry only
GameId and menu generation through a mechanical callback. Unknown/malformed IDs
leave the current runtime intact; repeated same-game requests return to its live
owner, and another entry uses the existing coherent checkpoint/join path. A live
recovery error blocks switching until explicit Stop/restart. Opening a saved game
expresses Resume intent; no new profile is allocated. Duplicate profile identities
are rejected in this initial isolated-profile policy. Import still uses scoped
document grants. Library display/switching remains Android-unqualified; relink/multi-disc support
is not implemented by this slice.

The next Rust launch adapter accepts only canonical
`machinerhythm-gcx://play/<GameId>` links. The exported singleTask activity forwards
new intents mechanically; Rust handles cold/warm requests through the same GameId
resolver. Cold invalid/unknown links leave the player unloaded, and warm requests
cannot create a second runtime or discard recovery RAM. Pending game requests
coalesce. No paths, media grants, native handles or save identities are supplied
by a link. Home screen pin requests are explicit from the game menu and target
this activity/package. Labels use the local title; no private artwork is copied.
Accepted requests and later own-app OS pin observations are separate saved fields;
unknown/failed observations retain prior status. OS reads occur on the Rust worker
while the guest is paused/unloaded, with no service or legacy install broadcast.
Unpin observations never remove a game/profile/save. This implementation follows
[Android's pin contract](https://developer.android.com/reference/android/content/pm/ShortcutManager#requestPinShortcut(android.content.pm.ShortcutInfo,%20android.content.IntentSender));
request acceptance is not confirmation. URI/policy/metadata host tests pass;
launcher confirmation, cold/warm/repeated taps and Android switching remain unrun.

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
The earlier ABI5 adapter exposes no snapshot restore. A card barrier alone does
not establish a coherent machine checkpoint or Android durability.

The pinned off-device [checkpoint audit](../native/README.md#off-device-checkpoint-audit-2026-10-22)
confirms that normal states omit staged card commands while restoring timing
events, SRAM and channel identity. The card completion handlers themselves do not
replay writes; restored guest execution can still write stale data. Movie payloads
can replace the raw card in RAM, and a forced barrier can then publish it. A future
state adapter must preserve protocol coherence independently of optional card
contents and report actual capture/load success. Dolphin's void SaveAs/LoadAs and
after-load callback do not provide that contract. Rust must validate the complete
generation before native mutation; failed partial loads must not flush mutated
card/SRAM during recovery. No state API or automatic resume is implemented by this
audit. Existing ordinary card save/Stop/reload evidence is unchanged.

The subsequent ABI6 candidate implements this contract with original Rust
checkpoint envelopes and save generations, plus targeted private native state
serialization. Card protocol is always included; card contents and movie-active
payloads are refused. Raw card and SRAM use one atomic Rust writer. The paused
machine, complete durable files and exact media/core/resource identities form one
generation, published before SavedSession references it. Interrupted writes never
roll ordinary files back. Restore validates before paused boot and again before
mutation; failed partial loads block persistence/resume through shutdown. Joined
checkpoint departures recheck ordinary generations. ABI7 also checks exact native card/SRAM reads through Rust; failed reads cannot
be treated as absence or publish fallback images. Every app boot waits paused
until initialization/restore is healthy. Rust captures on pause and
departure and restores after a fresh process; explicit Stop still requests an
ordinary-save cold Open. Files/errors survive rejection and explicit restart uses
current saves. The original native fresh-process CPU/staged-command/event and
failure fixtures pass; Android gameplay automatic resume remains unqualified.
See [implementation and reproduction](../native/README.md#rust-owned-checkpoint-candidate-2026-10-22).

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

While publishing, the paused core still lends the complete expected container.
After file/rename/directory sync, compare every persisted byte in bounded64KiB
chunks and reject truncation, extra bytes or read errors, then recheck current
ordinary-save generation. Do not allocate and deserialize a second machine image
merely for this write verification. Actual restore still independently validates
all binding, payload/container digests and current-save identities before native
mutation. This preserves the existing format/state ABI. ARM64 hashing uses the
pinned SHA library's runtime-detected acceleration and software fallback; never
replace media hashing with an untrusted size/mtime cache to reduce startup time.

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

ABI8 samples the Rust touch packet at `Pad::GetStatus`, under Dolphin's controller
state lock, immediately before its GameCube SI input read. Rust owns fixed16-slot
pointer captures, latest analog positions and digital presses retained until one
actual native poll; draw callbacks and the Android host tick never consume them.
All18 GameCube channels remain independent. The callback lives through native
join, and registration/reads share the controller lock. Rust drops its UI lock
before entering core/pad operations; the sampling callback only takes the short
Rust input lock and copies a stack packet. No input pulse or callback address is
a durable game/session identity or checkpoint field.

PSX-adapted following centers, continuous radial dead-zone rescaling and D-pad
hysteresis are positional input policy, with GameCube's octagonal gate retained.
There is no time interpolation of logical controls. The two render callbacks
share an immutable geometry allocation. A valid measurement with identical
effective input/surface mapping preserves captures; real changes, invalid
measurements and interruptions clear held and pending input. Content-inset-only
events refresh that mapping without an unconditional renderer resize.

Per-runtime bounded counters separately observe foreground Android event age,
dispatch-to-native-pad age and active pad-poll gaps. They retain no raw touch/game
data and log only summaries on checkpoint departure. They are software pipeline
evidence, not touch-to-photon, game movement, FPS or hard real-time guarantees.

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

## First alpha controls and presentation

Rust owns the branded native menu, per-screen control editor and independent test
pad. Small Java View/Canvas/SeekBar callbacks forward platform data only. Editor
state is guarded by Activity/menu generations. It cannot feed LiveRuntime; native
pause/checkpoint ownership remains with the existing Rust runtime owner.

`controls.json` is app preference data, separate from SavedSession/SaveProfile.
Validated, bounded profiles address named GameCube targets using normalized
control-safe rectangles. A whole valid profile is admitted at presentation;
unknown/colliding/tiny overrides fall back together. Apply atomically saves before
changing the active settings; it cancels old input and requires fresh geometry.
Save checks the current measured screen on Android's main thread, without
mistaking dialog focus loss for a change of physical display. Layout edits remain
separate for different screen sizes. Reset removes only this screen's overrides.
Corrupt settings stay intact and are reported; a settings failure does not reset
cards/checkpoints or silently restart a game.

Opacity is decorative. Adjustable radial dead zone and following-stick policy
retain the GameCube gate, independent analog travel/clicks and actual native-poll
press consumption. No smoothing/dwell or visual animation delays logical input.
Haptic requests respect Android's own preference and require no extra permission.
The render/input hot path does not clone persisted profiles each frame.
