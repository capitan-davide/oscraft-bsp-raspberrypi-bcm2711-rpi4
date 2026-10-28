//! The board's devices in hwinfo, for the drivers' defaults.
//!
//! The same devices as QNX's RPi4 BSP describes: the Ethernet controller
//! (GENET, with its MAC address, which `io-sock`'s driver takes from
//! here), the watchdog (for `wdtkick`) and the SD card controller's DMA
//! window (for `devb-sdmmc-bcm2711`).

use crate::{mbox, pm};
use oscraft_startup::Startup;
use oscraft_startup::fdt::{self, Fdt};
use oscraft_startup::syspage::hwinfo::{self, HWI_NULL_OFF};

/// GENET: registers and its two interrupts (SPI 157 and 158).
const GENET_BASE: u64 = 0xfd58_0000;
const GENET_SIZE: u32 = 0x8000;
const GENET_IRQS: [u32; 2] = [189, 190];
/// The PM block's register range.
const PM_SIZE: u32 = 0x114;

// ANCHOR: init_hwinfo
pub fn init_hwinfo(s: &mut Startup) {
    let fdt = s.fdt;
    let sp = &mut s.syspage;
    let (hw, st) = (&mut sp.hwinfo, &mut sp.strings);
    let bus = hwinfo::add_default(hw, st);
    let mem = sp.asinfo.find(st, "memory").unwrap_or(0);
    // Devices on the "internal" bus hang off the root; the others off the
    // unknown bus. The pnpid values are the BSP's.
    hw.add_item(st, "Device", "genet", 0, &2u32.to_le_bytes());
    for irq in GENET_IRQS {
        hw.add_location(st, GENET_BASE, GENET_SIZE, 0, mem);
        hw.add_irq(st, irq);
    }
    hw.add_nicaddr(st, &mbox::mac_address());

    hw.add_item(st, "Device", "wdog", bus, &7u32.to_le_bytes());
    hw.add_location(st, pm::BASE, PM_SIZE, 0, mem);
    hw.add_inputclk(st, 500, 1);
    hw.add_item(st, "Device", "wdt,options", bus, &0u32.to_le_bytes());
    hw.add_regname(st, "write_width", 32);
    hw.add_regname(st, "enable_width", 32);

    // The SD card controller reaches RAM through its bus's dma-ranges.
    // SAFETY: the device tree is where -u said, and startup reserved it.
    if let Some(dt) = fdt.and_then(|(addr, _)| unsafe { Fdt::at(addr) })
        && let Some((base, size)) = sdmmc_dma(&dt)
    {
        hw.add_item(st, "Device", "bcm2711-sdmmc", bus, &4u32.to_le_bytes());
        hw.add_location(st, base, size, 0, HWI_NULL_OFF);
    }
    hw.finish();
}
// ANCHOR_END: init_hwinfo

/// The bus address and size of the SD card controller's DMA window: the
/// first range of its parent bus's `dma-ranges` (child address, parent
/// address, size, in 2 + 2 + 1 cells).
fn sdmmc_dma(dt: &Fdt) -> Option<(u64, u32)> {
    let mmc = dt.find_compatible("brcm,bcm2711-emmc2")?;
    let ranges = dt.property(dt.parent(mmc)?, "dma-ranges")?;
    let mut cells = fdt::cells(ranges);
    let mut c = [0u32; 5];
    for cell in &mut c {
        *cell = cells.next()?;
    }
    Some((u64::from(c[0]) << 32 | u64::from(c[1]), c[4]))
}
