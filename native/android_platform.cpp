// SPDX-License-Identifier: GPL-2.0-or-later
// Mechanical adaptation of Dolphin AndroidCommon's native interface.
// Platform policy and document-provider operations belong to Rust.
#include "jni/AndroidCommon/AndroidCommon.h"
#include "bridge.h"
#include <cstdint>
#include <limits>

extern "C" int64_t gcx_platform_dispatch(uint32_t, const char*, const char*, char*, uintptr_t);
static int64_t Request(uint32_t op, std::string_view path = {}, std::string_view arg = {}, char* out = nullptr, size_t capacity = 0)
{
  return gcx_platform_dispatch(op, std::string(path).c_str(), std::string(arg).c_str(), out, capacity);
}
static std::string ReadText(uint32_t op, std::string_view path)
{
  const int64_t length = Request(op, path);
  if (length <= 0 || length > 1024 * 1024)
    return {};
  std::string value(static_cast<size_t>(length), '\0');
  if (Request(op, path, {}, value.data(), value.size()) != length)
    return {};
  return value;
}
bool IsPathAndroidContent(std::string_view uri) { return uri.starts_with("content://"); }
std::string OpenModeToAndroid(std::string mode)
{
  std::erase(mode, 'b');
  return mode;
}
std::string OpenModeToAndroid(std::ios_base::openmode mode)
{
  if (mode & std::ios_base::out)
    return mode & std::ios_base::in ? "r+" : "w";
  return "r";
}
int OpenAndroidContent(std::string_view uri, std::string_view mode) { return static_cast<int>(Request(1, uri, mode)); }
bool DeleteAndroidContent(std::string_view uri) { return Request(2, uri) > 0; }
jlong GetAndroidContentSizeAndIsDirectory(std::string_view uri) { return Request(3, uri); }
std::string GetAndroidContentDisplayName(std::string_view uri) { return ReadText(4, uri); }
std::vector<std::string> GetAndroidContentChildNames(std::string_view uri)
{
  const std::string names = ReadText(5, uri);
  std::vector<std::string> result;
  size_t start = 0;
  for (size_t i = 0; i < names.size(); ++i)
    if (names[i] == '\0') {
      result.emplace_back(names.substr(start, i - start));
      start = i + 1;
    }
  return result;
}
std::vector<std::string> DoFileSearchAndroidContent(std::string_view uri, std::span<const std::string_view>, bool)
{
  Request(6, uri);
  return {};
}
int GetNetworkIpAddress() { return static_cast<int>(Request(7)); }
int GetNetworkPrefixLength() { return static_cast<int>(Request(8)); }
int GetNetworkGateway() { return static_cast<int>(Request(9)); }
