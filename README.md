# gcx

**Your GameCube games. Open, play, leave, return.**

A game-first, Rust-first native GameCube player from Machine Rhythm. A separate
sister application to PSX and xbx, starting with stock Android ARM64 foldables.

The intended experience is a small local collection, comfortable GameCube controls,
dependable per-game progress and automatic resume, and optional Home Screen icons
that open individual games directly. No core-selection catalog, accounts, game
uploads or mandatory network connection.

## What exists today

**GCX-1 implementation in progress.** A Rust host now builds against pinned Dolphin
and passes an original GameCube CPU fixture, native input readback, atomic card
commit and teardown checks on Linux. The Android Rust player and narrow native
adapter link on ARM64. A private development APK passes ABI, alignment, signature
and resource checks. Visible gameplay, audible sound, ordinary in-game save/reload,
Android locking and automatic resume are unrun. See
[native reproduction and limits](native/README.md).

GameCube is the initial scope. Dolphin's Wii capability is not a promise of Wii
support in gcx. A Wii expansion, iOS implementation and multiple resident games
are separate decisions. Google Play and ordinary Apple App Store distribution
remain intended goals, not approvals or guaranteed performance.

## Begin

Read [AGENTS.md](AGENTS.md) and the [current state](docs/state.toml), then use the
[verified GitHub receipt](docs/github-state.toml) to locate actual live issues.
On a locally bootstrapped checkout, run `bash scripts/start-codex.sh` to begin the
included implementation assignment with your normal Codex permissions.

| Document | Purpose |
| --- | --- |
| [Product](docs/PRODUCT.md) | The game-first experience and boundaries |
| [Architecture](docs/ARCHITECTURE.md) | Native ownership, per-game sessions, cards and checkpoints |
| [GameCube feasibility](docs/GAMECUBE_FEASIBILITY.md) | Dolphin, boot, input, formats and mobile risks |
| [Acceptance](docs/ACCEPTANCE.md) | Separate build, game, save and lifecycle evidence |
| [Distribution](docs/DISTRIBUTION.md) | GPL lineage, bundled resources and store requirements |
| [Operations](docs/OPERATIONS.md) | Shared workstation, source and device safety |
| [PSX transfer](docs/PSX_TRANSFER.md) | Selective reuse without a product merger |
| [Sources](docs/SOURCES.md) | Observed primary sources and precise limits |
| [Master seed](handoff/MASTER_SEED.md) | Complete standalone fresh-chat handoff |
| [Local setup](handoff/LOCAL_SETUP.md) | Publish the foundation and start Codex |

## Engineering and source

New substantive first-party application, host, runtime, input, UI and persistence
logic is Rust. Borrowed C/C++ may be retained, repaired and replaced where justified.
A vendor directory or existing API is not an instruction to accumulate wrappers.
We borrowed an engine to avoid rebuilding a whole console, not to abandon control.

Original seed work uses [MIT](LICENSE). Incorporated code keeps its actual license;
Dolphin declares GPL-2.0-or-later. Our MIT license and a C ABI do not remove the
combined distribution's obligations. See [THIRD_PARTY.md](THIRD_PARTY.md).

No proprietary games, Nintendo firmware, private saves, keys or copied brand
artwork are supplied. gcx is an independent project, not affiliated with Nintendo
or the Dolphin project. The name is a project choice, not a trademark clearance.
