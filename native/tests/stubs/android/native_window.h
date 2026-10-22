// SPDX-License-Identifier: MIT
// Isolated ownership fixture, never included in the Android build.
#pragma once
#include <atomic>
struct ANativeWindow
{
  std::atomic<int> references{1};
};
void ANativeWindow_acquire(ANativeWindow*);
void ANativeWindow_release(ANativeWindow*);
