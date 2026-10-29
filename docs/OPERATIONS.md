> Current authority: NTX-5 and amendment6505289270; read docs/NTX_HANDOFF.md first. Existing safety/ownership rules below remain. Old seed/probe recipes are historical references, not current dispatch. Comprehensive app/package/data rename is successor work. No old-path alias or retired-storage recreation.

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

## Select the next bounded phase

Start from the latest source-bound acceptance result and open symptom. Keep an
unchanged qualified APK/native engine when no demonstrated cause requires a code
change. Read retained evidence first; select a focused host check or synthetic
fixture for the affected owner. A missing capture, diagnostic gap or unavailable
cache is not a game freeze. Do not rebuild unchanged binaries, re-fetch a donor,
replay an accepted campaign or broaden the game/control matrix just to show work.

Before a growing phase, resolve the exact source, dirty donor patch, dependency
and resource graph, SDK/NDK, signer and output paths. Shared tools can outlive the
project name in their cache path; verify the current owner and exact contents
rather than copying or deleting them. Record private local resolution separately
from public reproduction instructions. Do not trust retired external storage,
historical backup receipts or stale symlinks as surviving inputs or present
admission. Refuse the affected phase instead of recreating a mount or silently
switching its outputs to another filesystem.

Use existing source/tool caches in place and task-sized output estimates. Retain
one selected working candidate and its protected recovery/source/symbol bindings,
plus any earlier evidence still needed for an open incident. Propose exact inactive
reconstructible outputs to their owner; a directory called cache, target or local
is not blanket deletion authority. Verify actual free space and memory immediately
before the admitted growing phase. A second host is a separate useful test scope,
not an implicit migration or permission to start another mutable product worker.

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

Select the accountable prior handoff and pending undo for the bound physical
unit/profile. The newest receipt across a shared multi-device store can belong
to another unit; its timestamp neither replaces nor blocks the target's handoff.

### gcx-local test helper after Master6386389871

`scripts/android_packet.py` is the versioned repair of gcx's existing private
JSON packet helper, not a device service or application layer. Its private launch
wrapper must pass the admitted alias, a new private evidence directory,
`GCX_DEVICE_TASK`, and an absolute monotonic `GCX_OUTER_DEADLINE_MONO` established
before the outer timeout starts. A legacy foreground invocation runs inside the
existing exclusive `android-device transaction`; every nested command inherits
that transaction FD. Its default17-minute helper limit, five-minute cleanup and
one-minute return reserve apply to that foreground route. They are not universal
limits for independently prepared finite lab jobs. Those jobs use the installed
custodian with their declared work/cleanup/return budgets, unchanged absolute
deadlines and aggregate counters. Neither route grants its own custody. New tests
and owner waiting end at the actual task's cleanup cutoff.
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

For the separately owner-authorized automated Wind Waker setup, Start/A are the
existing named guest-contact route; the Pause fixture alone grants neither. If
model/tool turnaround expires an inspected capture, keep the30-second guard and
report a setup refusal, not a game freeze. A task may bind a complete inspected
control region to the exact source/APK, settings and native geometry, then compare
every RGB pixel against its newly captured counterpart before that one contact.
Require identical frame dimensions and an unchanged complete control/clear border;
changed pixels, profiles, geometry or a covering framework menu refuse. This is
no coordinate scaling or generic recognition route. Keep current focus/display,
playing/presentation, buffer, freshness and single-use guards; retain/open each
whole frame before any further guest action. Limit Start to ordinary title entry,
A to the inspected existing quest route and one observed-world action, preserve
the Pause/cleanup capture reserve, and stop at the first refusal. Never overwrite
a quest or interpret an intro as controllable play.
Matching controls alone does not identify the guest screen. An observed title
can enter its attract intro before the next contact while every control still
matches. The scoped Start setup must independently compare its inspected guest
title regions in that same fresh capture before contact; an intro, fade or
different scene refuses. Keep those game captures/private reference pixels out
of public source. A bounded pre-input title wait holds no control and supplies
no readiness proof by itself. Record authored contact duration separately from
product input policy, and preserve the zero-duration result as its own evidence.
The standard helper retains its eight-image default. The explicitly admitted
gameplay packet may select ten at construction, within the current stock-lab
maximum; per-command input cannot raise that ceiling. Its18-minute hard limit,
five-minute cleanup and45-second final-return reserve remain the original bounds.
Each new named observation starts its own bounded command interval inside those
original absolute deadlines. An interval left over from the preceding capture
must not reject a later observation before any transport call. Never renew the
transaction/cleanup deadlines to repair that helper error, or label it a native
timeout/freeze.
The wrapper's command wait must consult the existing cleanup predicate before
waiting for more operator input. A completed focused sequence or first failure
that requests cleanup enters the finalizer immediately, without requiring a
separate Finish command or waiting for the fixed cutoff. Keep both original
absolute deadlines and the return reserve intact.

An offline host checkpoint-inspector timeout is distinct from an invalid
checkpoint verdict and a device-transport timeout. Retain the first timeout.
The existing same-lease finalizer may make one further host-only verification
attempt against the same archive, with a command limit of 30 seconds and at
least 140 seconds of the original Stop/return reserve remaining. Never renew
the lease/deadlines, recollect an unbudgeted archive, retry an invalid verdict,
or continue after a timed-out device transport. Keep verifier/source identity
and both attempt results; a later pass does not erase the first failure.

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

A later independent operator may acquire the physical lock before gcx's host
audit. Record the original physical return, normal transaction exit and later
audit separately. A distinct holder's exclusive kernel FD lock on the identical
mount/inode/path can corroborate the earlier release; retain that evidence and
make no current free-device claim. Do not query or interrupt the new owner to
obtain a free-lock observation, or rewrite that owner's receipt/lock metadata.

A readiness check must require the newest presented frame after the most recent
surface init/termination, invalid measurement or unavailable-frame marker, plus
current playing acknowledgment. An old present record before Home is not proof
of reentry readiness. Measurement bursts retain a reserved share of the fixed
sample budget for new frame evidence; never work around missing proof with a
historical coordinate. Post-exit backup includes all current ordinary profiles,
collection/settings and the selected referenced state, with independent state
inspection; inactive state history is inventoried and retained in place rather
than repeatedly copied into each bounded archive.

For presentation timing, `cargo run --offline --locked -p gcx --bin
gcx-present-times -- PRIVATE_OBSERVATION.json` analyzes already collected text;
it performs no device operation. The input is `{"snapshots":[{"start_ns":0,
"end_ns":1,"stats":"..."}, ...]}` with monotonic host read bounds and the raw
output of `dumpsys SurfaceFlinger --latency EXACT_OWNED_LAYER`. Collect only under
fresh admitted custody, with current own foreground, process/start, display and
playing evidence. Discover only matching gcx layer names; require exactly one
advancing history and rediscover after a surface transition. Keep names and raw
histories private. Never omit the layer argument (global animation statistics),
use `--latency-clear`, or change OS/emulator timing settings for this observation.

The decoder follows the public Android13
[FrameTracker format](https://android.googlesource.com/platform/frameworks/native/+/refs/tags/android-13.0.0_r3/services/surfaceflinger/FrameTracker.cpp):
refresh period, then at most 127 desired/actual/ready timestamp rows. It uses the
actual column, excludes zero/invalid/pending fences, deduplicates overlap, and
requires the previous completed high-water timestamp in every next history.
Lost overlap withholds interval statistics. Missing/nonadvancing history is not
zero FPS or proof of a hung game. The bounded report retains long intervals,
initial progress observation and a trailing no-progress lower bound; its first
interval begins at the baseline's latest frame, which can predate the first
read. These are completed surface presentation intervals, not unique game FPS,
emulation speed, audio quality or touch-to-photon latency. At most 32 snapshots
and 512 KiB input keep this an offline diagnostic, not a monitoring service.

The same decoder describes the slow tail using a fixed, strict `>40 ms`
threshold, its fraction of completed intervals, consecutive runs and the largest
number of slow endpoints in a half-open one-second window. It retains ordered
positions/relative offsets privately; publish only aggregate counts and coarse
positions. Elapsed-time thirds include the baseline interval. Display-period
bins are `<1.5`, `[1.5,2.5)`, `[2.5,3.5)`, `[3.5,4.5)` and `>=4.5` times the
observed refresh period. They do not count missed emulated frames. A refresh
change withholds pooled interval statistics; history and refresh validity are
reported separately. Keep the original raw observations and decoder reports.

A matched presentation packet keeps source, scene/camera, settings, sampling
method and window size fixed, with a declared interval between paired windows.
Use repeated no-Home control pairs as well as Home-return pairs. Record exact
private process/start/layer identity and actual refresh; rediscover the owned
layer after Home. A no-Home pair also needs overlapping bridge histories across
its gap, not just two individually complete windows. A deliberate Home surface
transition is a separate segment, never a fabricated continuous stream. Refuse
to pool changed-process/layer/refresh, missing-history and nonadvancing cases.
Keep the finite profiling bound inside the original new-work/cleanup deadlines;
no refresh/speed/audio/save-durability change is part of this observation.

Master6425756029 accepts the recorded matched stationary comparison as complete
for that hypothesis. Do not rerun it or add a Home-specific timing fix without
new evidence. The next prepared current-build packet concerns ordinary progress.

For that packet, retain current collection/settings and every ordinary profile
before play. Use only individually guarded, source/scene-bound contacts; title
Start/A setup is not a general gameplay macro. Inspect the actual game Save menu
and confirmation before navigating it. Require visible movement/action/C Stick,
a recognizable marker the game persists and an observed save-completion message.
The first normal joined Stop protects its current generation independently;
only then retire the empty app, prove absence and launch a new process through
the game's own menus. Reject checkpoint-restore evidence as ordinary reload.
Confirm the persisted marker, then protect the final generation and Stop normally.
Keep each stage's evidence distinct and retain the first failure. Missing/capped
diagnostics or expired observations are not proof of the original freeze.

This two-Stop packet declares finite aggregate image/byte bounds at construction;
individual commands cannot enlarge them. Existing packets retain their earlier
caps. Reserve bytes for both remaining Stop backups before admitting more images,
and reserve cold loading plus final cleanup before the first Stop. Stage helpers
inherit absolute deadlines and aggregate counters; no deadline is renewed.

Use the installed android-device finite recipe route for qualified Pixel4 lab
work. Prepare the private authority/input graph, admit/work/cleanup/verify hooks
and finite budgets off-device before `request`; use the actual returned job ID
for `status`, `wait`, `resume` or `cancel`. Client wait duration is not job duration.
The installed custodian owns physical admission, safe handoff and actual release;
product hooks verify their own data/clients and return under that same lease.
Do not create a second controller or detach a legacy foreground transaction.
Reconcile current owner/task authority before submission; an older temporary
no-reacquisition instruction is not a permanent gate after an explicit amendment.
Preparation alone owns no device or storage. Follow the installed procedure and
current custodian for a nonprogressing startup, with its actual job ID in private
coordination; never clear launch/custody records yourself. A personal device still
needs its own fresh window and is not covered by a lab-device grant.

Keep live job admission separate from prepared audit copies: a template must never
overwrite the actual job ID or original deadline. Preserve failed originals and
use the installed cleanup-only recovery ancestry, with the exact original task
reference; successful recovery does not rewrite an earlier failed return as success.
Create the task's output directory with its required private mode before request.
Local receipt readers must see complete JSON across publication; atomic replacement
prevents a truncate/write interval from looking like missing state.

An unchanged APK may bind to an independently verified, pinned off-device signature
receipt when its actual readback digest matches exactly. Do not mislabel a bounded
host verifier's startup failure as a changed app signer. Prepare resource compatibility
before custody; the executor's memory/virtual-address bounds apply to child tools too.
The next packet keeps a constructor-declared stick duration, at most four seconds,
while earlier callers retain their one-second default. These are individual finite
contacts with full named-control matching and the same source/settings/geometry/
focus/process/single-use guards. Direct dispatch uses the actual inspected frame
within thirty seconds. A prepared, explicitly requested matching refresh may
instead capture a new frame and compare its complete named control AND inspected
guest-context region exactly before dispatch. The dispatched frame retains the
same age limit. Preserve/open that whole capture before any further guest action.
Changed pixels refuse; there is no blind gesture, implicit refresh or age extension.
This changes test pacing, not emulator input policy, game speed or performance
qualification. Tool/inspection expiry remains missing evidence, not a game freeze.

A later inspected navigation attempt moved into water and then returned to the
dock without another contact. Its changed context correctly refused the next
input. Preserve those images and the normal joined Stop/backup/return result;
do not call this a product freeze or an ordinary-save result. Use shorter steps
near edges. Under the current finite lab procedure, prepare enough time for
individual image inspection and both Stop protections before requesting another
job. The helper permits a constructor-declared ceiling up to96 captures, retaining
its eight-capture default, existing byte limit and strict per-contact guards.

Retain scoped diagnostic history across logd prefix eviction only when a fresh
read overlaps its exact prior tail under the same PID and process start time.
Keep all earlier invalidations and append new records in order. Empty/disjoint,
reordered, unscoped or oversized reads refuse input; an old ready flag alone is
never a fallback. A new process requires its own initialization history. Keep
the fresh focus, physical viewport, complete control/context and single-use
capture checks. This repairs evidence retention, not game timing or the freeze.
The missing ordinary-save packet may first finish its exact protected current
checkpoint through real resume/Pause/joined Stop, without changing card files by
hand. Its three Stop protections declare576MiB aggregate collected data. Earlier
callers retain their original byte defaults and bounds; the constructor maximum
is768MiB. A recovered/quiescent retirement remains distinct from normal Stop.

Complete logd eviction can remove even the overlap. Preserve that gap; it is not
a freeze diagnosis. The current fixed-layout Pixel4 ordinary-progress recipe may
guard a contact with current selected-game/profile intent, rechecked PID/start,
fresh focus/physical viewport and exact complete source-bound control plus
inspected guest-context pixels. It requires the initial native boot/presentation
acknowledgment once for each actual process. It never returns cached history as a
fresh log read. The complete Pause button uses the same current-frame proof;
paused/stopped/error metadata, changed pixels, source, layout or identity refuse.
Real checkpoint durability and joined Stop acknowledgments remain required.
The latest prepared task declares 90 minutes including five minutes of cleanup,
at most 96 images, 36 play/save contacts, 100 actions and 576 MiB collected data. Inspection time is
included; these bounds grant neither endurance stress nor renewed live deadlines.

For the inspected Wind Waker numeral context, scenery can move behind the text.
The task may compare its pinned opaque numeral palette instead of background
pixels: foreground position/shape and every selected RGB value must match exactly,
with a bounded nonempty text signal. This applies only to that declared guest HUD
context. Native buttons/thumb wells still require every complete-region pixel.
Keep menu/transition negatives and the actual moving-background fixture. Pause
uses the ordinary stock tap; held contacts remain GameCube game controls only.

Terminal current-frame cleanup attempts Pause independently of evictable log
history, then always reaches the existing protection/finalizer path when transport
permits. A fresh ordinary-process evidence directory copies the exact pinned
navigation proof and retains the immutable original command root. Preserve the
current actual checkpoint or clean stopped state; never reattach an old pointer
to avoid game navigation.

The installed custodian output monitor has an observed atomic-publication race:
an enumerated temporary receipt pathname can disappear before its later size
query. The resulting host failure does not establish a game stall. Preserve the
original in-flight operation and failed return, use installed named cleanup-only
recovery, and retain all recovery attempts. An isolated host reproduction runs
the actual installed function without phone or custody mutation. Correct that
monitor with its owner; do not edit shared tools or bypass output accounting from
the product checkout. No new play job is prepared as though the cause were fixed.

A separately admitted cleanup-only recovery has its own finite cleanup deadline;
it does not revive expired gameplay authority. Record fresh non-content focus,
display and keyguard evidence before any stock cleanup action. An unlocked
SystemUI shade may be collapsed only when it overlays the exact same owned
resumed gcx activity on the nonsecure lab device, then exact Activity AND focused
window guards must pass again. No notification capture, Back/game input, foreign
app interaction, launcher or settings changes. Keep normal joined Stop distinct
from independently protected quiescent retirement and post-exit backup.

The prepared continuation can protect an interrupted **capture** under the same
job without treating it as completed work. Require the inherited lease, matching
job/admission, `RETURNING` cleanup phase, positive retirement of preceding work,
and live PID/start identities showing that this cleanup process is the recorded
supervisor's direct current child. Preserve the original receipt and operation
before proceeding. Game/lifecycle input, unknown foreground, transport timeout,
missing facts, changed owner or surviving work require named recovery instead.
Recheck the exact acknowledged native process and game/profile, or actual
absence. Conservatively charge one interrupted image and a 16 MiB output allowance within the original
budget; those charges are not measured traffic or renewed authority.

During terminal cleanup, the known shade context must match fresh non-content
observations, native process identity, default display geometry and unlocked
nonsecure state before one stock collapse. Exact gcx Activity/window focus must
then return. This neither relaxes game-input guards nor closes the original
freeze. The refinement has 84 public helper and 67 private task checks, including
the retained shade metadata; device qualification is still due after the monitor
owner resolves its observed failure. No new device job is queued.

For a prepared stick contact, `motion="hold"` uses a single bounded stock swipe
whose start and end are the same inspected displaced point. The existing default
`motion="ramp"` keeps the center-to-target interpolation. Both retain the same
named-well radius, duration, source/frame/focus/geometry and single-use guards.
There is no additional motion-event stream or input client. This interpretation
follows the pinned [Android 13 input implementation](https://raw.githubusercontent.com/aosp-mirror/platform_frameworks_base/android-13.0.0_r1/services/core/java/com/android/server/input/InputShellCommand.java).

The current Rust policy starts from the well center and applies an in-well DOWN
displacement immediately. Focused host tests cover both sticks, fixed/following
preferences, first sample, steady held values, release and interruption. This
establishes a prepared test gesture, not device latency or game responsiveness.
Do not reuse old ramp durations as hold trajectories: begin with one short
individually inspected contact on safe ground. Keep prior movement observations
bound to their actual ramp gestures. The new packet has 86 helper and 69 task
checks plus two Rust tests; device qualification is still due. Product/engine
binaries, ordinary saves and protected checkpoints remain unchanged.

The next guarded world route also admits the already present **L click** button.
Its complete region derives from the unchanged Rust `Control::LClick` target and
matches retained current-build world/camera/cleanup frames. It is a short digital
contact, independent of L analog travel, and is excluded from Save menus and
cleanup input. Nintendo's [GameCube controls](https://www.guiasnintendo.com/2_GAMECUBE/Zelda_Wind_Waker/zeldawindwaker_sp/apie.html)
identify L for targeting/centering the camera. This repairs a missing test control;
it changes no application binary and does not infer partial trigger travel.

The installed monitor correction has now been read back against its owner’s source
and exercised with an actual atomic rename fixture. Subsequent gcx play jobs
earned normal joined Stop, protected ordinary profiles/generation and verified
return. Keep the original monitor interruption and recovery records; they remain
separate from the unresolved later-play freeze.

The next independently prepared ordinary-progress job may declare up to160 images
(the default remains eight),80 individually inspected play/save contacts and
768MiB collected data within its original finite90minute task. The prior35-contact
pass reached the house but exhausted navigation headroom without a new marker or
Save. Bounds cannot grow during execution; both Stop backups and return remain
reserved. An actual inspected item purchase with its exact rupee debit can be a
new persisted marker, as can collected rupees. An arbitrary changed count, unchanged
marker, sent input or card hash does not establish new saved progress.

A saved inventory acquisition is also a meaningful ordinary-progress marker.
The prepared continuation starts from visibly casual clothes, observes the actual
Hero’s Clothes acquisition text and the later green-clothes world, then requires
that outfit after actual Save, normal joined Stop and fresh-process game-menu
loading. An unchanged rupee count alone cannot satisfy this alternative. Keep the
actual acquisition, save completion and cold world captures distinct and private.

A retained ordinary-world refusal isolated one background-sensitive low-brightness
edge pixel in the numeral reference, while the displayed count stayed unchanged.
The next private proof narrows its declared exact palette to bright glyph interiors
(red=green, at least248). The actual pair retains120 selected pixels, above the
existing64-pixel minimum. No color tolerance is added. Changed interior RGB or
shape still refuses, as do title/menu/fade scenes; complete native-control regions
remain exact. Retain the original refusal and its separately verified normal
Stop/backup/return. This is a test reference repair, not an emulator fix.

The task command reader and interrupted-observation cleanup must use the same
prepared sequence ceiling. Cleanup keeps its old160-command default; a prepared
caller may declare at most256, with the original actual-job, live-parent, retired
work, inherited-lease and capture-only proofs unchanged. This is command numbering,
not an action, image, deadline or custody increase. A256-command task cannot rely
on the160 default for late interrupted observations.

A matching HUD proves its declared context, not unchanged terrain or character
position. Water/fall returns can change that position while the count remains
identical. After a transition, wait for a settled new whole frame and prefer the
existing direct route within30seconds; never extend its age or assume a previous
water view still applies. Shorter inspected bridge movements may use the task's
prepared80-contact budget, still inside100actions/160images and original cleanup
and return reserves. Do not enlarge any admitted job's bounds.

The mechanical helper can issue one balanced analog-trigger contact independently
of a digital click: a named L/R analog bar, an interior1..99percent point and a
100..500ms stock DOWN/MOVE/UP sequence. Rust still owns the horizontal travel
mapping. Pixel quantization is not an exact hardware fraction. Native age,
single-use, window/display and cleanup refusal checks remain; the task caller
must additionally prove the complete current bar pixels/source/settings/identity.
The prepared world-only test admits L analog and neither Save-menu nor cleanup
analog input. It does not combine travel/clicks or infer game-camera behavior from
the earlier unsuccessful digital-click centering attempt. No app/native changes
or new hardware result follow from the host checks.

Retain the actual component/display/geometry or coherence error from every read
of the existing12-observation wake/Home guard. A coherent but nonmatching
component previously failed without retaining that predicate evidence. Later
coherent Home in cleanup does not identify the earlier cause. Keep the same
guard, reads, wait bound and refusal of foreign input. A pre-launch failure can
use named cleanup-only recovery after proving gcx absent, independently verifying
the existing ordinary archive against unchanged current data and checking stock
physical return. Keep that recovery separate from the failed original return;
it earns no new normal Stop, play or in-game save result.

The runtime focus guard likewise retains its first coherent component mismatch
privately, including expected component and actual display/frame metadata, before
the unchanged refusal and forced cleanup. No extra query, window token, input,
capture, wait or readiness cache is added. A later cleanup refusal must not
overwrite that first observation.

A finite partial-play task may retain its actual scene through direct Home,
the existing independent complete-generation check and quiescent process
retirement before verified stock return. Use this only for observed world play
with active playing intent, no failure, foreground mismatch, timeout or inflight
operation, and the original cleanup reserve. Do not Pause before that departure
or probe launcher content. Failed protection retains the checkpoint, writer and
error for guarded cleanup; it must not silently fall back to Stop and discard
the scene. This departure is explicitly not normal joined Stop or ordinary-save
reload evidence. A later normal admission may continue only the actual current
checkpoint whose binding and bytes match its independently protected archive;
never reattach an archived pointer. Completed Save and cold-reload stages retain
normal cleanup. The missing ordinary-save row still requires actual new progress,
in-game Save completion, joined Stop and a fresh-process game-menu reload with
no checkpoint restoration.

A changed guest-context rectangle in an explicit matching contact is a known
pre-dispatch refusal. Keep its actual capture/process/reference and no-contact
record, invalidate the prior scene and capture, and require a new explicit whole
image observation. At most eight such observations fit this prepared task; the
ninth ends work. There is no automatic retry, deadline renewal or reused frame.
Transport, focus, process, source, complete-control and other refusals still end
work under the existing cleanup owner. Animated dialogue can use the existing
direct route from its newly inspected whole frame inside the same 30-second
limit. Prior failed attempts and their actual Stop/return evidence remain intact.

A typed pre-dispatch context refusal consumes capture evidence without consuming a
successful game contact. Number each attempted contact's image and diagnostics
independently of successful contacts, so a newly inspected observation can proceed
without colliding with the refused frame. Preserve both frames and the original
no-overwrite guard. Surface the contact result to the operator: a capture and a
null fatal error do not prove input was dispatched. The actual refusal/reobserve/
contact sequence has an isolated regression; existing action/image/deadline bounds
and the eight-refusal limit remain unchanged. No automatic retry is admitted.

After an explicitly authorized retention pass, reconcile the live input graph and
remaining current recovery set before resuming. Keep retired versions recorded as
historical evidence, not available files. Verify the exact retained ordinary cards,
SRAM, complete-generation hashes and signer/recovery binding; a filename suffix
alone does not distinguish a memory card from console SRAM. Qualify the existing
local output paths with a small original synchronized-write/rename/readback and
lock fixture, removing only that fixture. Historical temporary handoffs are not
runtime dependencies. Scope storage qualification to the declared next phase and
recheck capacity/concurrency before larger writes. Never repair a missing input by
restoring an obsolete checkpoint reference or reviving withdrawn storage.
