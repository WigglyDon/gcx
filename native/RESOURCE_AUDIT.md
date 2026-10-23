# Initial Android resource selection

The exact Dolphin source pin and `Data/Sys` tree are recorded in README.md.
`resources.txt` is the explicit non-settings allowlist. Packaging additionally
selects `GameSettings/G*.ini` and `GameSettings/D*.ini` for GameCube retail/demo
configuration. Those are upstream source configuration, retain Dolphin's terms,
and do not enable optional user cheats.

The GameCube font binaries are generated from Droid Sans by upstream's
`docs/gc-font-tool`; `GC/font-licenses.txt` identifies Google authorship and the
Apache 2.0 terms. The OSD font retains its Bitstream Vera copyright and license.
Copy those notices beside their fonts. Do not replace these with extracted IPL
fonts. This selection includes no IPL, DSP ROM/coefficient image, Wii NAND,
title database, disc banner, logo, platform artwork or startup artwork.

The initial route requests skipped IPL and DSP HLE. Any missing-resource failure
must be diagnosed against that route before expanding this allowlist. Resources
needed for optional cheats, LLE, Wii or a donor frontend are not admitted by a
missing-file message alone. Inventory and hash actual staged assets and the APK;
an allowlist in source is not proof of what a package contains.

The source checker also requires every selected resource, including discovered
GameCube G/D GameSettings INIs, to be tracked in the pinned source checkout.
Git path arguments are literal: a local filename containing a wildcard cannot
stand in for tracked inputs. Actual refusal tests injected an ordinary untracked
INI and a literal wildcard filename, verified rejection, then removed only those
owned fixtures and rechecked the source graph. No new packaged assets are admitted.

The ABI7 host configuration identity additionally records checked-profile-read1;
resource bytes remain the pinned selection. Native saved-image reads use Rust's
owned profile paths and exact expected image bytes rather than an unacknowledged
native fallback. This adds no packaged BIOS, firmware or startup artwork.
