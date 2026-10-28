# Devices in hwinfo

The system page's `hwinfo` section tells drivers about the board's
devices, so that they can start without board-specific options (the
library's book, [Chapter 6](../startup/06-system-page.html)). This
chapter shows what the board crate puts there: the same devices as QNX's
own BSP.

## The devices

*Table 8.1: The devices in hwinfo.*

| Device | Tags | Read by |
|---|---|---|
| `genet` | registers (`0xfd580000`, 32 KB), interrupts 189 and 190, MAC address | the Ethernet driver of `io-sock` |
| `wdog` | registers (the PM's), a 500 Hz input clock | `wdtkick` |
| `wdt,options` | register widths: `write_width`, `enable_width` (32) | `wdtkick` |
| `bcm2711-sdmmc` | the SD card controller's DMA window | `devb-sdmmc-bcm2711` |

The Ethernet controller's MAC address is programmed in the board's OTP
memory; the firmware returns it through the mailbox (Chapter 5).

The SD card controller (EMMC2) reaches RAM through a window in its bus's
address space: its bus maps bus addresses `0xc0000000` and up to physical
addresses 0 and up, for 1 GB. The device tree says so in the `dma-ranges`
property of the controller's parent bus, and the board crate reads it
there with the library's device tree reader; without a device tree, the
device is left out.

*Listing 8.1: The devices, `init_hwinfo()` (`startup-bcm2711-rpi4/src/hwinfo.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/hwinfo.rs:init_hwinfo}}
```

The `pnpid` values (2, 7, 0, 4), the order of the tags (a location before
each interrupt) and the owners (the Ethernet controller under the root,
the others under the bus `unknown`) are those of QNX's BSP, so that the
section reads the same to the drivers: Chapter 9 compares the two entry
by entry.
