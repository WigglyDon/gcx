// SPDX-License-Identifier: MIT
#![cfg(target_os = "android")]
mod n64_host;
mod platform;
mod player;

#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
    player::run(app);
}
