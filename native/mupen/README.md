# Pinned N64 integration boundary

Current successor implementation (2026-10-29): the full pinned ARM64 GLES3/HLE
core compiles. Rust owns the small libretro adapter, Android EGL/GLES resources,
audio delivery, measured N64 touch controls, per-profile ordinary generations and
bound machine generations in the same collection as direct Dolphin. The code is
being linked/qualified; no device playable/save/reload/continuity result follows.
Earlier preparatory findings below remain provenance, with these repairs applied:

- Actual restore result and buffer admission are checked. Completed save/load
  jobs yield after clearing the job, before further guest execution. Capture then
  commits the ordinary image from that same boundary before publishing a scene.
- The selected nonparallel workqueue executes its copy synchronously. Machine
  state includes FlashRAM mode/page buffer and device protocol state; ordinary
  EEPROM/Pak/SRAM/FlashRAM bytes remain a distinct bound generation. Current
  bytes are checked before restore, and failed restore memory cannot publish.
- Shutdown skips starting an unopened coroutine and acknowledges actual stopped
  core state before freeing it. Rust retains code/context/host/writer ownership
  if that acknowledgement or post-join publication fails.
- `check_boundary.py` compiles the actual pinned header/state/deinit entrypoints
  with bounded fixtures and reproduces both maintained patches exactly. These
  are boundary checks, not emulation/GPU or successful game-state evidence.
- See [selected resources/licenses](RESOURCE_AUDIT.md) and the exact 274-source
  manifest. N64 packaging retains component/source notices and GPLv3 text;
  candidate preservation includes original and complete patched source archives.

Selected core: [Mupen64Plus-Next at 15bada85906593376e8fd697f1b0b3af3a20f512](https://github.com/libretro/mupen64plus-libretro-nx/tree/15bada85906593376e8fd697f1b0b3af3a20f512).
[source-lock.json](source-lock.json) records its complete root tree identity,
recursive gitlink and the inspected file blobs. The repository has no .gitmodules
at this pin; do not invent a dependency URL for the gnulib gitlink. Full component
and selected-resource license audit/build reproduction remain pending.

The [Android makefile](https://github.com/libretro/mupen64plus-libretro-nx/blob/15bada85906593376e8fd697f1b0b3af3a20f512/libretro/jni/Android.mk)
selects the AArch64 dynarec, enables LLE/parallel components on ARM64 and chooses
GLESv3 with GLES3=1. Merely setting optional flags to zero on its command line
does not establish their exclusion: inspect the selected expanded graph. Its
Application.mk defaults to four ABIs/android-21; ntx must explicitly select
arm64-v8a and its supported API, output paths and 16 KiB ELF alignment. No upstream
CI template is a complete ntx reproduction recipe or actual device qualification.

The Rust n64 module now validates .z64/.v64/.n64 byte orders by cartridge magic,
normalizes a bounded private in-memory copy and fingerprints canonical bytes.
It does not derive/reallocate GameIds or scan/download games. Its own digital C
buttons and shoulders map to the pinned core's default, non-alternate libretro
mapping, independent from the GameCube pad. Cancellation and simultaneous input
have host fixtures. This module is preparatory policy, not a running host adapter.

Inspected actual boundaries:

| Boundary | Finding and implementation consequence |
| --- | --- |
| libretro.c: retro_load_game | need_fullpath=false; copies content; formats save memory before initialization. Load ordinary data after formatting and before first execution. Passing a real filesystem path also probes .ndd/.gb/.sav sidecars; use scoped cartridge bytes and no path to avoid unintended subsystem discovery. |
| libretro_memory.h / device headers | RETRO_MEMORY_SAVE_RAM exposes one 296960-byte image: EEPROM 2048, four Controller Paks of 32768, SRAM 32768, FlashRAM 131072. Rust parses these distinct regions. This is not a universal native-save converter; byte order/format validation of external saves remains due. |
| Controller mapping | Default core maps joypad B/Y to N64 A/B, Select to L, R2 to R, L2 to Z, X/A/L/R to C-up/down/left/right. Fix core options accordingly; do not use alternate mapping or emulate C buttons as a GameCube stick. |
| retro_serialize | Schedules a core job and switches coroutines until completion; returns retro_savestate_result. Caller owns buffer length and quiescence. |
| retro_unserialize | Waits for completion but returns true unconditionally after the wait. It cannot acknowledge successful restore as-is; repair/fixture this selected core boundary before offering resume. |
| savestates.c | Includes device protocol and FlashRAM page buffer/mode/status. Excluding ordinary filenames does not prove pending-save safety. Audit all cached/pending writes and failed-restore cleanup before enabling checkpoints. |
| GLES/threading | GLSM owns hardware-context callbacks; threaded unload switches coroutines and joins emuThread. Actual EGL context/thread, framebuffer, audio, option callbacks and completion-bearing shutdown are still required in the Rust host. No UI-thread frame loop. |

No core binary is downloaded or packaged, no RetroArch frontend is imported and
no real cartridge is read in this increment. The app explicitly refuses the N64
route until an execution adapter exists. Next work remains the linked ntx package
and protected migration qualification, then the Rust host/GLES/audio/core package
vertical slice. Ordinary N64 save durability, controllable play and cross-system
continuity remain open acceptance outcomes.

The root license is GPL version 2 text; inspected Mupen device files declare
GPL-2.0-or-later and GLSM declares MIT terms. These observations are not a complete
combined-distribution license conclusion. Retain all selected component notices
and corresponding source before packaging or public binary consideration.

## Source build preparation

The selected archive was verified against all 2275 upstream Git blob identities.
Its one Windows project CRLF export expansion was reversed to the exact pinned
Git blob; the source manifest records SHA256 for every original file. The
maintained patch and patched-file manifest separately identify the host-boundary
changes: checked state buffers/result, mechanical optional logger fallback,
instance-owned config/data directories and completed coroutine allocation release.
`cargo xtask mupen-source-check` validates that full active graph.

`cargo xtask build-mupen` uses the existing reviewed NDK, one job and isolated
build/ntx-mupen outputs. Command-line GLES3/ARM64/HLE options override upstream
make defaults. The expanded graph has 274 compiled sources and no parallel RSP,
parallel RDP, Angrylion or gnulib dependency; build execution/qualification is still
pending. Original optional dependencies remain pinned as excluded source.

The selected graph includes GLideNHQ xBRZ, whose source/header explicitly carry
GPLv3 and a MAME exception that does not apply to ntx. GPLv3 text is retained in
native/LICENSES/GPL-3.0-only.txt. Treat this selected core combination under its
actual applicable GPLv3 obligations, retain component notices and deliver exact
corresponding source/build inputs; root-license summaries are insufficient.
No public binary release or store/legal commitment is authorized.
