# gcx development

Read README.md, docs/state.toml, docs/github-state.toml, docs/PRODUCT.md,
docs/ARCHITECTURE.md and the active issue with its latest comments. Read
GAMECUBE_FEASIBILITY, DISTRIBUTION and OPERATIONS before choosing build/device
paths. Prepared issue files are scope, not proof of remote creation or execution.

## Continuing an existing assignment

Reconcile the latest live review with the current acceptance row before choosing
work. Initialization instructions are not a request to repeat completed boot,
ordinary-save, checkpoint or timing campaigns. Keep a qualified installed binary
stable while diagnosing an open incident; use retained evidence and the smallest
relevant check before requesting another build or device transaction.

Resolve source, tool and cache inputs against their actual current locations and
ownership. A historical workspace receipt, mount, symlink or cache label does not
prove those bytes still exist or grant capacity. If an input disappeared, report
that exact gap; do not revive a retired storage route, silently fall back to a
different filesystem, or copy a sibling checkout. Host placement and any second
platform work still require their own bounded assignment.

## Product and authority

Build an independent native GameCube player with the same game-first continuity
as PSX. Android ARM64 on the owner's stock Fold is the first desired playable
target. GameCube only initially; no Wii, Triforce, GBA-link, online service or
second platform is automatically admitted. Eventual mobile stores are goals.

All new substantive first-party logic is Rust. Use the mature Dolphin engine as
the first candidate. Inspect its native Android/core interfaces rather than
requiring libretro or inheriting the full Android/desktop frontend. Retain useful
C/C++ code, its authorship and licenses. Targeted donor fixes, optimizations,
proper APIs, bounded boot/HLE work and justified Rust replacements are authorized.
Mechanical platform shims and small donor-language edits are allowed. No Kotlin,
Java, C++ or Python application layer; no whole-core rewrite merely for language
purity. Upstream acceptance is not required to maintain our tested fork.

Prefer a coherent fix at the actual owner over layers, duplicated state or a tour
of alternative cores. Escalate material scope change, unknown technical model,
rights/security issues or data-loss risk, not the mere need to edit borrowed code.

## Core contracts

Distinguish GameId, SavedSession, SaveProfile and LiveRuntime. One foreground
runtime initially serves multiple durable games; it is not a permanent session
limit. Exactly one writer owns each card/profile/checkpoint through native teardown.
Keep process/task IDs and native handles out of permanent shortcuts/save identities.

GameCube needs a Control Stick, C Stick, D-pad, A/B/X/Y, Z, Start/Pause, and separate
L/R analog travel and digital clicks. Do not invent L3/R3 or reuse a PS1 button
mask. Preserve partial-trigger play, simultaneous touches, calibration and
release on interruptions. Rust owns the player UI and input policy.

Ordinary saves and machine snapshots are different evidence. Inspect raw-card
versus GCI-directory persistence and state restoration before deciding storage.
Do not import PS1's fixed card size, copy its restore order blindly, or use Xbox
hard-disk snapshot semantics. Do not restore stale card data over current progress.
Retain incompatible checkpoints and offer explicit restart with current saves.

Use Dolphin's existing emulated boot path as the initial BIOS-free candidate.
Do not impose original IPL import or develop a replacement BIOS without an actual
missing behavior. Audit all packaged resources and their rights. No copyrighted
firmware downloading, decompilation-derived laundering or proprietary assets in Git.

## Work and acceptance

GCX-1 is the first assignment. Inspect current local/remote work and acknowledge
with actual source binding before starting; do not duplicate an existing worker.
Move from pinned source to one executable Rust-hosted GameCube slice, not a
framework or another plan. Continue appropriate GCX-2 work once viable without
waiting for ceremonial closure; honor fresh Master decisions. The initial
multi-resident feature and full iOS port remain deferred.

Protect sibling projects, app IDs/signers, user games/cards/states and shared
resources. Build off-device. Read current host capacity/custodian policy before
heavy clones/builds. Never use global cleanup, unrelated caches, filesystem repair
or repeated expanded source archives to solve this project's space needs.
Personal Fold access is on request through the current exclusive stock-device
procedure. No root, unlock, wipe, system-package disabling or security changes.

Run focused tests, formatting, warnings-denied relevant lints, and actual native
packaging/ABI/signature/notices checks. Add tests for changed behavior; do not
restart an emulator-wide conformance campaign. Keep visible gameplay, audible
sound, card save/reload, automatic resume, shortcuts and measured speed separate.
Use isolated fixtures for faults; do not fill the owner's disk to test ENOSPC.

## Public handling

Public gcx source and ordinary reviewed commits/pushes are authorized. Original
work is MIT; borrowed/modified/translated GPL work retains applicable terms.
FFI, dynamic linking or another process is not automatic license separation.
Record exact source, patches, dependencies and corresponding-source inputs.
Only reuse selected public PSX files with provenance and license inspection.
No private sibling files, game/firmware data, raw logs, identifiers or keys in Git.
Store submission, public binary release, paid services, account/security changes
and legal commitments require their separate gates. Seed creation is not corporate
formation or a rights transfer. Keep real work and evidence in the issue and
lasting behavior in its small source documents. Never infer success from intent.
