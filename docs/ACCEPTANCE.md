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
