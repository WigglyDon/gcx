// SPDX-License-Identifier: MIT
use std::{fs, path::Path, process::Command};

pub fn check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .output()?;
    if !output.status.success() {
        return Err("cannot inventory active identity owners".into());
    }
    let mut failures = Vec::new();
    for name in output.stdout.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let name = std::str::from_utf8(name)?;
        let active = name.starts_with("crates/")
            || name.starts_with("xtask/")
            || name.starts_with("android/")
            || name.starts_with("assets/")
            || matches!(
                name,
                "Cargo.toml"
                    | "Cargo.lock"
                    | "native/bridge.cpp"
                    | "native/bridge.h"
                    | "native/CMakeLists.txt"
                    | "native/android_platform.cpp"
                    | "native/host_fixture.cpp"
                    | "native/tests/render_surface.cpp"
                    | "native/patches/0001-direct-adapter.patch"
                    | "scripts/android_packet.py"
                    | "scripts/tests/test_android_packet.py"
            );
        if !active {
            continue;
        }
        if name.to_ascii_lowercase().contains("gcx") {
            failures.push(name.to_owned());
        }
        let path = root.join(name);
        if !path.try_exists()? {
            continue;
        } // tracked rename before index update
        let bytes = fs::read(path)?;
        let Ok(contents) = std::str::from_utf8(&bytes) else {
            continue;
        };
        for (line, value) in contents.lines().enumerate() {
            if !value.to_ascii_lowercase().contains("gcx") {
                continue;
            }
            let allowed = matches!(
                name,
                "crates/ntx/src/legacy.rs" | "xtask/src/legacy.rs" | "xtask/src/identity.rs"
            ) || (name == "android/AndroidManifest.xml"
                && value.trim()
                    == "<data android:scheme=\"machinerhythm-gcx\" android:host=\"play\" android:pathPrefix=\"/\" />")
                || (name == "crates/ntx/src/lib.rs"
                    && value.contains("retained GCX1 fixture marker"))
                || (name == "crates/ntx/src/bin/ntx-checkpoint-probe.rs"
                    && value.trim() == "let marker = b\"GCXCR1\\0\\0\";")
                || (name == "native/patches/0001-direct-adapter.patch"
                    && value.contains("Legacy ABI1 GCX"));
            if !allowed {
                failures.push(format!("{name}:{}", line + 1));
            }
        }
    }
    if !failures.is_empty() {
        return Err(format!(
            "unclassified legacy identity references: {}",
            failures.join(", ")
        )
        .into());
    }
    println!(
        "Active implementation identity audit passed; explicit legacy readers/fixtures/provenance are listed in docs/IDENTITY_MIGRATION.md"
    );
    Ok(())
}
