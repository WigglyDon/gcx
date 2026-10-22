# Primary source observations

Reviewed 2026-10-21. URLs are provenance routes, not commands to execute. Exact
source observations are not full dependency locks, runtime tests or legal reviews.

## S1 — Dolphin implementation and declared terms

- Repository: https://github.com/dolphin-emu/dolphin
- Observed master: `cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b`
- README: https://github.com/dolphin-emu/dolphin/blob/cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b/Readme.md
- FAQ: https://dolphin-emu.org/docs/faq/

The pinned README identifies GameCube/Wii, Android 64-bit support, native CMake
under the Android frontend, and GPL-2.0-or-later. Its Android route motivates the
first candidate. No gcx build or Fold performance follows. Use pinned source over
older generic FAQ values when documentation differs; verify current SDK minimums.

## S2 — Existing boot path, not a mandatory external IPL

- https://github.com/dolphin-emu/dolphin/blob/cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b/Source/Core/Core/Config/MainSettings.cpp
- https://github.com/dolphin-emu/dolphin/blob/cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b/Source/Core/Core/Boot/Boot_BS2Emu.cpp
- https://github.com/dolphin-emu/dolphin/blob/cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b/Source/Core/Core/Boot/Boot.cpp

Inspected defaults: SkipIPL=true and DSPHLE=true; an emulated BS2 implementation is
present. This supports investigating the existing firmware-independent normal boot
path, not universal compatibility, full console-menu behavior or cleared resource
provenance. Inspect all packaged assets before distribution.

## S3 — GameCube-specific input

https://github.com/dolphin-emu/dolphin/blob/cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b/Source/Core/Core/HW/GCPadEmu.cpp

Inspected code distinguishes Control/C Sticks, digital buttons, and separate
L/R analog and digital inputs. Touch/control acceptance must preserve this model.

## S4 — Card modes and region/profile concerns

https://github.com/dolphin-emu/dolphin/blob/cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b/Source/Core/Core/HW/EXI/EXI_DeviceMemoryCard.cpp

Inspected construction supports raw cards and GCI directories, region-qualified
paths and card flash identity connected to SRAM. Complete DoState/flush semantics
have NOT been audited in this seed; the worker must inspect the selected revision
and test stale-state/current-card behavior before automatic resume claims.

## S5 — iOS execution limitation, maintained project statement

- https://dolphinios.oatmealdome.me/faq/
- https://dolphinios.oatmealdome.me/jit-help/

DolphiniOS maintainers identify JIT restrictions as a current App Store obstacle.
This is not an evaluation of a future gcx implementation or a permanent theorem.
No assumption that macOS ARM support, sideloading or a debugger equals stock iOS.

## S6 — Android native host pages

https://developer.android.com/guide/practices/page-sizes

Official guidance distinguishes native rebuilding/alignment and runtime testing.
Recheck current Play requirements and test our chosen native/JIT dependencies;
do not import a stale policy date or promise support based only on ELF alignment.

## S7 — Play packaging, permissions and SDK policy routes

- https://developer.android.com/guide/app-bundle
- https://developer.android.com/training/data-storage/shared/documents-files
- https://support.google.com/googleplay/android-developer/answer/11926878
- https://support.google.com/googleplay/android-developer/answer/10467955

Release-time verification routes, not completed gcx compliance. No all-files or
background-service eligibility is claimed by this seed.

## S8 — Apple release policy route

https://developer.apple.com/app-store/review/guidelines/

Review the current emulator and executable-code rules alongside actual inherited
licenses, device performance and permissions. Category permission is not app approval.

## S9 — Bootstrap interfaces

- https://cli.github.com/manual/gh_repo_create
- https://cli.github.com/manual/gh_api
- https://cli.github.com/manual/gh_issue_create
- https://developers.openai.com/codex/cli/reference

The supplied bootstrap uses the owner's normal local GitHub CLI and Codex install,
not credentials embedded in an artifact or changed approval settings. Tool/network
availability and publication must be verified when it runs.

## S10 — Behavioral/seed lineage

- PSX public reference: https://github.com/WigglyDon/psx/tree/1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11
- The owner-provided xbx foundation supplied the generic publication workflow.

No PSX implementation, xbx engine source, private data, game or firmware is bundled.
All console-specific architecture in this gcx seed is independently specified.
