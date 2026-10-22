// SPDX-License-Identifier: MIT
//! Atomic ordinary-card writes under the runtime's exclusive profile ownership.
use std::{
    ffi::{CStr, c_char},
    fs::{File, OpenOptions},
    io::{self, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::Mutex,
};
static ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);
static ERROR: Mutex<Option<String>> = Mutex::new(None);
pub struct CardStore;
impl CardStore {
    /// The caller holds ProfileLock until native teardown and this binding finish.
    pub fn bind(profile: &Path) -> io::Result<Self> {
        let mut active = ROOT
            .lock()
            .map_err(|_| io::Error::other("card store poisoned"))?;
        if active.is_some() {
            return Err(io::Error::other("another native card store is active"));
        }
        let profile = profile.canonicalize()?;
        // Earlier failures are delivered with their runtime identity before
        // teardown. Do not attribute an old profile's last write to a new one.
        let mut error = ERROR
            .lock()
            .map_err(|_| io::Error::other("card error store poisoned"))?;
        *error = None;
        *active = Some(profile);
        Ok(Self)
    }
}
impl Drop for CardStore {
    fn drop(&mut self) {
        if let Ok(mut root) = ROOT.lock() {
            *root = None;
        }
    }
}
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing storage parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other("missing storage name"))?;
    let mut temporary_name = name.to_os_string();
    temporary_name.push(".gcx-pending");
    let temporary = parent.join(temporary_name);
    if temporary
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err(io::Error::other("storage candidate is a symlink"));
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temporary, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
/// Native lends the path and complete card bytes only for this synchronous call.
/// Returns success only after file sync, atomic rename and directory sync.
/// # Safety
/// `path` is a valid NUL-terminated string and `data` lends `length` readable bytes.
pub unsafe extern "C" fn write_card(path: *const c_char, data: *const u8, length: usize) -> i32 {
    let result = (|| -> io::Result<()> {
        if path.is_null() || data.is_null() || length == 0 || length > 64 * 1024 * 1024 {
            return Err(io::Error::other("invalid native card buffer"));
        }
        // SAFETY: the native writer contract lends valid bytes for this call.
        let path = Path::new(std::ffi::OsStr::from_bytes(
            unsafe { CStr::from_ptr(path) }.to_bytes(),
        ));
        let bytes = unsafe { std::slice::from_raw_parts(data, length) };
        let root = ROOT
            .lock()
            .map_err(|_| io::Error::other("card store poisoned"))?;
        let root = root
            .as_ref()
            .ok_or_else(|| io::Error::other("no owned card store"))?;
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            || !path
                .parent()
                .ok_or_else(|| io::Error::other("missing card parent"))?
                .canonicalize()?
                .starts_with(root)
        {
            return Err(io::Error::other("card path escaped its owned profile"));
        }
        atomic_replace(path, bytes)
    })();
    match result {
        Ok(()) => {
            if let Ok(mut saved) = ERROR.lock() {
                *saved = None;
            }
            1
        }
        Err(error) => {
            if let Ok(mut saved) = ERROR.lock() {
                *saved = Some(error.to_string());
            }
            0
        }
    }
}
pub fn last_card_error() -> Option<String> {
    ERROR.lock().ok().and_then(|e| e.clone())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_writer_requires_one_bound_profile_and_rejects_other_paths() {
        let root = std::env::temp_dir().join(format!("gcx-bound-card-{}", std::process::id()));
        let owned = root.join("owned");
        std::fs::create_dir_all(&owned).unwrap();
        let outside = root.join("outside.raw");
        std::fs::write(&outside, b"retain").unwrap();
        let store = CardStore::bind(&owned).unwrap();
        assert!(CardStore::bind(&owned).is_err());
        let outside_path = std::ffi::CString::new(outside.as_os_str().as_bytes()).unwrap();
        assert_eq!(
            unsafe { write_card(outside_path.as_ptr(), b"new".as_ptr(), 3) },
            0
        );
        assert_eq!(std::fs::read(&outside).unwrap(), b"retain");
        assert!(last_card_error().is_some());
        let card = owned.join("card.raw");
        let card_path = std::ffi::CString::new(card.as_os_str().as_bytes()).unwrap();
        assert_eq!(
            unsafe { write_card(card_path.as_ptr(), b"new".as_ptr(), 3) },
            1
        );
        assert_eq!(std::fs::read(&card).unwrap(), b"new");
        assert!(last_card_error().is_none());
        drop(store);
        assert_eq!(
            unsafe { write_card(card_path.as_ptr(), b"old".as_ptr(), 3) },
            0
        );
        assert_eq!(std::fs::read(&card).unwrap(), b"new");
        assert!(last_card_error().is_some());
        let rebound = CardStore::bind(&owned).unwrap();
        assert!(last_card_error().is_none());
        drop(rebound);
        std::fs::remove_file(card).unwrap();
        std::fs::remove_file(outside).unwrap();
        std::fs::remove_dir(owned).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn failed_candidate_keeps_previous_card_and_success_replaces_it() {
        let root = std::env::temp_dir().join(format!("gcx-atomic-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let card = root.join("card.raw");
        std::fs::write(&card, b"durable old generation").unwrap();
        let pending = root.join("card.raw.gcx-pending");
        std::fs::create_dir(&pending).unwrap();
        assert!(atomic_replace(&card, b"new generation").is_err());
        assert_eq!(std::fs::read(&card).unwrap(), b"durable old generation");
        std::fs::remove_dir(&pending).unwrap();
        atomic_replace(&card, b"new generation").unwrap();
        assert_eq!(std::fs::read(&card).unwrap(), b"new generation");
        assert!(!pending.exists());
        std::fs::remove_file(card).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
