# Evidence and acceptance

There are no earned gcx runtime results at seed creation. Each report binds the
exact source, patches, resources, build identity and tested path. Source inspection,
automated host tests, APK checks and owner gameplay are separate evidence classes.

## GCX-1: native engine slice

Verify a complete reproducible Dolphin input pin and licenses. Build an Android
ARM64 native component with irrelevant frontends disabled. Prove actual exported
ABI, resource lookup, callback lifetimes, native threading and orderly shutdown.
The slice must use our Rust host, our separate app identity and a native render/
audio/input path. A standalone donor launch is a comparison, not completion.

In a fresh authorized Fold window, launch one owner-selected GameCube game with
the chosen boot path; observe moving gameplay, responsive simultaneous input and
audible sound. Demonstrate ordinary in-game save, stop the process, then load that
save through the game. Record runtime errors/performance honestly. Lack of a game
or device window permits build work, not invented hardware acceptance.

## GCX-2: the promised player experience

- Library: persistent entries, content-based detection, scoped import, cancellation,
  large/compressed inputs without whole-file allocation, and missing-file recovery.
- Controls: independent Control/C Stick axes, diagonals and dead zones, A/B/X/Y/Z,
  Start/Pause, simultaneous direction/action and distinct partial/full-click L/R.
  Exercise interruption, stray touches and any actual controller disconnect.
- Layout: tiny initial surfaces, actual drawable/cutouts, outer/inner screens and
  rotations; aspect-correct picture clear of controls; editor Save/Cancel/Reset
  and saved profiles survive restart/update.
- Ordinary progress: valid card save/reload in a fresh process, both supported card
  operations and corrupt/unavailable input handled without reformatting data.
- Automatic resume: a known scene and pause intent return after fresh process and
  app update with verified state/card generation; absent/incompatible/stale state
  is retained and explained, never silently rolled back or discarded.
- Switches: A -> B -> A and B -> A -> B preserve independent cards and scenes,
  with no duplicate native owner, crossed input, audio bleed or shared save path.
- Shortcuts: actual launcher confirmation and real cold/warm taps, repeated requests,
  another active game, invalid IDs, relink, update survival and unpin without deletion.
- Multi-disc: one identity/card profile, remembered disc and a requested runtime swap,
  with no reset or hash-based creation of a second unrelated game entry. If no
  suitable input exists, explicitly leave this acceptance row unrun.
- Resource behavior: bounded repeatable play, measured emulation speed separately
  from game and present FPS; available CPU/GPU/memory/thermal observations, audio
  interruptions and cold shader-compilation versus warmed behavior.

Protect live progress first. Storage failure, corruption, killed writer, interrupted
checkpoint publication and stale-card restore tests use isolated fixtures/copies.
No deliberate owner-disk exhaustion, damaged only-copy saves or kill during an
unbacked in-game write. Observe platform behavior without assuming onStop/onDestroy
will always run before process death. Preserve the last accepted checkpoint.

## Earned scene return and next guarded qualification

ABI8 source `0e5417e328383e0ba7ff0d3b0d5cf01bb3f5b744`, installed APK
`c7194951d9420e45622c7eefb29baaa5365fec694e6cefa015a7049dc724e5c9`, subsequently
earned the bounded Pixel4 intentional-pause cold-process scene return on2026-10-24.
Its93,250,868byte payload and complete coherent generation were independently
validated/backed up before retirement. New PID/start identity, paused restore until
Resume and matching unsaved bridge position/camera were verified. The owner
explicitly confirmed restored sound, movement/action, C Stick and control feel.
Normal Stop occurred only for final cleanup after this evidence. Final backup,
unchanged ordinary card, clean joined Stop and verified return are recorded in
[the ABI8 handback](https://github.com/WigglyDon/gcx/issues/2#issuecomment-6379745602).
This supersedes ABI8's earlier unqualified scene-return status, while preserving
the historical ABI7 evidence below. Automatic paused=false return, two-game library
return, real shortcuts, full controls and a new ABI8 ordinary-save/reload remain due.

ABI7 source `dae7e3f1326fb320a231efa5ffd89c853482f417`, APK
`261a4976606117c29b70df2a7485f48188a64067d6224c5d1721b546c3763b75`, earned
the bounded Pixel4 intentional-pause cold-process scene return on2026-10-23.
Complete checkpoint/card/SRAM/session generations were validated and privately
backed up before controlled retirement. New process/start identity, retained pause
intent, and actual Vulkan return to unsaved bridge and villager-dialogue scenes
were observed. Owner confirmed the second scene, audible resumed sound, controls
and clear Home handle, with a slight reload hitch. No Stop or card-menu load
prepared/substituted for those returns. See [device evidence](https://github.com/WigglyDon/gcx/issues/2#issuecomment-6371910629).
The automatic paused=false variant remains unrun. This does not transfer ordinary
card-save acceptance from older builds, qualify broad bar transitions, or qualify
the subsequent ABI8 input/UI increment.

[Master review6356548750](https://github.com/WigglyDon/gcx/issues/2#issuecomment-6356548750)
accepts the preserved ABI7 increment for qualification, not completed GCX-2
acceptance or phone custody. Keep the existing prepared runtime and backups;
unchanged binaries and completed host campaigns need no rerun.

The unchanged ABI8 candidate has now earned its separate single-scene row above.
After a fresh owner Pixel4 offer and successful exclusive admission, continue the
remaining library/shortcut rows without repeating that completed row. Preserve
both earned pairs and backups. Any future changed candidate's resume check
still requires checkpoint → genuinely new process → exact controllable scene return.
Verify completed payload and durable session-reference publication, then back up
the quiescent generation before controlled process retirement. Confirm actual
Vulkan picture, usable input, restored sound and pause intent. An intentionally
paused scene must stay paused until Resume. Never use Stop game to prepare this
test: it clears the scene reference. Game-menu card loading is separate evidence.

Only after single-game scene return succeeds, use the remaining admitted window
for Wind Waker → Twilight Princess → Wind Waker and actual game shortcuts. Keep
pin request, OS pin observation and real cold/warm/repeated taps distinct. A
second-title failure preserves the first checkpoint and leaves switching unqualified.
Retain ordinary in-game-save/clean-Stop/reload as a separate resulting-build check.

Record actual phone checkpoint bytes, capture/publication and restore timing
intervals, observed process memory and audio interruptions. Label observation
bounds where unchanged diagnostics cannot separate internal phases. Observe the
Home handle during relevant transitions without repeating broad layout approval.
Protect cleanup time: a single-game return is a useful handback, with later rows
left unrun. Retain a failing checkpoint/error and symbols; return the device before
heavy repair. Fold/siblings stay untouched; no public binary release is admitted.

## Input responsiveness

Prove touch delivery at the actual complete native GameCube pad read, separately
from a game's movement/acceleration and from display latency. A press/release
between host ticks must reach one native poll; a held control must remain held.
Latest analog position, both sticks, independent partial L/R travel and clicks,
simultaneous pointers and independent releases must pass. Interruptions and real
geometry changes cancel held and unsampled input. Equivalent inset measurements
must retain gestures and shared draw/touch mapping. Logical input has no timed
visual interpolation or extra host-frame wait.

Use focused Rust regressions and the native `gcx-probe --input-poll` fixture.
On a future device window, collect per-runtime bounded event-to-dispatch and
dispatch-to-pad age histograms, plus native pad-poll cadence. Invalid source
timestamps are explicit; intentional pause/departure gaps are excluded. These
software timestamps are not touch-to-photon, character response, FPS or a hard
real-time guarantee. Confirm real touch/control feel with both actual games under
ordinary play; do not infer hardware latency from host timing or a build.

## Release gates distinct from playable acceptance

App Bundle delivery, current target requirements, 4/16 KiB native runtime support,
accessibility for the custom interface, appropriate storage/background permissions,
complete source/notices and a truthful privacy model remain release qualification.
Stock-iOS gameplay and store rights are separate. No binary/store publication is
implied by a development APK or upstream store listing.

Full handback: source/modified-engine identities; build and corresponding-source
reproduction; actual game/boot/render/audio/card/state/shortcut results; measured
limits; preserved data and verified device return. Publish only sanitized summaries.
Do not claim broad compatibility, battery superiority, perpetual residency, or full
completion from build success. Keep acceptance pending until Master review.
