> Historical source-selection research, retained at its original scope. Current work follows AGENTS.md, NTX-5 and docs/ARCHITECTURE.md. Legacy product names here describe that research.

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
paths and card flash identity connected to SRAM. The 2026-10-22 pinned
[checkpoint audit](../native/README.md#off-device-checkpoint-audit-2026-10-22)
records the inspected raw-card/protocol, timing, SRAM and state completion paths.
It identifies concrete gaps rather than proving safe restore. Isolated
stale-state/current-card tests and a completion-bearing adapter are still required
before automatic resume claims.

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

Selected public MIT PSX landscape geometry has since been adapted as recorded in
PSX_TRANSFER.md. No sibling engine source, private data, game or firmware is bundled.
All console-specific architecture in this gcx seed is independently specified.

## S11 — Android app-window geometry and immersive mode

- https://developer.android.com/develop/ui/views/layout/immersive
- https://developer.android.com/develop/ui/views/layout/edge-to-edge
- https://developer.android.com/reference/android/view/WindowInsets
- https://developer.android.com/reference/android/view/WindowInsetsController
- https://github.com/aosp-mirror/platform_frameworks_base/blob/android-13.0.0_r8/core/java/android/app/NativeActivity.java

Read for the bounded fullscreen/inset continuation after Master6348389764.
NativeActivity takes the window surface/input queue; its content rectangle can be
inset within that surface. Rust queries current bounds rather than assuming a bar
height or equating screen, input and buffer coordinates. API30+ uses the platform
insets controller with transient swipe access; API28/29 uses supported legacy
immersive-sticky flags. Stable bar clearance protects controls during transient
reveal; this conservative choice is independent of successful hiding. Framework
callbacks are mechanical forwarding only. No AndroidX/managed frontend is added.
These sources inform implementation; they do not establish device qualification.

After the full-drawable Pixel4 sizing acceptance, the owner reported a misplaced
Home gesture handle following bar reveal. The requested-hidden navigation window
still had its normal bottom frame; scoped evidence is retained privately. The
API31+ candidate now uses BEHAVIOR_DEFAULT, following the reviewed public PSX
platform owner (exact blob in PSX_TRANSFER.md). Android documents that mode as
allowing navigation gestures while bars are hidden. API30 retains transient mode.
This is a supported policy adjustment pending device verification, not an
established explanation or repair of the handle artifact.

## S12 — GameId launch and pinned shortcut contract

- https://developer.android.com/reference/android/content/pm/ShortcutManager
- https://developer.android.com/develop/ui/compose/system/shortcuts/creating-shortcuts
- Pinned Android37.0 SDK android.jar: ShortcutManager/ShortcutInfo.Builder JVM signatures inspected using the existing JDK.

The foreground app can request launcher confirmation; the API's positive return
is not a confirmed pin. Own-app pinned-shortcut observations are separate, run on
the Rust worker because the query can block, and never delete progress. New-intent
forwarding is mechanical; URI parsing, identity lookup, pin policy and persistence
are original Rust. No AndroidX frontend, service or proprietary icon is added.
This is source/API preparation; physical launcher results remain unqualified.
