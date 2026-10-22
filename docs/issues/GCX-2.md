# GCX-2: Game-first library, safe resume and Home Screen games

## Outcome

Extend the qualified gcx engine slice into the same quality of everyday experience
as the owner's PSX player: choose a game, leave it, open another, return safely.
Use the existing gcx worker after GCX-1 viability; do not spawn another worker.

Implement the shared Rust GameId / SaveProfile / SavedSession / LiveRuntime boundary
in actual launch/persistence code. The first policy has one live runtime and many
saved games; a permanent singleton session identity is not acceptable.

Deliver local scoped library/import and validated compressed-disc access, stable
relink, multi-disc identity and switching, comfortable GameCube control/editor
profiles, native pause/volume and lifecycle, and Android pinned game shortcuts.
Do not expose a core catalog or arbitrary paths via launch intents. Same-game warm
return preserves state; other-game launch safely checkpoints/stops/switches.

Choose and qualify raw-card or GCI-directory storage with region and card identity.
Use one writer per save profile through teardown. Prove ordinary in-game save/reload
separately from automatic machine resume. Inspect exact native state/card behavior;
retain incompatible or stale checkpoints and refuse silent progress rollback.
Implement bounded, durable publication with recovery after worker death or failed
writes. Shared-card cross-title behavior needs an explicit coherent policy, not
an accidental global writable directory. Existing user saves must not be formatted.

Verify actual A -> B -> A, process restart, update, fold/rotation, audio focus,
partial/full L/R, layout persistence, cold/warm/repeated launcher taps, unknown IDs,
missing-file recovery and normal card/current-state consistency. Use scratch
fixtures for failure tests and private recoverable backups for updates.

Retain portable iOS launch semantics without claiming an iOS emulator exists.
Stock-iOS execution, Wii and multiple resident processes remain outside this pass.
Full acceptance and evidence distinctions are in docs/ACCEPTANCE.md. Keep the issue
open for Master review; distinguish unrun rows and real blockers from completion.
