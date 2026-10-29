// SPDX-License-Identifier: MIT
//! Exact source graph and bounded upstream build for the N64 execution adapter.
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const PIN: &str = "15bada85906593376e8fd697f1b0b3af3a20f512";
pub fn source_check(root: &Path) -> Result<PathBuf> {
    let source = root.join(".local/source/mupen64plus-next");
    let mut expected = BTreeSet::new();
    let patched: std::collections::BTreeMap<_, _> =
        include_str!("../../native/mupen/patched-files.sha256")
            .lines()
            .map(|s| {
                s.split_once("  ")
                    .map(|(digest, name)| (name, digest))
                    .ok_or("invalid patched source manifest")
            })
            .collect::<std::result::Result<_, _>>()?;
    for row in include_str!("../../native/mupen/source-files.sha256").lines() {
        let (digest, name) = row
            .split_once("  ")
            .ok_or("invalid Mupen source manifest")?;
        if Path::new(name).is_absolute()
            || Path::new(name)
                .components()
                .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            return Err("unsafe Mupen source path".into());
        }
        let path = source.join(name);
        if !path.symlink_metadata()?.is_file() {
            return Err("nonregular pinned Mupen input".into());
        }
        let mut input = fs::File::open(path)?;
        let mut hash = Sha256::new();
        let mut bytes = [0; 65536];
        loop {
            let n = input.read(&mut bytes)?;
            if n == 0 {
                break;
            }
            hash.update(&bytes[..n]);
        }
        let active_digest = patched.get(name).copied().unwrap_or(digest);
        if format!("{:x}", hash.finalize()) != active_digest {
            return Err(format!("Mupen source differs: {name}").into());
        }
        if !expected.insert(name.to_owned()) {
            return Err("duplicate Mupen source identity".into());
        }
    }
    fn inventory(dir: &Path, root: &Path, files: &mut BTreeSet<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            let info = path.symlink_metadata()?;
            if info.file_type().is_symlink() {
                return Err("symlink in Mupen source graph".into());
            }
            if info.is_dir() {
                inventory(&path, root, files)?;
            } else if info.is_file() {
                files.insert(
                    path.strip_prefix(root)?
                        .to_str()
                        .ok_or("non-UTF8 Mupen path")?
                        .to_owned(),
                );
            } else {
                return Err("nonregular Mupen source graph".into());
            }
        }
        Ok(())
    }
    let mut actual = BTreeSet::new();
    inventory(&source, &source, &mut actual)?;
    if actual != expected {
        return Err("Mupen source graph has missing or unreviewed extra files".into());
    }
    if patched.keys().any(|name| !expected.contains(*name)) {
        return Err("patch names an unpinned file".into());
    }
    println!(
        "Verified {} exact Mupen source files at {PIN} plus the declared state-result repair; optional gnulib gitlink is outside the selected HLE build",
        expected.len()
    );
    Ok(source)
}
pub fn build(root: &Path) -> Result<()> {
    let source = source_check(root)?;
    crate::run(Command::new("python3").arg(root.join("native/mupen/check_boundary.py")))?;
    let ndk = PathBuf::from(
        std::env::var_os("ANDROID_NDK_HOME").ok_or("set the existing reviewed ANDROID_NDK_HOME")?,
    );
    if !fs::read_to_string(ndk.join("source.properties"))?
        .lines()
        .any(|s| s.trim() == "Pkg.Revision = 30.0.16248370")
    {
        return Err("expected reviewed NDK r30".into());
    }
    let out = root.join("build/ntx-mupen");
    let mut command = Command::new(ndk.join("ndk-build"));
    command
        .arg("-j1")
        .arg("-C")
        .arg(&source)
        .arg(format!("NDK_PROJECT_PATH={}", source.display()))
        .arg(format!(
            "APP_BUILD_SCRIPT={}",
            source.join("libretro/jni/Android.mk").display()
        ))
        .arg(format!(
            "NDK_APPLICATION_MK={}",
            source.join("libretro/jni/Application.mk").display()
        ))
        .args([
            "APP_ABI=arm64-v8a",
            "APP_PLATFORM=android-28",
            "GLES3=1",
            "LLE=0",
            "HAVE_PARALLEL_RSP=0",
            "HAVE_PARALLEL_RDP=0",
            "HAVE_THR_AL=0",
            "APP_LDFLAGS=-Wl,-z,max-page-size=16384",
        ])
        .arg(format!("GIT_VERSION={PIN}"))
        .arg(format!("NDK_OUT={}", out.join("obj").display()))
        .arg(format!("NDK_LIBS_OUT={}", out.join("lib").display()));
    crate::run(&mut command)?;
    let input = out.join("obj/local/arm64-v8a/libretro.so");
    let output = out.join("lib/arm64-v8a/libntx_mupen64plus_next.so");
    fs::copy(input, output)?;
    Ok(())
}
