// SPDX-License-Identifier: MIT
//! Durable identities and host session metadata; contains no native handles or machine snapshot.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameId(pub String);
#[derive(Clone, Serialize, Deserialize)]
pub struct SaveProfileId(pub String);
#[derive(Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub game_id: GameId,
    pub profile_id: SaveProfileId,
    pub uri: String,
    pub title: String,
    pub paused: bool,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub restart_required: bool,
}
impl SavedSession {
    pub fn fail(&mut self, error: String) {
        self.paused = true;
        self.restart_required = true;
        self.last_error = Some(error.chars().take(8192).collect());
    }
    /// Explicitly cold boot using the same profile; never selects an older save image.
    pub fn request_restart(&mut self) {
        self.paused = false;
        self.restart_required = false;
    }
}
#[derive(Serialize, Deserialize)]
pub struct Collection {
    pub schema_version: u32,
    pub entries: Vec<SavedSession>,
    pub selected: Option<String>,
}
impl Default for Collection {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: Vec::new(),
            selected: None,
        }
    }
}
pub fn valid_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
impl Collection {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != 1 {
            return Err("unsupported collection schema; original retained");
        }
        if self.entries.len() > 4096 {
            return Err("collection has too many entries; original retained");
        }
        let mut games = BTreeSet::new();
        let mut documents = BTreeSet::new();
        for e in &self.entries {
            if !valid_id(&e.game_id.0)
                || !valid_id(&e.profile_id.0)
                || !e.uri.starts_with("content://")
                || e.uri.len() > 65536
                || e.title.len() > 16384
                || e.last_error.as_ref().is_some_and(|s| s.len() > 32768)
            {
                return Err("invalid collection metadata; original retained");
            }
            if !games.insert(e.game_id.0.as_str()) || !documents.insert(e.uri.as_str()) {
                return Err("ambiguous collection identity; original retained");
            }
        }
        if self
            .selected
            .as_ref()
            .is_some_and(|id| !games.contains(id.as_str()))
        {
            return Err("selected game is absent; original retained");
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entry() -> SavedSession {
        SavedSession {
            game_id: GameId("a".repeat(32)),
            profile_id: SaveProfileId("b".repeat(32)),
            uri: "content://fixture/game".into(),
            title: "Original fixture".into(),
            paused: false,
            last_error: None,
            restart_required: false,
        }
    }
    #[test]
    fn failure_survives_relaunch_and_explicit_restart_keeps_identity_and_error() {
        let mut session = entry();
        session.fail("write failed".into());
        let mut restored: SavedSession =
            serde_json::from_slice(&serde_json::to_vec(&session).unwrap()).unwrap();
        assert!(restored.paused && restored.restart_required);
        restored.request_restart();
        assert!(!restored.paused && !restored.restart_required);
        assert_eq!(restored.last_error.as_deref(), Some("write failed"));
        assert_eq!(restored.profile_id.0, session.profile_id.0);
        assert_eq!(restored.game_id.0, session.game_id.0);
    }
    #[test]
    fn unsupported_schema_and_ambiguous_or_unsafe_identity_are_retained_as_errors() {
        let mut c = Collection {
            entries: vec![entry()],
            selected: Some("a".repeat(32)),
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schema_version = 2;
        assert!(c.validate().is_err());
        c.schema_version = 1;
        c.entries.push(entry());
        assert!(c.validate().is_err());
        c.entries.pop();
        c.entries[0].profile_id.0 = "../other-profile".into();
        assert!(c.validate().is_err());
        c.entries[0] = entry();
        c.selected = Some("c".repeat(32));
        assert!(c.validate().is_err());
    }
}
