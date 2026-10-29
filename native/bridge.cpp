// SPDX-License-Identifier: GPL-2.0-or-later
// Dolphin Host implementations adapted from MainNoGUI.cpp at cf5cb7ef81c3e8e7af5c9d566334219ffda9f68b.
// Copyright 2008 Dolphin Emulator Project; ntx adapter changes 2026.
#include "bridge.h"
#include "Common/Config/Config.h"
#include "Common/FileUtil.h"
#include "Common/MsgHandler.h"
#include "Common/WindowSystemInfo.h"
#include "Core/Boot/Boot.h"
#include "Core/BootManager.h"
#include "Core/Config/MainSettings.h"
#include "Core/ConfigManager.h"
#include "Core/Core.h"
#include "Core/Host.h"
#include "Core/HW/Memmap.h"
#include "Core/HW/GCPad.h"
#include "Core/HW/GCMemcard/GCMemcardRaw.h"
#include "Core/HW/EXI/EXI.h"
#include "Core/HW/EXI/EXI_DeviceMemoryCard.h"
#include "InputCommon/GCPadStatus.h"
#include "Core/System.h"
#include "Core/State.h"
#include "InputCommon/ControllerInterface/Touch/InputOverrider.h"
#include "InputCommon/ControllerEmu/ControllerEmu.h"
#include "UICommon/UICommon.h"
#include "VideoCommon/Present.h"
#include "VideoCommon/VideoBackendBase.h"
#include "VideoCommon/VideoEvents.h"
#include <imgui.h>
#include <cstring>
#include <algorithm>
#include <atomic>

static ntx_card_write_fn s_card_writer = nullptr;
static ntx_saved_read_fn s_saved_reader = nullptr;
static ntx_event_fn s_event = nullptr;
static void* s_event_context = nullptr;
static ntx_platform_fn s_platform = nullptr;
static ntx_ui_fn s_ui = nullptr;
static Common::EventHook s_overlay_hook;
static Common::EventHook s_present_hook;
static std::atomic<bool> s_checkpoint_failed = false;
static std::atomic<bool> s_persistence_blocked = false;
static void Event(uint32_t id, const char* message) { if (s_event) s_event(s_event_context, id, message); }
std::vector<std::string> Host_GetPreferredLocales() { return {}; }
void Host_PPCSymbolsChanged() {}
void Host_PPCBreakpointsChanged() {}
bool Host_UIBlocksControllerState() { return false; }
void Host_Message(HostMessageID id) { Event(static_cast<uint32_t>(id), ""); }
void Host_UpdateTitle(const std::string& title) { Event(1, title.c_str()); }
void Host_UpdateDisasmDialog() {}
void Host_JitCacheInvalidation() {}
void Host_JitProfileDataWiped() {}
void Host_RequestRenderWindowSize(int width, int height) {}
bool Host_RendererHasFocus() { return true; }
bool Host_RendererHasFullFocus() { return true; }
bool Host_RendererIsFullscreen() { return false; }
bool Host_TASInputHasFocus() { return false; }
void Host_YieldToUI() {}
void Host_TitleChanged() {}
void Host_UpdateDiscordClientID(const std::string& client_id) {}
bool Host_UpdateDiscordPresenceRaw(const std::string& details, const std::string& state,
    const std::string& large_image_key, const std::string& large_image_text,
    const std::string& small_image_key, const std::string& small_image_text,
    int64_t start_timestamp, int64_t end_timestamp, int party_size, int party_max) { return false; }
std::unique_ptr<GBAHostInterface> Host_CreateGBAHost(std::weak_ptr<HW::GBA::Core> core) { return nullptr; }

static ntx_input_fn s_input_sampler = nullptr;
void ntx_engine_input_sampler(ntx_input_fn sampler)
{
  const auto lock = ControllerEmu::EmulatedController::GetStateLock();
  s_input_sampler = sampler;
}
uint32_t ntx_engine_abi() { return 8; }
void ntx_engine_card_writer(ntx_card_write_fn writer) { s_card_writer = writer; }
void ntx_engine_saved_reader(ntx_saved_read_fn reader) { s_saved_reader = reader; }
void ntx_engine_platform(ntx_platform_fn platform) { s_platform = platform; }
void ntx_engine_ui(ntx_ui_fn ui) { s_ui = ui; }
extern "C" int64_t ntx_platform_dispatch(uint32_t op, const char* path, const char* argument, char* output, uintptr_t capacity)
{
  return s_platform ? s_platform(op, path, argument, output, capacity) : -1;
}
int ntx_engine_init(const char* user, const char* sys, ntx_event_fn event, void* event_context, const char* video, const char* audio)
{
  s_event = event;
  s_checkpoint_failed = false;
  s_persistence_blocked = false;
  s_event_context = event_context;
  Common::RegisterMsgAlertHandler([](const char* caption, const char* text, bool, Common::MsgType style) {
    Event(style == Common::MsgType::Critical ? 2 : 3, text);
    return false;
  });
#ifdef ANDROID
  static std::string resource_directory;
  if (resource_directory.empty())
  {
    File::SetSysDirectory(sys);
    resource_directory = sys;
  }
  else if (resource_directory != sys)
  {
    Event(2, "Engine resources cannot change during this process.");
    Common::RegisterMsgAlertHandler(nullptr);
    s_event = nullptr;
  s_event_context = nullptr;
    return 0;
  }
#endif
  if (!s_saved_reader)
  {
    Event(2, "A checked saved-image reader is required before initialization.");
    Common::RegisterMsgAlertHandler(nullptr);
    s_event = nullptr;
    s_event_context = nullptr;
    return 0;
  }
  ExpansionInterface::SetImageReadHandler([](const std::string& path, std::span<u8> bytes, size_t* length) {
    return s_saved_reader ? s_saved_reader(path.c_str(), bytes.data(), bytes.size(), length) : -1;
  });
  UICommon::SetUserDirectory(user);
  UICommon::CreateDirectories();
  UICommon::Init();
  // Mechanical binding to the Rust-owned profile: imported absolute host paths
  // are not portable save identities. Dolphin resolves defaults in this user root.
  Config::SetBaseOrCurrent(Config::MAIN_MEMCARD_A_PATH, std::string{});
  Config::SetBaseOrCurrent(Config::MAIN_MEMCARD_B_PATH, std::string{});
  Config::SetBaseOrCurrent(Config::MAIN_SLOT_A, ExpansionInterface::EXIDeviceType::MemoryCard);
  Config::SetBaseOrCurrent(Config::MAIN_SLOT_B, ExpansionInterface::EXIDeviceType::None);
  MemoryCard::SetWriteHandler([](const std::string& path, std::span<const u8> data) {
    if (s_persistence_blocked || ExpansionInterface::ImageReadFailed())
      return false;
    const bool ok = s_card_writer && s_card_writer(path.c_str(), data.data(), data.size()) != 0;
    if (!ok)
      Event(20, "Could not durably write the memory card. The runtime must be retained until retry succeeds.");
    return ok;
  });
  ExpansionInterface::SetSramWriteHandler([](const std::string& path, std::span<const u8> data) {
    if (s_persistence_blocked || ExpansionInterface::ImageReadFailed())
      return false;
    const bool ok = s_card_writer && s_card_writer(path.c_str(), data.data(), data.size()) != 0;
    if (!ok)
      Event(20, "Could not durably write console data. Retain the paused runtime for retry.");
    return ok;
  });
  Config::SetBaseOrCurrent(Config::MAIN_SKIP_IPL, true);
  Config::SetBaseOrCurrent(Config::MAIN_DSP_HLE, true);
  Config::SetBaseOrCurrent(Config::MAIN_GFX_BACKEND, std::string(video));
  Config::SetBaseOrCurrent(Config::MAIN_AUDIO_BACKEND, std::string(audio));
  VideoBackendBase::ActivateBackend(video);
  UICommon::InitControllers(WindowSystemInfo{});
  ciface::Touch::RegisterGameCubeInputOverrider(0);
  Pad::SetInputPollCallback([](int index) {
    if (index == 0 && s_input_sampler) {
      double controls[18]{};
      s_input_sampler(controls);
      ntx_engine_input(controls);
    }
  });
  auto& events = GetVideoEvents();
  s_overlay_hook = events.draw_overlay_event.Register([] {
    if (s_ui) {
      const auto size = ImGui::GetIO().DisplaySize;
      s_ui(1, size.x, size.y, g_presenter->CalculateDrawAspectRatio());
    }
  });
  s_present_hook = events.before_present_event.Register([](PresentInfo&) {
    if (s_ui && g_presenter) {
      const auto rect = s_ui(0, static_cast<float>(g_presenter->GetBackbufferWidth()),
        static_cast<float>(g_presenter->GetBackbufferHeight()), g_presenter->CalculateDrawAspectRatio());
      g_presenter->SetContentArea({rect.left, rect.top, rect.right, rect.bottom});
    }
  });
  return 1;
}
static int Boot(const char* media, void* native_window, bool paused)
{
  auto boot = BootParameters::GenerateFromFile(media);
  if (!boot)
    return 0;
  const auto* disc = std::get_if<BootParameters::Disc>(&boot->parameters);
  const auto* executable = std::get_if<BootParameters::Executable>(&boot->parameters);
  if (!((disc && disc->volume->GetVolumeType() == DiscIO::Platform::GameCubeDisc) ||
        (executable && !executable->reader->IsWii())))
    return 0;
  WindowSystemInfo wsi;
#ifdef ANDROID
  if (native_window)
    wsi = WindowSystemInfo(WindowSystemType::Android, nullptr, native_window, native_window);
#endif
  SConfig::GetInstance().bBootToPause = paused;
  return BootManager::BootCore(Core::System::GetInstance(), std::move(boot), wsi);
}
int ntx_engine_boot(const char* media, void* native_window) { return Boot(media, native_window, false); }
int ntx_engine_boot_paused(const char* media, void* native_window) { return Boot(media, native_window, true); }
int ntx_engine_inspect(const char* media, ntx_media_info* info)
{
  if (!info)
    return 0;
  auto boot = BootParameters::GenerateFromFile(media);
  if (!boot)
    return 0;
  const auto* disc = std::get_if<BootParameters::Disc>(&boot->parameters);
  if (!disc || disc->volume->GetVolumeType() != DiscIO::Platform::GameCubeDisc)
    return 0;
  *info = {};
  const std::string id = disc->volume->GetGameID();
  std::memcpy(info->game_code, id.data(), std::min(id.size(), sizeof(info->game_code) - 1));
  info->region = static_cast<uint32_t>(disc->volume->GetRegion());
  info->revision = disc->volume->GetRevision().value_or(0);
  info->disc = disc->volume->GetDiscNumber().value_or(0);
  return 1;
}
void ntx_engine_dispatch() { Core::HostDispatchJobs(Core::System::GetInstance()); }
int ntx_engine_state() { return static_cast<int>(Core::GetState(Core::System::GetInstance())); }
uint32_t ntx_engine_probe_word()
{
  auto& system = Core::System::GetInstance();
  const Core::CPUThreadGuard guard(system);
  return system.GetMemory().Read_U32(0x10000);
}
void ntx_engine_pause(int paused)
{
  if (!paused && ntx_engine_recovery_required())
    return;
  Core::SetState(Core::System::GetInstance(), paused ? Core::State::Paused : Core::State::Running);
  if (paused && Core::IsRunning(Core::System::GetInstance()))
  {
    const Core::CPUThreadGuard guard(Core::System::GetInstance());
  }
}
int ntx_engine_surface(void* native_window)
{
  if (!g_presenter)
    return 0;
  g_presenter->ChangeSurface(native_window);
  return 1;
}
void ntx_engine_resize() { if (g_presenter) g_presenter->ResizeSurface(); }
void ntx_engine_input(const double* controls)
{
  const auto lock = ControllerEmu::EmulatedController::GetStateLock();
  for (int i = 0; i < 18; ++i)
    ciface::Touch::SetControlState(0, static_cast<ciface::Touch::ControlID>(i), controls[i]);
}
void ntx_engine_pad_status(ntx_pad_status* status)
{
  if (!status)
    return;
  const auto lock = ControllerEmu::EmulatedController::GetStateLock();
  const auto pad = Pad::PollAndGetStatus(0);
  *status = {pad.button, {pad.stickX, pad.stickY, pad.substickX, pad.substickY},
             {pad.triggerLeft, pad.triggerRight}, static_cast<uint8_t>(pad.isConnected), 0};
}
int ntx_engine_flush_cards()
{
  auto& system = Core::System::GetInstance();
  if (!Core::IsRunning(system) || s_persistence_blocked || ExpansionInterface::ImageReadFailed())
    return 0;
  const Core::CPUThreadGuard guard(system);
  auto* device = system.GetExpansionInterface().GetDevice(ExpansionInterface::Slot::A);
  if (!device || device->m_device_type != ExpansionInterface::EXIDeviceType::MemoryCard)
    return 0;
  return static_cast<ExpansionInterface::CEXIMemoryCard*>(device)->FlushToStorage() &&
         system.GetExpansionInterface().FlushSRAM();
}
uint32_t ntx_engine_checkpoint_abi() { return 1; }
int ntx_engine_recovery_required() { return s_checkpoint_failed || ExpansionInterface::ImageReadFailed(); }
int ntx_engine_checkpoint_capture(uintptr_t limit, ntx_checkpoint_write_fn writer, void* context)
{
  auto& system = Core::System::GetInstance();
  if (!writer || !limit || Core::GetState(system) != Core::State::Paused || ntx_engine_recovery_required())
    return 0;
  const Core::CPUThreadGuard guard(system);
  if (!ntx_engine_flush_cards())
    return 0;
  // Dolphin is built without C++ exceptions. No exception can unwind over the
  // ABI; allocation is bounded before the engine buffer grows.
  return State::CaptureNtx(system, limit, [writer, context](std::span<const u8> bytes) {
    return writer(context, bytes.data(), bytes.size()) != 0;
  });
}
int ntx_engine_checkpoint_load(uint8_t* bytes, uintptr_t length)
{
  auto& system = Core::System::GetInstance();
  if (!bytes || !length || Core::GetState(system) != Core::State::Paused || ntx_engine_recovery_required())
    return 0;
  const Core::CPUThreadGuard guard(system);
  s_persistence_blocked = true;
  const bool loaded = State::LoadNtx(system, {bytes, length});
  if (!loaded)
  {
    s_checkpoint_failed = true;
    Event(2, "Checkpoint load failed. Restart explicitly with current saves; persistence is blocked until shutdown.");
    return 0;
  }
  s_persistence_blocked = false;
  return 1;
}
void ntx_engine_stop() { Core::Stop(Core::System::GetInstance()); }
void ntx_engine_shutdown()
{
  Core::Shutdown(Core::System::GetInstance());
  MemoryCard::SetWriteHandler(nullptr);
  ExpansionInterface::SetSramWriteHandler(nullptr);
  ExpansionInterface::SetImageReadHandler(nullptr);
  s_card_writer = nullptr;
  s_saved_reader = nullptr;
  s_overlay_hook = {};
  s_present_hook = {};
  Pad::SetInputPollCallback(nullptr);
  s_input_sampler = nullptr;
  ciface::Touch::UnregisterGameCubeInputOverrider(0);
  UICommon::ShutdownControllers();
  UICommon::Shutdown();
  Common::RegisterMsgAlertHandler(nullptr);
  s_event = nullptr;
  s_event_context = nullptr;
  s_ui = nullptr;
  s_platform = nullptr;
}
void ntx_ui_rect(float left, float top, float right, float bottom, uint32_t color)
{ ImGui::GetForegroundDrawList()->AddRectFilled({left, top}, {right, bottom}, color, 12.0f); }
void ntx_ui_circle(float x, float y, float radius, uint32_t color)
{ ImGui::GetForegroundDrawList()->AddCircleFilled({x, y}, radius, color); }
void ntx_ui_text(float x, float y, float size, const char* text, uint32_t color)
{ ImGui::GetForegroundDrawList()->AddText(ImGui::GetFont(), size, {x, y}, color, text); }
