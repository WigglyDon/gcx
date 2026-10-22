// SPDX-License-Identifier: MIT
use crate::run;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub struct Tools {
    pub build: PathBuf,
    pub platform: PathBuf,
}
impl Tools {
    pub fn existing() -> Result<Self> {
        let sdk = PathBuf::from(
            std::env::var_os("ANDROID_HOME")
                .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
                .ok_or("set ANDROID_HOME to the existing SDK")?,
        );
        let build = sdk.join("build-tools/37.0.0");
        let platform = sdk.join("platforms/android-37.0/android.jar");
        if !platform.is_file() || !build.join("aapt2").is_file() || !build.join("d8").is_file() {
            return Err("reviewed Android 37 platform/build-tools 37.0.0 unavailable".into());
        }
        Ok(Self { build, platform })
    }
}
fn copy(source: &Path, destination: &Path) -> Result<()> {
    if source.symlink_metadata()?.file_type().is_symlink() || !source.is_file() {
        return Err("package input must be an ordinary file".into());
    }
    fs::create_dir_all(destination.parent().ok_or("missing package parent")?)?;
    fs::copy(source, destination)?;
    Ok(())
}
pub fn prepare(root: &Path, source: &Path) -> Result<()> {
    crate::receipt::write(root, source)?;
    let tools = Tools::existing()?;
    let out = root.join("build/android-package");
    let assets = out.join("assets");
    if assets.exists() {
        fs::remove_dir_all(&assets)?;
    }
    fs::create_dir_all(&assets)?;
    let sys = source.join("Data/Sys");
    let mut resources: Vec<String> = include_str!("../../native/resources.txt")
        .lines()
        .map(str::to_owned)
        .collect();
    for entry in fs::read_dir(sys.join("GameSettings"))? {
        let path = entry?.path();
        let name = path
            .file_name()
            .ok_or("missing settings name")?
            .to_str()
            .ok_or("non-UTF8 settings name")?;
        if name.ends_with(".ini") && (name.starts_with('G') || name.starts_with('D')) {
            resources.push(format!("GameSettings/{name}"));
        }
    }
    resources.sort();
    resources.dedup();
    for relative in &resources {
        copy(&sys.join(relative), &assets.join("Sys").join(relative))?;
    }
    fs::write(
        assets.join("resource-index.txt"),
        resources.join("\n") + "\n",
    )?;
    let notices = assets.join("notices");
    copy(&root.join("LICENSE"), &notices.join("gcx-MIT.txt"))?;
    copy(
        &root.join("native/COPYING"),
        &notices.join("Dolphin-COPYING.txt"),
    )?;
    copy(
        &root.join("native/README.md"),
        &notices.join("engine-source.md"),
    )?;
    copy(
        &root.join("native/RESOURCE_AUDIT.md"),
        &notices.join("resource-audit.md"),
    )?;
    for name in ["binding.json", "source-files.sha256"] {
        copy(
            &root.join("build/source-receipt").join(name),
            &notices.join("build-source").join(name),
        )?;
    }
    let sysroot = PathBuf::from(output(Command::new("rustc").args(["--print", "sysroot"]))?.trim());
    let rustdoc = sysroot.join("share/doc/rust");
    copy(
        &rustdoc.join("COPYRIGHT-library.html"),
        &notices.join("rust-toolchain/COPYRIGHT-library.html"),
    )?;
    for entry in fs::read_dir(rustdoc.join("licenses"))? {
        let path = entry?.path();
        if path.is_file() {
            copy(
                &path,
                &notices
                    .join("rust-toolchain/licenses")
                    .join(path.file_name().ok_or("missing Rust license name")?),
            )?;
        }
    }
    fs::write(
        notices.join("rust-toolchain/version.txt"),
        output(Command::new("rustc").arg("-vV"))?,
    )?;
    if let Some(ndk) = std::env::var_os("ANDROID_NDK_HOME") {
        let ndk = PathBuf::from(ndk);
        for name in ["NOTICE", "NOTICE.toolchain", "source.properties"] {
            copy(&ndk.join(name), &notices.join("ndk").join(name))?;
        }
    }
    for path in [
        "native/dolphin-gitlinks.tsv",
        "native/dependencies.txt",
        "native/patches/0001-direct-adapter.patch",
        "Cargo.lock",
    ] {
        copy(&root.join(path), &notices.join(path))?;
    }
    for entry in fs::read_dir(source.join("LICENSES"))? {
        let p = entry?.path();
        if p.is_file() {
            copy(
                &p,
                &notices
                    .join("Dolphin-LICENSES")
                    .join(p.file_name().ok_or("missing license name")?),
            )?;
        }
    }
    // Preserve dependency notices recursively, including component-specific terms.
    // Corresponding-source delivery remains a separate, required artifact.
    fn licenses(from: &Path, base: &Path, to: &Path) -> Result<()> {
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            if name == ".git" || entry.file_type()?.is_symlink() {
                continue;
            }
            if entry.file_type()?.is_dir() {
                licenses(&path, base, to)?;
            } else {
                let upper = name.to_string_lossy().to_ascii_uppercase();
                if upper.starts_with("LICENSE")
                    || upper.starts_with("LICENCE")
                    || upper.starts_with("COPYING")
                    || upper == "NOTICE"
                    || upper == "AUTHORS"
                    || upper.starts_with("COPYRIGHT")
                {
                    copy(&path, &to.join(path.strip_prefix(base)?))?;
                }
            }
        }
        Ok(())
    }
    licenses(
        &source.join("Externals"),
        &source.join("Externals"),
        &notices.join("native-dependencies"),
    )?;
    let metadata = output(Command::new("cargo").args([
        "metadata",
        "--offline",
        "--locked",
        "--format-version",
        "1",
        "--filter-platform",
        "aarch64-linux-android",
    ]))?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata)?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("missing Cargo packages")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing Cargo graph")?;
    let android = packages
        .iter()
        .find(|p| p["name"] == "gcx-android")
        .ok_or("missing Android package")?;
    let mut pending = vec![
        android["id"]
            .as_str()
            .ok_or("missing Cargo identity")?
            .to_owned(),
    ];
    let mut selected = std::collections::BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !selected.insert(id.clone()) {
            continue;
        }
        let node = nodes
            .iter()
            .find(|n| n["id"] == id)
            .ok_or("incomplete Cargo dependency graph")?;
        for dep in node["dependencies"]
            .as_array()
            .ok_or("missing Cargo dependencies")?
        {
            pending.push(
                dep.as_str()
                    .ok_or("invalid Cargo dependency identity")?
                    .to_owned(),
            );
        }
    }
    let mut rust_inventory = String::new();
    for package in packages
        .iter()
        .filter(|p| p["id"].as_str().is_some_and(|id| selected.contains(id)))
    {
        let name = package["name"].as_str().ok_or("missing crate name")?;
        let version = package["version"].as_str().ok_or("missing crate version")?;
        let manifest = PathBuf::from(
            package["manifest_path"]
                .as_str()
                .ok_or("missing crate manifest")?,
        );
        let base = manifest.parent().ok_or("missing crate directory")?;
        let destination = notices
            .join("rust-dependencies")
            .join(format!("{name}-{version}"));
        if package["source"].is_null() {
            copy(&root.join("LICENSE"), &destination.join("LICENSE"))?;
        } else {
            licenses(base, base, &destination)?;
            if let Some(file) = package["license_file"].as_str() {
                copy(&base.join(file), &destination.join("declared-license.txt"))?;
            }
            if !destination.exists() {
                let supplement = root
                    .join("native/rust-notices")
                    .join(format!("{name}-{version}"));
                let vcs: serde_json::Value =
                    serde_json::from_slice(&fs::read(base.join(".cargo_vcs_info.json"))?)?;
                let revision = vcs["git"]["sha1"]
                    .as_str()
                    .ok_or("missing crate source revision")?;
                let origin = fs::read_to_string(supplement.join("SOURCE"))?;
                if !origin.contains(revision) {
                    return Err("supplemental license revision differs from crate".into());
                }
                copy(
                    &supplement.join("LICENSE-MIT"),
                    &destination.join("LICENSE-MIT"),
                )?;
                copy(&supplement.join("SOURCE"), &destination.join("SOURCE"))?;
            }
        }
        rust_inventory.push_str(&format!(
            "{name}\t{version}\t{}\n",
            package["license"].as_str().unwrap_or("license-file")
        ));
    }
    fs::write(notices.join("rust-dependencies.tsv"), rust_inventory)?;
    let mut inventory = String::new();
    for relative in &resources {
        let output = Command::new("sha256sum").arg(sys.join(relative)).output()?;
        if !output.status.success() {
            return Err("resource hashing failed".into());
        }
        let digest = String::from_utf8(output.stdout)?
            .split_whitespace()
            .next()
            .ok_or("missing digest")?
            .to_owned();
        inventory.push_str(&format!("{digest}  Sys/{relative}\n"));
    }
    fs::write(assets.join("resource-sha256.txt"), inventory)?;
    let classes = out.join("classes");
    if classes.exists() {
        fs::remove_dir_all(&classes)?;
    }
    fs::create_dir_all(&classes)?;
    run(Command::new("java")
        .args([
            "-m",
            "jdk.compiler/com.sun.tools.javac.Main",
            "-source",
            "8",
            "-target",
            "8",
            "-Xlint:all,-options",
            "-Werror",
            "-bootclasspath",
        ])
        .arg(&tools.platform)
        .arg("-d")
        .arg(&classes)
        .arg(root.join("android/GcxActivity.java")))?;
    let dex = out.join("dex");
    fs::create_dir_all(&dex)?;
    run(Command::new(tools.build.join("d8"))
        .args(["--release", "--min-api", "28", "--lib"])
        .arg(&tools.platform)
        .arg("--output")
        .arg(&dex)
        .args(
            fs::read_dir(classes.join("com/machinerhythm/gcx"))?
                .map(|entry| entry.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()?,
        ))?;
    println!(
        "Prepared audited resource selection and mechanical Android trampoline; no APK or execution claimed"
    );
    Ok(())
}

fn output(command: &mut Command) -> Result<String> {
    let result = command.output()?;
    if !result.status.success() {
        return Err(format!("package inspection failed: {command:?}").into());
    }
    Ok(String::from_utf8(result.stdout)?)
}
fn audit_elf(file: &Path, ndk: &Path) -> Result<()> {
    let readelf = ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf");
    let header = output(Command::new(&readelf).args(["-h"]).arg(file))?;
    if !header.contains("AArch64") || !header.contains("ELF64") {
        return Err("non-ARM64 native library".into());
    }
    let segments = output(Command::new(&readelf).args(["-W", "-l"]).arg(file))?;
    let mut loads = 0;
    for line in segments
        .lines()
        .filter(|line| line.trim_start().starts_with("LOAD "))
    {
        let last = line
            .split_whitespace()
            .last()
            .ok_or("ELF load alignment missing")?;
        if u64::from_str_radix(last.trim_start_matches("0x"), 16)? < 16384 {
            return Err(format!(
                "native library lacks 16 KiB load alignment: {}",
                file.display()
            )
            .into());
        }
        loads += 1;
    }
    if loads == 0 {
        return Err("no native load segments".into());
    }
    let dynamic = output(Command::new(&readelf).args(["-d"]).arg(file))?;
    for line in dynamic.lines().filter(|line| line.contains("(NEEDED)")) {
        let name = line
            .split_once('[')
            .and_then(|(_, s)| s.split_once(']'))
            .map(|(s, _)| s)
            .ok_or("invalid NEEDED entry")?;
        let allowed = [
            "libgcx_engine.so",
            "libc++_shared.so",
            "libandroid.so",
            "liblog.so",
            "libc.so",
            "libm.so",
            "libdl.so",
            "libz.so",
            "libEGL.so",
            "libGLESv2.so",
            "libGLESv3.so",
            "libOpenSLES.so",
            "libaaudio.so",
            "libvulkan.so",
        ];
        if !allowed.contains(&name) {
            return Err(format!("unresolved package dependency {name}").into());
        }
    }
    Ok(())
}
pub fn package(root: &Path, source: &Path) -> Result<()> {
    let tools = Tools::existing()?;
    let ndk = PathBuf::from(
        std::env::var_os("ANDROID_NDK_HOME").ok_or("set ANDROID_NDK_HOME to reviewed NDK r30")?,
    );
    if !fs::read_to_string(ndk.join("source.properties"))?
        .lines()
        .any(|s| s.trim() == "Pkg.Revision = 30.0.16248370")
    {
        return Err("expected NDK 30.0.16248370".into());
    }
    let engine = root.join("build/android/gcx/libgcx_engine.so");
    if !engine.is_file() {
        return Err("build the pinned Android native engine first".into());
    }
    // Preserve this exact debug image before a later native link replaces it.
    // The package receipt records its content hash beside the stripped copy.
    let engine_hash = crate::receipt::digest(&engine)?;
    let symbols = root.join(".local/symbols");
    fs::create_dir_all(&symbols)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&symbols, fs::Permissions::from_mode(0o700))?;
    }
    let debug = symbols.join(format!("{engine_hash}.libgcx_engine.so"));
    if !debug.exists() {
        copy(&engine, &debug)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&debug, fs::Permissions::from_mode(0o600))?;
        }
    }
    if crate::receipt::digest(&debug)? != engine_hash {
        return Err("retained native debug image differs from the package input".into());
    }
    prepare(root, source)?;
    let linker =
        ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android28-clang");
    run(Command::new("cargo")
        .args([
            "rustc",
            "--offline",
            "--locked",
            "--release",
            "-p",
            "gcx-android",
            "--target",
            "aarch64-linux-android",
            "--",
            "-C",
            "link-arg=-Wl,-z,max-page-size=16384",
        ])
        .env(
            "GCX_ENGINE_LIB_DIR",
            engine.parent().ok_or("missing engine directory")?,
        )
        .env("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", linker))?;
    let out = root.join("build/android-package");
    let payload = out.join("payload");
    if payload.exists() {
        fs::remove_dir_all(&payload)?;
    }
    let native = payload.join("lib/arm64-v8a");
    fs::create_dir_all(&native)?;
    let rust = root.join("target/aarch64-linux-android/release/libgcx_android.so");
    let cpp=ndk.join("toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so");
    let mut native_hashes = String::from("source_sha256\tpackaged_sha256\tname\n");
    for p in [&engine, &rust, &cpp] {
        audit_elf(p, &ndk)?;
        let name = p.file_name().ok_or("missing native name")?;
        let packaged = native.join(name);
        copy(p, &packaged)?;
        // Retain the unstripped build for debugging. Ship only the executable
        // image and dynamic symbols; donor dependencies can contain DWARF even
        // in Release configuration.
        run(
            Command::new(ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"))
                .arg("--strip-unneeded")
                .arg(&packaged),
        )?;
        audit_elf(&packaged, &ndk)?;
        native_hashes.push_str(&format!(
            "{}\t{}\t{}\n",
            crate::receipt::digest(p)?,
            crate::receipt::digest(&packaged)?,
            name.to_str().ok_or("invalid native name")?
        ));
    }
    fs::write(out.join("native-libraries.tsv"), native_hashes)?;
    let symbols = output(
        Command::new(ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf"))
            .args(["--dyn-syms", "--wide"])
            .arg(native.join("libgcx_android.so")),
    )?;
    for symbol in [
        "ANativeActivity_onCreate",
        "Java_com_machinerhythm_gcx_GcxActivity_documentResult",
        "Java_com_machinerhythm_gcx_GcxActivity_uiAction",
        "Java_com_machinerhythm_gcx_GcxActivity_beforeDestroy",
        "Java_com_machinerhythm_gcx_GcxActivity_windowChanged",
        "Java_com_machinerhythm_gcx_GcxActivity_windowFocus",
    ] {
        if !symbols.lines().any(|line| {
            line.split_whitespace().last() == Some(symbol)
                && !line.split_whitespace().any(|s| s == "UND")
        }) {
            return Err(format!("missing defined Android entry point {symbol}").into());
        }
    }
    let exports = output(
        Command::new(ndk.join("toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf"))
            .args(["--dyn-syms", "--wide"])
            .arg(native.join("libgcx_engine.so")),
    )?;
    for line in include_str!("../../native/bridge.h")
        .lines()
        .filter(|s| s.starts_with("GCX_EXPORT "))
    {
        let symbol = line
            .split('(')
            .next()
            .and_then(|s| s.split_whitespace().last())
            .ok_or("invalid native ABI declaration")?;
        if !exports.lines().any(|line| {
            line.split_whitespace().last() == Some(symbol)
                && !line.split_whitespace().any(|s| s == "UND")
        }) {
            return Err(format!("missing defined native ABI export {symbol}").into());
        }
    }
    copy(&out.join("dex/classes.dex"), &payload.join("classes.dex"))?;
    let unsigned = out.join("unsigned.apk");
    run(Command::new(tools.build.join("aapt2"))
        .args(["link", "--manifest"])
        .arg(root.join("android/AndroidManifest.xml"))
        .arg("-I")
        .arg(&tools.platform)
        .arg("-A")
        .arg(out.join("assets"))
        .arg("-o")
        .arg(&unsigned))?;
    run(Command::new("zip")
        .args(["-q", "-0", "-r"])
        .arg(&unsigned)
        .args(["classes.dex", "lib"])
        .current_dir(&payload))?;
    let aligned = out.join("aligned.apk");
    run(Command::new(tools.build.join("zipalign"))
        .args(["-f", "-P", "16", "4"])
        .arg(&unsigned)
        .arg(&aligned))?;
    let signing = root.join(".local/signing");
    fs::create_dir_all(&signing)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&signing, fs::Permissions::from_mode(0o700))?;
    }
    let key = signing.join("gcx-development.p12");
    let password = signing.join("password.private");
    if !key.exists() {
        if password.exists() {
            return Err("partial gcx signer exists; preserve and reconcile it".into());
        }
        use std::io::{Read, Write};
        let mut bytes = [0_u8; 32];
        fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
        let secret: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&password)?;
        file.write_all(secret.as_bytes())?;
        file.sync_all()?;
        run(Command::new("keytool")
            .args(["-genkeypair", "-keystore"])
            .arg(&key)
            .args([
                "-storetype",
                "PKCS12",
                "-alias",
                "gcx-development",
                "-keyalg",
                "RSA",
                "-keysize",
                "3072",
                "-validity",
                "10000",
                "-dname",
                "CN=gcx local development",
                "-storepass:file",
            ])
            .arg(&password))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key, fs::Permissions::from_mode(0o600))?;
        }
        let backup = root.join(".local/backups/signer-initial");
        fs::create_dir_all(&backup)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&backup, fs::Permissions::from_mode(0o700))?;
        }
        copy(&key, &backup.join("gcx-development.p12"))?;
        copy(&password, &backup.join("password.private"))?;
    }
    if !password.exists() {
        return Err("gcx signer password unavailable; preserve signer".into());
    }
    let apk = out.join("gcx-development.apk");
    run(Command::new(tools.build.join("apksigner"))
        .args(["sign", "--ks"])
        .arg(&key)
        .args(["--ks-key-alias", "gcx-development", "--ks-pass"])
        .arg(format!("file:{}", password.display()))
        .arg("--out")
        .arg(&apk)
        .arg(&aligned))?;
    let verification = output(
        Command::new(tools.build.join("apksigner"))
            .args(["verify", "--verbose", "--print-certs"])
            .arg(&apk),
    )?;
    fs::write(out.join("signature.txt"), verification)?;
    run(Command::new(tools.build.join("zipalign"))
        .args(["-c", "-P", "16", "4"])
        .arg(&apk))?;
    let manifest = output(
        Command::new(tools.build.join("aapt2"))
            .args(["dump", "badging"])
            .arg(&apk),
    )?;
    if !manifest.contains("package: name='com.machinerhythm.gcx'")
        || !manifest.contains("native-code: 'arm64-v8a'")
        || manifest.contains("uses-permission:")
    {
        return Err("APK namespace/ABI/permission contract differs".into());
    }
    fs::write(out.join("manifest.txt"), manifest)?;
    let entries = output(Command::new("unzip").arg("-Z1").arg(&apk))?;
    for line in entries.lines() {
        if line.starts_with("assets/Sys/") {
            let name = line.trim_start_matches("assets/Sys/");
            let index = fs::read_to_string(out.join("assets/resource-index.txt"))?;
            if !index.lines().any(|s| s == name) {
                return Err("unreviewed Sys asset in APK".into());
            }
        }
        if line.ends_with(".so") && !line.starts_with("lib/arm64-v8a/") {
            return Err("unexpected native ABI".into());
        }
    }
    fs::write(out.join("apk-entries.txt"), entries)?;
    for relative in fs::read_to_string(out.join("assets/resource-index.txt"))?.lines() {
        let result = Command::new("unzip")
            .args(["-p"])
            .arg(&apk)
            .arg(format!("assets/Sys/{relative}"))
            .output()?;
        if !result.status.success()
            || result.stdout != fs::read(out.join("assets/Sys").join(relative))?
        {
            return Err("actual APK resource bytes differ from reviewed staging".into());
        }
    }
    for name in ["libgcx_engine.so", "libgcx_android.so", "libc++_shared.so"] {
        let result = Command::new("unzip")
            .args(["-p"])
            .arg(&apk)
            .arg(format!("lib/arm64-v8a/{name}"))
            .output()?;
        if !result.status.success() || result.stdout != fs::read(native.join(name))? {
            return Err("actual APK native bytes differ from audited libraries".into());
        }
    }
    run(Command::new("unzip").arg("-tq").arg(&apk))?;
    let digest = output(Command::new("sha256sum").arg(&apk))?;
    fs::write(out.join("apk-sha256.txt"), digest)?;
    println!(
        "Private development APK built and checked; no install, gameplay or public release claimed"
    );
    Ok(())
}
