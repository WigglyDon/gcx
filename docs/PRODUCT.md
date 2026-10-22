# Product: GameCube games as durable applications

## The intended experience

The owner's working PSX PS1 player is the behavioral reference: select a game,
play comfortably, leave it, try another game, fold the phone, and return to the
correct scene after process restart. gcx carries that experience to GameCube as
its own application and identity. It is not PSX renamed or a multi-console merger.

The normal path is **choose a local game -> play -> leave -> return**. The engine
is implementation machinery, not a choice ordinary users need to configure.
Open gcx for a small collection and Add games. A game-specific Home Screen icon
enters that game's durable session directly. Settings stay compact and actionable.

Initial scope is GameCube, one local player and one live runtime. Wii is a plausible
later extension because the candidate engine supports it, but it introduces other
controllers, storage and services and is not active scope. Do not show fake Wii
support. GBA link, microphone, broadband adapter, Triforce and netplay are separate
compatibility/features, not silent requirements for ordinary GameCube play.

## Native feel, not an emulator control panel

The first desired device is the stock Android ARM64 Fold. Fit the complete
aspect-correct game picture around camera/system insets and comfortable controls.
Keep controls off the picture by default. Do not stretch or crop game-written
borders without explicit player choice. Use the actual reported game geometry,
not a permanent 4:3 or 60-game-FPS assumption.

Preserve touch capture, multi-finger combinations, neutralization on interruption,
short press haptics and per-display editable layout profiles. The GameCube layout
must be deliberately designed: prominent A, B/X/Y, Control Stick, C Stick, D-pad,
Z, Start/Pause and L/R. Trigger travel and full-press click are independent signals.
Users must be able to hold a partial press and move/aim without an accidental click.
Document and test the touch gesture, then evaluate it by actual gameplay.
Physical controllers feed the same logical input and must neutralize on disconnect.
Keep a usable touch route; an external controller is not a required workaround.

Pause feedback is immediate while state storage completes safely. Explicit user
pause survives fold/rotation/focus changes. Background means paused and silent by
default, with durable recovery; no fake audio service or CPU keepalive. Bound
queues and discard stale audio/input rather than replaying them on return.

## Library, formats and identity

Use scoped document/folder access as a complete path. Explicit rescans may inspect
selected roots; no mandatory all-files permission or startup-wide storage crawl.
Inspect container and platform metadata rather than trusting ISO/RVZ filename
extensions. Start with ordinary GameCube ISO/GCM and RVZ where the chosen decoder
supports them; document what actually works before advertising more formats.
Keep compressed content streaming; do not inflate every game into another full
copy. Missing access or media must lead to recovery of the same entry and saves.

A multi-disc game has one durable game identity and save profile, with distinct
media identity, region/revision and remembered disc. Do not merge unrelated games
by similar filename, discard a second disc, or reset a session when it asks for a
swap. A local title/cover can be edited without changing saves. Use local artwork
when valid or an original placeholder; no mandatory cover service or scraping.

## Continuity and completeness

Ordinary memory-card saves are durable game progress; complete machine checkpoints
are convenient continuation. Cards, state, selected media and controller topology
must agree before automatic resume. Do not silently restart an incompatible state
or overwrite newer card progress to make an old snapshot load.

Local export/import is the desired migration route, not a cloud-account product.
Preserve card format, region and identifiers; validate imports rather than treating
arbitrary bytes as a card. No sharing of a writable card across independently
paused game machines by accident. Cross-title save interactions require an explicit
shared-profile design and regression checks, not global writable default storage.

Aim for install -> choose game -> play without external firmware. The initial
Dolphin boot/HLE path is already a source-identified route, not a new BIOS project.
Audit its resources, verify actual gameplay and retain a legitimate optional IPL
route only when a real feature requires it. Do not bundle proprietary startup art,
console menus or game assets to imitate branding.

## Scope control

All new substantive first-party logic is Rust. Borrow, modify and maintain the
mature engine where it helps. No arbitrary donor-hopping, whole-core rewrite,
shared-studio framework, online account system or enormous settings catalog.
Keep future multi-resident sessions possible through identity/ownership, but
implement one runtime first. Android and iOS may provide the same durable game
experience with different process and execution policies. Do not promise equal
engine availability or speed on all platforms without evidence.
