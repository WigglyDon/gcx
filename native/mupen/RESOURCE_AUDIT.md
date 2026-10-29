# Selected N64 code and resources

This audit applies to pin 15bada85906593376e8fd697f1b0b3af3a20f512 and
the 274 compilation inputs in selected-sources.txt. The complete 2275-file
source identity and each maintained modification are checked separately.
The Android recipe uses ARM64/GLES3/GLideN64/HLE and one build job. Parallel
RSP/RDP, Angrylion, CXD4 and its optional gnulib gitlink are excluded.

| Component | Inspected provenance and packaging treatment |
| --- | --- |
| Mupen core and HLE RSP | Per-file GPL-2.0-or-later notices, upstream LICENSES/author lists retained. Boot uses original HLE PIF/CIC implementation; no proprietary boot ROM or firmware payload is copied into the APK. |
| GLideN64 / GlideHQ | Root and per-component notices retained, including Rice1964 GPL-2.0-or-later filters and included gles2n64/Glow/FXAA/queue notices. Generated combiner/shader strings and numeric blue-noise/Z tables come from the exact pinned source; no game texture pack or downloaded shader is added. |
| xBRZ | Selected TextureFilters_xbrz.cpp and its header explicitly require GPLv3. The MAME exception is inapplicable. GPLv3 text is supplied; the combined N64 package is treated under the applicable GPLv3 obligations rather than summarized as GPLv2-only. |
| libretro-common / libco / GLSM | Individual file terms and copyright notices retained with corresponding source, including MIT/BSD and GPL-covered modules. No RetroArch frontend, core marketplace, account or downloader is imported. |
| zlib / minizip / libpng | Embedded source copyright/license notices retained (including zlib.h/png.h); copies of source notices accompany the package and exact source archive. These components do not supply game content. |
| xxHash / MD5 / resampler | BSD-2-Clause xxHash and Aladdin's permissive MD5 notice, plus selected libretro resampler notices, retained. |
| TextDrawer | Pinned GLideN64 source identifies OpenGL Programming Wikibook provenance. The pin has stripped its FreeType/glyph loading; no font.ttf or FreeType library is selected or packaged. This upstream provenance remains visible in the source notices; public binary/store clearance remains separately gated. |
| ntx presentation | New Rust EGL/GLES/audio and N64 input/session/persistence code is original MIT work. Button glyphs are small original code-defined shapes, with no copied font, logo, cartridge art or game screenshot bundled. |

APK staging preserves every upstream file named LICENSE/LICENCE/COPYING/NOTICE/
AUTHORS/COPYRIGHT recursively, plus source-header notice files for dependencies
whose license lives inside code. The private candidate includes both the exact
original archive and the complete verified patched source, first-party Git
bundle, Rust vendor set, build/tool bindings and reproduction instructions.
Patched files retain their upstream terms and dated ntx change notices.

The source archive contains optional upstream components that are not compiled.
Their original notices remain intact; their presence is not a claim they are
enabled. No cartridge, save, firmware, signing material or private device data
belongs in this inventory. Public binary release and store/legal commitments
remain outside this implementation authorization.
