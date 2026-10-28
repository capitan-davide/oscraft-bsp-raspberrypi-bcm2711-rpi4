# The Mailbox and the RAM

On the Raspberry Pi, the GPU's firmware owns much of the board's
configuration: how the RAM is split between the GPU and the ARM, the
clocks, the board's identity. Startup asks it through a *mailbox*. This
chapter explains the mailbox, then how the board crate describes the RAM
with what it learns.

## The BCM2711's address map

The ARM cores see the BCM2711 as in Table 5.1 (in the SoC's "low
peripheral" mode, which the firmware selects).

*Table 5.1: The BCM2711's physical address map, as the ARM sees it.*

| Range | Contents |
|---|---|
| `0x0_0000_0000` - `0x0_3fff_ffff` | RAM: the ARM's, then at its top the GPU's |
| `0x0_4000_0000` - `0x0_fbff_ffff` | RAM, on boards with more than 1 GB |
| `0x0_fc00_0000` - `0x0_ff7f_ffff` | The main peripherals: UARTs, mailbox, GPIO, the power manager |
| `0x0_ff80_0000` - `0x0_ffff_ffff` | The ARM's local peripherals, including the GIC-400 |
| `0x1_0000_0000` and up | RAM above 4 GB, on 8 GB boards |

The peripherals sit in the top 64 MB below 4 GB, so a board with 4 GB or
more has RAM in two parts, around them.

## The mailbox

The mailbox is a pair of hardware FIFOs between the ARM and the GPU, at
`0xfe00b880`. The ARM talks to the firmware on its *property channel*
(8) in messages: a buffer in RAM that holds a list of *tags*, each a
request with room for the answer. The ARM writes the buffer's address,
with the channel number in its low 4 bits (so the buffer must be 16-byte
aligned), into mailbox 1; the firmware answers in the same buffer and
posts the address back in mailbox 0.

*Listing 5.1: A mailbox call (`startup-bcm2711-rpi4/src/mbox.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/mbox.rs:mbox_call}}
```

Startup runs with the caches off, so the GPU sees the message as the ARM
wrote it, and the ARM sees the answer, with no cache maintenance. The
buffer is in startup's own data, below 1 GB, where the GPU can reach it.

The board crate asks for five things:

*Table 5.2: What startup asks the firmware.*

| Tag | Answer | Used for |
|---|---|---|
| `0x00010002` board revision | A 32-bit code | RAM size, board name, maker, SoC |
| `0x00010004` board serial | 64 bits | `_CS_HW_SERIAL` |
| `0x00010006` VC memory | Base and size of the GPU's RAM | Reserved, named `vcram` |
| `0x00030002` clock rate, ARM | Hz | The CPUs' speed in cpuinfo |
| `0x00010003` MAC address | 6 bytes | The Ethernet controller's hwinfo (Chapter 8) |

## The board revision

The revision code describes the board in bit fields:
`NOQuuuWuFMMMCCCCPPPPTTTTTTTTRRRR`, where `MMM` is the RAM size (from 256 MB
to 8 GB), `CCCC` the maker, `PPPP` the processor and `TTTTTTTT` the board
type (0x11 for a Pi 4 B). `revision.rs` decodes it, with unit tests; QEMU's
2 GB `raspi4b` reports `0xb03115`.

## The RAM

With the revision's RAM size and the GPU's share from the mailbox,
`init_ram()` (Listing 5.2) describes the RAM as QNX's own BSP does.

*Listing 5.2: Describing the RAM, `init_ram()` (`startup-bcm2711-rpi4/src/board.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/board.rs:init_ram}}
```

- The RAM is one range, or two around the peripherals on 4 and 8 GB
  boards.
- The first page holds armstub8 and the spin table, which the other
  cores read until startup releases them (Chapter 3): it is reserved.
- The GPU's RAM is reserved too, and named `vcram`.
- `below1G` names the RAM below the GPU's. The BCM2711's older DMA
  engines, and the SD card controller's, only reach the first gigabyte;
  drivers ask for memory there by this name (QNX's SD card driver takes
  `mem name=below1G`).

The library then turns what is free into `sysram` (the library's book,
[Chapter 5](../startup/05-ram.html)). On QEMU's 2 GB board the kernel
reports 1952 MB free of 2048, the same as with QNX's startup.
