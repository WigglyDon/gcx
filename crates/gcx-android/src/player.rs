// SPDX-License-Identifier: MIT
use crate::platform::{self, Command};
use android_activity::{
    AndroidApp, InputStatus, MainEvent, PollEvent,
    input::{InputEvent, MotionAction},
};
use gcx::checkpoint::{self, Binding, Checkpoint};
use gcx::library::{Collection, GameId, GameOpen, SaveProfileId, SavedSession, valid_id};
use gcx::lifecycle::{StopResult, StopRuntime, stop_game};
use gcx::{
    Pad, ProfileLock,
    engine::*,
    touch::Target,
    viewport::{PlayerView, WindowGeometry},
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
static UI: Mutex<PlayerView> = Mutex::new(PlayerView::new());
pub(crate) fn measure_window(geometry: Option<WindowGeometry>) {
    if let Ok(mut ui) = UI.lock() {
        ui.measure(geometry);
    }
}

struct SaveProfile {
    path: PathBuf,
    _writer: ProfileLock,
}
struct LiveRuntime {
    native_alive: bool,
    events: Box<RuntimeEvents>,
    _cards: gcx::storage::CardStore,
    _profile: SaveProfile,
    _boot_window: NativeWindow,
    binding: Binding,
    pending_restore: Option<Checkpoint>,
    accepted_checkpoint: Option<String>,
    accepted_generation: Option<gcx::storage::SaveGeneration>,
    failed_restore: bool,
}
struct RuntimeEvents {
    game_id: String,
    epoch: u64,
    stopped: AtomicBool,
    teardown_error: Mutex<Option<String>>,
}
impl LiveRuntime {
    fn commit_for_departure(&self) -> bool {
        if let Ok(mut touch) = UI.lock() {
            touch.cancel();
        }
        // SAFETY: this live owner retains the native runtime and profile lock.
        // Keep execution paused after the barrier until stop/join completes.
        unsafe {
            gcx_engine_pause(1);
            gcx_engine_input(Pad::default().values().as_ptr());
            // A partially loaded machine is quarantined through teardown.
            gcx_engine_checkpoint_failed() != 0 || gcx_engine_flush_cards() != 0
        }
    }
}
/// Publish the immutable complete generation before changing its durable session
/// reference. The profile lock is still owned, including across metadata sync.
fn save_checkpoint(
    active: &mut LiveRuntime,
    root: &Path,
    collection: &mut Collection,
    index: usize,
) -> Result<()> {
    if active.pending_restore.is_some() {
        return Err("Checkpoint restore is still starting.".into());
    }
    if !active.native_alive {
        return Err("The native runtime is already unloaded.".into());
    }
    if let Ok(mut touch) = UI.lock() {
        touch.cancel();
    }
    unsafe {
        gcx_engine_pause(1);
        gcx_engine_input(Pad::default().values().as_ptr());
    }
    let id = unsafe {
        checkpoint::capture(
            &active._profile.path,
            &active.binding,
            collection.entries[index].paused,
        )?
    };
    let generation = gcx::storage::save_generation(&active._profile.path)?;
    let earlier = collection.entries[index].checkpoint.replace(id.clone());
    if let Err(error) = save_collection(root, collection) {
        // Even if rename preceded the sync error, both possible references still
        // reach complete files. Never retire either on unacknowledged metadata.
        collection.entries[index].checkpoint = earlier;
        return Err(error);
    }
    active.accepted_generation = Some(generation);
    if let Some(old) = active.accepted_checkpoint.replace(id.clone())
        && let Err(error) = checkpoint::retire_previous(&active._profile.path, &old, &id)
    {
        platform::diagnostic(&format!("Prior successful checkpoint retained: {error}"));
    }
    platform::diagnostic("Checkpoint complete: native paused, saves and session reference durable");
    Ok(())
}
impl LiveRuntime {
    fn verify_departure(&self) -> std::result::Result<(), String> {
        self.accepted_generation
            .as_ref()
            .ok_or_else(|| "No acknowledged checkpoint generation.".to_owned())?
            .verify(&self._profile.path)
            .map_err(|e| e.to_string())
    }
    fn restore(&mut self) -> Result<Option<()>> {
        if unsafe { gcx_engine_state() } != 1 {
            return Ok(None);
        }
        let Some(mut state) = self.pending_restore.take() else {
            return Ok(None);
        };
        state.verify_current_saves(&self._profile.path)?;
        if unsafe { gcx_engine_checkpoint_load(state.payload.as_mut_ptr(), state.payload.len()) }
            == 0
        {
            self.failed_restore = true;
            return Err("Checkpoint restore failed. Its file and current saves are retained; restart explicitly with current saves.".into());
        }
        self.accepted_generation = Some(gcx::storage::save_generation(&self._profile.path)?);
        platform::diagnostic(
            "Checkpoint restored into paused native runtime; current saves validated",
        );
        Ok(Some(()))
    }
}
fn pause_for_background(
    root: &Path,
    runtime: &mut Option<LiveRuntime>,
    collection: &mut Collection,
    selected: Option<usize>,
    native_paused: &mut Option<bool>,
) {
    if let Ok(mut touch) = UI.lock() {
        touch.cancel();
    }
    let Some(active) = runtime.as_mut() else {
        return;
    };
    unsafe {
        gcx_engine_pause(1);
        gcx_engine_input(Pad::default().values().as_ptr());
        if !matches!(gcx_engine_state(), 1 | 2) {
            *native_paused = None;
            return;
        }
    }
    if *native_paused != Some(true)
        && active.pending_restore.is_none()
        && let Some(index) = selected
        && !collection.entries[index].restart_required
        && let Err(error) = save_checkpoint(active, root, collection, index)
    {
        platform::push(Command::Error(format!(
            "Could not checkpoint the game: {error}. Current saves and paused runtime retained."
        )));
    }
    *native_paused = Some(true);
}
impl StopRuntime for LiveRuntime {
    fn epoch(&self) -> u64 {
        self.events.epoch
    }
    fn pause_commit(&mut self) -> std::result::Result<(), String> {
        if self.commit_for_departure() {
            Ok(())
        } else {
            Err("Could not durably commit the card. The paused game is retained; Stop game can be retried.".into())
        }
    }
    fn discard_checkpoint(&mut self) {
        if let Some(id) = &self.accepted_checkpoint
            && let Err(error) = checkpoint::discard_successful(&self._profile.path, id)
        {
            platform::diagnostic(&format!(
                "Stopped checkpoint retained after cleanup error: {error}"
            ));
        }
    }
    fn join(&mut self) -> std::result::Result<(), String> {
        if self.native_alive {
            if let Ok(mut error) = self.events.teardown_error.lock() {
                *error = None;
            }
            platform::diagnostic("Native shutdown begin; profile writer still owned");
            // SAFETY: same serialized native owner. Keep all Rust fields/profile
            // alive until the stop caller has durably published its result.
            unsafe {
                self.failed_restore |= gcx_engine_checkpoint_failed() != 0;
                gcx_engine_stop();
                gcx_engine_shutdown();
            }
            self.native_alive = false;
            platform::diagnostic("Native shutdown joined; profile writer still owned");
            // SAFETY: shutdown has joined native workers; this only reads core state.
            if unsafe { gcx_engine_state() } != 0 {
                return Err("Native shutdown did not reach the unloaded state.".into());
            }
        }
        if self.failed_restore {
            return Err("Failed checkpoint machine discarded without writing card or console data; current saves retained.".into());
        }
        if let Some(error) = gcx::storage::last_card_error().or_else(|| {
            self.events
                .teardown_error
                .lock()
                .ok()
                .and_then(|e| e.clone())
        }) {
            return Err(error);
        }
        Ok(())
    }
}
impl Drop for LiveRuntime {
    fn drop(&mut self) {
        if let Ok(mut touch) = UI.lock() {
            touch.cancel();
        }
        // SAFETY: the one android_main thread owns this runtime; callbacks are
        // static and the profile lock outlives completed native teardown.
        unsafe {
            if self.native_alive {
                gcx_engine_input(Pad::default().values().as_ptr());
            }
        }
        let _ = self.join();
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
            if let Ok(mut error) = context.teardown_error.lock() {
                *error = Some(message.clone());
            }
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
extern "C" fn draw(phase: u32, width: f32, height: f32, aspect: f32) -> Rect {
    let (geometry, pad) = {
        let Ok(mut ui) = UI.lock() else {
            return Rect::default();
        };
        let Some(geometry) = ui.present(width, height).cloned() else {
            return Rect::default();
        };
        (geometry, ui.pad())
    };
    let layout = &geometry.layout;
    if phase == 1 {
        let value = |control: gcx::Control| pad.values()[control as usize] as f32;
        // SAFETY: called on the video thread with the donor ImGui frame active.
        unsafe {
            for zone in &layout.zones {
                let a = zone.area;
                let (x, y) = a.center();
                let radius = (a.right - a.left) * 0.5;
                match zone.target {
                    Target::Stick(cx, cy) => {
                        gcx_ui_circle(x, y, radius, 0x703c3b3a);
                        let reach = radius * if cx == gcx::Control::CX { 0.6 } else { 0.5 };
                        let knob_x = x + value(cx) * reach;
                        let knob_y = y - value(cy) * reach;
                        gcx_ui_circle(
                            knob_x,
                            knob_y,
                            radius * if cx == gcx::Control::CX { 0.38 } else { 0.48 },
                            if cx == gcx::Control::CX {
                                0xff51cfe6
                            } else {
                                0xffb5b3ae
                            },
                        );
                        if cx == gcx::Control::CX {
                            let font = radius * 0.4;
                            gcx_ui_text(
                                knob_x - font * 0.3,
                                knob_y - font * 0.5,
                                font,
                                c"C".as_ptr(),
                                0xff252629,
                            );
                        }
                        continue;
                    }
                    Target::Button(control) => {
                        let color = zone.target.color();
                        let color = if value(control) > 0.0 {
                            0xffeeeeee
                        } else {
                            color
                        };
                        if (a.right - a.left - (a.bottom - a.top)).abs() < 1.0 {
                            gcx_ui_circle(x, y, radius, color);
                        } else {
                            gcx_ui_rect(a.left, a.top, a.right, a.bottom, color);
                        }
                    }
                    Target::Travel(control) => {
                        gcx_ui_rect(a.left, a.top, a.right, a.bottom, 0xe0475057);
                        gcx_ui_rect(
                            a.left,
                            a.bottom - (a.bottom - a.top) * 0.25,
                            a.left + (a.right - a.left) * value(control),
                            a.bottom,
                            0xffb5b3ae,
                        );
                    }
                    Target::Dpad => {
                        let step = (a.right - a.left) / 3.0;
                        gcx_ui_rect(a.left + step, a.top, a.right - step, a.bottom, 0xffaaa6a2);
                        gcx_ui_rect(a.left, a.top + step, a.right, a.bottom - step, 0xffaaa6a2);
                        continue;
                    }
                    Target::Pause => gcx_ui_rect(a.left, a.top, a.right, a.bottom, 0xb03c3b3a),
                }
                let count = zone.label.to_bytes().len() as f32;
                let density = geometry.density;
                let font = ((a.bottom - a.top) * 0.4)
                    .min((a.right - a.left) / (count * 0.65 + 1.0))
                    .clamp(10.0 * density, 32.0 * density);
                gcx_ui_text(
                    x - count * font * 0.3,
                    y - font * 0.5,
                    font,
                    zone.label.as_ptr(),
                    if matches!(zone.target, Target::Button(_) | Target::Dpad) {
                        0xff252629
                    } else {
                        0xffeeeeee
                    },
                );
            }
        }
    }
    let a = geometry.picture(aspect);
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
        teardown_error: Mutex::new(None),
    });
    // SAFETY: serialized owner, valid strings and stable process-long callbacks.
    unsafe {
        if gcx_engine_abi() != 6 {
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
        let mut runtime = LiveRuntime {
            native_alive: true,
            events,
            _cards: cards,
            _profile: profile,
            _boot_window: window.clone(),
            binding: Binding {
                game_id: entry.game_id.0.clone(),
                profile_id: entry.profile_id.0.clone(),
                engine: BUILD_ENGINE_SHA256.into(),
                state_abi: gcx_engine_checkpoint_abi(),
                resources: BUILD_RESOURCE_ID.into(),
                media_sha256: String::new(),
                disc: 0,
            },
            pending_restore: None,
            accepted_checkpoint: None,
            accepted_generation: None,
            failed_restore: false,
        };
        let mut info = MediaInfo::default();
        if gcx_engine_inspect(uri.as_ptr(), &mut info) == 0 {
            return Err("Choose a supported GameCube disc image (ISO/GCM or RVZ).".into());
        }
        runtime.binding.disc = info.disc;
        runtime.binding.media_sha256 =
            checkpoint::media_digest(platform::open_document(&entry.uri)?)?;
        if let Some(id) = &entry.checkpoint {
            // No guest has started. Reject mismatches before paused native boot.
            runtime.pending_restore = Some(checkpoint::load(
                &runtime._profile.path,
                id,
                &runtime.binding,
            )?);
        }
        let admitted = if runtime.pending_restore.is_some() {
            gcx_engine_boot_paused(uri.as_ptr(), window.ptr().as_ptr().cast())
        } else {
            gcx_engine_boot(uri.as_ptr(), window.ptr().as_ptr().cast())
        };
        if admitted == 0 {
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
        platform::diagnostic(&format!("Player startup/host failure: {error}"));
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
    platform::diagnostic("Resources ready; loading collection");
    let mut collection = load_collection(&root)?;
    let mut interrupted = false;
    for entry in &mut collection.entries {
        interrupted |= entry.recover_interrupted();
    }
    if interrupted && save_collection(&root, &collection).is_err() {
        platform::diagnostic(
            "Could not persist interrupted-runtime recovery; the original intent marker remains retained.",
        );
    }
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
    } else if let Some(index) = selected.filter(|i| collection.entries[*i].stopped) {
        platform::unloaded_menu(app, &collection.entries[index].title, None);
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
                        measure_window(None);
                        platform::refresh_window(app);
                        surface_pending = runtime.is_some();
                    }
                    MainEvent::TerminateWindow { .. } => {
                        // Complete pause/checkpoint while the old drawable is still
                        // retained; surface loss never requests a fresh boot.
                        pause_for_background(
                            &root,
                            &mut runtime,
                            &mut collection,
                            selected,
                            &mut native_paused,
                        );
                        window = None;
                        measure_window(None);
                        if let Ok(mut touch) = UI.lock() {
                            touch.cancel();
                        }
                    }
                    MainEvent::GainedFocus => {
                        foreground = true;
                        platform::refresh_window(app);
                    }
                    MainEvent::LostFocus | MainEvent::Pause | MainEvent::Stop => {
                        foreground = false;
                        pause_for_background(
                            &root,
                            &mut runtime,
                            &mut collection,
                            selected,
                            &mut native_paused,
                        );
                        if let Ok(mut touch) = UI.lock() {
                            touch.cancel();
                        }
                    }
                    MainEvent::Destroy => destroyed = true,
                    MainEvent::WindowResized { .. }
                    | MainEvent::ContentRectChanged { .. }
                    | MainEvent::ConfigChanged { .. } => {
                        measure_window(None);
                        platform::refresh_window(app);
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
                                        && window.is_some()
                                        && let Ok(mut touch) = UI.lock()
                                    {
                                        match motion.action() {
                                            MotionAction::Down | MotionAction::PointerDown => {
                                                let pointer =
                                                    motion.pointer_at_index(motion.pointer_index());
                                                if touch.down(
                                                    pointer.pointer_id(),
                                                    pointer.x(),
                                                    pointer.y(),
                                                ) && let Some(index) = selected
                                                {
                                                    collection.entries[index].paused = true;
                                                    touch.cancel();
                                                    hidden = false;
                                                    if let Some(active) = &runtime {
                                                        platform::player_menu(
                                                            app,
                                                            active.events.epoch,
                                                            !collection.entries[index]
                                                                .restart_required,
                                                            None,
                                                        );
                                                    }
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
                                                let pointer =
                                                    motion.pointer_at_index(motion.pointer_index());
                                                touch.up(pointer.pointer_id());
                                            }
                                            MotionAction::Cancel => touch.cancel(),
                                            _ => {}
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
                Command::Game(id) => match collection
                    .open_game(&id, runtime.as_ref().map(|r| r.events.game_id.as_str()))
                {
                    Ok(GameOpen::ResumeCurrent) => {
                        if let Some(index) = selected {
                            collection.entries[index].paused = false;
                        }
                        platform::menu(app, false, None);
                        continue;
                    }
                    Ok(GameOpen::Select(index)) => {
                        Command::Selected(collection.entries[index].uri.clone())
                    }
                    Err(error) => {
                        platform::notice(app, error.into());
                        continue;
                    }
                },
                other => other,
            };
            match command {
                Command::NativeError { .. } | Command::Game(_) => continue,
                Command::Library => {
                    hidden = false;
                    if let Some(index) = selected.filter(|_| runtime.is_some()) {
                        collection.entries[index].paused = true;
                    }
                    pause_for_background(
                        &root,
                        &mut runtime,
                        &mut collection,
                        selected,
                        &mut native_paused,
                    );
                    platform::library_menu(
                        app,
                        collection
                            .entries
                            .iter()
                            .map(|e| (e.game_id.0.clone(), e.title.clone()))
                            .collect(),
                        runtime.as_ref().map(|r| r.events.epoch),
                        selected.is_some_and(|i| !collection.entries[i].restart_required),
                    );
                }
                Command::Choose => {
                    if !picker_pending {
                        platform::picker(app);
                        picker_pending = true;
                    }
                }
                Command::Restart => restart_requested = true,
                Command::Resume(epoch) => {
                    if runtime.as_ref().is_some_and(|r| r.events.epoch == epoch)
                        && let Some(index) = selected
                        && !collection.entries[index].restart_required
                    {
                        collection.entries[index].paused = false;
                        platform::menu(app, false, None);
                    }
                }
                Command::Stop(epoch) => {
                    if let Some(index) = selected {
                        match stop_game(&mut runtime, epoch, &mut collection, index, |c| {
                            save_collection(&root, c).map_err(|e| e.to_string())
                        }) {
                            StopResult::Ignored => {}
                            StopResult::Retained(error) => {
                                hidden = false;
                                native_paused = Some(true);
                                platform::player_menu(app, epoch, false, Some(error));
                            }
                            StopResult::Unloaded { error } => {
                                hidden = false;
                                native_paused = None;
                                surface_pending = false;
                                restart_requested = false;
                                platform::diagnostic(if error.is_none() {
                                    "Stop complete: native workers joined, clean metadata durable, profile released; game unloaded"
                                } else {
                                    "Stop unloaded the native runtime; recovery metadata/error retained"
                                });
                                platform::unloaded_menu(
                                    app,
                                    &collection.entries[index].title,
                                    error,
                                );
                            }
                        }
                    }
                }
                Command::Open => {
                    if runtime.is_none()
                        && let Some(index) = selected
                        && collection.entries[index].stopped
                        && !collection.entries[index].restart_required
                    {
                        collection.entries[index].request_restart();
                        if save_collection(&root, &collection).is_ok() {
                            platform::menu(app, false, None);
                        } else {
                            collection.entries[index].stopped = true;
                            platform::notice(
                                app,
                                "Could not preserve launch intent; the game remains unloaded."
                                    .into(),
                            );
                        }
                    }
                }
                Command::Selected(uri) => {
                    picker_pending = false;
                    if runtime.is_some() {
                        if selected.is_some_and(|i| collection.entries[i].restart_required) {
                            platform::notice(app, "The current game needs recovery. Use Stop or Restart with current saves before opening another game.".into());
                            continue;
                        }
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
                            if let Some(index) = selected {
                                collection.entries[index].paused = false;
                            }
                            platform::menu(app, false, None);
                            continue;
                        }
                    }
                    // Save the departing scene and join while its profile remains
                    // exclusively owned. A failed checkpoint keeps its live RAM.
                    if let (Some(active), Some(index)) = (runtime.as_mut(), selected) {
                        let saved = save_checkpoint(active, &root, &mut collection, index);
                        if let Err(error) = saved {
                            native_paused = Some(true);
                            platform::push(Command::Error(format!(
                                "Could not leave the current game: {error}"
                            )));
                            continue;
                        }
                        let joined = active.join().and_then(|()| active.verify_departure());
                        collection.entries[index].runtime_incomplete = joined.is_err();
                        if let Err(error) = joined {
                            collection.entries[index].fail(error);
                        }
                        if let Err(error) = save_collection(&root, &collection) {
                            collection.entries[index].runtime_incomplete = true;
                            collection.entries[index]
                                .fail(format!("Could not preserve the departing session: {error}"));
                            let _ = save_collection(&root, &collection);
                            hidden = false;
                            platform::menu(app, true, collection.entries[index].last_error.clone());
                            drop(runtime.take());
                            native_paused = None;
                            continue;
                        }
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
                                runtime_incomplete: false,
                                stopped: false,
                                checkpoint: None,
                            });
                            collection.entries.len() - 1
                        };
                    collection.selected = Some(collection.entries[index].game_id.0.clone());
                    selected = Some(index);
                    if collection.entries[index].stopped
                        && !collection.entries[index].restart_required
                    {
                        collection.entries[index].request_restart();
                    }
                    if !collection.entries[index].restart_required {
                        collection.entries[index].paused = false;
                    }
                    if save_collection(&root, &collection).is_err() {
                        platform::notice(app,"Could not durably save the collection. Files are retained; choose the document again to retry.".into());
                        continue;
                    }
                    if collection.entries[index].restart_required {
                        platform::menu(app, true, collection.entries[index].last_error.clone());
                    } else {
                        platform::menu(app, false, None);
                    }
                }
                Command::MenuHidden(generation) => {
                    if platform::menu_current(generation) {
                        hidden = true;
                    }
                }
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
                        if let Some(active) = &runtime {
                            platform::player_menu(
                                app,
                                active.events.epoch,
                                false,
                                Some(format!("{error}{persistence}")),
                            );
                        }
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
                if runtime
                    .as_ref()
                    .is_some_and(|active| !active.commit_for_departure())
                {
                    native_paused = Some(true);
                    platform::push(Command::Error("The memory card could not be committed. The current runtime is retained; restart has not occurred.".into()));
                    continue;
                }
                let mut teardown_error = None;
                if let Some(active) = runtime.as_mut() {
                    let quarantined =
                        active.failed_restore || unsafe { gcx_engine_checkpoint_failed() != 0 };
                    let joined = active.join();
                    if !quarantined {
                        teardown_error = joined.err();
                    }
                    // A failed-load quarantine has protected durable ordinary
                    // files through shutdown; only this explicit action cold boots.
                }
                if let Some(error) = teardown_error {
                    collection.entries[index].fail(format!(
                        "Persistence failed during restart teardown: {error}"
                    ));
                    hidden = false;
                    platform::menu(app, true, collection.entries[index].last_error.clone());
                    let _ = save_collection(&root, &collection);
                    drop(runtime.take());
                    continue;
                }
                // Preserve the lock until restart intent is durably recorded.
                let earlier = collection.entries[index].clone();
                collection.entries[index].request_restart();
                if let Err(error) = save_collection(&root, &collection) {
                    collection.entries[index] = earlier;
                    collection.entries[index]
                        .fail(format!("Could not preserve restart intent: {error}"));
                    let _ = save_collection(&root, &collection);
                    hidden = false;
                    platform::menu(app, true, collection.entries[index].last_error.clone());
                } else {
                    platform::menu(app, false, None);
                }
                drop(runtime.take());
                native_paused = None;
            }
        }
        if runtime.is_none()
            && hidden
            && foreground
            && let (Some(index), Some(window)) = (selected, &window)
            && collection.entries[index].boot_allowed()
            && window.width() > 32
            && window.height() > 32
        {
            collection.entries[index].runtime_incomplete = true;
            if save_collection(&root, &collection).is_err() {
                collection.entries[index]
                    .fail("Could not durably record the runtime start. No game was booted.".into());
                hidden = false;
                platform::menu(app, true, collection.entries[index].last_error.clone());
                continue;
            }
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
                if foreground
                    && window.is_some()
                    && !surface_pending
                    && let (Some(active), Some(index)) = (runtime.as_mut(), selected)
                    && active.pending_restore.is_some()
                    && !collection.entries[index].restart_required
                {
                    match active.restore() {
                        Ok(Some(())) => {
                            // SavedSession owns the latest durable UI intent;
                            // Resume after the capture may have changed it.
                            let paused = collection.entries[index].paused;
                            active.accepted_checkpoint =
                                collection.entries[index].checkpoint.clone();
                            native_paused = Some(true);
                            if paused {
                                hidden = false;
                                platform::player_menu(app, active.events.epoch, true, None);
                            }
                        }
                        Ok(None) => {}
                        Err(error) => {
                            collection.entries[index].fail(error.to_string());
                            platform::push(Command::Error(error.to_string()));
                            native_paused = Some(true);
                        }
                    }
                }
                if matches!(gcx_engine_state(), 1 | 2) {
                    let restoring = runtime
                        .as_ref()
                        .is_some_and(|r| r.pending_restore.is_some());
                    let paused = restoring
                        || !hidden
                        || !foreground
                        || window.is_none()
                        || selected.is_some_and(|i| collection.entries[i].paused);
                    if native_paused != Some(paused) {
                        gcx_engine_pause(i32::from(paused));
                        native_paused = Some(paused);
                        if paused
                            && !restoring
                            && let (Some(active), Some(index)) = (runtime.as_mut(), selected)
                            && !collection.entries[index].restart_required
                            && let Err(error) =
                                save_checkpoint(active, &root, &mut collection, index)
                        {
                            platform::push(Command::Error(format!(
                                "Could not checkpoint the paused game: {error}"
                            )));
                        }
                        if save_collection(&root, &collection).is_err() {
                            if let Some(index) = selected {
                                collection.entries[index].paused = true;
                            }
                            gcx_engine_pause(1);
                            native_paused = Some(true);
                            platform::push(Command::Error("Could not preserve session settings. The paused game and previous checkpoint are retained.".into()));
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
                let joined = runtime.as_mut().expect("checked live runtime").join();
                hidden = false;
                if let Some(index) = selected {
                    collection.entries[index].runtime_incomplete = joined.is_err();
                    collection.entries[index].stopped = true;
                    if let Err(error) = joined {
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
                        if save_collection(&root, &collection).is_err() {
                            collection.entries[index].runtime_incomplete = true;
                            collection.entries[index]
                                .fail("Could not preserve native stop metadata.".into());
                            let _ = save_collection(&root, &collection);
                        }
                        platform::unloaded_menu(
                            app,
                            &collection.entries[index].title,
                            collection.entries[index]
                                .restart_required
                                .then(|| collection.entries[index].last_error.clone())
                                .flatten(),
                        );
                    }
                } else {
                    platform::menu(app, true, None);
                }
                drop(runtime.take());
            }
        }
    }
    if let (Some(active), Some(index)) = (runtime.as_mut(), selected) {
        let checkpoint = if collection.entries[index].restart_required {
            Err("The runtime already requires explicit recovery.".into())
        } else if let Some(state) = &active.pending_restore {
            state
                .verify_current_saves(&active._profile.path)
                .map_err(Into::into)
        } else {
            save_checkpoint(active, &root, &mut collection, index)
        };
        let barrier = active.pause_commit();
        let joined = active.join().and_then(|()| {
            if let Some(state) = &active.pending_restore {
                state
                    .verify_current_saves(&active._profile.path)
                    .map_err(|e| e.to_string())
            } else {
                active.verify_departure()
            }
        });
        collection.entries[index].runtime_incomplete =
            checkpoint.is_err() || barrier.is_err() || joined.is_err();
        // Activity destruction is a resumable departure, never explicit Stop.
        if let Some(error) = checkpoint
            .err()
            .or_else(|| barrier.err().map(Into::into))
            .or_else(|| joined.err().map(Into::into))
        {
            collection.entries[index].fail(format!("Interrupted departure: {error}"));
        }
    }
    let saved = save_collection(&root, &collection);
    drop(runtime.take());
    saved?;
    Ok(())
}
