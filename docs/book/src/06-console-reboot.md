# Console, Reboot and Watchdog

Two of the board's devices serve both startup and the kernel: the
mini-UART, which is the console, and the power manager's watchdog, which
can reset the board. This chapter shows how each is used by startup, and
handed to the kernel as callouts.

## The mini-UART

The BCM2711 has five PL011 UARTs and one simpler *mini-UART* (UART1),
part of its "auxiliary" peripherals. With `enable_uart=1`, the firmware
routes the mini-UART to GPIO 14 and 15, the console pins of the
Raspberry Pi's header, and sets it to 115200 baud. Startup does not
configure it again; it only sends bytes, polling the line status register
until the transmitter has room. That is the `DebugDevice` the console uses
(`startup-bcm2711-rpi4/src/miniuart.rs`).

> [!WARNING]
> The mini-UART's baud rate is derived from the GPU's core clock. If the
> clock changes, so does the baud rate: `force_turbo=1` in `config.txt`
> keeps it fixed. The serial driver the boot script starts,
> `devc-serminiuart`, is told the clock (`-c500000000`).

## The kernel's debug channel

The kernel prints its own messages (with `procnto -v`, and when it
crashes) through the `debug` callouts of the system page. They do in
assembly what startup's driver does in Rust, at the mini-UART's virtual
address, which the board maps with `map_device()` and patches in (the
library's book, [Chapter 7](../startup/07-callouts.html)).

*Listing 6.1: The mini-UART's callouts (`startup-bcm2711-rpi4/src/miniuart.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/miniuart.rs:miniuart_callouts}}
```

`display_char` waits for room, writes the character, and then waits for
the transmitter to be idle: the kernel may print just before it resets
the board, and the character must leave first. `poll_key` returns a
received byte or -1. The mini-UART cannot detect a break, so
`break_detect` returns 0.

## The power manager and its watchdog

The BCM2711's power manager (PM, at `0xfe100000`) has a watchdog: a
counter that counts down in ticks of about 15 µs (65,536 Hz) and, when it
expires, resets what `PM_RSTC` says. Its registers only accept writes
with a password, 0x5a, in the top byte.

With `-W`, startup starts the watchdog (Listing 6.2), and something in the
running system must reload it in time: QNX's `wdtkick` utility does, given
the PM's registers and values. QNX's BSP starts it in the boot script when
startup has `-W`.

*Listing 6.2: Starting the watchdog, `watchdog_enable()` (`startup-bcm2711-rpi4/src/pm.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/pm.rs:watchdog}}
```

## Reboot

The same watchdog resets the board for the kernel: the `reboot` callout
(Listing 6.3) sets it to 10 ticks, asks for a full reset, and waits.

*Listing 6.3: The reboot callout (`startup-bcm2711-rpi4/src/pm.rs`).*

```rust
{{#include ../../../startup-bcm2711-rpi4/src/pm.rs:reboot_callout}}
```

QNX's reboot callout keeps the PM's address in a read/write area of the
system page, which its patcher fills in. Patching the address into the
code, as here, does the same with the library's single patching
mechanism.

> [!NOTE]
> QEMU's model of the BCM2711's watchdog resets the machine as soon as
> the watchdog is set up, instead of counting down. So `-W` makes an
> image reset at once in QEMU, and is only for the board.
