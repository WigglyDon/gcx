// SPDX-License-Identifier: MIT
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};
fn update_file(hash: &mut Sha256, path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
    let mut file = fs::File::open(path).expect("available pinned native/resource input");
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes).expect("read native/resource input");
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
}
fn main() {
    println!("cargo:rerun-if-env-changed=GCX_ENGINE_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_NATIVE_ENGINE").is_some() {
        let path = std::env::var("GCX_ENGINE_LIB_DIR")
            .expect("set GCX_ENGINE_LIB_DIR to the built gcx_engine library directory");
        println!("cargo:rustc-link-search=native={path}");
        println!("cargo:rustc-link-lib=dylib=gcx_engine");
        let mut engine = Sha256::new();
        update_file(&mut engine, &Path::new(&path).join("libgcx_engine.so"));
        println!("cargo:rustc-env=GCX_ENGINE_SHA256={:x}", engine.finalize());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let sys = root.join(".local/source/dolphin/Data/Sys");
        let selection = root.join("native/resources.txt");
        println!("cargo:rerun-if-changed={}", selection.display());
        let mut names: Vec<String> = fs::read_to_string(selection)
            .expect("resource selection")
            .lines()
            .map(str::to_owned)
            .collect();
        for item in fs::read_dir(sys.join("GameSettings")).expect("pinned game settings") {
            let item = item.expect("game settings entry");
            let name = item.file_name().into_string().expect("resource name");
            if name.ends_with(".ini") && (name.starts_with('G') || name.starts_with('D')) {
                names.push(format!("GameSettings/{name}"));
            }
        }
        names.sort();
        names.dedup();
        let mut resources = Sha256::new();
        resources
            .update(b"gcx:skip-ipl;dsp-hle;raw-card-a;no-card-b;gamecube-pad-p1;checkpoint-abi1;checked-profile-read1\0");
        for name in names {
            resources.update(name.as_bytes());
            resources.update([0]);
            update_file(&mut resources, &sys.join(name));
        }
        println!("cargo:rustc-env=GCX_RESOURCE_ID={:x}", resources.finalize());
    }
}
