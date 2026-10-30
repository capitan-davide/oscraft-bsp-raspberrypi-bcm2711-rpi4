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
[Chapter 8](../startup/08-gic.html)), which also registers the GIC's
part of each core's setup: `init_boot_cpu()` runs it on the boot core,
and each other core when it starts. A device's interrupt vector is then
its GIC interrupt ID, such as 125 for the mini-UART (SPI 93), the number
the boot script gives `devc-serminiuart`.

> [!NOTE]
> The BCM2711 also has the older interrupt controller of the earlier
> Raspberry Pis. Like QNX's startup, this one does not describe it: the
> GIC handles all the interrupts.

## The PCI Express MSIs

The Raspberry Pi 4 connects its USB controller, a VL805, over PCI
Express, so USB depends on the PCI server, and the PCI server on
*message-signaled interrupts* (MSIs). A PCI Express device does not have
an interrupt line: it raises an interrupt by writing to an address. The
BCM2711's PCIe controller catches those writes, sets one of 32 bits in
its MSI status register, and raises one GIC interrupt, ID 180, while any
is set.

To the kernel, the controller is a second interrupt controller,
*cascaded* into the GIC: when interrupt 180 arrives, the kernel asks the
MSI controller's `id` callout which MSI it is, and handles that one. The
board describes it as a third `intrinfo` entry (Listing 7.1): 32 vectors
from 0x200, `cascade_vector` 180, and the `INTR_FLAG_MSI` flag, which tells
the PCI server these vectors are for MSIs.

*Listing 7.1: The MSI controller, `init_msi()` (`startup-bcm2711-rpi4/src/msi.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/msi.rs:init_msi}}
```

The callouts (Listing 7.2) are ported from QNX's BSP. `id` takes the
lowest pending MSI that is not masked, masks it and clears its status
bit, so that a level-like interrupt does not fire again while the driver
handles it; `eoi` unmasks it, unless the kernel keeps it masked (a
non-zero mask count in `x22`, which `INTR_GENFLAG_LOAD_INTRMASK` asks the
kernel to load), and signals the controller's end of interrupt.
`INTR_GENFLAG_NOGLITCH` tells the kernel not to add its own check that the
controller really raised the interrupt: the MSI controller only signals
the GIC when an MSI is pending.

*Listing 7.2: The MSI callouts (`startup-bcm2711-rpi4/src/msi.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/msi.rs:msi_callouts}}
```

## The timer

The system counter runs at 54 MHz on the board and 62.5 MHz in QEMU; the
firmware writes the frequency to `CNTFRQ_EL0`, and the library reads it
there. The system tick is the virtual timer's PPI, 27, as on most AArch64
boards.

## The cores

The four Cortex-A72 cores wait on the firmware's spin table (Chapter 3).
The library calls the board's `release_cpu()` (Listing 7.3) for each in
turn, with the physical address of its entry code.

*Listing 7.3: Releasing a core (`startup-bcm2711-rpi4/src/board.rs`).*

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
