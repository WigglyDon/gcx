// SPDX-License-Identifier: GPL-2.0-or-later
// Host-only mechanical access for original Rust regression fixtures. Not packaged.
#include "bridge.h"
#include "Core/Core.h"
#include "Core/System.h"
#include "Core/HW/EXI/EXI.h"
#include "Core/HW/EXI/EXI_Device.h"
#include "Core/HW/SI/SI_DeviceGCController.h"
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

extern "C" GCX_EXPORT uint64_t gcx_fixture_si_request(uint32_t operation)
{
  auto& system = Core::System::GetInstance();
  const Core::CPUThreadGuard guard(system);
  SerialInterface::CSIDevice_GCController controller(
      system, SerialInterface::SIDevices::SIDEVICE_GC_CONTROLLER, 0);
  u32 high = 0, low = 0;
  int size = -1;
  if (operation == 3)
  {
    if (controller.GetData(high, low) == SerialInterface::DataResponse::Success)
      size = 8;
  }
  else if (operation <= 2)
  {
    // Status/origin probe and actual direct data request, through the donor's
    // real SI command path rather than a second implementation of its mapping.
    u8 buffer[64]{};
    buffer[0] = operation == 0 ? 0x00 : operation == 1 ? 0x41 : 0x40;
    size = controller.RunBuffer(buffer, operation == 2 ? 3 : 1);
    for (int i = 0; i < 4; ++i)
      high = (high << 8) | buffer[i];
  }
  return (static_cast<uint64_t>(static_cast<uint32_t>(size)) << 32) | high;
}
