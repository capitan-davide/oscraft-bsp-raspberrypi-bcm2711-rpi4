# The Board Crate

`startup-bcm2711-rpi4` is the Raspberry Pi 4's part of startup. This
chapter gives an overview of it: its modules, and its `main()`, which the
following chapters take apart.

## Modules

*Table 4.1: The board crate's modules.*

| Module | Contents | Chapter |
|---|---|---|
| `startup-bcm2711-rpi4/src/board.rs` | `main()`, the RAM layout, the spin table | 4, 5, 7 |
| `startup-bcm2711-rpi4/src/mbox.rs` | The VideoCore mailbox | 5 |
| `startup-bcm2711-rpi4/src/revision.rs` | Decoding the board revision | 5 |
| `startup-bcm2711-rpi4/src/miniuart.rs` | The mini-UART: startup's console, the kernel's debug callouts | 6 |
| `startup-bcm2711-rpi4/src/pm.rs` | The watchdog and the reboot callout | 6 |
| `startup-bcm2711-rpi4/src/hwinfo.rs` | The devices in hwinfo | 8 |
| `startup-bcm2711-rpi4/src/lib.rs` | The crate root | 4 |

The crate is a `no_std` static library for AArch64; its modules that touch
hardware are only built for AArch64, so that the rest
(`revision.rs`) can be unit-tested on the host.

## `main()`

Listing 4.1 is the whole of the board's `main()`. It follows the order the
library requires (the library's book,
[Chapter 14](../startup/14-board-crate.html)), with the board's facts at
each step.

*Listing 4.1: The board's `main()` (`startup-bcm2711-rpi4/src/board.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/board.rs:board_main}}
```

In order:

1. **The console** is the mini-UART, which the firmware has set up
   (Chapter 6).
2. **The options**: the library's, plus `-W`, which starts the watchdog
   (Chapter 6).
3. **The board revision and the ARM clock** come from the GPU's firmware,
   through the mailbox (Chapter 5).
4. **Four CPUs**, released through the spin table (Chapter 7).
5. **The RAM**, from the board revision and the mailbox (Chapter 5).
6. **The page tables**, by the library.
7. **The GIC-400** at its addresses, and the **timer**'s PPI (Chapter 7).
8. **The boot CPU**, at the ARM clock's speed.
9. **The kernel's debug channel and reboot**: the mini-UART's and the
   watchdog's callouts (Chapter 6).
10. **The strings** `uname` and `confstr()` report: the board, its maker,
    the SoC, the serial number.
11. **The devices** for the drivers (Chapter 8).
12. **`boot()`**: the library loads the kernel, starts the CPUs, writes the
    system page and jumps.
