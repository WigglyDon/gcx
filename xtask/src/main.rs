// SPDX-License-Identifier: MIT
mod android;
mod corresponding;
mod receipt;
use std::{path::PathBuf, process::Command};
const PIN: &str = "cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b";
fn run(command: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    if !command.status()?.success() {
        return Err(format!("command failed: {command:?}").into());
    }
    Ok(())
}
fn source_check(source: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() || String::from_utf8(output.stdout)?.trim() != PIN {
        return Err("wrong Dolphin revision".into());
    }
    let diff = Command::new("git")
        .arg("-C")
        .arg(source)
        .args(["diff", "--no-ext-diff", "--ignore-submodules=all", "HEAD"])
        .output()?;
    let patch = include_str!("../../native/patches/0001-direct-adapter.patch");
    let patch = &patch[patch
        .find("diff --git ")
        .ok_or("missing native patch body")?..];
    if !diff.status.success() || String::from_utf8(diff.stdout)? != patch {
        return Err("native source changes differ from reviewed adapter patch".into());
    }
    let expected = include_str!("../../native/dolphin-gitlinks.tsv");
    let mut actual = String::new();
    let tree = Command::new("git")
        .arg("-C")
        .arg(source)
        .args(["ls-tree", "-r", "HEAD"])
        .output()?;
    if !tree.status.success() {
        return Err("cannot read Dolphin graph".into());
    }
    for line in String::from_utf8(tree.stdout)?
        .lines()
        .filter(|line| line.starts_with("160000 "))
    {
        let fields: Vec<_> = line.split_whitespace().collect();
        actual.push_str(&format!("{}\t{}\n", fields[2], fields[3]));
    }
    if actual != expected {
        return Err("Dolphin gitlink graph differs from reviewed lock".into());
    }
    for path in include_str!("../../native/dependencies.txt").lines() {
        let wanted = expected
            .lines()
            .find_map(|line| {
                let (sha, name) = line.split_once('\t')?;
                (name == path).then_some(sha)
            })
            .ok_or("dependency absent from graph")?;
        let output = Command::new("git")
            .arg("-C")
            .arg(source.join(path))
            .args(["rev-parse", "HEAD"])
            .output()?;
        if !output.status.success() || String::from_utf8(output.stdout)?.trim() != wanted {
            return Err(format!("unavailable or unpinned dependency: {path}").into());
        }
        let dirty = Command::new("git")
            .arg("-C")
            .arg(source.join(path))
            .args([
                "status",
                "--porcelain",
                "--untracked-files=no",
                "--ignore-submodules=all",
            ])
            .output()?;
        if !dirty.status.success() || !dirty.stdout.is_empty() {
            return Err(format!("modified dependency: {path}").into());
        }
    }
    println!("Verified Dolphin revision and selected dependency gitlinks");
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&root)?;
    let source = root.join(".local/source/dolphin");
    let action = std::env::args().nth(1).unwrap_or_default();
    match action.as_str() {
        "fetch-dependencies" => {
            let mut command = Command::new("timeout");
            command
                .args([
                    "300",
                    "git",
                    "-c",
                    "http.lowSpeedLimit=1024",
                    "-c",
                    "http.lowSpeedTime=30",
                    "-C",
                ])
                .arg(&source)
                .args([
                    "submodule",
                    "update",
                    "--init",
                    "--depth",
                    "1",
                    "--jobs",
                    "2",
                    "--",
                ])
                .args(include_str!("../../native/dependencies.txt").lines());
            run(&mut command)?;
            source_check(&source)?;
        }
        "source-check" => {
            source_check(&source)?;
        }
        "source-receipt" => {
            source_check(&source)?;
            receipt::write(&root, &source)?;
        }
        "prepare-android" => {
            source_check(&source)?;
            android::prepare(&root, &source)?;
        }
        "package-android" => {
            source_check(&source)?;
            android::package(&root, &source)?;
        }
        "preserve-candidate" => {
            source_check(&source)?;
            corresponding::preserve(&root, &source)?;
        }
        "configure-host" | "configure-android" => {
            source_check(&source)?;
            let android = action == "configure-android";
            let mut command = Command::new("cmake");
            command
                .arg("-S")
                .arg(&source)
                .arg("-B")
                .arg(if android {
                    "build/android"
                } else {
                    "build/host"
                })
                .args([
                    "-G",
                    "Ninja",
                    "-UX11_*",
                    "-UXRANDR_*",
                    "-DCMAKE_BUILD_TYPE=Release",
                    "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
                    "-DUSE_SYSTEM_LIBS=OFF",
                    "-DENABLE_QT=OFF",
                    "-DENABLE_X11=OFF",
                    "-DENABLE_NOGUI=OFF",
                    "-DENABLE_CLI_TOOL=OFF",
                    "-DENABLE_TESTS=OFF",
                    "-DENABLE_SDL=OFF",
                    "-DUSE_MGBA=OFF",
                    "-DUSE_UPNP=OFF",
                    "-DUSE_DISCORD_PRESENCE=OFF",
                    "-DUSE_RETRO_ACHIEVEMENTS=OFF",
                    "-DENABLE_ANALYTICS=OFF",
                    "-DENABLE_AUTOUPDATE=OFF",
                    "-DENCODE_FRAMEDUMPS=OFF",
                    "-DENABLE_LLVM=OFF",
                    "-DENABLE_EVDEV=OFF",
                    "-DENABLE_HWDB=OFF",
                    "-DENABLE_ALSA=OFF",
                    "-DENABLE_PULSEAUDIO=OFF",
                    "-DUSE_SANITIZERS=OFF",
                    "-DBUNDLE_SPEEX=ON",
                    "-DDISTRIBUTOR=gcx",
                ])
                .arg(format!(
                    "-DGCX_ADAPTER_DIR={}",
                    root.join("native").display()
                ));
            if android {
                let ndk = PathBuf::from(
                    std::env::var_os("ANDROID_NDK_HOME")
                        .ok_or("set ANDROID_NDK_HOME to existing NDK r30")?,
                );
                if !std::fs::read_to_string(ndk.join("source.properties"))?
                    .lines()
                    .any(|line| line.trim() == "Pkg.Revision = 30.0.16248370")
                {
                    return Err("expected reviewed NDK 30.0.16248370".into());
                }
                command
                    .arg(format!(
                        "-DCMAKE_TOOLCHAIN_FILE={}",
                        ndk.join("build/cmake/android.toolchain.cmake").display()
                    ))
                    .args([
                        "-DANDROID_ABI=arm64-v8a",
                        "-DANDROID_PLATFORM=android-28",
                        "-DANDROID_STL=c++_shared",
                        "-DENABLE_EGL=ON",
                    ]);
            } else {
                command.args([
                    "-DCMAKE_C_COMPILER=clang",
                    "-DCMAKE_CXX_COMPILER=clang++",
                    "-DENABLE_EGL=OFF",
                ]);
            }
            run(&mut command)?;
        }
        "build-host" | "build-android" => {
            source_check(&source)?;
            run(Command::new("cmake").args([
                "--build",
                if action == "build-android" {
                    "build/android"
                } else {
                    "build/host"
                },
                "--target",
                "gcx_engine",
                "--parallel",
                "1",
            ]))?;
        }
        _ => {
            return Err(
                "usage: cargo xtask fetch-dependencies|source-check|configure-host|build-host|configure-android|build-android"
                    .into(),
            );
        }
    }
    Ok(())
}
