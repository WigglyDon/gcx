# Source lineage and dependency obligations

Original ntx Rust host, player, input, storage and build logic uses [MIT](LICENSE).
The foundation's supplied xbx bootstrap attribution remains intact. Landscape
geometry adapts selected public PSX MIT layout concepts at
`4277d4ca651a4c9c2e8e88113a0ef092e330f01c`; its copyright notice is retained in
LICENSE and the packaged MIT notice. See [transfer review](docs/PSX_TRANSFER.md).
The alpha icon also adapts the public PSX MIT button composition at
`123e831e4afb6c7cec2d8e05a030e3cd82a72bba`; the same notice is retained.
No private sibling data or emulation/persistence code is reused.

The engine is official [Dolphin](https://github.com/dolphin-emu/dolphin), pinned at
`cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b`, with a maintained patch in
`native/patches/0001-direct-adapter.patch`. Its GPL-2.0-or-later terms and authorship
remain applicable. `native/COPYING` retains upstream's licensing statement,
including its aggregate GPLv3 compatibility statement; the GPL text for the adapter
is retained in `native/LICENSES`. Packaging also preserves upstream's full license
directory and dependency-specific terms. The native
Host adapter derives from the pinned MainNoGUI implementation and retains GPL
attribution. New donor-owner repairs retain the donor file licenses and recorded
change provenance. A C ABI does not provide license separation.

`native/dolphin-gitlinks.tsv` records every root gitlink; `native/dependencies.txt`
selects the materialized build inputs. Rust crates are fixed by Cargo.lock and
retained registry checksums. Packaging preserves dependency notices, with missing
crate-package license texts supplemented from their exact VCS revisions under
`native/rust-notices`. These are source notices, not copied application logic.

The staged GameCube font/settings selection and rights are described in
[native/RESOURCE_AUDIT.md](native/RESOURCE_AUDIT.md). Font notices accompany the
selected assets. No proprietary game, firmware, startup artwork or private save
is part of the app source or staged resources. Staging is not an APK audit.

Reproduction and actual limits are in [native/README.md](native/README.md).
Corresponding-source inputs, toolchain/resource identity, actual packaged notices
and a recoverable binary/source pair remain mandatory before distribution.
Public binary release and store submission are separate gates. No affiliation,
endorsement or trademark rights are claimed.
