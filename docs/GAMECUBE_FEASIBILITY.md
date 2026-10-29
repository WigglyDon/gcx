> Historical source-selection research, retained at its original scope. Current work follows AGENTS.md, NTX-5 and docs/ARCHITECTURE.md. Legacy product names here describe that research.

# GameCube feasibility and first investigation

Reviewed 2026-10-21. These are source observations and proposed tests, not a build,
performance benchmark, full provenance audit or application release approval.

## First engine: official Dolphin

Observed source: `dolphin-emu/dolphin` at `cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b`. The README declares
GameCube/Wii emulation, Android support with 64-bit ARM or x86-64, native CMake
builds beneath its Android Gradle frontend, and GPL-2.0-or-later [S1]. This is a
strong initial integration candidate without starting a new console core.

The new Master should compare that observation with actual current source and a
suitable reproducible upstream revision, then hold one fully pinned candidate
stable during qualification. Record submodules, generated resources, toolchains,
local modifications and licenses. Do not chase nightlies by default or download
an opaque APK and call it an embedded engine. No fork must be created before a
small maintained patch proves necessary; targeted forks are allowed.

Inspect native Host/Core/Boot/State and Android adapter ownership first. Use a
thin C ABI into our Rust runtime where it is coherent; libretro is not a product
requirement. Do not adopt the donor's entire Kotlin/Java/Qt UI. Retain compatibility
fixes, accurate execution, backend code and internal threads that serve the game.
Fix the actual layer when integration exposes a defect. No blanket vendor immunity.

## Firmware is not an unconditional first-run requirement

The inspected defaults set SkipIPL and DSPHLE to true, and the source contains an
emulated BS2 boot implementation [S2]. The starting route is therefore the existing
emulated boot path, not a demand that every user import an original IPL image.
Qualify it with complete game boots, sound, saves and resume. A source setting is
not proof that every title or optional console-menu feature works without a dump.

Audit the exact built resource set, including fonts and any DSP/boot resources,
for source/provenance and retained terms. Bundle only qualified redistributable
inputs. Never add Nintendo startup graphics/sound, a proprietary IPL or copied
firmware as branding. A legitimate local IPL import may be a distinct optional
feature; gcx does not need to reproduce the original console menu to be complete.
An unavailable boot capability can justify first-party Rust work, but there is
no reason to invent a new BIOS project before testing the established boot route.

## Input is a real GameCube controller

The pinned input code separates Control Stick and C Stick, digital buttons,
analog L/R channels and digital L/R clicks [S3]. Do not reuse a PlayStation pad
layout unchanged or assume generic two-trigger events are sufficient. Touch must
allow useful partial presses as well as full-click behavior, alongside movement
and aiming. Preserve octagonal-gate intent and per-axis range calibration. Start
with one ordinary GameCube pad; allow the internal model to identify ports without
turning multiplayer/peripheral support into the first milestone.

A pressure-sensitive screen is not required: a deliberately designed positional
or separate partial/full touch control can represent the signals. The design must
be playable and measured, not a hidden hard-coded pressure that breaks a mechanic.
Actual hardware controllers vary; missing trigger axes require honest mapping,
not silently inferred analog values. Do not assume a connected controller exists.

## Disc and storage semantics

Identify GameCube media using the chosen DiscIO parser, not extensions alone.
ISO/GCM and RVZ are initial input targets to verify. RVZ is a compressed container,
not an instruction to duplicate a whole decompressed image. Retain region, revision,
disc number and content identity; group disc sets deliberately and test runtime
swaps. File moves and format conversion need a stable entry plus validated relink,
not a newly allocated save identity by default.

The inspected card device has raw and GCI-directory paths, region handling and
card/SRAM identity [S4]. Inspect concrete DoState and flush implementation in the
selected revision before choosing a profile model. Ordinary card exports and
version-bound emulator checkpoints are not interchangeable. In-flight native
writes, delayed callbacks and state restoration must not undo newer progress.

## Android qualification

Source-build the ARM64 engine with only needed frontends/features. Verify which
JNI/global services its native components still expect when our Rust application
owns the UI. Make the minimum coherent adapter, not a shell that launches the
standalone emulator or copies frames from another app. Verify JIT/fast-memory and
native-library behavior on the actual host page size and GPU; packaging alignment
alone does not prove 16 KiB runtime support [S6].

First obtain actual moving gameplay, useful touch, sound and ordinary card save /
fresh-process load inside gcx. Then qualify automatic resume and A -> B -> A.
A donor-app baseline is diagnostic, not acceptance of our host. Record sustained
bounded observations, heat, memory, audio gaps and first-compile shader stutter
separately. Do not promise a thermal/endurance result from one title screen.

## iOS: preserve the goal, state the actual gap

DolphiniOS's maintainers currently identify JIT as a blocker to their App Store
release [S5]. Treat that as evidence of a serious execution/distribution gap, not
proof that gcx is either approved or permanently impossible. A macOS ARM build is
not an iPhone result. Keep core/platform boundaries portable; a stock-iOS route
requires its own actual non-debugger gameplay and legal/distribution evaluation.
No complete iOS port or new CPU-recompiler campaign is dispatched by this seed.

## First decision and genuine missing inputs

No gcx game, private image, device window or local toolchain has been verified by
this seed. Ask for one owner-chosen GameCube title/input when needed. Later request
a second game to prove independent sessions; a multi-disc title qualifies swapping
only when its complete disc set is available. Do not choose a title for the user
or silently reuse PSX/xbx input directories. Build/source work can start off-device.

The first handback should contain one reproducible native/Rust runtime result,
exact source binding, a measured path toward the target player and concrete limits.
No synthetic framework skeleton is a substitute for the engine slice. If a major
architectural campaign is required, report it while preserving working increments.
