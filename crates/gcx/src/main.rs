// SPDX-License-Identifier: MIT
//! Off-device native CPU/lifecycle probe, not gameplay/audio acceptance.
use gcx::engine::*;
use gcx::{Control, Pad, ProfileLock};
use std::ffi::{CStr, CString, c_char};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static STOP: AtomicBool = AtomicBool::new(false);
static ERROR: AtomicBool = AtomicBool::new(false);
static CARD_ERROR: AtomicBool = AtomicBool::new(false);
struct EngineGuard;
impl Drop for EngineGuard {
    fn drop(&mut self) {
        // SAFETY: created only after this owner thread initializes the engine;
        // joined teardown precedes the caller's card binding/profile lock drop.
        unsafe {
            gcx_engine_stop();
            gcx_engine_shutdown();
        }
    }
}
unsafe extern "C" fn event(_: *mut std::ffi::c_void, id: u32, message: *const c_char) {
    if id == 10 {
        STOP.store(true, Ordering::Relaxed);
    }
    if id == 2 {
        ERROR.store(true, Ordering::Relaxed);
    }
    if id == 20 {
        CARD_ERROR.store(true, Ordering::Relaxed);
    }
    if id == 2 || id == 20 {
        // SAFETY: bridge lends a NUL-terminated message only for this callback.
        let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
        let _ = writeln!(std::io::stderr().lock(), "native alert: {message}");
    }
}
fn c_path(path: &std::path::Path) -> Result<CString, Box<dyn std::error::Error>> {
    use std::os::unix::ffi::OsStrExt;
    Ok(CString::new(path.as_os_str().as_bytes())?)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(args.len() == 2
        || args.len() == 3
            && (args[2] == "--unavailable-audio"
                || args[2] == "--reentry"
                || args[2] == "--card-retry"))
    {
        return Err("usage: gcx-probe <new-private-directory> <Dolphin-Data/Sys> [--unavailable-audio|--reentry|--card-retry]".into());
    }
    let root = PathBuf::from(&args[0]);
    let sys = PathBuf::from(&args[1]).canonicalize()?;
    let local_sys = std::env::current_exe()?
        .parent()
        .ok_or("missing executable parent")?
        .join("Sys");
    if !local_sys.exists() {
        std::os::unix::fs::symlink(&sys, &local_sys)?;
    }
    if local_sys.canonicalize()? != sys {
        return Err("host Sys resource path has another owner".into());
    }
    // Always refuse existing top-level directories, including for the reentry fault fixture.
    std::fs::create_dir(&root)?;
    let root = root.canonicalize()?;
    if args.len() == 3 && args[2] == "--reentry" {
        let a = root.join("A");
        let b = root.join("B");
        std::fs::create_dir(&a)?;
        std::fs::create_dir(&b)?;
        fixture(&a, &sys, false, false)?;
        let card = raw_card(&a)?;
        let mut expected = std::fs::read(&card)?;
        let marker = b"original gcx card reentry fixture";
        if expected.len() < marker.len() {
            return Err("native card unexpectedly short".into());
        }
        let offset = expected.len() - marker.len();
        expected[offset..].copy_from_slice(marker);
        {
            let _writer = ProfileLock::acquire(&a)?;
            gcx::storage::atomic_replace(&card, &expected)?;
        }
        fixture(&b, &sys, false, false)?;
        if std::fs::read(raw_card(&b)?)? == expected {
            return Err("native fixture profiles unexpectedly shared a card".into());
        }
        fixture(&a, &sys, false, false)?;
        if std::fs::read(&card)? != expected {
            return Err("native card marker was lost across profile reentry".into());
        }
        println!(
            "PASS: same-process native A/B/A profile reentry retained an isolated ordinary-card marker. No game save or gameplay continuity acceptance claimed."
        );
    } else {
        fixture(
            &root,
            &sys,
            args.len() == 3 && args[2] == "--unavailable-audio",
            args.len() == 3 && args[2] == "--card-retry",
        )?;
    }
    println!(
        "PASS: Rust host executed the original GameCube fixture, verified native pad and card commit, neutralized input and completed teardown. Graphics/audio/gameplay/card-save/resume/device checks remain unrun."
    );
    Ok(())
}
fn raw_card(profile: &std::path::Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cards = std::fs::read_dir(profile.join("GC"))?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    let cards: Vec<_> = cards
        .into_iter()
        .filter(|p| {
            p.extension().is_some_and(|e| e == "raw")
                && p.file_name()
                    .is_some_and(|n| n.as_encoded_bytes().starts_with(b"MemoryCardA."))
        })
        .collect();
    if cards.len() != 1 {
        return Err("fixture requires exactly one native raw card".into());
    }
    Ok(cards[0].clone())
}
fn fixture(
    profile: &std::path::Path,
    sys: &std::path::Path,
    unavailable_audio: bool,
    card_retry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    STOP.store(false, Ordering::Relaxed);
    ERROR.store(false, Ordering::Relaxed);
    CARD_ERROR.store(false, Ordering::Relaxed);
    let _writer = ProfileLock::acquire(profile)?;
    let _cards = gcx::storage::CardStore::bind(profile)?;
    let media = profile.join("cpu-probe.dol");
    std::fs::write(&media, gcx::cpu_probe_dol())?;
    let user = c_path(profile)?;
    let sys = c_path(sys)?;
    let media = c_path(&media)?;
    let mut observed = false;
    let mut input_verified = false;
    let mut shutdown_obstruction = None;
    let mut pad = Pad::default();
    pad.set(Control::A, 1.0)?;
    pad.set(Control::StickX, 0.5)?;
    pad.set(Control::CY, -0.6)?;
    pad.set(Control::LTravel, 0.25)?;
    pad.set(Control::RTravel, 0.4)?;
    pad.set(Control::LClick, 1.0)?;
    // SAFETY: one host thread owns all native entrypoints and borrowed C strings.
    // The callback uses atomics, never unwinds, and remains alive through shutdown.
    unsafe {
        if gcx_engine_abi() != 7 {
            return Err("incompatible engine ABI".into());
        }
        gcx_engine_card_writer(gcx::storage::write_card);
        gcx_engine_saved_reader(gcx::storage::read_image);
        if gcx_engine_init(
            user.as_ptr(),
            sys.as_ptr(),
            event,
            std::ptr::null_mut(),
            c"Null".as_ptr(),
            if unavailable_audio {
                c"Cubeb".as_ptr()
            } else {
                c"No Audio Output".as_ptr()
            },
        ) == 0
        {
            return Err("engine initialization rejected".into());
        }
        let _engine = EngineGuard;
        let booted = gcx_engine_boot(media.as_ptr(), std::ptr::null_mut()) != 0;
        eprintln!(
            "fixture boot admitted={booted}, state={}",
            gcx_engine_state()
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        while booted && Instant::now() < deadline && !STOP.load(Ordering::Relaxed) {
            gcx_engine_dispatch();
            if matches!(gcx_engine_state(), 1 | 2) && gcx_engine_probe_word() == 0x4743_5831 {
                observed = true;
                gcx_engine_pause(1);
                gcx_engine_input(pad.values().as_ptr());
                let mut status = PadStatus::default();
                gcx_engine_pad_status(&mut status);
                eprintln!("fixture native pad={status:?}");
                // These are Dolphin's GameCube PadButton values, scoped to this
                // diagnostic. The Rust product pad remains named channels.
                input_verified = status.connected == 1
                    && status.buttons & 0x0160 == 0x0140
                    && (190..=193).contains(&status.axes[0])
                    && (49..=53).contains(&status.axes[3])
                    && (62..=65).contains(&status.triggers[0])
                    && (100..=103).contains(&status.triggers[1]);
                gcx_engine_input(Pad::default().values().as_ptr());
                gcx_engine_pad_status(&mut status);
                eprintln!("fixture neutral pad={status:?}");
                input_verified &=
                    status.buttons == 0 && status.axes == [128; 4] && status.triggers == [0; 2];
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let committed = gcx_engine_flush_cards() != 0;
        eprintln!(
            "fixture observed={observed}, input={input_verified}, card_commit={committed}, final_state={}",
            gcx_engine_state()
        );
        input_verified &= committed;
        if committed && card_retry {
            // Isolated fault: a directory blocks only this fixture's candidate
            // file. Do not fill the disk or touch any existing profile.
            let card = raw_card(profile)?;
            let before = std::fs::read(&card)?;
            let mut pending = card.as_os_str().to_os_string();
            pending.push(".gcx-pending");
            let pending = PathBuf::from(pending);
            std::fs::create_dir(&pending)?;
            let failed = gcx_engine_flush_cards() == 0;
            let retained = gcx_engine_probe_word() == 0x4743_5831
                && std::fs::read(&card).is_ok_and(|bytes| bytes == before)
                && gcx::storage::last_card_error().is_some();
            std::fs::remove_dir(&pending)?;
            let reported = CARD_ERROR.swap(false, Ordering::Relaxed);
            let retried =
                gcx_engine_flush_cards() != 0 && gcx::storage::last_card_error().is_none();
            input_verified &= failed && retained && reported && retried;
            eprintln!(
                "fixture card failure={failed}, retained={retained}, reported={reported}, retry={retried}"
            );
            // A successful barrier followed by paused teardown must not issue
            // another fallible write of the same image from the native worker.
            std::fs::create_dir(&pending)?;
            shutdown_obstruction = Some(pending);
        }
    }
    if let Some(pending) = shutdown_obstruction {
        std::fs::remove_dir(pending)?;
        eprintln!("fixture completed joined teardown without a redundant post-barrier card write");
    }
    // _writer remains held past native teardown, including joined card workers.
    if !observed
        || !input_verified
        || ERROR.load(Ordering::Relaxed)
        || CARD_ERROR.load(Ordering::Relaxed)
    {
        return Err("native CPU/lifecycle probe failed; private evidence retained".into());
    }
    Ok(())
}
