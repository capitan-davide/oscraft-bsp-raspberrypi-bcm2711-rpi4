/*
 * fwstub - stand-in for the Raspberry Pi 4 firmware + armstub8 under QEMU.
 *
 * QEMU's raspi4b machine, given an ELF with -kernel, starts every core at the
 * ELF entry in EL3 and passes nothing in x0. The real board enters the kernel
 * image differently: the VideoCore firmware loads the image at 0x80000 and the
 * DTB somewhere in RAM, then armstub8 (at physical 0x0) drops every core to
 * non-secure EL2, enters the image on core 0 with x0 = DTB address, and parks
 * cores 1-3 on a spin table at 0xd8 + 8 * cpu (the DTB's cpu-release-addr).
 *
 * This stub reproduces that hand-off so the same image boots unchanged in QEMU
 * and on hardware. The image and DTB themselves are placed by QEMU's generic
 * loader device (see tools/qemu/run.sh).
 */

#ifndef KERNEL_ADDR
#define KERNEL_ADDR     0x80000
#endif
#ifndef DTB_ADDR
#define DTB_ADDR        0x1f000000
#endif

#define GICD_BASE       0xff841000
#define GICD_CTLR       0x000
#define GICD_TYPER      0x004
#define GICD_TYPER_SECURITYEXTN_BIT 10
#define GICD_IGROUPR    0x080
#define GICD_NUM_IGROUPR 7          /* 224 interrupt lines on the BCM2711 */
#define GICC_BASE       0xff842000
#define GICC_CTLR       0x000
#define GICC_PMR        0x004

/* SCR_EL3: RW (EL2 is AArch64), HCE (HVC allowed), SMD (no SMC), RES1, NS */
#define SCR_EL3_VAL     0x5b1
/* SPSR_EL3: return to EL2h with D, A, I, F masked */
#define SPSR_EL2H       0x3c9

    .section .text.boot, "ax"
    .global _start
_start:
    b       el3_entry

    /* Spin table, at the same address as armstub8's. */
    .org    0xd8
    .global spin_table
spin_table:
    .quad   0, 0, 0, 0

    .org    0x100
el3_entry:
    mrs     x5, mpidr_el1
    and     x5, x5, #0xff

    /*
     * GIC: like armstub8, hand all interrupts to the non-secure world
     * (group 1). Only meaningful if the GIC has the security extensions;
     * QEMU's raspi4b GIC does not, and there group 1 would stay disabled
     * because the non-secure software only enables group 0.
     */
    ldr     x1, =GICD_BASE
    ldr     w0, [x1, #GICD_TYPER]
    tbz     w0, #GICD_TYPER_SECURITYEXTN_BIT, 3f
    /* The distributor is shared, so core 0 sets it up once. */
    cbnz    x5, 2f
    mov     w0, #3
    str     w0, [x1, #GICD_CTLR]
    mov     w0, #-1
    mov     x2, #1
1:  add     x3, x1, #GICD_IGROUPR
    str     w0, [x3, x2, lsl #2]
    add     x2, x2, #1
    cmp     x2, #GICD_NUM_IGROUPR
    b.lo    1b

    /* Banked per-core state: SGIs/PPIs to group 1, open the CPU interface. */
2:  mov     w0, #-1
    str     w0, [x1, #GICD_IGROUPR]
    ldr     x1, =GICC_BASE
    mov     w0, #0x1e7
    str     w0, [x1, #GICC_CTLR]
    mov     w0, #0xff
    str     w0, [x1, #GICC_PMR]

3:  /* Drop to non-secure EL2. */
    ldr     x0, =SCR_EL3_VAL
    msr     scr_el3, x0
    mov     x0, #SPSR_EL2H
    msr     spsr_el3, x0
    adr     x0, el2_entry
    msr     elr_el3, x0
    eret

el2_entry:
    mrs     x5, mpidr_el1
    and     x5, x5, #0xff
    cbz     x5, primary

    /* Cores 1-3: wait until the kernel writes our release address. */
    adr     x6, spin_table
    add     x6, x6, x5, lsl #3
3:  wfe
    ldr     x4, [x6]
    cbz     x4, 3b
    mov     x0, xzr
    b       enter

primary:
    ldr     x0, =DTB_ADDR
    ldr     x4, =KERNEL_ADDR
enter:
    mov     x1, xzr
    mov     x2, xzr
    mov     x3, xzr
    br      x4

    .ltorg
