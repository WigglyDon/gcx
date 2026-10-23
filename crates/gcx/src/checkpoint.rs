// SPDX-License-Identifier: MIT
//! Private complete machine generations. Ordinary saves remain authoritative.
//! Callers hold ProfileLock and keep native execution paused until publication.
use crate::{
    library::valid_id,
    storage::{self, SaveGeneration},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 8] = b"GCXCP001";
const HEADER_LIMIT: usize = 65536;
// Safety ceiling for this private format, not a promised device allocation.
// Record actual native payload sizes; an oversized capture retains the runtime.
pub const PAYLOAD_LIMIT: usize = 256 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub game_id: String,
    pub profile_id: String,
    pub engine: String,
    pub state_abi: u32,
    pub resources: String,
    pub media_sha256: String,
    pub disc: u8,
}
impl Binding {
    fn validate(&self) -> io::Result<()> {
        if !valid_id(&self.game_id)
            || !valid_id(&self.profile_id)
            || !storage::valid_digest(&self.engine)
            || !storage::valid_digest(&self.media_sha256)
            || self.state_abi != 1
            || self.resources.is_empty()
            || self.resources.len() > 4096
        {
            return Err(io::Error::other(
                "unsupported checkpoint binding; current saves retained",
            ));
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
struct Metadata {
    binding: Binding,
    generation: SaveGeneration,
    paused: bool,
    payload_sha256: String,
    payload_length: usize,
}
pub struct Checkpoint {
    pub payload: Vec<u8>,
    pub paused: bool,
    generation: SaveGeneration,
}
impl Checkpoint {
    /// Recheck immediately before native mutation, including after paused boot.
    pub fn verify_current_saves(&self, profile: &Path) -> io::Result<()> {
        self.generation.verify(profile)
    }
}
fn directory(profile: &Path) -> io::Result<PathBuf> {
    let path = profile.join("checkpoints");
    if path
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err(io::Error::other("checkpoint directory is a symlink"));
    }
    Ok(path)
}
fn path(profile: &Path, id: &str) -> io::Result<PathBuf> {
    if !storage::valid_digest(id) {
        return Err(io::Error::other("invalid checkpoint identity"));
    }
    Ok(directory(profile)?.join(format!("{id}.gcx")))
}
fn parts_digest(parts: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part);
    }
    format!("{:x}", hash.finalize())
}
/// Stream a selected media descriptor without copying the game image.
pub fn media_digest(mut file: impl Read) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
/// Synchronous native capture directly into the durable Rust container. The
/// caller owns the initialized, paused engine and its ProfileLock throughout.
///
/// # Safety
/// Serialize this call with all native lifecycle work; retain callbacks, profile
/// ownership and native runtime until it returns.
#[cfg(feature = "native-engine")]
pub unsafe fn capture(profile: &Path, binding: &Binding, paused: bool) -> io::Result<String> {
    struct Writer<'a> {
        profile: &'a Path,
        binding: &'a Binding,
        paused: bool,
        result: Option<io::Result<String>>,
    }
    unsafe extern "C" fn write(
        context: *mut std::ffi::c_void,
        bytes: *const u8,
        length: usize,
    ) -> i32 {
        if context.is_null() || bytes.is_null() || length == 0 || length > PAYLOAD_LIMIT {
            return 0;
        }
        // SAFETY: synchronous callback; native lends its complete bounded buffer
        // and the stack context remains live until capture returns.
        let writer = unsafe { &mut *context.cast::<Writer<'_>>() };
        let result = publish(writer.profile, writer.binding, writer.paused, unsafe {
            std::slice::from_raw_parts(bytes, length)
        });
        let ok = result.is_ok();
        writer.result = Some(result);
        i32::from(ok)
    }
    let mut writer = Writer {
        profile,
        binding,
        paused,
        result: None,
    };
    // SAFETY: public engine lifecycle is serialized by the host, and the bridge
    // refuses an absent/running/failed runtime before borrowing the context.
    let acknowledged = unsafe {
        crate::engine::gcx_engine_checkpoint_capture(
            PAYLOAD_LIMIT,
            write,
            std::ptr::from_mut(&mut writer).cast(),
        )
    };
    let result = writer.result.unwrap_or_else(|| {
        Err(io::Error::other(
            "native checkpoint capture refused; paused runtime and prior checkpoint retained",
        ))
    });
    if acknowledged == 0 && result.is_ok() {
        return Err(io::Error::other(
            "native checkpoint did not acknowledge completion",
        ));
    }
    result
}
/// Publish an immutable payload first. The caller atomically publishes this ID
/// in SavedSession afterward; its old reference stays usable on metadata error.
pub fn publish(
    profile: &Path,
    binding: &Binding,
    paused: bool,
    payload: &[u8],
) -> io::Result<String> {
    binding.validate()?;
    if payload.is_empty() || payload.len() > PAYLOAD_LIMIT {
        return Err(io::Error::other(
            "invalid checkpoint size; paused runtime retained",
        ));
    }
    let generation = storage::save_generation(profile)?;
    if !generation.images.contains_key("GC/SRAM.raw")
        || !generation
            .images
            .keys()
            .any(|name| name.starts_with("GC/MemoryCardA."))
    {
        return Err(io::Error::other(
            "checkpoint requires acknowledged card and console data",
        ));
    }
    let metadata = serde_json::to_vec(&Metadata {
        binding: binding.clone(),
        generation,
        paused,
        payload_sha256: storage::digest(payload),
        payload_length: payload.len(),
    })?;
    if metadata.len() > HEADER_LIMIT {
        return Err(io::Error::other("checkpoint metadata too large"));
    }
    let length = (metadata.len() as u32).to_le_bytes();
    let parts: [&[u8]; 4] = [MAGIC, &length, &metadata, payload];
    let id = parts_digest(&parts);
    fs::create_dir_all(directory(profile)?)?;
    storage::atomic_replace_parts(&path(profile, &id)?, &parts)?;
    // Verify actual persisted bytes before the session may refer to this file.
    let restored = load(profile, &id, binding)?;
    if restored.payload != payload || restored.paused != paused {
        return Err(io::Error::other(
            "checkpoint readback failed; prior reference retained",
        ));
    }
    Ok(id)
}
/// Validate all inputs and current ordinary files before native deserialization.
/// Every error leaves the checkpoint and ordinary saves in place.
pub fn load(profile: &Path, id: &str, expected: &Binding) -> io::Result<Checkpoint> {
    expected.validate()?;
    let path = path(profile, id)?;
    if path.symlink_metadata()?.file_type().is_symlink() {
        return Err(io::Error::other("checkpoint file is a symlink"));
    }
    let mut file = File::open(path)?;
    let file_length = file.metadata()?.len();
    if file_length > (PAYLOAD_LIMIT + HEADER_LIMIT + 12) as u64 {
        return Err(io::Error::other(
            "checkpoint is too large; original retained",
        ));
    }
    let mut header = [0; 12];
    file.read_exact(&mut header)?;
    if &header[..8] != MAGIC {
        return Err(io::Error::other(
            "unsupported checkpoint format; original retained",
        ));
    }
    let length = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    if length > HEADER_LIMIT {
        return Err(io::Error::other("invalid checkpoint header"));
    }
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    let metadata: Metadata = serde_json::from_slice(&bytes)?;
    if metadata.binding != *expected {
        return Err(io::Error::other(
            "checkpoint core/resources/game/disc changed; restart with current saves",
        ));
    }
    if metadata.payload_length == 0
        || metadata.payload_length > PAYLOAD_LIMIT
        || file_length != (12 + length + metadata.payload_length) as u64
    {
        return Err(io::Error::other("invalid checkpoint payload length"));
    }
    metadata.generation.verify(profile)?;
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(metadata.payload_length)
        .map_err(io::Error::other)?;
    file.take(metadata.payload_length as u64 + 1)
        .read_to_end(&mut payload)?;
    if payload.len() != metadata.payload_length
        || storage::digest(&payload) != metadata.payload_sha256
        || parts_digest(&[&header, &bytes, &payload]) != id
    {
        return Err(io::Error::other(
            "checkpoint bytes are damaged; current saves retained",
        ));
    }
    Ok(Checkpoint {
        payload,
        paused: metadata.paused,
        generation: metadata.generation,
    })
}
/// Only after the new SavedSession reference is durably acknowledged, retire a
/// known earlier successful checkpoint. Never pass a rejected checkpoint here.
pub fn retire_previous(profile: &Path, old: &str, current: &str) -> io::Result<()> {
    if old == current {
        return Ok(());
    }
    discard_successful(profile, old)
}
/// Only after a durable explicit Stop or replacement reference; never delete a
/// rejected or unacknowledged checkpoint. Caller retains the profile lock.
pub fn discard_successful(profile: &Path, id: &str) -> io::Result<()> {
    let old = path(profile, id)?;
    match fs::remove_file(old) {
        Ok(()) => File::open(directory(profile)?)?.sync_all(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProfileLock, storage::CardStore};
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    fn setup(name: &str) -> (PathBuf, ProfileLock, CardStore, Binding) {
        let root = std::env::temp_dir().join(format!(
            "gcx-checkpoint-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("GC")).unwrap();
        let lock = ProfileLock::acquire(&root).unwrap();
        let writer = CardStore::bind(&root).unwrap();
        write(&root.join("GC/MemoryCardA.fixture.raw"), b"original card");
        write(&root.join("GC/SRAM.raw"), b"original console identity");
        let binding = Binding {
            game_id: "a".repeat(32),
            profile_id: "b".repeat(32),
            engine: "c".repeat(64),
            state_abi: 1,
            resources: "original resource fixture".into(),
            media_sha256: "d".repeat(64),
            disc: 0,
        };
        (root, lock, writer, binding)
    }
    fn write(path: &Path, bytes: &[u8]) {
        let path = CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(
            unsafe { storage::write_card(path.as_ptr(), bytes.as_ptr(), bytes.len()) },
            1
        );
    }
    #[test]
    fn complete_generation_and_pause_intent_survive_reopen_but_newer_saves_refuse_restore() {
        let _serial = storage::STORE_TEST_LOCK.lock().unwrap();
        let (root, lock, writer, binding) = setup("roundtrip");
        let id = publish(&root, &binding, true, b"original complete-machine fixture").unwrap();
        drop(writer);
        let writer = CardStore::bind(&root).unwrap();
        let restored = load(&root, &id, &binding).unwrap();
        assert!(restored.paused);
        assert_eq!(restored.payload, b"original complete-machine fixture");
        let before = fs::read(path(&root, &id).unwrap()).unwrap();
        write(
            &root.join("GC/MemoryCardA.fixture.raw"),
            b"newer durable progress",
        );
        assert!(restored.verify_current_saves(&root).is_err());
        assert!(load(&root, &id, &binding).is_err());
        assert_eq!(fs::read(path(&root, &id).unwrap()).unwrap(), before);
        assert_eq!(
            fs::read(root.join("GC/MemoryCardA.fixture.raw")).unwrap(),
            b"newer durable progress"
        );
        drop(writer);
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_payload_or_session_publication_keeps_the_previous_reachable_checkpoint() {
        let _serial = storage::STORE_TEST_LOCK.lock().unwrap();
        let (root, lock, writer, binding) = setup("publication");
        let old = publish(&root, &binding, true, b"earlier machine").unwrap();
        let old_bytes = fs::read(path(&root, &old).unwrap()).unwrap();
        let pending = directory(&root)
            .unwrap()
            .join(format!("{old}.gcx.gcx-pending"));
        fs::create_dir(&pending).unwrap();
        assert!(publish(&root, &binding, true, b"earlier machine").is_err());
        assert_eq!(fs::read(path(&root, &old).unwrap()).unwrap(), old_bytes);
        fs::remove_dir(pending).unwrap();
        let new = publish(&root, &binding, false, b"later complete machine").unwrap();
        let session = root.join("session-fixture.json");
        storage::atomic_replace(&session, old.as_bytes()).unwrap();
        let blocked = root.join("session-fixture.json.gcx-pending");
        fs::create_dir(&blocked).unwrap();
        assert!(storage::atomic_replace(&session, new.as_bytes()).is_err());
        assert_eq!(fs::read(&session).unwrap(), old.as_bytes());
        assert_eq!(
            load(&root, &old, &binding).unwrap().payload,
            b"earlier machine"
        );
        assert_eq!(
            load(&root, &new, &binding).unwrap().payload,
            b"later complete machine"
        );
        fs::remove_dir(blocked).unwrap();
        storage::atomic_replace(&session, new.as_bytes()).unwrap();
        retire_previous(&root, &old, &new).unwrap();
        assert!(!path(&root, &old).unwrap().exists());
        assert!(!load(&root, &new, &binding).unwrap().paused);
        drop(writer);
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn incompatible_inputs_and_corruption_retain_the_payload_and_current_saves() {
        let _serial = storage::STORE_TEST_LOCK.lock().unwrap();
        let (root, lock, writer, binding) = setup("rejection");
        let id = publish(&root, &binding, false, b"machine fixture").unwrap();
        let original = fs::read(path(&root, &id).unwrap()).unwrap();
        let mut alternatives = vec![binding.clone(); 6];
        alternatives[0].engine = "e".repeat(64);
        alternatives[1].resources.push_str("changed");
        alternatives[2].media_sha256 = "f".repeat(64);
        alternatives[3].disc = 1;
        alternatives[4].profile_id = "c".repeat(32);
        alternatives[5].state_abi = 2;
        for other in alternatives {
            assert!(load(&root, &id, &other).is_err());
        }
        assert_eq!(fs::read(path(&root, &id).unwrap()).unwrap(), original);
        let mut damaged = original.clone();
        *damaged.last_mut().unwrap() ^= 1;
        fs::write(path(&root, &id).unwrap(), &damaged).unwrap();
        assert!(load(&root, &id, &binding).is_err());
        assert_eq!(fs::read(path(&root, &id).unwrap()).unwrap(), damaged);
        damaged[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        fs::write(path(&root, &id).unwrap(), &damaged).unwrap();
        assert!(load(&root, &id, &binding).is_err());
        assert_eq!(
            fs::read(root.join("GC/MemoryCardA.fixture.raw")).unwrap(),
            b"original card"
        );
        assert!(load(&root, "../escape", &binding).is_err());
        drop(writer);
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }
}
