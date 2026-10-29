// SPDX-License-Identifier: MIT
//! Preserve one checked binary/source pair, without archiving private workspaces.
use crate::{receipt::digest, run};
use std::{fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn output(command: &mut Command) -> Result<Vec<u8>> {
    let result = command.output()?;
    if !result.status.success() {
        return Err(format!("source preservation failed: {command:?}").into());
    }
    Ok(result.stdout)
}
fn bundle(repo: &Path, out: &Path, name: &str) -> Result<()> {
    let archive = out.join(format!("{name}.bundle"));
    run(Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["bundle", "create"])
        .arg(&archive)
        .arg("HEAD"))?;
    let shallow = output(Command::new("git").arg("-C").arg(repo).args([
        "rev-parse",
        "--git-path",
        "shallow",
    ]))?;
    let shallow = String::from_utf8(shallow)?;
    let shallow = repo.join(shallow.trim());
    // Git bundles omit the shallow boundary. Preserve it explicitly, otherwise
    // a successful bundle verification still leaves missing parent commits.
    let boundary = if shallow.exists() {
        fs::read(shallow)?
    } else {
        Vec::new()
    };
    fs::write(out.join(format!("{name}.shallow")), &boundary)?;
    let verify = out.join("verify.git");
    run(Command::new("git")
        .args(["init", "--bare", "--quiet"])
        .arg(&verify))?;
    if !boundary.is_empty() {
        fs::write(verify.join("shallow"), boundary)?;
    }
    run(Command::new("git")
        .arg("-C")
        .arg(&verify)
        .args(["bundle", "unbundle"])
        .arg(&archive))?;
    let head = output(
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["rev-parse", "HEAD"]),
    )?;
    let head = String::from_utf8(head)?;
    run(Command::new("git").arg("-C").arg(&verify).args([
        "update-ref",
        "refs/heads/source",
        head.trim(),
    ]))?;
    run(Command::new("git").arg("-C").arg(&verify).args([
        "fsck",
        "--no-reflogs",
        "--connectivity-only",
    ]))?;
    // Only this newly constructed, checked duplicate object store is retired.
    fs::remove_dir_all(verify)?;
    Ok(())
}
pub fn preserve(root: &Path, source: &Path) -> Result<()> {
    let status = output(
        Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root),
    )?;
    if !status.is_empty() {
        return Err(
            "commit reviewed public source before preserving the binary/source pair".into(),
        );
    }
    let package = root.join("build/ntx-android-package");
    let apk = package.join("ntx-development.apk");
    let binding = fs::read(root.join("build/source-receipt/binding.json"))?;
    let recorded: serde_json::Value = serde_json::from_slice(&binding)?;
    let head = String::from_utf8(output(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root),
    )?)?;
    if recorded["first_party_head"].as_str() != Some(head.trim()) {
        return Err("package the committed source before preserving its candidate".into());
    }
    let embedded = output(
        Command::new("unzip")
            .args(["-p"])
            .arg(&apk)
            .arg("assets/notices/build-source/binding.json"),
    )?;
    if embedded != binding {
        return Err("APK source binding differs from current build receipt".into());
    }
    let inventory = fs::read(root.join("build/source-receipt/source-files.sha256"))?;
    for line in String::from_utf8(inventory.clone())?.lines() {
        let (wanted, path) = line.split_once("  ").ok_or("invalid source inventory")?;
        if digest(&root.join(path))? != wanted {
            return Err("source changed after packaging".into());
        }
    }
    let identity = digest(&apk)?;
    let out = root.join(".local/candidates").join(&identity);
    // Refuse to overwrite even a partial preservation attempt.
    fs::create_dir_all(out.parent().ok_or("missing candidate parent")?)?;
    fs::create_dir(&out)?;
    for name in [
        "ntx-development.apk",
        "signature.txt",
        "manifest.txt",
        "apk-entries.txt",
        "apk-sha256.txt",
        "native-libraries.tsv",
    ] {
        fs::copy(package.join(name), out.join(name))?;
    }
    fs::write(out.join("binding.json"), binding)?;
    fs::write(out.join("source-files.sha256"), inventory)?;
    bundle(root, &out, "ntx")?;
    bundle(source, &out, "dolphin")?;
    let mupen = crate::mupen::source_check(root)?;
    run(Command::new("tar")
        .args(["-czf"])
        .arg(out.join("mupen-active-source.tar.gz"))
        .arg("-C")
        .arg(mupen.parent().ok_or("missing Mupen parent")?)
        .arg("mupen64plus-next"))?;
    fs::copy(
        root.join(".local/source-archives/mupen64plus-next-15bada85.tar.gz"),
        out.join("mupen-upstream-15bada85.tar.gz"),
    )?;
    let mut graph = String::from("dolphin\t.local/source/dolphin\n");
    for (index, path) in include_str!("../../native/dependencies.txt")
        .lines()
        .enumerate()
    {
        let name = format!("dependency-{index:02}");
        bundle(&source.join(path), &out, &name)?;
        graph.push_str(&format!("{name}\t.local/source/dolphin/{path}\n"));
    }
    fs::write(out.join("repositories.tsv"), graph)?;
    let vendor = root.join(".local/corresponding-source-vendor");
    run(Command::new("cargo")
        .args(["vendor", "--offline", "--locked"])
        .arg(&vendor))?;
    run(Command::new("tar")
        .args(["-czf"])
        .arg(out.join("cargo-vendor.tar.gz"))
        .arg("-C")
        .arg(root.join(".local"))
        .arg("corresponding-source-vendor"))?;
    fs::write(
        out.join("REBUILD.txt"),
        include_str!("../../native/CORRESPONDING_SOURCE.txt"),
    )?;
    let mut files = fs::read_dir(&out)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.sort();
    let mut hashes = String::new();
    for file in files {
        hashes.push_str(&format!(
            "{}  {}\n",
            digest(&file)?,
            file.file_name()
                .ok_or("missing artifact name")?
                .to_str()
                .ok_or("invalid artifact name")?
        ));
    }
    fs::write(out.join("SHA256SUMS"), hashes)?;
    println!(
        "Preserved checked private binary/source pair: {}",
        out.display()
    );
    Ok(())
}
