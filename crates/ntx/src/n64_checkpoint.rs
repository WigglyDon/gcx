// SPDX-License-Identifier: MIT
//! N64 continuation binds the pinned machine to its coherent ordinary save
//! generation. No core/architecture/option fingerprint is rewritten on import.
use crate::{
    library::valid_id,
    n64_save::{Generation, SaveStore},
    storage,
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
const MAGIC: &[u8; 8] = b"NTXN6401";
const LIMIT: u64 = 64 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub game_id: String,
    pub profile_id: String,
    pub media_sha256: String,
    pub core_sha256: String,
    pub options_sha256: String,
    pub architecture: String,
}
impl Binding {
    fn validate(&self) -> io::Result<()> {
        if !valid_id(&self.game_id)
            || !valid_id(&self.profile_id)
            || ![&self.media_sha256, &self.core_sha256, &self.options_sha256]
                .iter()
                .all(|s| storage::valid_digest(s))
            || self.architecture != "aarch64-android-gles3-mupen-state1"
        {
            return Err(error(
                "Unsupported N64 scene binding; current saves retained",
            ));
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
struct Metadata {
    schema: u32,
    binding: Binding,
    ordinary: Generation,
    payload_sha256: String,
    payload_bytes: usize,
}
pub struct Checkpoint {
    pub payload: Vec<u8>,
    ordinary: Generation,
}
impl Checkpoint {
    pub fn verify_current(&self, store: &SaveStore) -> io::Result<()> {
        store.verify_current()?;
        if store.generation() != Some(&self.ordinary) {
            return Err(error(
                "N64 scene belongs to an older ordinary-save generation; retain it and cold start explicitly with current saves",
            ));
        }
        Ok(())
    }
}
fn error(s: &str) -> io::Error {
    io::Error::other(s)
}
fn path(profile: &Path, id: &str) -> io::Result<PathBuf> {
    if !storage::valid_digest(id) {
        return Err(error("Invalid N64 scene identity"));
    }
    let dir = profile.join("n64-checkpoints");
    if dir.symlink_metadata().is_ok_and(|m| !m.is_dir()) {
        return Err(error("Unsafe N64 scene directory"));
    }
    Ok(dir.join(format!("{id}.ntx")))
}
/// The caller has yielded immediately after the actual native save job, then
/// committed the ordinary image from that same boundary. Keep the writer owned
/// until the collection references this returned complete generation durably.
pub fn publish(
    profile: &Path,
    binding: &Binding,
    store: &SaveStore,
    payload: &[u8],
) -> io::Result<String> {
    binding.validate()?;
    store.verify_current()?;
    if payload.is_empty() || payload.len() as u64 > LIMIT {
        return Err(error("Invalid N64 scene size"));
    }
    let metadata = Metadata {
        schema: 1,
        binding: binding.clone(),
        ordinary: store
            .generation()
            .ok_or_else(|| error("N64 scene has no committed ordinary generation"))?
            .clone(),
        payload_sha256: storage::digest(payload),
        payload_bytes: payload.len(),
    };
    let header = serde_json::to_vec(&metadata)?;
    if header.len() > 65536 {
        return Err(error("N64 scene header too large"));
    }
    let mut bytes = Vec::with_capacity(12 + header.len() + payload.len());
    bytes.extend(MAGIC);
    bytes.extend((header.len() as u32).to_le_bytes());
    bytes.extend(header);
    bytes.extend(payload);
    let id = storage::digest(&bytes);
    let destination = path(profile, &id)?;
    fs::create_dir_all(destination.parent().unwrap())?;
    if destination.try_exists()? {
        if fs::read(&destination)? != bytes {
            return Err(error("N64 retained scene collision"));
        }
    } else {
        storage::atomic_replace(&destination, &bytes)?;
    }
    if fs::read(&destination)? != bytes {
        return Err(error("N64 scene readback differs"));
    }
    store.verify_current()?;
    Ok(id)
}
pub fn load(
    profile: &Path,
    id: &str,
    binding: &Binding,
    store: &SaveStore,
) -> io::Result<Checkpoint> {
    binding.validate()?;
    let source = path(profile, id)?;
    let info = source.symlink_metadata()?;
    if !info.is_file() || info.len() < 12 || info.len() > LIMIT + 65548 {
        return Err(error("Invalid N64 scene file"));
    }
    let bytes = fs::read(source)?;
    if storage::digest(&bytes) != id || &bytes[..8] != MAGIC {
        return Err(error("N64 scene integrity differs"));
    }
    let size = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if size > 65536 || 12 + size >= bytes.len() {
        return Err(error("N64 scene header is truncated"));
    }
    let metadata: Metadata = serde_json::from_slice(&bytes[12..12 + size])?;
    let payload = &bytes[12 + size..];
    if metadata.schema != 1
        || metadata.binding != *binding
        || metadata.payload_bytes != payload.len()
        || storage::digest(payload) != metadata.payload_sha256
    {
        return Err(error(
            "N64 scene compatibility differs; exact bytes and current progress retained",
        ));
    }
    let state = Checkpoint {
        payload: payload.to_vec(),
        ordinary: metadata.ordinary,
    };
    state.verify_current(store)?;
    Ok(state)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_scene_and_wrong_game_cannot_restore_over_newer_progress() {
        let root = std::env::temp_dir().join(format!("ntx-n64-scene-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let mut store = SaveStore::open(&root).unwrap();
        let mut ordinary = vec![0; crate::n64::SAVE_IMAGE_BYTES];
        store.commit(&ordinary).unwrap();
        let mut binding = Binding {
            game_id: "1234567890abcdef".repeat(2),
            profile_id: "abcdef1234567890".repeat(2),
            media_sha256: "a".repeat(64),
            core_sha256: "b".repeat(64),
            options_sha256: "c".repeat(64),
            architecture: "aarch64-android-gles3-mupen-state1".into(),
        };
        let id = publish(&root, &binding, &store, b"retained native state").unwrap();
        let scene = load(&root, &id, &binding, &store).unwrap();
        ordinary[0] = 9;
        store.commit(&ordinary).unwrap();
        assert!(scene.verify_current(&store).is_err());
        assert!(load(&root, &id, &binding, &store).is_err());
        binding.game_id = "b".repeat(32);
        assert!(load(&root, &id, &binding, &store).is_err());
        assert_eq!(store.load().unwrap().unwrap()[0], 9);
        assert!(path(&root, &id).unwrap().exists());
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}
