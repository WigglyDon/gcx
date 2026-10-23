// SPDX-License-Identifier: MIT
//! Original CPU/EXI checkpoint regression. No games, firmware or owner profiles.
#[cfg(target_os = "android")]
fn main() {
    eprintln!("Host-only fixture; not packaged on Android.");
}
#[cfg(not(target_os = "android"))]
mod host {
    use gcx::{
        ProfileLock,
        checkpoint::{self, Binding},
        engine::*,
        storage,
    };
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use std::{
        ffi::{CStr, CString, c_char, c_void},
        fs,
        io::{self, Write},
        path::{Path, PathBuf},
        process::Command,
        time::{Duration, Instant},
    };
    type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
    unsafe extern "C" {
        fn gcx_fixture_card(operation: u32, value: u32) -> u32;
        fn gcx_fixture_gpr4() -> u32;
    }
    #[derive(Serialize, Deserialize)]
    struct Reference {
        id: String,
        binding: Binding,
        counter: u32,
        gpr4: u32,
    }
    struct Native;
    impl Drop for Native {
        fn drop(&mut self) {
            unsafe {
                gcx_engine_stop();
                gcx_engine_shutdown();
            }
        }
    }
    unsafe extern "C" fn event(_: *mut c_void, id: u32, message: *const c_char) {
        if matches!(id, 2 | 20) {
            let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
            let _ = writeln!(io::stderr().lock(), "fixture native event {id}: {message}");
        }
    }
    fn c_path(path: &Path) -> Result<CString> {
        use std::os::unix::ffi::OsStrExt;
        Ok(CString::new(path.as_os_str().as_bytes())?)
    }
    fn counter_dol() -> Vec<u8> {
        let mut bytes = gcx::cpu_probe_dol();
        for (i, instruction) in [
            0x3c60_8001_u32,
            0x3880_0000,
            0x3884_0001,
            0x9083_0000,
            0x4bff_fff8,
        ]
        .into_iter()
        .enumerate()
        {
            bytes[0x100 + i * 4..0x104 + i * 4].copy_from_slice(&instruction.to_be_bytes());
        }
        bytes
    }
    fn binding(profile: &Path) -> Result<Binding> {
        Ok(Binding {
            game_id: "a".repeat(32),
            profile_id: "b".repeat(32),
            engine: BUILD_ENGINE_SHA256.into(),
            state_abi: unsafe { gcx_engine_checkpoint_abi() },
            resources: BUILD_RESOURCE_ID.into(),
            media_sha256: format!(
                "{:x}",
                Sha256::digest(fs::read(profile.join("counter-probe.dol"))?)
            ),
            disc: 0,
        })
    }
    fn start(profile: &Path, sys: &Path, paused: bool) -> Result<Native> {
        let user = c_path(profile)?;
        let sys = c_path(sys)?;
        let media = c_path(&profile.join("counter-probe.dol"))?;
        unsafe {
            if gcx_engine_abi() != 6 {
                return Err("wrong native ABI".into());
            }
            gcx_engine_card_writer(storage::write_card);
            if gcx_engine_init(
                user.as_ptr(),
                sys.as_ptr(),
                event,
                std::ptr::null_mut(),
                c"Null".as_ptr(),
                c"No Audio Output".as_ptr(),
            ) == 0
            {
                return Err("native init refused".into());
            }
            let native = Native;
            let admitted = if paused {
                gcx_engine_boot_paused(media.as_ptr(), std::ptr::null_mut())
            } else {
                gcx_engine_boot(media.as_ptr(), std::ptr::null_mut())
            };
            if admitted == 0 {
                return Err("original fixture boot refused".into());
            }
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline {
                gcx_engine_dispatch();
                let state = gcx_engine_state();
                if paused && state == 1
                    || !paused && matches!(state, 1 | 2) && gcx_engine_probe_word() > 100
                {
                    gcx_engine_pause(1);
                    return Ok(native);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        Err("fixture boot timed out".into())
    }
    unsafe extern "C" fn copy_payload(
        context: *mut c_void,
        bytes: *const u8,
        length: usize,
    ) -> i32 {
        if bytes.is_null() || context.is_null() || length == 0 || length > checkpoint::PAYLOAD_LIMIT
        {
            return 0;
        }
        let out = unsafe { &mut *context.cast::<Vec<u8>>() };
        if out.try_reserve_exact(length).is_err() {
            return 0;
        }
        out.extend_from_slice(unsafe { std::slice::from_raw_parts(bytes, length) });
        1
    }
    fn capture(limit: usize) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        if unsafe {
            gcx_engine_checkpoint_capture(limit, copy_payload, (&mut out as *mut Vec<u8>).cast())
        } == 0
        {
            return Err("native capture refused".into());
        }
        Ok(out)
    }
    fn card(operation: u32, value: u32) -> Result<u32> {
        let result = unsafe { gcx_fixture_card(operation, value) };
        if result == u32::MAX {
            return Err("native fixture card unavailable".into());
        }
        Ok(result)
    }
    fn command(bytes: &[u8], finish: bool) -> Result {
        card(0, 1)?;
        for byte in bytes {
            card(1, u32::from(*byte))?;
        }
        if finish {
            card(0, 0)?;
        }
        Ok(())
    }
    fn reference(profile: &Path, name: &str) -> Result<Reference> {
        Ok(serde_json::from_slice(&fs::read(profile.join(name))?)?)
    }
    fn save_reference(profile: &Path, name: &str, binding: &Binding) -> Result {
        let id = unsafe { checkpoint::capture(profile, binding, true)? };
        let payload = checkpoint::load(profile, &id, binding)?.payload;
        let reference = Reference {
            id,
            binding: binding.clone(),
            counter: unsafe { gcx_engine_probe_word() },
            gpr4: unsafe { gcx_fixture_gpr4() },
        };
        storage::atomic_replace(&profile.join(name), &serde_json::to_vec(&reference)?)?;
        println!(
            "fixture checkpoint payload={} bytes, counter={}, register4={}",
            payload.len(),
            reference.counter,
            reference.gpr4
        );
        Ok(())
    }
    fn raw_card(profile: &Path) -> Result<PathBuf> {
        let mut names = fs::read_dir(profile.join("GC"))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.as_encoded_bytes().starts_with(b"MemoryCardA."))
            });
        let path = names.next().ok_or("missing card")?;
        if names.next().is_some() {
            return Err("ambiguous fixture card".into());
        }
        Ok(path)
    }
    fn phase(profile: &Path, sys: &Path, phase: &str) -> Result {
        let _lock = ProfileLock::acquire(profile)?;
        let _store = storage::CardStore::bind(profile)?;
        let expected = binding(profile)?;
        match phase {
            "capture" => {
                let native = start(profile, sys, false)?;
                command(&[0x81, 1], true)?; // Enable card completion interrupt.
                command(&[0x89], true)?; // Clear prior status.
                command(&[0xf1, 1, 0x10], true)?; // Erase at0x22000; delayed completion remains queued.
                assert_eq!(card(3, 0)?, 0);
                command(&[0xf2, 1, 0, 0, 0, 0x11, 0x22], false)?; // Half a page-program command at0x20000.
                assert!(capture(1).is_err()); // Bounded refusal retains the paused machine.
                save_reference(profile, "captured.json", &expected)?;
                drop(native); // Keep profile/card ownership until all native writers end.
                let r = reference(profile, "captured.json")?;
                checkpoint::load(profile, &r.id, &expected)?.verify_current_saves(profile)?;
                println!(
                    "PASS capture: staged card command and completion event captured; clean shutdown retained generation"
                );
            }
            "restore" => {
                let r = reference(profile, "captured.json")?;
                let mut state = checkpoint::load(profile, &r.id, &expected)?;
                assert!(state.paused);
                let native = start(profile, sys, true)?;
                assert_eq!(
                    unsafe { gcx_engine_probe_word() },
                    0,
                    "no guest execution before restore"
                );
                state.verify_current_saves(profile)?;
                assert_eq!(
                    unsafe {
                        gcx_engine_checkpoint_load(state.payload.as_mut_ptr(), state.payload.len())
                    },
                    1
                );
                assert_eq!(unsafe { gcx_engine_probe_word() }, r.counter);
                assert_eq!(unsafe { gcx_fixture_gpr4() }, r.gpr4);
                assert_eq!(card(3, 0)?, 0);
                unsafe {
                    gcx_engine_pause(0);
                }
                let deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < deadline && card(3, 0)? == 0 {
                    unsafe {
                        gcx_engine_dispatch();
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                unsafe {
                    gcx_engine_pause(1);
                }
                assert_eq!(card(3, 0)?, 1, "restored delayed completion event fired");
                assert_ne!(
                    unsafe { gcx_engine_probe_word() },
                    r.counter,
                    "guest CPU continued"
                );
                let bytes = fs::read(raw_card(profile)?)?;
                assert_eq!(
                    &bytes[0x20000..0x20004],
                    &[0xff; 4],
                    "completion did not replay a card write"
                );
                card(1, 0x33)?;
                card(1, 0x44)?;
                card(0, 0)?;
                assert_eq!(unsafe { gcx_engine_flush_cards() }, 1);
                let bytes = fs::read(raw_card(profile)?)?;
                assert_eq!(
                    &bytes[0x20000..0x20004],
                    &[0x11, 0x22, 0x33, 0x44],
                    "staged protocol survived fresh process"
                );
                assert!(
                    checkpoint::load(profile, &r.id, &expected).is_err(),
                    "newer card rejects old state"
                );
                save_reference(profile, "latest.json", &expected)?;
                drop(native);
                println!(
                    "PASS restore: new process restored CPU/registers, completion event and staged card bytes; newer generation refused old state"
                );
            }
            "reject-newer" => {
                let r = reference(profile, "captured.json")?;
                assert!(checkpoint::load(profile, &r.id, &expected).is_err());
                assert_eq!(unsafe { gcx_engine_state() }, 0);
                println!(
                    "PASS gate: newer durable saves refused before any native startup/mutation"
                );
            }
            "fail-truncated" | "fail-card-image" => {
                let r = reference(profile, "latest.json")?;
                let mut state = checkpoint::load(profile, &r.id, &expected)?;
                let generation = storage::save_generation(profile)?;
                let native = start(profile, sys, true)?;
                state.verify_current_saves(profile)?;
                if phase == "fail-truncated" {
                    state.payload.truncate(state.payload.len() - 16);
                } else {
                    let marker = b"GCXCR1\0\0";
                    let offsets: Vec<_> = state
                        .payload
                        .windows(8)
                        .enumerate()
                        .filter_map(|(i, x)| (x == marker).then_some(i))
                        .collect();
                    assert_eq!(offsets.len(), 1);
                    state.payload[offsets[0] + 8] = 1; // Refuse a payload requesting old card-image restoration.
                }
                assert_eq!(
                    unsafe {
                        gcx_engine_checkpoint_load(state.payload.as_mut_ptr(), state.payload.len())
                    },
                    0
                );
                assert_eq!(unsafe { gcx_engine_checkpoint_failed() }, 1);
                unsafe {
                    gcx_engine_pause(0);
                }
                assert_eq!(
                    unsafe { gcx_engine_state() },
                    1,
                    "failed machine cannot resume"
                );
                assert_eq!(
                    unsafe { gcx_engine_flush_cards() },
                    0,
                    "persistence blocked after failed load"
                );
                drop(native);
                generation.verify(profile)?;
                assert!(
                    profile
                        .join("checkpoints")
                        .join(format!("{}.gcx", r.id))
                        .is_file()
                );
                println!(
                    "PASS {phase}: failure retained checkpoint and durable card/SRAM through native teardown"
                );
            }
            _ => return Err("unknown fixture phase".into()),
        }
        Ok(())
    }
    pub fn run() -> Result {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() == 4 && args[0] == "--phase" {
            return phase(
                &PathBuf::from(&args[2]),
                &PathBuf::from(&args[3]),
                args[1].to_str().ok_or("phase encoding")?,
            );
        }
        if args.len() != 2 {
            return Err("usage: gcx-checkpoint-probe <NEW-private-root> <Dolphin-Data/Sys>".into());
        }
        let root = PathBuf::from(&args[0]);
        fs::create_dir(&root)?;
        let profile = root.canonicalize()?.join("profile");
        fs::create_dir(&profile)?;
        fs::write(profile.join("counter-probe.dol"), counter_dol())?;
        let sys = PathBuf::from(&args[1]).canonicalize()?;
        let exe = std::env::current_exe()?;
        let local_sys = exe.parent().ok_or("executable parent")?.join("Sys");
        if !local_sys.exists() {
            std::os::unix::fs::symlink(&sys, &local_sys)?;
        }
        if local_sys.canonicalize()? != sys {
            return Err("host Sys has another owner".into());
        }
        for step in [
            "capture",
            "restore",
            "reject-newer",
            "fail-truncated",
            "fail-card-image",
        ] {
            let status = Command::new(&exe)
                .arg("--phase")
                .arg(step)
                .arg(&profile)
                .arg(&sys)
                .status()?;
            if !status.success() {
                return Err(format!("checkpoint fixture phase {step} failed: {status}").into());
            }
        }
        println!(
            "PASS: original fresh-process CPU/card-protocol checkpoint fixture. No gameplay, graphics, sound, Android resume or shortcut acceptance."
        );
        Ok(())
    }
}
#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    host::run()
}
