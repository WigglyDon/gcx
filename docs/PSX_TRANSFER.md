# Selective PSX transfer, not a merger

Behavioral reference: Don reports that PSX's PS1 player resumes independent games
across closures, fresh processes and folds and feels good in ordinary play.
Historical public source binding: `WigglyDon/psx` at `1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11`.
This is a retained reference, not a claim to be its latest branch head or to prove
every source path was tested. Re-read current public files before actual reuse.

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
