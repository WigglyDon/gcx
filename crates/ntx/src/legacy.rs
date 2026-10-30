// SPDX-License-Identifier: MIT
//! Immutable legacy identifiers, used only while reading retained data.
pub(crate) const PLAY_URI_PREFIX: &str = "machinerhythm-gcx://play/";
pub(crate) const CHECKPOINT_MAGIC: &[u8; 8] = b"GCXCP001";
pub(crate) const CHECKPOINT_EXTENSION: &str = "gcx";
/// Exact schema-1 diagnostic for a retained import that failed disc inspection.
pub(crate) const UNSUPPORTED_DISC_IMPORT: &str =
    "Choose a supported GameCube disc image (ISO/GCM or RVZ).";
