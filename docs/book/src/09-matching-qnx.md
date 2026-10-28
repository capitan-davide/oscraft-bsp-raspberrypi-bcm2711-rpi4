# Matching QNX's Startup

The BSP's startup has a reference: QNX's own startup for the same board,
in the Raspberry Pi 4 BSP package. This chapter shows how the two are
compared, what the comparison found, and what is still missing.

## The method

`make qnx-reference` builds the image of `images/rpi4.build` with QNX's
startup instead of ours: the only difference between the two images is
the startup program. Both boot in QEMU. Three comparisons follow:

1. **The boot.** Both reach the shell, and `make qemu-test` checks both
   the same way (`QEMU_TEST_IMAGE=build/qnx-reference.ifs`).
2. **Startup's own report.** With `-vvvv`, QNX's startup prints its view
   of the board and the whole system page before it starts the kernel.
3. **The kernel's view.** `pidin syspage` on the running system prints
   every section of the system page as the kernel has it. A `diff` of the
   two outputs, with addresses masked, shows every difference in the
   contract.

## The results

*Table 9.1: Our system page compared with QNX's startup's, on QEMU.*

| Section | Result |
|---|---|
| `cpuinfo` | The same: CPU ID, speed, flags (`0xc0f00c7a`), affinity, caches |
| `cacheattr` | The same: 48 KB L1 instruction, 32 KB L1 data, 64-byte lines |
| `idreg_dump`, `gic_map` | The same |
| `qtime` | The same frequency, period, scale and interrupt; ours adds `QTIME_FLAG_GLOBAL_CLOCKCYCLES`, which the QNX startup specification requires |
| `asinfo` | The same tree; the free RAM is the same to the kernel (1952 MB of 2048) |
| `hwinfo` | The same entries and tags |
| `typed_strings` | The same, but for the serial number: QEMU provides none, and ours reports zeros where QNX's startup reports the board revision code |
| `intrinfo` | The same three entries: the GIC's two and the PCI Express MSI controller's; our MSI `id` callout is 8 bytes shorter (two instructions of a bounds check that cannot fail) |
| `callout`, `smp` | Different addresses, the same roles |
| `gicr_map` | Zeros; QNX's has leftover values (it is for GICv3 only) |
| `TCR_EL1` | The same, `0x14b5183519` |

The sections are in a different order in memory, which the kernel does
not depend on.

## What is missing

- **Hypervisor support and compressed images**, which QNX's startup
  library has and the BSP does not use.
- **The board itself.** Everything above was checked in QEMU. QEMU models
  the BCM2711's peripherals that startup touches, but not their timing or
  every detail. The reboot and the watchdog, UART3 (`-D pl011-3`, which
  has no serial port in QEMU's `raspi4b`), the MSI controller under a real
  PCI device, and
  `-u arg` under U-Boot can only be checked on the board.

## Checked in QEMU besides the comparison

- `-D pl011` moves startup's output and the kernel's to UART0, QEMU's
  first serial port; an unknown `-D` name keeps the default, with a
  message.
- `-u arg` without U-Boot's arguments reports that it finds no device
  tree address, and the boot goes on without the device tree.

> [!WARNING]
> When the BSP runs on the board, check the `display_char` callout's
> output first (`procnto -v` prints through it): a wrong mini-UART address
> or clock shows as silence after "Starting next program".
