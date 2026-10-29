# ntx development

Continue this existing project under [NTX-5](https://github.com/WigglyDon/ntx/issues/5)
and its binding [architecture amendment6505289270](https://github.com/WigglyDon/ntx/issues/5#issuecomment-6505289270).
Read their latest comments, README.md, docs/NTX_HANDOFF.md, docs/state.toml,
docs/github-state.toml, docs/PRODUCT.md and docs/ARCHITECTURE.md before choosing work.
Read docs/DISTRIBUTION.md and docs/OPERATIONS.md before build/device paths.
The handoff's local-only supplement locates retained inputs and recovery material.
Old GCX issue/seed/probe instructions are historical evidence, not current priority.

## Continuation and ownership

This is succession of the same public repository (stable ID1429193278), history,
working direct Dolphin GameCube implementation and protected progress. A fresh
session acknowledges ownership on NTX-5 with actual checkout/branch/HEAD/dirt and
continues the first unfinished slice. Do not start another implementation worker,
clone a duplicate, reset donor edits, replay the seed or restart completed campaigns.
Resolve inputs/output locations after relocation; old absolute paths and cached
executables are not proof of readiness. Do not create a permanent old-path alias,
recreate retired storage or silently switch host/filesystem.

## Product and engineering

ntx is the Nintendo member of the five emulator products, with PSX as the
established game-first experience and architecture reference. Public original
first-party work is MIT; borrowed code retains its licenses/provenance. All new
substantive first-party player/host/persistence/input logic is Rust. Retain useful
C/C++ and use mechanical platform/JNI/ABI glue where necessary. No managed
application layer, speculative framework or whole-emulator rewrite.

Complete the comprehensive rename: active source/crates, native exports/JNI,
build/tooling/artifacts/docs/branding and canonical Android package
com.machinerhythm.ntx, data root ntx, links machinerhythm-ntx://play/<GameId>.
Legacy names belong only to immutable history/provenance or isolated compatibility
readers/fixtures. Preserve the signer and opaque game/profile identities. A new
Android package does not inherit private data or document grants: implement staged,
validated, idempotent migration without overwriting newer target progress. Keep
source installation and rollback material until migration is verified. Never
rewrite snapshot fingerprints to force compatibility or discard unsaved progress.

Keep Dolphin's working direct GameCube adapter. N64 through a pinned
Mupen64Plus-Next core and small Rust libretro host adapter is ACTIVE now; this does
not convert Dolphin to libretro or import RetroArch's frontend. FN64 is a possible
later N64 backend, independent of game identity and ordinary progress. Do not
modify FN64, PSX or other siblings. Wii/other systems, all-platform ports, online
services, public binary releases and paid infrastructure remain separately scoped.

## Shared product contracts

Keep host platform/device, emulated system and execution backend independent.
Portable Rust owns collection/session/save/resume/input policy and scheduling;
host adapters own platform resources; backend adapters own console execution and
console-specific persistence. Android/PID/window/GPU/JIT handles are ephemeral.
Platform agnosticism is required now, not a claim all ports are implemented.

Keep GameId, SaveProfile, SavedSession and LiveRuntime distinct. Multiple games
independently open/resumable on the same or different systems are the product
model. Executing, resident-paused, checkpointed/unloaded and recovery-pending are
placements/states, not new identities. One foreground live slot is the first
scheduling policy, not a permanent singleton architecture. Future runtime slots
must not require game/save reidentification; qualify native isolation before
simultaneous instantiation. Background pause/silence is the default.

Separate immutable media, ordinary committed progress, machine resume generations
and host/control preferences. Inspect each core's capture/restore/pending-write
semantics. Publish coherent generations, validate current ordinary data before
restore, and prevent stale/partial restore from flushing over newer saves. Retain
incompatibility/errors and offer explicit cold start with current saves. Exactly
one writer owns each profile through native join and durable metadata publication.
No assumption of cross-backend/architecture snapshot compatibility.

Use actual system controls and save formats: GameCube Control/C sticks and separate
L/R analog travel/clicks; N64 one analog stick, four C buttons, digital L/R and its
cartridge/Controller Pak storage. Preserve cancellation and simultaneous input.
Do not transplant PS1 masks, GameCube cards or Xbox storage semantics into N64.

## Finite implementation and verification

Order: full ntx identity/migration with retained GameCube progress; first playable
N64 in the same collection; supported continuity/cross-system switch and alpha
comfort. Use representative affected-boundary tests, focused Rust/native/package
checks and warnings-denied relevant lints. Preserve earned boot/save/scene/library
and matched-timing evidence. Quest-playing/speedrunner automation is retired.
Keep unresolved Pixel Pause/reentry and separate Fold incidents honest; neither
an unrepeatable incident nor a blocked optional shortcut halts all product work.

Build off-device under current shared-host capacity/concurrency rules. Preserve
signer, current cards/states, recovery candidates and unique retained inputs. No
global cleanup, sibling cache deletion, retired mount resurrection or automatic
heavy build. Installed android-device procedure owns admission/return. Pixel4
standing lab work uses normal exclusive admission; Fold needs its fresh personal
window. No root, wipe, security/launcher changes or peer preemption. Backups and
actual cleanup/return must be ready under the held lease. Phone tasks require
ready artifacts; this succession itself starts no device tests.

Public source commits/pushes are authorized. No secrets, private device details,
games, firmware, saves, raw logs or signing bytes in Git. Audit licenses/resources
and corresponding-source reproduction. Store submissions, legal commitments and
public binary release retain their separate gates. Report actual outcomes and
limits on NTX-5; keep acceptance issues open.
