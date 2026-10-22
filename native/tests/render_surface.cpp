// SPDX-License-Identifier: MIT
// Exercise the actual patched surface owner with a framework-like JNI requirement.
#include "Common/WindowSystemInfo.h"
#include <cassert>
#include <thread>

static thread_local bool attached = false;
static std::atomic<int> preparations{0};
static std::atomic<int> releases{0};
static std::atomic<int> unsafe_releases{0};
extern "C" void gcx_prepare_render_thread()
{
  attached = true;
  ++preparations;
}
void ANativeWindow_acquire(ANativeWindow* window) { ++window->references; }
void ANativeWindow_release(ANativeWindow* window)
{
  if (!attached)
    ++unsafe_releases;
  --window->references;
  ++releases;
}
int main()
{
  ANativeWindow window;
  assert(!AcquireRenderSurface(WindowSystemType::Headless, &window));
  assert(!AcquireRenderSurface(WindowSystemType::Android, nullptr));
  auto queued = AcquireRenderSurface(WindowSystemType::Android, &window);
  auto renderer = queued;
  assert(window.references == 2);
  queued.reset();
  assert(releases == 0);
  // Stand in for Android relinquishing its own reference before native teardown.
  --window.references;
  std::thread video([owner = std::move(renderer)]() mutable {
    assert(!attached);
    owner.reset();
  });
  video.join();
  assert(window.references == 0);
  assert(preparations == 1 && releases == 1 && unsafe_releases == 0);
}
