// SPDX-License-Identifier: MIT
//! Unsafe declarations for native/bridge.h, ABI 8. One Rust host thread owns
//! lifecycle calls. Callbacks must not unwind and remain valid through teardown.
use std::ffi::{c_char, c_void};
pub const BUILD_ENGINE_SHA256: Option<&str> = option_env!("NTX_ENGINE_SHA256");
pub const BUILD_RESOURCE_ID: Option<&str> = option_env!("NTX_RESOURCE_ID");

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
#[derive(Default)]
pub struct MediaInfo {
    pub game_code: [c_char; 8],
    pub region: u32,
    pub revision: u16,
    pub disc: u8,
    pub reserved: u8,
}
#[repr(C)]
#[derive(Default, Debug)]
pub struct PadStatus {
    pub buttons: u16,
    pub axes: [u8; 4],
    pub triggers: [u8; 2],
    pub connected: u8,
    pub reserved: u8,
}
pub type Event = unsafe extern "C" fn(*mut c_void, u32, *const c_char);
pub type Platform = extern "C" fn(u32, *const c_char, *const c_char, *mut c_char, usize) -> i64;
pub type Ui = extern "C" fn(u32, f32, f32, f32) -> Rect;
pub type CheckpointWriter = unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32;
unsafe extern "C" {
    pub fn ntx_engine_abi() -> u32;
    pub fn ntx_engine_saved_reader(
        reader: unsafe extern "C" fn(*const c_char, *mut u8, usize, *mut usize) -> i32,
    );
    pub fn ntx_engine_card_writer(
        writer: unsafe extern "C" fn(*const c_char, *const u8, usize) -> i32,
    );
    pub fn ntx_engine_flush_cards() -> i32;
    pub fn ntx_engine_platform(platform: Platform);
    pub fn ntx_engine_ui(ui: Ui);
    pub fn ntx_engine_init(
        user: *const c_char,
        sys: *const c_char,
        event: Event,
        event_context: *mut c_void,
        video: *const c_char,
        audio: *const c_char,
    ) -> i32;
    pub fn ntx_engine_inspect(media: *const c_char, info: *mut MediaInfo) -> i32;
    pub fn ntx_engine_boot(media: *const c_char, window: *mut c_void) -> i32;
    pub fn ntx_engine_boot_paused(media: *const c_char, window: *mut c_void) -> i32;
    pub fn ntx_engine_checkpoint_abi() -> u32;
    pub fn ntx_engine_checkpoint_capture(
        limit: usize,
        writer: CheckpointWriter,
        context: *mut c_void,
    ) -> i32;
    pub fn ntx_engine_checkpoint_load(bytes: *mut u8, length: usize) -> i32;
    pub fn ntx_engine_recovery_required() -> i32;
    pub fn ntx_engine_dispatch();
    pub fn ntx_engine_state() -> i32;
    pub fn ntx_engine_probe_word() -> u32;
    pub fn ntx_engine_pause(paused: i32);
    pub fn ntx_engine_surface(window: *mut c_void) -> i32;
    pub fn ntx_engine_resize();
    pub fn ntx_engine_input(controls: *const f64);
    /// CPU-thread callback at the actual complete native pad poll, through join.
    pub fn ntx_engine_input_sampler(sampler: Option<unsafe extern "C" fn(*mut f64)>);
    pub fn ntx_engine_pad_status(status: *mut PadStatus);
    pub fn ntx_engine_stop();
    pub fn ntx_engine_shutdown();
    pub fn ntx_ui_rect(left: f32, top: f32, right: f32, bottom: f32, color: u32);
    pub fn ntx_ui_circle(x: f32, y: f32, radius: f32, color: u32);
    pub fn ntx_ui_text(x: f32, y: f32, size: f32, text: *const c_char, color: u32);
}
