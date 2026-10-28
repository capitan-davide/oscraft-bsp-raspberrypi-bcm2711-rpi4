# Interrupts, Timer and Cores

The library provides the GICv2, the generic timer and the SMP protocol;
the board only supplies where things are and how its cores are released.
This chapter covers the BCM2711's part.

## The GIC-400

The BCM2711's interrupt controller is Arm's GIC-400, a GICv2, among the
ARM's local peripherals: its distributor at `0xff841000`, its CPU
interface at `0xff842000`. It has 224 interrupt lines: the 32 SGIs and
PPIs, and 192 SPIs for the SoC's devices. The board passes the two
addresses to `gic_v2::init()` (the library's book,
[Chapter 8](../startup/08-gic.html)); a device's interrupt vector is then
its GIC interrupt ID, such as 125 for the mini-UART (SPI 93), the number
the boot script gives `devc-serminiuart`.

> [!NOTE]
> The BCM2711 also has the older interrupt controller of the earlier
> Raspberry Pis, and an MSI controller for PCI Express, cascaded into the
> GIC. QNX's startup describes the MSI controller as a third `intrinfo`
> entry, for the PCI server; this BSP does not yet (Chapter 9).

## The timer

The system counter runs at 54 MHz on the board and 62.5 MHz in QEMU; the
firmware writes the frequency to `CNTFRQ_EL0`, and the library reads it
there. The system tick is the virtual timer's PPI, 27, as on most AArch64
boards.

## The cores

The four Cortex-A72 cores wait on the firmware's spin table (Chapter 3).
The library calls the board's `release_cpu()` (Listing 7.1) for each in
turn, with the physical address of its entry code.

*Listing 7.1: Releasing a core (`startup-bcm2711-rpi4/src/board.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/board.rs:spin_table}}
```

The write goes straight to memory, since startup runs with the caches off,
and the waiting core reads it with its own caches off. `dsb sy` makes sure
the write is done before `sev` wakes the cores from `wfe`; each core
rereads its word, and the one whose word is now set leaves the loop.

The core then runs the library's code for secondary CPUs (the library's
book, [Chapter 12](../startup/12-smp.html)). On QEMU, `pidin -f bl`
shows the kernel's four idle threads, one on each core, and other threads
moving between them.
