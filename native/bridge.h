// SPDX-License-Identifier: GPL-2.0-or-later
// Maintained direct adapter to Dolphin. Lifecycle calls have one Rust host owner;
// explicitly documented synchronous callbacks also run on native worker threads.
#pragma once
#include <stdint.h>
#if defined(__GNUC__)
#define NTX_EXPORT __attribute__((visibility("default")))
#else
#define NTX_EXPORT
#endif
#ifdef __cplusplus
extern "C" {
#endif
typedef int (*ntx_saved_read_fn)(const char* path, uint8_t* bytes, uintptr_t capacity, uintptr_t* length);
typedef int (*ntx_card_write_fn)(const char* path, const uint8_t* bytes, uintptr_t length);
typedef int (*ntx_checkpoint_write_fn)(void* context, const uint8_t* bytes, uintptr_t length);
typedef void (*ntx_event_fn)(void* context, uint32_t event, const char* message);
// ABI8: called at the actual complete native pad poll. Fill exactly18 doubles;
// synchronous CPU-thread callback, registered before init or while quiescent,
// retained through join. Registration and reads share the controller state lock.
typedef void (*ntx_input_fn)(double* controls);
typedef int64_t (*ntx_platform_fn)(uint32_t operation, const char* path, const char* argument, char* output, uintptr_t capacity);
// ABI 5 additionally requires platform operation 10: attach the calling native
// thread to the JVM through thread exit, return 1 on success. Empty strings and
// no output are supplied. This precedes the final Android surface release.
typedef struct ntx_rect { int32_t left, top, right, bottom; } ntx_rect;
typedef struct ntx_media_info { char game_code[8]; uint32_t region; uint16_t revision; uint8_t disc; uint8_t reserved; } ntx_media_info;
typedef struct ntx_pad_status { uint16_t buttons; uint8_t axes[4]; uint8_t triggers[2]; uint8_t connected; uint8_t reserved; } ntx_pad_status;
typedef ntx_rect (*ntx_ui_fn)(uint32_t phase, float width, float height, float aspect);
NTX_EXPORT uint32_t ntx_engine_abi(void);
NTX_EXPORT void ntx_engine_card_writer(ntx_card_write_fn writer);
// ABI7: 1 exact/query read, 0 genuinely absent, -1 failure. Null/zero queries size.
NTX_EXPORT void ntx_engine_saved_reader(ntx_saved_read_fn reader);
NTX_EXPORT int ntx_engine_flush_cards(void);
NTX_EXPORT void ntx_engine_platform(ntx_platform_fn platform);
NTX_EXPORT void ntx_engine_ui(ntx_ui_fn ui);
NTX_EXPORT int ntx_engine_init(const char* user, const char* sys, ntx_event_fn event, void* event_context, const char* video, const char* audio);
NTX_EXPORT int ntx_engine_boot(const char* media, void* native_window);
NTX_EXPORT int ntx_engine_boot_paused(const char* media, void* native_window);
// Private NTX state ABI 1; requires a paused runtime and host generation checks.
// Synchronous borrowed-buffer callback; success is not host-file durability.
NTX_EXPORT uint32_t ntx_engine_checkpoint_abi(void);
NTX_EXPORT int ntx_engine_checkpoint_capture(uintptr_t limit, ntx_checkpoint_write_fn writer, void* context);
NTX_EXPORT int ntx_engine_checkpoint_load(uint8_t* bytes, uintptr_t length);
// Failed deserialization blocks all persistence until joined shutdown/new init.
NTX_EXPORT int ntx_engine_recovery_required(void);
NTX_EXPORT int ntx_engine_inspect(const char* media, ntx_media_info* info);
NTX_EXPORT void ntx_engine_dispatch(void);
NTX_EXPORT int ntx_engine_state(void);
NTX_EXPORT uint32_t ntx_engine_probe_word(void);
NTX_EXPORT void ntx_engine_pause(int paused);
NTX_EXPORT int ntx_engine_surface(void* native_window);
NTX_EXPORT void ntx_engine_resize(void);
NTX_EXPORT void ntx_engine_input(const double* controls);
NTX_EXPORT void ntx_engine_input_sampler(ntx_input_fn sampler);
NTX_EXPORT void ntx_engine_pad_status(ntx_pad_status* status);
NTX_EXPORT void ntx_engine_stop(void);
NTX_EXPORT void ntx_engine_shutdown(void);
NTX_EXPORT void ntx_ui_rect(float left, float top, float right, float bottom, uint32_t color);
NTX_EXPORT void ntx_ui_circle(float x, float y, float radius, uint32_t color);
NTX_EXPORT void ntx_ui_text(float x, float y, float size, const char* text, uint32_t color);
#ifdef __cplusplus
}
#endif
