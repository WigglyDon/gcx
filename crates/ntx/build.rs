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
    println!("cargo:rerun-if-env-changed=NTX_ENGINE_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_NATIVE_ENGINE").is_some() {
        // cargo check can validate Rust/Android without a native rebuild. A
        // linked build still requires this real library, and runtime binding
        // cannot be constructed without measured engine/resource fingerprints.
        println!("cargo:rustc-link-lib=dylib=ntx_engine");
        let Some(path) = std::env::var_os("NTX_ENGINE_LIB_DIR") else {
            println!(
                "cargo:warning=Native fingerprints unavailable; Rust type checking only, no runnable backend claim"
            );
            return;
        };
        let path = path.to_str().expect("UTF8 native library directory");
        println!("cargo:rustc-link-search=native={path}");
        let mut engine = Sha256::new();
        update_file(&mut engine, &Path::new(&path).join("libntx_engine.so"));
        println!("cargo:rustc-env=NTX_ENGINE_SHA256={:x}", engine.finalize());
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
            .update(b"ntx:skip-ipl;dsp-hle;raw-card-a;no-card-b;gamecube-pad-p1;checkpoint-abi1;checked-profile-read1\0");
        for name in names {
            resources.update(name.as_bytes());
            resources.update([0]);
            update_file(&mut resources, &sys.join(name));
        }
        println!("cargo:rustc-env=NTX_RESOURCE_ID={:x}", resources.finalize());
    }
}
