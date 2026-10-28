//! The PCI Express controller's MSI interrupts.
//!
//! A PCI Express device signals a *message-signaled interrupt* (MSI) by
//! writing to an address; the BCM2711's PCIe controller catches the
//! writes and sets a bit, one of 32, in its MSI status register, and
//! raises one GIC interrupt (SPI 148, interrupt ID 180) while any bit is
//! set. To the kernel the controller is a second level of interrupt
//! controller, *cascaded* into that GIC interrupt: 32 more vectors, from
//! 0x200, with their own callouts. The PCI server hands them to drivers.

use oscraft_startup::Startup;
use oscraft_startup::abi::syspage::{
    INTR_FLAG_MSI, INTR_GENFLAG_LOAD_INTRMASK, INTR_GENFLAG_NOGLITCH, IntrgenData, IntrinfoEntry,
};

/// The PCIe controller's MSI registers.
const MSI_BASE: u64 = 0xfd50_0000 + 0x4000;
/// The GIC interrupt the controller raises.
const MSI_CASCADE: u32 = 160 + 20;
/// The kernel's vectors for the 32 MSIs.
const MSI_VECTOR_BASE: u32 = 0x200;

// ANCHOR: msi_callouts
// The register offsets: EOI 0x060, STATUS 0x500, CLEAR 0x508,
// MASK_STATUS 0x50c, MASK_SET 0x510, MASK_CLEAR 0x514. Each callout is
// patched with their virtual address.
//
// id: the lowest pending, unmasked MSI, in x19 (-1 if none); it is masked
// and its status cleared, so it does not fire again until eoi.
oscraft_startup::callout!(pub static ID, "bcm2711_msi_id", patches = 1,
    "mov     x7, #0", "movk x7, #0, lsl #16", "movk x7, #0, lsl #32", "movk x7, #0, lsl #48",
    "mov     x19, #-1",
    "ldr     w0, [x7, #0x50c]",     // MASK_STATUS
    "ldr     w1, [x7, #0x500]",     // STATUS
    "bics    w1, w1, w0",
    "cbz     w1, 0f",
    "rbit    w2, w1",
    "clz     w1, w2",               // the lowest set bit
    "mov     w19, w1",
    "mov     w2, #1",
    "lsl     w1, w2, w19",
    "str     w1, [x7, #0x510]",     // MASK_SET
    "str     w1, [x7, #0x508]",     // CLEAR
    "0:",
);

// eoi: unmask the MSI (unless the kernel keeps it masked: x22, the mask
// count, is not 0) and signal the end of the interrupt.
oscraft_startup::callout!(pub static EOI, "bcm2711_msi_eoi", patches = 1,
    "mov     x7, #0", "movk x7, #0, lsl #16", "movk x7, #0, lsl #32", "movk x7, #0, lsl #48",
    "cbnz    x22, 0f",
    "cmp     w19, #32",
    "b.hs    0f",
    "mov     w2, #1",
    "lsl     w1, w2, w19",
    "str     w1, [x7, #0x514]",     // MASK_CLEAR
    "str     w2, [x7, #0x060]",     // EOI
    "0:",
);

// mask/unmask(syspage, vector): set the vector's bit in MASK_SET /
// MASK_CLEAR; -1 for a vector beyond 31.
macro_rules! mask_callout {
    ($name:ident, $sym:literal, $reg:literal) => {
        oscraft_startup::callout!(pub static $name, $sym, patches = 1,
            "mov     x7, #0", "movk x7, #0, lsl #16", "movk x7, #0, lsl #32", "movk x7, #0, lsl #48",
            "mov     x0, #-1",
            "cmp     w1, #32",
            "b.hs    0f",
            "mov     w2, #1",
            "lsl     w1, w2, w1",
            concat!("str     w1, [x7, #", $reg, "]"),
            "mov     x0, #0",
            "0: ret",
        );
    };
}
mask_callout!(MASK, "bcm2711_msi_mask", "0x510");
mask_callout!(UNMASK, "bcm2711_msi_unmask", "0x514");
// ANCHOR_END: msi_callouts

// ANCHOR: init_msi
/// Describe the MSI controller as a third interrupt controller, after the
/// GIC's two ranges. The kernel runs the id callout with glitch
/// protection (`NOGLITCH`), and loads the interrupt's mask count for eoi.
pub fn init_msi(s: &mut Startup) {
    let regs = s.map_device(MSI_BASE, 0x1000);
    let sp = &mut s.syspage;
    let msi = IntrinfoEntry {
        vector_base: MSI_VECTOR_BASE,
        num_vectors: 32,
        cascade_vector: MSI_CASCADE,
        flags: INTR_FLAG_MSI,
        id: IntrgenData {
            genflags: INTR_GENFLAG_NOGLITCH,
            rtn: sp.callout(&ID, &[regs]),
            ..Default::default()
        },
        eoi: IntrgenData {
            genflags: INTR_GENFLAG_LOAD_INTRMASK,
            rtn: sp.callout(&EOI, &[regs]),
            ..Default::default()
        },
        mask: sp.callout(&MASK, &[regs]),
        unmask: sp.callout(&UNMASK, &[regs]),
        ..Default::default()
    };
    sp.intrinfo.push(msi);
}
// ANCHOR_END: init_msi
