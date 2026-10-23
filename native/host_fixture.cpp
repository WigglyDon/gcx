// SPDX-License-Identifier: GPL-2.0-or-later
// Host-only mechanical access for original Rust regression fixtures. Not packaged.
#include "bridge.h"
#include "Core/Core.h"
#include "Core/System.h"
#include "Core/HW/EXI/EXI.h"
#include "Core/HW/EXI/EXI_Device.h"
#include "Core/PowerPC/PowerPC.h"

extern "C" GCX_EXPORT uint32_t gcx_fixture_card(uint32_t operation, uint32_t value)
{
  auto& system = Core::System::GetInstance();
  const Core::CPUThreadGuard guard(system);
  auto* card = system.GetExpansionInterface().GetDevice(ExpansionInterface::Slot::A);
  if (!card || card->m_device_type != ExpansionInterface::EXIDeviceType::MemoryCard)
    return 0xffffffff;
  switch (operation)
  {
  case 0: card->SetCS(static_cast<int>(value)); return 0;
  case 1: card->ImmWrite(value << 24, 1); return 0;
  case 2: return card->ImmRead(1) >> 24;
  case 3: return card->IsInterruptSet();
  default: return 0xffffffff;
  }
}
extern "C" GCX_EXPORT uint32_t gcx_fixture_gpr4()
{
  auto& system = Core::System::GetInstance();
  const Core::CPUThreadGuard guard(system);
  return system.GetPPCState().gpr[4];
}
