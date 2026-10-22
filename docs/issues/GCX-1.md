# GCX-1: Qualify Dolphin and run a Rust-hosted GameCube slice

## Assignment

Initial implementation assignment, authorized after the local foundation is
published and the actual worker acknowledges. Read AGENTS.md, product, architecture,
GameCube feasibility, distribution, operations and current Master amendments.
No second worker is requested. The seed itself has not started an engine build.

Produce a reproducible native engine integration and one real GameCube game inside
our Rust Android host. This is not an emulator survey, a document-only investigation
or permission to replace the product with the standalone Dolphin app.

## Work

1. Reconcile local/remote source and unknown changes; verify separate gcx identity,
   available tools and actual shared-host capacity before bulk downloads/builds.
2. Inspect official Dolphin, starting from the recorded observation, then pin an
   appropriate complete revision/submodule/resource graph. Preserve GPL lineage.
3. Inspect actual native lifecycle/thread/render/audio/input/state/card ownership.
   Choose a small C ABI/direct engine boundary; libretro is not required. Targeted
   core repairs and bounded new Rust services are permitted. No managed app layer.
4. Source-build ARM64 with only needed frontends/resources. Preserve native JIT,
   accurate timing and backend behavior. Confirm real exported symbols and callbacks.
5. Implement a Rust-hosted executable slice with a game picker, native game picture,
   usable GameCube touch including partial/full L/R, audio, pause and clean shutdown.
   Existing donor emulated boot is the initial no-external-IPL route to test.
6. In an authorized device window, prove moving gameplay, simultaneous input,
   audible sound and an ordinary card save loaded after a fresh process. Record
   measured emulation speed, audio interruptions, memory and available heat data.

One owner-selected local GameCube title is needed for gameplay. Ask only when
necessary; do not publish its bytes/paths or scan unrelated directories. Missing
input/custody does not block independent source/build work. No game has been chosen
by the seed. Use isolated scratch save profiles while qualifying.

## Exit and continuation

Hand back source/build/resource identities, exact reproducible commands, modified
native boundary, actual test outcomes and concrete blockers. Builds do not prove
16 KiB runtime, iOS, gameplay, cards or snapshot recovery. Keep issue open for Master
acceptance. Once the slice is viable, continue the already defined GCX-2 work in the
same worker without ceremonial idle time; read fresh decisions before integration.
No unrelated cleanup, whole-core rewrite, second process pool, Wii scope, store
submission, public binary release or spending follows from this assignment.
