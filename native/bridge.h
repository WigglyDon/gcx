// SPDX-License-Identifier: GPL-2.0-or-later
// Maintained direct adapter to Dolphin. Lifecycle calls have one Rust host owner;
// explicitly documented synchronous callbacks also run on native worker threads.
#pragma once
#include <stdint.h>
#if defined(__GNUC__)
#define GCX_EXPORT __attribute__((visibility("default")))
#else
#define GCX_EXPORT
#endif
#ifdef __cplusplus
extern "C" {
#endif
typedef int (*gcx_saved_read_fn)(const char* path, uint8_t* bytes, uintptr_t capacity, uintptr_t* length);
typedef int (*gcx_card_write_fn)(const char* path, const uint8_t* bytes, uintptr_t length);
typedef int (*gcx_checkpoint_write_fn)(void* context, const uint8_t* bytes, uintptr_t length);
typedef void (*gcx_event_fn)(void* context, uint32_t event, const char* message);
// ABI8: called at the actual complete native pad poll. Fill exactly18 doubles;
// synchronous CPU-thread callback, registered before init or while quiescent,
// retained through join. Registration and reads share the controller state lock.
typedef void (*gcx_input_fn)(double* controls);
typedef int64_t (*gcx_platform_fn)(uint32_t operation, const char* path, const char* argument, char* output, uintptr_t capacity);
// ABI 5 additionally requires platform operation 10: attach the calling native
// thread to the JVM through thread exit, return 1 on success. Empty strings and
// no output are supplied. This precedes the final Android surface release.
typedef struct gcx_rect { int32_t left, top, right, bottom; } gcx_rect;
typedef struct gcx_media_info { char game_code[8]; uint32_t region; uint16_t revision; uint8_t disc; uint8_t reserved; } gcx_media_info;
typedef struct gcx_pad_status { uint16_t buttons; uint8_t axes[4]; uint8_t triggers[2]; uint8_t connected; uint8_t reserved; } gcx_pad_status;
typedef gcx_rect (*gcx_ui_fn)(uint32_t phase, float width, float height, float aspect);
GCX_EXPORT uint32_t gcx_engine_abi(void);
GCX_EXPORT void gcx_engine_card_writer(gcx_card_write_fn writer);
// ABI7: 1 exact/query read, 0 genuinely absent, -1 failure. Null/zero queries size.
GCX_EXPORT void gcx_engine_saved_reader(gcx_saved_read_fn reader);
GCX_EXPORT int gcx_engine_flush_cards(void);
GCX_EXPORT void gcx_engine_platform(gcx_platform_fn platform);
GCX_EXPORT void gcx_engine_ui(gcx_ui_fn ui);
GCX_EXPORT int gcx_engine_init(const char* user, const char* sys, gcx_event_fn event, void* event_context, const char* video, const char* audio);
GCX_EXPORT int gcx_engine_boot(const char* media, void* native_window);
GCX_EXPORT int gcx_engine_boot_paused(const char* media, void* native_window);
// Private GCX state ABI 1; requires a paused runtime and host generation checks.
// Synchronous borrowed-buffer callback; success is not host-file durability.
GCX_EXPORT uint32_t gcx_engine_checkpoint_abi(void);
GCX_EXPORT int gcx_engine_checkpoint_capture(uintptr_t limit, gcx_checkpoint_write_fn writer, void* context);
GCX_EXPORT int gcx_engine_checkpoint_load(uint8_t* bytes, uintptr_t length);
// Failed deserialization blocks all persistence until joined shutdown/new init.
GCX_EXPORT int gcx_engine_recovery_required(void);
GCX_EXPORT int gcx_engine_inspect(const char* media, gcx_media_info* info);
GCX_EXPORT void gcx_engine_dispatch(void);
GCX_EXPORT int gcx_engine_state(void);
GCX_EXPORT uint32_t gcx_engine_probe_word(void);
GCX_EXPORT void gcx_engine_pause(int paused);
GCX_EXPORT int gcx_engine_surface(void* native_window);
GCX_EXPORT void gcx_engine_resize(void);
GCX_EXPORT void gcx_engine_input(const double* controls);
GCX_EXPORT void gcx_engine_input_sampler(gcx_input_fn sampler);
GCX_EXPORT void gcx_engine_pad_status(gcx_pad_status* status);
GCX_EXPORT void gcx_engine_stop(void);
GCX_EXPORT void gcx_engine_shutdown(void);
GCX_EXPORT void gcx_ui_rect(float left, float top, float right, float bottom, uint32_t color);
GCX_EXPORT void gcx_ui_circle(float x, float y, float radius, uint32_t color);
GCX_EXPORT void gcx_ui_text(float x, float y, float size, const char* text, uint32_t color);
#ifdef __cplusplus
}
#endif
