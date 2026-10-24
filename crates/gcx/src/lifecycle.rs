// SPDX-License-Identifier: MIT
//! Explicit ordinary-save stop. The runtime retains its profile through publication.
use crate::library::Collection;
use std::sync::atomic::{AtomicU32, Ordering};

/// A fixed budget for the narrow lifecycle diagnostic burst, shared by callbacks.
/// It does not gate player behavior or collect per-frame data.
pub struct DiagnosticBudget(AtomicU32);
impl DiagnosticBudget {
    pub const fn new(limit: u32) -> Self {
        Self(AtomicU32::new(limit))
    }
    pub fn reset(&self, limit: u32) {
        self.0.store(limit, Ordering::Relaxed);
    }
    pub fn take(&self) -> bool {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
            .is_ok()
    }
}

pub trait StopRuntime {
    fn epoch(&self) -> u64;
    fn pause_commit(&mut self) -> Result<(), String>;
    /// Join every native worker, retaining the profile/card binding until Drop.
    fn join(&mut self) -> Result<(), String>;
    /// Optional cleanup of a known successful scene after durable explicit Stop.
    /// Rejected/unacknowledged checkpoints must remain retained.
    fn discard_checkpoint(&mut self) {}
}
#[derive(Debug, PartialEq, Eq)]
pub enum StopResult {
    Ignored,
    Retained(String),
    Unloaded { error: Option<String> },
}

pub fn stop_game<R: StopRuntime>(
    runtime: &mut Option<R>,
    epoch: u64,
    collection: &mut Collection,
    index: usize,
    mut publish: impl FnMut(&Collection) -> Result<(), String>,
) -> StopResult {
    let Some(active) = runtime.as_mut().filter(|r| r.epoch() == epoch) else {
        return StopResult::Ignored;
    };
    let entry = &mut collection.entries[index];
    entry.paused = true;
    if let Err(error) = active.pause_commit() {
        entry.fail(error.clone());
        entry.runtime_incomplete = true;
        let message = if publish(collection).is_err() {
            format!("{error}\nCould not preserve the error on disk.")
        } else {
            error
        };
        return StopResult::Retained(message);
    }
    // No further guest execution after the acknowledged barrier. Native handles
    // end here; R still owns its OS-backed writer lock and storage binding.
    let joined = active.join();
    let entry = &mut collection.entries[index];
    entry.stopped = true;
    // Explicit Stop requests an ordinary-save cold boot on the next Open.
    // Keep immutable files, including any previously rejected generation.
    entry.checkpoint = None;
    entry.runtime_incomplete = joined.is_err();
    entry.restart_required = joined.is_err();
    let mut error = joined.err();
    if let Some(message) = &error {
        entry.fail(format!("Native teardown failed: {message}"));
    }
    if let Err(message) = publish(collection) {
        let entry = &mut collection.entries[index];
        entry.runtime_incomplete = true;
        let message = format!(
            "The game is unloaded, but clean session metadata was not acknowledged: {message}{}",
            error
                .as_ref()
                .map(|e| format!("\nEarlier error: {e}"))
                .unwrap_or_default()
        );
        entry.fail(message.clone());
        // A rename may have succeeded before a directory-sync error. Attempt to
        // retain truthful recovery metadata; never claim the failed publication.
        let _ = publish(collection);
        error = Some(message);
    }
    if error.is_none() {
        active.discard_checkpoint();
    }
    drop(runtime.take());
    StopResult::Unloaded { error }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{GameId, SaveProfileId, SavedSession};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn diagnostic_callbacks_share_a_hard_cap_without_underflow() {
        let budget = std::sync::Arc::new(DiagnosticBudget::new(64));
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let budget = budget.clone();
                std::thread::spawn(move || (0..100).filter(|_| budget.take()).count())
            })
            .collect();
        assert_eq!(
            threads
                .into_iter()
                .map(|t| t.join().unwrap())
                .sum::<usize>(),
            64
        );
        assert!(!budget.take());
        budget.reset(2); // a new Activity receives its own finite budget
        assert!(budget.take());
        assert!(budget.take());
        assert!(!budget.take());
    }

    struct Runtime {
        epoch: u64,
        fail_barrier: bool,
        fail_join: bool,
        trace: Rc<RefCell<Vec<&'static str>>>,
        profile: Option<std::path::PathBuf>,
        _writer: Option<crate::ProfileLock>,
    }
    impl StopRuntime for Runtime {
        fn epoch(&self) -> u64 {
            self.epoch
        }
        fn pause_commit(&mut self) -> Result<(), String> {
            self.trace.borrow_mut().push("barrier");
            if self.fail_barrier {
                Err("card barrier failed; runtime retained".into())
            } else {
                Ok(())
            }
        }
        fn discard_checkpoint(&mut self) {
            self.trace.borrow_mut().push("discard-checkpoint");
        }
        fn join(&mut self) -> Result<(), String> {
            self.trace.borrow_mut().push("join");
            if let Some(profile) = &self.profile {
                assert!(
                    matches!(crate::ProfileLock::acquire(profile), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock)
                );
            }
            if self.fail_join {
                Err("late writer failure".into())
            } else {
                Ok(())
            }
        }
    }
    impl Drop for Runtime {
        fn drop(&mut self) {
            self.trace.borrow_mut().push("release");
        }
    }
    fn collection() -> Collection {
        Collection {
            entries: vec![SavedSession {
                game_id: GameId("a".repeat(32)),
                profile_id: SaveProfileId("b".repeat(32)),
                uri: "content://fixture/game".into(),
                title: "Original fixture".into(),
                paused: false,
                last_error: None,
                restart_required: false,
                runtime_incomplete: true,
                stopped: false,
                checkpoint: Some("c".repeat(64)),
                shortcut_requested: false,
                shortcut_pinned: None,
            }],
            selected: Some("a".repeat(32)),
            ..Collection::default()
        }
    }
    fn runtime(trace: &Rc<RefCell<Vec<&'static str>>>) -> Option<Runtime> {
        Some(Runtime {
            epoch: 7,
            fail_barrier: false,
            fail_join: false,
            trace: trace.clone(),
            profile: None,
            _writer: None,
        })
    }
    #[test]
    fn clean_stop_publishes_before_release_and_does_not_reboot_or_duplicate() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut live = runtime(&trace);
        let mut c = collection();
        assert_eq!(
            stop_game(&mut live, 7, &mut c, 0, |c| {
                assert!(c.entries[0].stopped && !c.entries[0].runtime_incomplete);
                assert!(c.entries[0].checkpoint.is_none());
                trace.borrow_mut().push("publish");
                Ok(())
            }),
            StopResult::Unloaded { error: None }
        );
        assert_eq!(
            *trace.borrow(),
            [
                "barrier",
                "join",
                "publish",
                "discard-checkpoint",
                "release"
            ]
        );
        assert!(live.is_none());
        assert!(!c.entries[0].boot_allowed());
        let loaded: Collection = serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        assert!(!loaded.entries[0].boot_allowed());
        assert_eq!(loaded.entries[0].profile_id.0, "b".repeat(32));
        assert_eq!(
            stop_game(&mut live, 7, &mut c, 0, |_| panic!("duplicate publication")),
            StopResult::Ignored
        );
        live = runtime(&trace);
        live.as_mut().unwrap().epoch = 8;
        assert_eq!(
            stop_game(&mut live, 7, &mut c, 0, |_| panic!("stale stop")),
            StopResult::Ignored
        );
        assert_eq!(live.as_ref().unwrap().epoch, 8);
    }
    #[test]
    fn failed_barrier_retains_live_owner_and_retry_can_complete() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut live = runtime(&trace);
        let mut c = collection();
        live.as_mut().unwrap().fail_barrier = true;
        assert!(matches!(
            stop_game(&mut live, 7, &mut c, 0, |_| Ok(())),
            StopResult::Retained(_)
        ));
        assert_eq!(*trace.borrow(), ["barrier"]);
        assert!(
            c.entries[0].paused && c.entries[0].runtime_incomplete && c.entries[0].restart_required
        );
        live.as_mut().unwrap().fail_barrier = false;
        assert_eq!(
            stop_game(&mut live, 7, &mut c, 0, |_| Ok(())),
            StopResult::Unloaded { error: None }
        );
        assert!(!c.entries[0].runtime_incomplete && !c.entries[0].boot_allowed());
        c.entries[0].request_restart();
        assert!(c.entries[0].boot_allowed());
        assert!(c.entries[0].last_error.is_some());
    }
    #[test]
    fn late_failure_and_unacknowledged_metadata_retain_recovery() {
        for late_native in [true, false] {
            let trace = Rc::new(RefCell::new(Vec::new()));
            let mut live = runtime(&trace);
            let mut c = collection();
            live.as_mut().unwrap().fail_join = late_native;
            let result = stop_game(&mut live, 7, &mut c, 0, |_| {
                if late_native {
                    Ok(())
                } else {
                    Err("sync failed".into())
                }
            });
            assert!(matches!(result, StopResult::Unloaded { error: Some(_) }));
            assert!(live.is_none());
            assert!(
                c.entries[0].stopped
                    && c.entries[0].runtime_incomplete
                    && c.entries[0].restart_required
            );
            assert!(c.entries[0].last_error.is_some());
            assert!(!c.entries[0].boot_allowed());
            assert_eq!(trace.borrow().last(), Some(&"release"));
            assert!(!trace.borrow().contains(&"discard-checkpoint"));
        }
    }
    #[test]
    fn actual_profile_lock_spans_join_and_metadata_publication() {
        let path = std::env::temp_dir().join(format!("gcx-stop-lock-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut live = runtime(&trace);
        live.as_mut().unwrap()._writer = Some(crate::ProfileLock::acquire(&path).unwrap());
        live.as_mut().unwrap().profile = Some(path.clone());
        let mut c = collection();
        let result = stop_game(&mut live, 7, &mut c, 0, |_| {
            assert!(
                matches!(crate::ProfileLock::acquire(&path), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock)
            );
            Ok(())
        });
        assert_eq!(result, StopResult::Unloaded { error: None });
        drop(crate::ProfileLock::acquire(&path).unwrap());
        std::fs::remove_file(path.join("writer.lock")).unwrap();
        std::fs::remove_dir(path).unwrap();
    }
}
