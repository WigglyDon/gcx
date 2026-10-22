# Architecture contracts

Status: initial design for Master evaluation, not implemented capabilities.
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
