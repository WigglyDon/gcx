# GCX-4: Optional paused-resident games after measured single-runtime success

## Not dispatched

Preserve independent durable sessions now; do NOT implement this feature as a
prerequisite for the first GameCube player. A future Master/owner assignment must
activate it after single-runtime continuity is proven.

Possible Android feature: Keep another game ready. One game plays while another
is paused resident; Android may reclaim it, in which case durable resume restores
it. Distinguish saved sessions, resident processes, tasks and simultaneous execution.
No unlimited-running promise, root, service abuse or fake audio keepalive.

Before implementation, measure baseline checkpoint-switch cost against bounded
resident slots, RAM/native/GPU/JIT use, thermals and foreground audio/pacing. Each
worker has one core and exclusive card/profile ownership. Core globals, renderer
ownership and safe teardown must be respected. Process/task identity never becomes
GameId or a shortcut target. Recover after OS worker kill, full restart, missing
storage, failed checkpoints, duplicate launch requests and stale-card conflicts.

A separate single-engine worker required by a donor lifecycle is not equivalent
to this multi-resident feature and can be justified independently. iOS may retain
only saved-session continuity; no arbitrary per-game process promise. Do not
expose a toggle until there is measured benefit and tested recovery.
