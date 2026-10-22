# Selective PSX transfer, not a merger

Behavioral reference: Don reports that PSX's PS1 player resumes independent games
across closures, fresh processes and folds and feels good in ordinary play.
Historical public source binding: `WigglyDon/psx` at `1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11`.
This is a retained reference, not a claim to be its latest branch head or to prove
every source path was tested. Re-read current public files before actual reuse.

## Landscape adaptation, 2026-10-22

Owner requested the public PSX layout as gcx's clean default landscape reference.
Reviewed public revision `4277d4ca651a4c9c2e8e88113a0ef092e330f01c`, MIT LICENSE
(Machine Rhythm contributors, 2026), and `crates/player/src/input.rs`, blob
`c3d90f7ca21e1a18b1f945cef999287af054ec9c`. Relevant owners: `side_rails`, the
density-independent control metrics, `ps2_camera_rails` spacing, `PlayerViewport`
and high, centered clear-picture placement. Also inspected the public Android
platform's fullscreen/inset handling; it is not transplanted into gcx.

`crates/gcx/src/touch.rs` adapts the side-rail arrangement and spacing to a bounded
GameCube layout: large green A, red B, gray X/Y and L/R, purple Z, gray Control
Stick and yellow C Stick; independent trigger travel/clicks, a captured eight-way
D-pad, Start and a separate gcx Pause action. No PS1 masks, PS2 stick clicks,
fixed PS1 aspect, saved profiles, GL or emulator/storage code is transferred.
Rust uses actual display density and the engine's reported aspect. The MIT
copyright notice is preserved in the root license and therefore APK notices.
Host tests cover non-overlap, bounds/minimum targets, all 18 reachable channels,
diagonal D-pad and simultaneous independent trigger release. Device landscape,
insets/cutouts and full physical control qualification remain due.

This is selected public-source adaptation. PSX's checkout, private files, app
resources and active work were not read or changed for it.

Candidate reusable concepts: validated opaque GameId/OpenGame requests, input
capture and cancellation, aspect-correct layout/editor behavior, atomic settings,
private-source separation, document grants, launch-intent handling, native Android
lifecycle and focused packaging checks. Check coupling and actual licenses first.
Record the source revision, file path, license, modifications and tests for each
transferred file. Preserve attribution. Do not copy private files or full history.

Do NOT inherit PS1 button masks, card sizes, BIOS bytes, libretro timing/TLS,
assumed 4:3 geometry, snapshot-size limits, firmware restore order or specific GL
state ownership. Do not import PS2 dual-analog semantics as GameCube trigger/click
semantics. xbx's hard-disk/EEPROM snapshot design is not gcx's memory-card design.

Recent PSX failure lessons to include in tests, not claims of shared code defects:
transient tiny native windows can invalidate geometry; a standard-library lock
API may compile but be unsupported on Android; a C++ adapter can hide mangled
exports from a C ABI. Verify actual platform primitives and exported symbols.

Maintain separate app IDs, signers, storage and release/source history. A small
port can reuse source without a studio-wide framework extraction. Do not modify
PSX/xbx/FN64/Kitten or share mutable cards/caches without the respective authority.
The new Master owns gcx decisions and consults actual source when transfer helps.
