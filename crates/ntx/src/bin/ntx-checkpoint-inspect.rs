// SPDX-License-Identifier: MIT
//! Offline inspection of a protected GameCube profile copy; no native library.
use ntx::checkpoint::{self, Binding};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "usage: ntx-checkpoint-inspect PROTECTED_PROFILE CHECKPOINT_ID TRUSTED_BINDING_JSON"
                .into(),
        );
    }
    let profile = PathBuf::from(&args[0]);
    let id = args[1].to_str().ok_or("checkpoint identity encoding")?;
    let binding_file = PathBuf::from(&args[2]);
    let info = binding_file.symlink_metadata()?;
    if !info.is_file() || info.len() > 8192 {
        return Err(io::Error::other(
            "expected binding must be a bounded regular independent receipt",
        )
        .into());
    }
    let mut bytes = Vec::new();
    File::open(binding_file)?
        .take(8193)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8192 {
        return Err("expected binding grew beyond its limit".into());
    }
    let expected: Binding = serde_json::from_slice(&bytes)?;
    let state = checkpoint::load(&profile, id, &expected)?;
    state.verify_current_saves(&profile)?;
    println!(
        "PASS read-only checkpoint inspection: {} payload bytes; captured pause intent={}. No native boot, restore or profile mutation.",
        state.payload.len(),
        state.paused
    );
    Ok(())
}
