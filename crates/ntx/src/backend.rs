// SPDX-License-Identifier: MIT
//! Portable routing. Host resources and native instances never identify a game.
use serde::{Deserialize, Serialize};

#[cfg(feature = "native-engine")]
pub mod dolphin;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum System {
    GameCube,
    Nintendo64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backend {
    Dolphin,
    Mupen64PlusNext,
}

impl Backend {
    pub fn system(self) -> System {
        match self {
            Self::Dolphin => System::GameCube,
            Self::Mupen64PlusNext => System::Nintendo64,
        }
    }
}
