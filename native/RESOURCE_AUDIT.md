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
