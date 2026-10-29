// SPDX-License-Identifier: MIT
//! Signing identity is immutable provenance, not a branding choice.
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const SIGNER_ALIAS: &str = "gcx-development";

pub fn verify_certificate(
    root: &Path,
    key: &Path,
    password: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    let export = std::process::Command::new("keytool")
        .args(["-exportcert", "-keystore"])
        .arg(key)
        .args(["-alias", SIGNER_ALIAS, "-storepass:file"])
        .arg(password)
        .output()?;
    if !export.status.success() {
        return Err("retained signing certificate could not be exported for verification".into());
    }
    let digest = format!("{:x}", Sha256::digest(export.stdout));
    let receipt = fs::read_to_string(root.join(".local/candidates/b4c921276bb21f5b4392a5f058ad75168203dfdf9099484cb8e3c816f253ff3a/signature.txt"))?;
    if !receipt.lines().any(|line| {
        line.split_once("certificate SHA-256 digest: ")
            .is_some_and(|(_, value)| value == digest)
    }) {
        return Err(
            "signing certificate differs from protected installed candidate receipt".into(),
        );
    }
    Ok(())
}

pub fn preserved_signer(root: &Path) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let signing = root.join(".local/signing");
    let backup = root.join(".local/backups/signer-initial");
    let original = signing.join("gcx-development.p12");
    let canonical = signing.join("ntx-development.p12");
    let password = signing.join("password.private");
    // No key generation fallback. Missing recovery material is an error.
    for (actual, recovery) in [
        (&original, backup.join("gcx-development.p12")),
        (&password, backup.join("password.private")),
    ] {
        if !actual.symlink_metadata()?.is_file()
            || !recovery.symlink_metadata()?.is_file()
            || fs::read(actual)? != fs::read(recovery)?
        {
            return Err(
                "retained signer differs from independent recovery; preserve and reconcile".into(),
            );
        }
    }
    if !canonical.try_exists()? {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&canonical)?;
        output.write_all(&fs::read(&original)?)?;
        output.sync_all()?;
        fs::File::open(&signing)?.sync_all()?;
    }
    if !canonical.symlink_metadata()?.is_file() || fs::read(&canonical)? != fs::read(&original)? {
        return Err(
            "canonical signer differs from retained identity; no replacement generated".into(),
        );
    }
    Ok((canonical, password))
}
