# ntx identity and protected migration

Active crates, probes, native exports, JNI/activity, Android resources, package,
private root, links, helper environment and build outputs now use ntx. Dolphin
remains a direct adapter. New CMake output roots avoid relocated legacy caches.
This is source implementation; a new linked APK and actual device migration have
not yet been qualified. The legacy installation and recovery candidates remain.

Collection schema 2 separates `system` and `backend`. `Collection::decode` maps
schema 1 explicitly to GameCube/Dolphin while preserving opaque IDs, selected
game, profile, pause/error and checkpoint fields. Unknown/ambiguous schemas and
invalid routes refuse. Android/native resources never become durable game IDs.

The existing signer is retained. `cargo xtask check-signer` compares original key
and password with independent initial recovery, copies the same key to its ntx
filename without replacing the original, exports its certificate and compares
the digest with the installed-candidate verification receipt. Packaging performs
the same checks. Missing or divergent material refuses; no new key is generated. Independent
source reproduction uses package-android-unsigned without owner signing material;
its artifact cannot update an existing installation.

## Offline development import

`cargo run --offline --locked -p ntx --bin ntx-migrate -- PROTECTED_EXPORT PRIVATE_TARGET_PARENT`
imports only into PRIVATE_TARGET_PARENT/ntx. This command performs no device
operation and is not an exporter, archive extractor or phone-installation recipe.
A prepared development transfer can place that protected export in files/ntx-import.
At startup, the new app uses the same Rust validator and atomic publisher before
creating files/ntx. An existing target is never replaced; the incoming export stays
retained. The app holds its store lock for its entire subsequent runtime.
Use an already protected, quiescent export obtained under separately admitted
custody; independently retain source installation, ordinary profiles and complete
generations. Never extract an untrusted archive directly into the app's live root.

The importer validates the collection, ordinary generation hashes and referenced
checkpoint container identity, takes profile writer locks, inventories bounded
regular files, rejects symlinks/hard links/unfinished publications, and streams
copies into a private stage. It preserves profile contents and control preferences
byte-for-byte; packaged Sys resources are reconstructed by the new application.
It rereads the staged hashes and source before writing a receipt, syncing and
renaming into place under the app's matching store lock. Interrupted stages stay
retained. Retries succeed only for the identical published tree. Any existing
changed target refuses, including newer progress; there is no merge/overwrite.

New checkpoint containers use NTXCP001 and .ntx. The isolated legacy reader accepts
the old envelope/extension but still checks the complete container hash, exact
engine/resource/media binding and current ordinary generation. It never changes
fingerprints. Migration retains the checkpoint and marks scene recovery explicitly;
it does not silently bootstrap a changed engine into unsaved play. Explicit
Restart uses current saves and leaves rejected checkpoint bytes retained.

New package grants are not inferred from old URI strings. Migrated entries remain
blocked for media access. Selecting the same original document through the new
picker can durably reacquire access without creating a new game/profile; where a
checkpoint exists, its media digest must match. Internal game links do not count
as grant acquisition. A changed locator is accepted only against the preserved media fingerprint;
otherwise only the exact original locator can be reacquired. Optional protected
media-identities.json exports or retained checkpoint bindings supply that original
fingerprint. New successful boots preserve it independently of checkpoints. Package-targeted old
shortcuts may require re-pinning. No automatic legacy-app forwarding is installed.

Pixel and Fold transfer/rollback, linked package/ABI checks, actual progress and
short representative changed-boundary play/lifecycle checks remain unearned.
No source uninstall or cleanup is allowed on the strength of host fixtures.

## Residual legacy references

`cargo xtask check-identity` audits active implementation paths/content. The
remaining exceptions have explicit meaning:

| Owner | Allowed legacy meaning |
| --- | --- |
| crates/ntx/src/legacy.rs | Old URI and checkpoint magic/extension readers |
| android/AndroidManifest.xml | Old public URI scheme reader alongside canonical ntx links |
| xtask/src/legacy.rs | Immutable signer alias, retained source/recovery filenames and certificate receipt |
| xtask/src/identity.rs | The audit's literal exception patterns |
| crates/ntx/src/lib.rs and checkpoint probe | Original PowerPC fixture marker and ABI1 corruption fixture bytes |
| Maintained Dolphin patch | Two inherited ABI1 payload cookies; comments identify unchanged serialized bytes |
| native/patches/history and docs/history | Original patches, source/build identities, incidents and historical recipes |
| handoff/LOCAL_SETUP.md and MASTER_SEED.md; issue/research documents | Explicitly historical bootstrap/research/issue evidence, not active dispatch |
| AGENTS/current guidance/state | Descriptions of the migration boundary and exact retained legacy evidence |
| LICENSE | Original attribution/provenance |

Vendor names, licenses, commit hashes, game/save payloads and private historical
recipes are not bulk-renamed. The ignored Dolphin edits are amended mechanically
at their own hooks and exactly matched by the maintained source patch; the original
patch is retained separately. No dependency/sibling reset or old-path alias occurs.
