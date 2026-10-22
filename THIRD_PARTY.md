# Source lineage and future dependency obligations

This seed incorporates no emulator code, firmware, game, font or native library.
Original foundation documents/scripts use the repository MIT license. The generic
bootstrap workflow is adapted from Machine Rhythm's supplied xbx seed; its original
notice is retained. Console-specific design is replaced for GameCube.

First engine candidate: Dolphin, https://github.com/dolphin-emu/dolphin, observed at
`cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b`. Its README declares GPL-2.0-or-later. This is a research binding,
not an incorporated dependency or complete inventory. Preserve exact per-file,
submodule, resource and tool licenses when integrating. Modified or translated
borrowed implementation retains its applicable obligations. No MIT/FFI exemption.

Future selective PSX Rust reuse requires a file-level review and source pin with
all applicable notices. Neither a public repository nor its root MIT license
turns every vendored file into original MIT code. No private source/history is
included. See docs/PSX_TRANSFER.md and docs/DISTRIBUTION.md.

Before any distributed native build, replace this candidate-only inventory with
actual source/resources/dependencies and corresponding-source reproduction.
Do not claim upstream authorship, affiliation, endorsement or trademark rights.
