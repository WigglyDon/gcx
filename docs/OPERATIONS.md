# Working environment and delivery safety

## Independent project

Work only in the assigned gcx checkout. Proposed Android application ID:
`com.machinerhythm.gcx`; scheme `machinerhythm-gcx://play/<GameId>`.
Use a separately preserved local signing identity and private storage namespace.
Never install over PSX, xbx, Dolphin or another app; do not borrow their signing
keys or mutate their user directories. These names are design choices, not proof
of registration. No studio registry admission or legal formation is implied.

Inspect actual Git status, remote, history, issue and worker acknowledgments before
starting. Preserve unknown/uncommitted work. Normal reviewed gcx commits/pushes are
within scope; no force push, destructive reset or automatic deletion of others'
work. The initial worker runs in the seeded checkout. A fresh Master evaluates and
guides that worker; it does not spawn another merely because the chat is new.

## Shared workstation capacity

The owner uses a shared development workstation that has experienced real storage
allocation pressure. Before substantial source downloads, submodules, native
builds, archives or backups: read current custodian policy and actual available
capacity, metadata/allocation reserve on Btrfs where applicable, competing jobs
and expected peak growth. Do not infer sufficient capacity from one `df` number.
A small seed-copy check is not admission for a multi-gigabyte engine build.

Respect protected margins and schedule bounded build concurrency. Do not inherit
another product's numerical reserve as a measured gcx quota. Reuse authorized
existing SDK/NDK/tool caches without modifying their ownership or clearing them.
No new server, paid capacity or background service is granted by this seed.

No global cargo clean, git clean, shared cache purge, filesystem balance/repair,
or erasing unrecognized files. Remove only inactive, verified reconstructible
gcx-owned outputs after preserving source, failure evidence and reconstruction
instructions. `.local` is private durable state, not disposable scratch. Keep
accepted APK/source pairs, active candidate, recovery baseline and private backups.
Do not multiply full expanded source bundles merely to inspect or revalidate them.

Use Rust xtask for first-party build/verification orchestration once implemented.
Reuse upstream CMake/Gradle/NDK and dependency build tools where required; this does
not introduce a non-Rust product layer. The initial Bash bootstrap only publishes
this small project foundation and launches the existing Codex installation.

## Device custody

Personal Fold access remains on request under the current stock-device and
exclusive-transaction procedure. Read it locally when a device action becomes
necessary. A connected/unlocked phone or historical authorization is not current
custody. Build and prepare off-device; request a bounded window only for the ready
candidate. Respect other projects' locks; never preempt or replay actions after a
connection changes without fresh verification/admission.

No rooting, bootloader unlock, wipe, system-package disabling, security weakening,
console firmware flashing or unrelated app changes. Backup actual gcx data before
updates/migration, retain the signer/application ID and a workable restore route.
No active progress is sacrificed for a test. Return the device in the agreed state
and verify screen/lock/lease release as required; disclose any unresolved return.

### gcx-local test helper after Master6386389871

`scripts/android_packet.py` is the versioned repair of gcx's existing private
JSON packet helper, not a device service or application layer. Its private launch
wrapper must pass the admitted alias, a new private evidence directory,
`GCX_DEVICE_TASK`, and an absolute monotonic `GCX_OUTER_DEADLINE_MONO` established
before the outer timeout starts. Run it only inside the existing exclusive
`android-device transaction`; every nested command inherits that transaction FD.
It does not grant custody. The earlier of the outer deadline and the helper's
17-minute limit controls a five-minute cleanup reserve, including one minute
reserved for final return. New tests and owner waiting end at the cleanup cutoff.
For unattended packets set `GCX_AUTOMATED_PACKET=1`: the input loop immediately
enters its finalizer at that cutoff (or forced cleanup), without waiting for an
agent response. The prepared task-local wrapper may then run the existing bounded
recovery inside the SAME transaction and ORIGINAL hard deadline, preserving the
pending receipt first. It must leave real backup/validation/return time and refuse
recovery after a timed-out transport. There is no renewed lease or deadline.

The two focus metadata reads share one wrapped ADB call, reducing repeated USB
wrapper overhead without caching either observation. Own UI input/captures require
freshly matching resumed Activity and focused
window on the current live display. Use `dumpsys window displays`, not the first
focus line in a full window dump: retained ANR snapshots can precede the live
section. Android13 omits the global top-display ID from this subcommand; one
live display is unambiguous, while several require an explicit top-display ID.
Framework button taps use the currently dumped layout, with a second
focus observation before the single tap. Owner-assisted Pause uses a real owner
tap. For owner-authorized automated checks, one named native button tap is bound
to a just-captured, agent-inspected own image, matching live display frame/rotation
and focus, observed bounds, and a thirty-second freshness limit. Each tap consumes
that capture; historical coordinates and batches are refused. Cleanup permits
only native Pause and framework Stop. Optional `press_ms` of0..250 is a bounded
stationary game-button test contact, not product input dwell; Pause cannot hold
across the new dialog window. A mismatch ends new work; unexpected app content
is not retained. Home, nonsecure wake and ordinary app entry are separate named
steps. An admitted autonomous packet may explicitly authorize its initial ordinary
gcx entry from the freshly observed foreground, preserving that app's state; this
exception is consumed by a launch or own-UI observation and never overrides a
later mismatch. Subsequent unexpected foreground ends new work. Raw UI/capture/hash
batches are refused. Hashing uses nonempty, validated literal own-file arguments,
never a quoted `sh -c` string or an input-less hash command.
Ordinary entry uses stock `am start -W` within the operation timeout, rather than
assuming that a scheduled launch already has a focused window. Missing/ambiguous
focus still refuses; retain at most eight filtered metadata failures privately.

The helper does not kill an unverified live writer to simulate Stop. It records
pending recovery before releasing custody if normal cleanup cannot complete.
For an authorized fresh-process checkpoint test, its named `retire_checkpoint`
operation requires an unchanged complete backup and independent inspector verdict,
matching intent/game/profile/checkpoint, fresh hashes of the actual checkpoint,
card, SRAM, generation and collection, observed Home foreground, exact current
PID/start identity and paused/publication evidence. Only then may it retire the
own process. It records that this is not normal Stop; cleanup cannot invoke it
as a substitute. A separately named cleanup-only `recover_checkpoint` keeps all
those proof gates and records controlled recovery, never normal Stop. Save actual
current hash/process/quiescence observations before retirement. Any changed data/process or missing quiescence refuses before
retirement and ends new test work. Raw process/UI batches remain refused.
Keep failed quiescence observations too, with distinct bounded evidence names;
missing proof must leave the writer intact and its return pending. A successful
archive inspection alone never authorizes process retirement or safe handback.
Timeout means no further device commands from that helper. Clean Stop metadata,
native join/profile release, own-client absence and physical display/keyguard
return are separate observations; a helper flag alone is not acceptance. Confirm
actual transaction/lease release after exit. Test guards off-device with
`python3 -m unittest discover -s scripts/tests -v`.
Allow a finite, deadline-bounded display/keyguard observation sequence after
Sleep; an immediate read can precede the asynchronous transition. If it never
settles, record a pending return. Keep stdin interactive for a live packet;
EOF deliberately finalizes the helper. Re-observe the actual app PID/start after
reentry before filtering logs; a Home return is not proof of process continuity.

## Private materials

Keep game images, original firmware, saves, checkpoints, card icons extracted from
private saves, device identifiers, raw logs/captures, signing keys and credentials
outside public Git. The user need not upload any of those to the Master chat.
Use only locally selected/authorized inputs and bounded directories; no broad
scanning of unrelated disks or sibling projects. Preserve original files unless
an exact separate cleanup assignment permits more.

Public evidence should be sanitized results, reproducible public-source commands,
code/build identities and qualified outcome summaries. Internal tests can use
first-party fixtures and lawfully redistributable homebrew with verified terms.
No game download or firmware provisioning service is part of gcx.

## Boundaries to report

Continue independent safe work while waiting for a game, firmware genuinely needed,
capacity or device window. Report the exact blocker and completed increments rather
than a claim of completion. Do not stop merely because borrowed source needs a fix.
Escalate a major new engine campaign, unresolved rights/security model, insufficient
resource authority or data-loss risk. No public binary release/store submission,
spending, account enrollment or weakened approval configuration is authorized.

The first alpha helper also recognizes a verified fresh install with no imported
game/collection, so closing the empty library does not require or claim native
Stop. The installer must prove package absence before setting
`GCX_VERIFIED_NEW_INSTALL=1`; never use it to relabel existing application data.

A cleanup-only `recover_pending_restore` covers a fully traced single paused
bootstrap that departed before its first restore/play. It keeps all matching
complete-archive/current-file/PID/start gates, requires checkpoint validation and
paused native boot, and refuses any restore attempt, playing state, capture,
second boot or truncated trace. It does not require a new capture that the product
correctly skips while restore is pending. Ordinary post-play recovery keeps its
publication requirement. At an already observed physical screenOFF/keyguard,
cleanup needs no wake or fabricated Home component. It is controlled process
retirement, never normal Stop/join acceptance. Keep original failed receipts and
reconcile them only after actual physical and kernel-lease return observations.

Fold captures must resolve the focused logical display to exactly one valid active
local physical viewport, then recheck it and focus after shell-v2 capture. No
first/default-panel fallback; discard pixels after a panel change. Keep numeric
physical IDs private. Return and screen-off recovery require every observed
physical `Display State` to be OFF; an inactive outer panel is insufficient.
Collection text uses shell-v2 status/stderr: a successful exec-out stream containing
a missing-file message is not proof of an absent collection. The current helper
has43 offline regressions covering these gates. The installed Alpha 1 APK is
unchanged by test-helper repairs.

Semantic button input must intersect its bounds with every ancestor container,
including ScrollView clipping, and preserve negative coordinates. Screen bounds
alone cannot establish a visible button. For the next alpha pass, prepare
automatic checkpoint backup and normal Stop/recovery inside the held transaction
before launch; the cleanup reserve must not depend on model turnaround or a
second admission. EOF finalizes an interactive helper; use an actual open PTY.

The owner requested automated tests without human control takeover. The bounded
fixed-footer fixture may drive native **Pause only** from the exact installed
source's actual current-process `present.ready` rectangle, with no covering native
framework menu, playing-state acknowledgment, live own Activity/window/display,
fresh single-use PNG and matching buffer geometry. It must refuse stale/missing
source geometry or a different foreground. This avoids guessing historical
coordinates while removing model turnaround from a known gcx control. Resolve
Resume/Stop through current semantic button bounds; retain/open the own-app
captures for visual review. This is not a general arbitrary-coordinate or guest
input route. Keep automatic backup/validation/Stop-or-guarded-recovery in the SAME
held lease, and wait finitely for coherent Home after the authorized stock
nonsecure wake before launching. A successful short functional fixture
does not qualify game speed, sound, control feel or extended lifecycle behavior.

A process-loss first failure must also retain gcx's OS exit/scoped crash evidence
and current data, even when there is no PID to Pause/Stop. The prepared post-exit
finalizer performs no UI, process kill or intent rewrite: validate actual file
paths/sizes against remaining budget, archive ordinary profiles/settings and the
selected referenced state, compare hashes before/after and recheck process absence.
Retain inactive state history in place and record the archive's exact scope. Inspect a referenced checkpoint only
against the actual device-local binding; preserve archive/errors on mismatch or
failed validation, and never imply normal Stop. Five new offline fixtures cover
absent runtime, newly appearing writer, size refusal, foreign binding and failed
state inspection. This follow-up path was added AFTER the first Fold world
window; that window's latest-generation backup remains unqualified. A deferred
read-only window is not permission to reacquire or to install another candidate.

After an accountable later return, keep the current canonical custody receipt
consistent: returned phase, released custody, resolved recovery and no active
resource/return/cleanup-error flags. Preserve earlier attempts verbatim with
hashes and explicit history/resolution references; do not leave a pending phase
beside returned=true. A free lock alone does not prove handback. Cite actual
client retirement, physical display/keyguard and kernel-release evidence. Product
failures or a deferred read-only backup remain product follow-up, not an active
lease; never relabel them as a successful native Stop to clear custody.

A readiness check must require the newest presented frame after the most recent
surface init/termination, invalid measurement or unavailable-frame marker, plus
current playing acknowledgment. An old present record before Home is not proof
of reentry readiness. Measurement bursts retain a reserved share of the fixed
sample budget for new frame evidence; never work around missing proof with a
historical coordinate. Post-exit backup includes all current ordinary profiles,
collection/settings and the selected referenced state, with independent state
inspection; inactive state history is inventoried and retained in place rather
than repeatedly copied into each bounded archive.
