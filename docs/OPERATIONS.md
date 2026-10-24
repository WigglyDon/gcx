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

Own UI input/captures require freshly matching resumed Activity and focused
window. Framework button taps use the currently dumped layout, with a second
focus observation before the single tap. Native Pause uses the owner's real tap,
not stale guessed coordinates. A mismatch ends new work; unexpected app content
is not retained. Home, nonsecure wake and ordinary app entry are separate named
steps; unexpected foreground requires owner clarification. Raw UI/capture/hash
batches are refused. Hashing uses nonempty, validated literal own-file arguments,
never a quoted `sh -c` string or an input-less hash command.

The helper does not kill an unverified live writer to simulate Stop. It records
pending recovery before releasing custody if normal cleanup cannot complete.
Timeout means no further device commands from that helper. Clean Stop metadata,
native join/profile release, own-client absence and physical display/keyguard
return are separate observations; a helper flag alone is not acceptance. Confirm
actual transaction/lease release after exit. Test guards off-device with
`python3 -m unittest discover -s scripts/tests -v`.

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
