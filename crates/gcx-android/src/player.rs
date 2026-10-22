// SPDX-License-Identifier: MIT
use crate::platform::{self, Command};
use android_activity::{
    AndroidApp, InputStatus, MainEvent, PollEvent,
    input::{InputEvent, MotionAction},
};
use gcx::library::{Collection, GameId, SaveProfileId, SavedSession, valid_id};
use gcx::{
    Pad, ProfileLock,
    engine::*,
    touch::{Layout, Target, TouchPad},
};
use ndk::native_window::NativeWindow;
use std::{
    ffi::{CStr, CString, c_char},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
static OWNER: AtomicBool = AtomicBool::new(false);
static NEXT_RUNTIME: AtomicU64 = AtomicU64::new(1);
static UI: Mutex<TouchPad> = Mutex::new(TouchPad::new());

struct SaveProfile {
    path: PathBuf,
    _writer: ProfileLock,
}
struct LiveRuntime {
    events: Box<RuntimeEvents>,
    _cards: gcx::storage::CardStore,
    _profile: SaveProfile,
    _boot_window: NativeWindow,
}
struct RuntimeEvents {
    game_id: String,
    epoch: u64,
    stopped: AtomicBool,
}
impl Drop for LiveRuntime {
    fn drop(&mut self) {
        if let Ok(mut touch) = UI.lock() {
            touch.cancel();
        }
        // SAFETY: the one android_main thread owns this runtime; callbacks are
        // static and the profile lock outlives completed native teardown.
        unsafe {
            gcx_engine_input(Pad::default().values().as_ptr());
            gcx_engine_stop();
            gcx_engine_shutdown();
        }
    }
}
unsafe extern "C" fn event(context: *mut std::ffi::c_void, id: u32, message: *const c_char) {
    // SAFETY: this box is retained in LiveRuntime until every native worker joins.
    let context = unsafe { &*context.cast::<RuntimeEvents>() };
    if id == 10 {
        context.stopped.store(true, Ordering::Relaxed);
    }
    if id == 2 || id == 20 {
        // SAFETY: native lends a NUL-terminated message during this callback.
        let message = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        let message = if id == 20 {
            format!(
                "{message} {}",
                gcx::storage::last_card_error().unwrap_or_default()
            )
        } else {
            message
        };
        platform::push(Command::NativeError {
            game_id: context.game_id.clone(),
            epoch: context.epoch,
            message,
        });
    }
    if id == 3 {
        // SAFETY: same borrowed native message contract as critical alerts.
        let message = unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned();
        platform::push(Command::Notice(message));
    }
}
extern "C" fn draw(phase: u32, width: f32, height: f32, _aspect: f32) -> Rect {
    if !width.is_finite() || !height.is_finite() || width < 1.0 || height < 1.0 {
        return Rect::default();
    }
    let layout = Layout::new(width, height);
    if phase == 1 {
        let pad = UI.lock().map(|touch| touch.pad()).unwrap_or_default();
        let value = |control: gcx::Control| pad.values()[control as usize] as f32;
        // SAFETY: called on the video thread with the donor ImGui frame active.
        unsafe {
            for zone in &layout.zones {
                let a = zone.area;
                let (x, y) = a.center();
                let radius = (a.right - a.left) * 0.5;
                match zone.target {
                    Target::Stick(cx, cy) => {
                        gcx_ui_circle(x, y, radius, 0xe0475057);
                        gcx_ui_circle(
                            x + value(cx) * radius * 0.7,
                            y - value(cy) * radius * 0.7,
                            radius * 0.28,
                            0xff78cbd4,
                        );
                    }
                    Target::Button(control) => gcx_ui_circle(
                        x,
                        y,
                        radius,
                        if value(control) > 0.0 {
                            0xff529777
                        } else {
                            0xe0475057
                        },
                    ),
                    Target::Travel(control) => {
                        gcx_ui_rect(a.left, a.top, a.right, a.bottom, 0xe0475057);
                        gcx_ui_rect(
                            a.left,
                            a.bottom - (a.bottom - a.top) * 0.25,
                            a.left + (a.right - a.left) * value(control),
                            a.bottom,
                            0xff78cbd4,
                        );
                    }
                    Target::Pause => gcx_ui_rect(a.left, a.top, a.right, a.bottom, 0xe0475057),
                }
                let count = zone.label.to_bytes().len() as f32;
                let font = ((a.bottom - a.top) * 0.4)
                    .min((a.right - a.left) / (count * 0.65 + 1.0))
                    .clamp(12.0, 48.0);
                gcx_ui_text(
                    x - count * font * 0.3,
                    y - font * 0.5,
                    font,
                    zone.label.as_ptr(),
                    0xffeeeeee,
                );
            }
        }
    }
    let a = layout.game;
    Rect {
        left: a.left as i32,
        top: a.top as i32,
        right: a.right as i32,
        bottom: a.bottom as i32,
    }
}
fn id() -> Result<String> {
    let mut bytes = [0_u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn load_collection(root: &Path) -> Result<Collection> {
    let path = root.join("collection.json");
    if !path.exists() {
        return Ok(Collection::default());
    }
    if path.metadata()?.len() > 4 * 1024 * 1024 {
        return Err("collection metadata is too large; original retained".into());
    }
    let collection: Collection = serde_json::from_slice(&std::fs::read(path)?)?;
    collection.validate()?;
    Ok(collection)
}
fn save_collection(root: &Path, collection: &Collection) -> Result<()> {
    gcx::storage::atomic_replace(
        &root.join("collection.json"),
        &serde_json::to_vec(collection)?,
    )?;
    Ok(())
}
fn resources(app: &AndroidApp, root: &Path) -> Result<PathBuf> {
    let manager = app.asset_manager();
    let mut index = String::new();
    let mut asset = manager
        .open(c"resource-index.txt")
        .ok_or("missing resource inventory")?;
    if asset.length() > 4 * 1024 * 1024 {
        return Err("resource inventory exceeds its size limit".into());
    }
    asset.read_to_string(&mut index)?;
    let sys = root.join("Sys");
    for relative in index.lines() {
        if Path::new(relative)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err("invalid resource inventory path".into());
        }
        let name = CString::new(format!("Sys/{relative}"))?;
        let mut bytes = Vec::new();
        let mut asset = manager.open(&name).ok_or("missing engine resource")?;
        if asset.length() > 4 * 1024 * 1024 {
            return Err("engine resource exceeds its size limit".into());
        }
        asset.read_to_end(&mut bytes)?;
        let destination = sys.join(relative);
        std::fs::create_dir_all(destination.parent().ok_or("resource parent unavailable")?)?;
        if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
            gcx::storage::atomic_replace(&destination, &bytes)?;
        }
    }
    Ok(sys)
}
fn boot(
    root: &Path,
    sys: &Path,
    entry: &SavedSession,
    window: &NativeWindow,
) -> Result<LiveRuntime> {
    if !valid_id(&entry.profile_id.0) {
        return Err("invalid profile identity".into());
    }
    let path = root.join("profiles").join(&entry.profile_id.0);
    std::fs::create_dir_all(&path)?;
    let profile = SaveProfile {
        _writer: ProfileLock::acquire(&path)?,
        path,
    };
    let user = CString::new(profile.path.to_str().ok_or("invalid user path")?)?;
    let sys = CString::new(sys.to_str().ok_or("invalid resource path")?)?;
    let uri = CString::new(entry.uri.as_str())?;
    let mut events = Box::new(RuntimeEvents {
        game_id: entry.game_id.0.clone(),
        epoch: NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed),
        stopped: AtomicBool::new(false),
    });
    // SAFETY: serialized owner, valid strings and stable process-long callbacks.
    unsafe {
        if gcx_engine_abi() != 4 {
            return Err("engine ABI mismatch".into());
        }
        let cards = gcx::storage::CardStore::bind(&profile.path)?;
        gcx_engine_card_writer(gcx::storage::write_card);
        gcx_engine_platform(platform::dispatch);
        gcx_engine_ui(draw);
        if gcx_engine_init(
            user.as_ptr(),
            sys.as_ptr(),
            event,
            std::ptr::from_mut(events.as_mut()).cast(),
            c"Vulkan".as_ptr(),
            c"Cubeb".as_ptr(),
        ) == 0
        {
            return Err("engine initialization rejected".into());
        }
        let runtime = LiveRuntime {
            events,
            _cards: cards,
            _profile: profile,
            _boot_window: window.clone(),
        };
        let mut info = MediaInfo::default();
        if gcx_engine_inspect(uri.as_ptr(), &mut info) == 0 {
            return Err("Choose a supported GameCube disc image (ISO/GCM or RVZ).".into());
        }
        if gcx_engine_boot(uri.as_ptr(), window.ptr().as_ptr().cast()) == 0 {
            return Err("The native engine could not boot this GameCube game.".into());
        }
        Ok(runtime)
    }
}
pub fn run(app: AndroidApp) {
    if OWNER
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let result = run_owned(&app);
    if let Err(error) = result {
        platform::notice(&app, format!("gcx: {error}"));
    }
    platform::shutdown();
    OWNER.store(false, Ordering::Release);
}
fn run_owned(app: &AndroidApp) -> Result<()> {
    platform::init(app)?;
    let root = app
        .internal_data_path()
        .ok_or("private storage unavailable")?
        .join("gcx");
    std::fs::create_dir_all(&root)?;
    let sys = resources(app, &root)?;
    let mut collection = load_collection(&root)?;
    let mut selected = collection
        .selected
        .as_ref()
        .and_then(|id| collection.entries.iter().position(|e| e.game_id.0 == *id));
    let mut runtime: Option<LiveRuntime> = None;
    let mut window: Option<NativeWindow> = None;
    let mut foreground = false;
    let mut destroyed = false;
    let mut hidden = false;
    let mut picker_pending = false;
    let mut native_paused = None;
    let mut surface_pending = false;
    let mut restart_requested = false;
    if let Some(index) = selected.filter(|i| collection.entries[*i].restart_required) {
        platform::menu(
            app,
            true,
            Some(format!(
                "gcx paused\n\n{}",
                collection.entries[index]
                    .last_error
                    .as_deref()
                    .unwrap_or("The previous runtime needs an explicit restart.")
            )),
        );
    } else if selected.is_some() {
        platform::menu(app, false, None);
    } else {
        platform::menu(app, true, None);
    }
    while !destroyed {
        app.poll_events(Some(Duration::from_millis(8)), |event| {
            if let PollEvent::Main(event) = event {
                match event {
                    MainEvent::InitWindow { .. } => {
                        window = app.native_window();
                        surface_pending = runtime.is_some();
                    }
                    MainEvent::TerminateWindow { .. } => {
                        if runtime.is_some() {
                            unsafe {
                                gcx_engine_pause(1);
                                gcx_engine_input(Pad::default().values().as_ptr());
                                if matches!(gcx_engine_state(), 1 | 2)
                                    && gcx_engine_flush_cards() == 0
                                {
                                    platform::push(Command::Error("Could not commit the card when the window closed. The runtime is retained; retry before restarting.".into()));
                                }
                            }
                            native_paused = Some(true);
                        }
                        window = None;
                        if let Ok(mut touch) = UI.lock() {
                            touch.cancel();
                        }
                    }
                    MainEvent::GainedFocus => foreground = true,
                    MainEvent::LostFocus | MainEvent::Pause | MainEvent::Stop => {
                        foreground = false;
                        if let Ok(mut touch) = UI.lock() {
                            touch.cancel();
                        }
                    }
                    MainEvent::Destroy => destroyed = true,
                    MainEvent::WindowResized { .. } | MainEvent::ContentRectChanged { .. } => {
                        if let Ok(mut touch) = UI.lock() {
                            touch.cancel();
                        }
                        if runtime.is_some() {
                            unsafe {
                                gcx_engine_resize();
                            }
                        }
                    }
                    MainEvent::InputAvailable => {
                        if let Ok(mut input) = app.input_events_iter() {
                            while input.next(|event| {
                                if let InputEvent::MotionEvent(motion) = event {
                                    if hidden
                                        && runtime.is_some()
                                        && let Some(window) = &window
                                    {
                                        let layout = Layout::new(
                                            window.width() as f32,
                                            window.height() as f32,
                                        );
                                        if let Ok(mut touch) = UI.lock() {
                                            match motion.action() {
                                                MotionAction::Down | MotionAction::PointerDown => {
                                                    let pointer = motion
                                                        .pointer_at_index(motion.pointer_index());
                                                    if touch.down(
                                                        pointer.pointer_id(),
                                                        pointer.x(),
                                                        pointer.y(),
                                                        &layout,
                                                    ) && let Some(index) = selected
                                                    {
                                                        collection.entries[index].paused =
                                                            !collection.entries[index].paused;
                                                        touch.cancel();
                                                        let message =
                                                            if collection.entries[index].paused {
                                                                "Paused"
                                                            } else {
                                                                "Playing"
                                                            };
                                                        platform::notice(app, message.into());
                                                    }
                                                }
                                                MotionAction::Move => {
                                                    for pointer in motion.pointers() {
                                                        touch.moved(
                                                            pointer.pointer_id(),
                                                            pointer.x(),
                                                            pointer.y(),
                                                        );
                                                    }
                                                }
                                                MotionAction::Up | MotionAction::PointerUp => {
                                                    let pointer = motion
                                                        .pointer_at_index(motion.pointer_index());
                                                    touch.up(pointer.pointer_id());
                                                }
                                                MotionAction::Cancel => touch.cancel(),
                                                _ => {}
                                            }
                                        }
                                    }
                                    InputStatus::Handled
                                } else {
                                    InputStatus::Unhandled
                                }
                            }) {}
                        }
                    }
                    _ => {}
                }
            }
        });
        for command in platform::commands() {
            let command = match command {
                Command::NativeError {
                    game_id,
                    epoch,
                    message,
                } => {
                    if runtime
                        .as_ref()
                        .is_some_and(|active| active.events.epoch == epoch)
                    {
                        Command::Error(message)
                    } else {
                        if let Some(index) = collection
                            .entries
                            .iter()
                            .position(|entry| entry.game_id.0 == game_id)
                        {
                            collection.entries[index].last_error =
                                Some(message.chars().take(8192).collect());
                            if save_collection(&root, &collection).is_err() {
                                platform::notice(
                                    app,
                                    "Could not preserve an earlier runtime error on disk.".into(),
                                );
                            }
                            if runtime.is_none()
                                && selected == Some(index)
                                && collection.entries[index].restart_required
                            {
                                hidden = false;
                                platform::menu(app, true, Some(message));
                            } else {
                                platform::notice(app, format!("Earlier runtime: {message}"));
                            }
                        }
                        continue;
                    }
                }
                other => other,
            };
            match command {
                Command::NativeError { .. } => continue,
                Command::Choose => {
                    if !picker_pending {
                        platform::picker(app);
                        picker_pending = true;
                    }
                }
                Command::Restart => restart_requested = true,
                Command::Selected(uri) => {
                    picker_pending = false;
                    if runtime.is_some() {
                        let mut info = MediaInfo::default();
                        let valid=CString::new(uri.as_str()).is_ok_and(|candidate| unsafe {gcx_engine_inspect(candidate.as_ptr(),&mut info)}!=0);
                        if !valid {
                            platform::notice(app,"Choose a readable GameCube ISO/GCM or RVZ. The current runtime is retained.".into());
                            continue;
                        }
                        if selected.is_some_and(|i| {
                            collection.entries[i].uri == uri
                                && !collection.entries[i].restart_required
                        }) {
                            platform::menu(app, false, None);
                            continue;
                        }
                    }
                    // Retain the old writer and its RAM on failed card durability.
                    if runtime.is_some() && unsafe { gcx_engine_flush_cards() } == 0 {
                        platform::push(Command::Error("The current memory card could not be committed; its runtime is retained. Retry after resolving storage.".into()));
                        continue;
                    }
                    drop(runtime.take());
                    native_paused = None;
                    let index =
                        if let Some(index) = collection.entries.iter().position(|e| e.uri == uri) {
                            index
                        } else {
                            let title = platform::display_name(&uri)
                                .unwrap_or_else(|_| "GameCube game".into());
                            collection.entries.push(SavedSession {
                                game_id: GameId(id()?),
                                profile_id: SaveProfileId(id()?),
                                uri,
                                title,
                                paused: false,
                                last_error: None,
                                restart_required: false,
                            });
                            collection.entries.len() - 1
                        };
                    collection.selected = Some(collection.entries[index].game_id.0.clone());
                    selected = Some(index);
                    if save_collection(&root, &collection).is_err() {
                        platform::notice(app,"Could not durably save the collection. Files are retained; choose the document again to retry.".into());
                        continue;
                    }
                    platform::menu(app, false, None);
                }
                Command::MenuHidden => hidden = true,
                Command::Canceled => picker_pending = false,
                Command::Error(error) => {
                    picker_pending = false;
                    if runtime.is_some()
                        && let Some(index) = selected
                    {
                        collection.entries[index].fail(error.clone());
                        if let Ok(mut touch) = UI.lock() {
                            touch.cancel();
                        }
                        hidden = false;
                        unsafe {
                            gcx_engine_pause(1);
                            gcx_engine_input(Pad::default().values().as_ptr());
                        }
                        native_paused = Some(true);
                        let persistence = if save_collection(&root, &collection).is_err() {
                            "\nCould not preserve this error on disk; it remains on this screen."
                        } else {
                            ""
                        };
                        platform::menu(
                            app,
                            true,
                            Some(format!("gcx paused\n\n{error}{persistence}")),
                        );
                    } else {
                        platform::notice(app, error);
                    }
                }
                Command::Notice(message) => platform::notice(app, message),
            }
        }
        if restart_requested {
            restart_requested = false;
            if let Some(index) = selected {
                if runtime.is_some() && unsafe { gcx_engine_flush_cards() } == 0 {
                    platform::push(Command::Error("The memory card could not be committed. The current runtime is retained; restart has not occurred.".into()));
                    continue;
                }
                drop(runtime.take());
                collection.entries[index].request_restart();
                if save_collection(&root, &collection).is_err() {
                    platform::notice(
                        app,
                        "Could not preserve restart settings; retry when storage is available."
                            .into(),
                    );
                } else {
                    platform::menu(app, false, None);
                }
            }
        }
        if runtime.is_none()
            && hidden
            && foreground
            && let (Some(index), Some(window)) = (selected, &window)
            && !collection.entries[index].restart_required
            && window.width() > 32
            && window.height() > 32
        {
            match boot(&root, &sys, &collection.entries[index], window) {
                Ok(active) => {
                    runtime = Some(active);
                    surface_pending = false;
                    native_paused = None;
                }
                Err(error) => {
                    collection.entries[index].fail(error.to_string());
                    let retained = save_collection(&root, &collection).is_ok();
                    hidden = false;
                    platform::menu(
                        app,
                        true,
                        Some(format!(
                            "Could not start this game.\n\n{error}{}",
                            if retained {
                                ""
                            } else {
                                "\nCould not preserve this error on disk."
                            }
                        )),
                    );
                }
            }
        }
        if runtime.is_some() {
            // SAFETY: all lifecycle work runs on this owner thread; surface strong
            // references are held by Rust and the native queue/backend owners.
            unsafe {
                gcx_engine_dispatch();
                if surface_pending && let Some(window) = &window {
                    surface_pending = gcx_engine_surface(window.ptr().as_ptr().cast()) == 0;
                }
                if matches!(gcx_engine_state(), 1 | 2) {
                    let paused = !foreground
                        || window.is_none()
                        || selected.is_some_and(|i| collection.entries[i].paused);
                    if native_paused != Some(paused) {
                        gcx_engine_pause(i32::from(paused));
                        native_paused = Some(paused);
                        if paused && gcx_engine_flush_cards() == 0 {
                            platform::push(Command::Error("Could not commit the card while pausing. The runtime is retained; retry before restarting.".into()));
                        }
                        if save_collection(&root, &collection).is_err() {
                            if let Some(index) = selected {
                                collection.entries[index].paused = true;
                            }
                            gcx_engine_pause(1);
                            native_paused = Some(true);
                            platform::notice(app,"Could not save session settings. The game is retained and paused; tap Pause to retry.".into());
                        }
                    }
                    let pad = if paused {
                        Pad::default()
                    } else {
                        UI.lock().map(|touch| touch.pad()).unwrap_or_default()
                    };
                    gcx_engine_input(pad.values().as_ptr());
                }
            }
            if runtime
                .as_ref()
                .is_some_and(|active| active.events.stopped.load(Ordering::Relaxed))
            {
                drop(runtime.take());
                hidden = false;
                if let Some(index) = selected {
                    if let Some(error) = gcx::storage::last_card_error() {
                        collection.entries[index].fail(format!(
                            "Card persistence failed during native stop: {error}"
                        ));
                    }
                    if collection.entries[index].restart_required {
                        let mut message = collection.entries[index]
                            .last_error
                            .clone()
                            .unwrap_or_else(|| "The runtime needs an explicit restart.".into());
                        if save_collection(&root, &collection).is_err() {
                            message.push_str("\nCould not preserve this error on disk; it remains on this screen.");
                        }
                        platform::menu(app, true, Some(message));
                    } else {
                        selected = None;
                        platform::menu(app, true, None);
                    }
                } else {
                    platform::menu(app, true, None);
                }
            }
        }
    }
    drop(runtime.take());
    if let Some(error) = gcx::storage::last_card_error()
        && let Some(index) = selected
    {
        collection.entries[index].fail(format!("Card persistence failed during teardown: {error}"));
    }
    save_collection(&root, &collection)?;
    Ok(())
}
