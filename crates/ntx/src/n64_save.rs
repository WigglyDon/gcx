// SPDX-License-Identifier: MIT
//! N64 ordinary progress belongs to a save profile, independently of a core
//! instance or machine checkpoint. Only the quiescent runtime may publish it.
use crate::{ProfileLock, n64::SaveImage, storage};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generation {
    pub schema: u32,
    pub number: u64,
    pub image_sha256: String,
}
pub struct SaveStore {
    directory: PathBuf,
    accepted: Option<Generation>,
    _writer: ProfileLock,
}
fn error(message: &str) -> io::Error {
    io::Error::other(message)
}
fn read_generation(directory: &Path) -> io::Result<Option<Generation>> {
    let path = directory.join("ordinary.json");
    match path.symlink_metadata() {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
        Ok(info) if !info.is_file() || info.len() > 4096 => {
            return Err(error("invalid N64 ordinary generation"));
        }
        _ => {}
    }
    let generation: Generation = serde_json::from_slice(&fs::read(path)?)?;
    if generation.schema != 1
        || generation.number == 0
        || !storage::valid_digest(&generation.image_sha256)
    {
        return Err(error("unsupported N64 ordinary generation"));
    }
    Ok(Some(generation))
}
fn image_path(directory: &Path, generation: &Generation) -> PathBuf {
    directory.join(format!("{}.m64ram", generation.image_sha256))
}
fn read_image(directory: &Path, generation: &Generation) -> io::Result<Vec<u8>> {
    let path = image_path(directory, generation);
    let info = path.symlink_metadata()?;
    if !info.is_file() || info.len() != crate::n64::SAVE_IMAGE_BYTES as u64 {
        return Err(error("invalid pinned N64 ordinary image"));
    }
    let bytes = fs::read(path)?;
    SaveImage::parse(&bytes)?;
    if storage::digest(&bytes) != generation.image_sha256 {
        return Err(error(
            "N64 ordinary data changed outside its writer; retain runtime and recovery",
        ));
    }
    Ok(bytes)
}
impl SaveStore {
    pub fn open(profile: &Path) -> io::Result<Self> {
        fs::create_dir_all(profile)?;
        if !profile.symlink_metadata()?.is_dir() {
            return Err(error("unsafe N64 profile"));
        }
        let writer = ProfileLock::acquire(profile)?;
        let directory = profile.join("N64");
        fs::create_dir_all(&directory)?;
        if !directory.symlink_metadata()?.is_dir() {
            return Err(error("unsafe N64 ordinary directory"));
        }
        let accepted = read_generation(&directory)?;
        if let Some(generation) = &accepted {
            read_image(&directory, generation)?;
        }
        Ok(Self {
            directory,
            accepted,
            _writer: writer,
        })
    }
    pub fn generation(&self) -> Option<&Generation> {
        self.accepted.as_ref()
    }
    /// Validate actual bytes again immediately before native restore or flush.
    pub fn verify_current(&self) -> io::Result<()> {
        if read_generation(&self.directory)? != self.accepted {
            return Err(error(
                "N64 ordinary generation advanced; stale runtime cannot publish",
            ));
        }
        if let Some(generation) = &self.accepted {
            read_image(&self.directory, generation)?;
        }
        Ok(())
    }
    pub fn load(&self) -> io::Result<Option<Vec<u8>>> {
        self.verify_current()?;
        self.accepted
            .as_ref()
            .map(|g| read_image(&self.directory, g))
            .transpose()
    }
    /// Publish immutable image bytes before their durable generation reference.
    /// On any failure the caller retains its paused runtime and profile writer.
    pub fn commit(&mut self, bytes: &[u8]) -> io::Result<Generation> {
        SaveImage::parse(bytes)?;
        self.verify_current()?;
        let hash = storage::digest(bytes);
        if let Some(current) = &self.accepted
            && current.image_sha256 == hash
        {
            return Ok(current.clone());
        }
        let generation = Generation {
            schema: 1,
            number: self
                .accepted
                .as_ref()
                .map_or(Some(1), |g| g.number.checked_add(1))
                .ok_or_else(|| error("N64 generation exhausted"))?,
            image_sha256: hash,
        };
        let path = image_path(&self.directory, &generation);
        if path.try_exists()? {
            if read_image(&self.directory, &generation)? != bytes {
                return Err(error("retained N64 image collision"));
            }
        } else {
            storage::atomic_replace(&path, bytes)?;
        }
        if read_image(&self.directory, &generation)? != bytes {
            return Err(error("N64 ordinary image readback failed"));
        }
        self.verify_current()?;
        storage::atomic_replace(
            &self.directory.join("ordinary.json"),
            &serde_json::to_vec(&generation)?,
        )?;
        if read_generation(&self.directory)?.as_ref() != Some(&generation) {
            return Err(error("N64 ordinary generation readback failed"));
        }
        self.accepted = Some(generation.clone());
        Ok(generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("ntx-n64-{name}-{}", std::process::id()));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn ordinary_progress_reopens_independently_and_a_writer_cannot_adopt_outside_changes() {
        let root = root("generation");
        let mut store = SaveStore::open(&root).unwrap();
        assert!(store.load().unwrap().is_none());
        assert!(SaveStore::open(&root).is_err());
        let mut bytes = vec![0; crate::n64::SAVE_IMAGE_BYTES];
        bytes[0x800] = 23;
        let generation = store.commit(&bytes).unwrap();
        assert_eq!(generation.number, 1);
        drop(store);
        let mut store = SaveStore::open(&root).unwrap();
        assert_eq!(store.load().unwrap().unwrap(), bytes);
        fs::write(
            image_path(&store.directory, &generation),
            vec![1; bytes.len()],
        )
        .unwrap();
        assert!(store.commit(&bytes).is_err());
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_metadata_publication_keeps_prior_committed_generation_and_profile_lock() {
        let root = root("publication");
        let mut store = SaveStore::open(&root).unwrap();
        let mut bytes = vec![0; crate::n64::SAVE_IMAGE_BYTES];
        let old = store.commit(&bytes).unwrap();
        fs::create_dir(store.directory.join("ordinary.json.ntx-pending")).unwrap();
        bytes[0] = 9;
        assert!(store.commit(&bytes).is_err());
        assert_eq!(store.generation(), Some(&old));
        assert!(SaveStore::open(&root).is_err());
        assert_eq!(store.load().unwrap().unwrap()[0], 0);
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}
