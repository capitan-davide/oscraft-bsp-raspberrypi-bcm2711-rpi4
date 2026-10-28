# Building, Running and Testing

This chapter shows how to build the startup and the boot image, boot it
in QEMU, and run the tests.

## What you need

- **The QNX SDP 8.0**, with the Raspberry Pi 4 BSP package installed
  through the QNX Software Center. Source its environment script before
  running `make`, from bash:

  ```text
  source <sdp>/qnxsdp-env.sh
  ```

  It sets `QNX_HOST` and `QNX_TARGET` and puts `mkifs` and the SDP's
  compilers on the `PATH`. The Makefile takes everything from there, and
  finds the BSP package's zip in the SDP's `bsp/` directory; `QNX_BSP_ZIP`
  overrides it.
- **Rust**: `rust-toolchain.toml` pins the version and the target,
  `aarch64-unknown-none-softfloat`, which `rustup` installs on first use.
- **`clang` and `ld.lld`**: `ld.lld` links the startup (below), `clang`
  assembles the QEMU firmware stub.
- **QEMU** with the `raspi4b` machine (QEMU 9.0 or later).
- **mdBook**, for the books.

## The Makefile

*Table 2.1: The Makefile's targets.*

| Target | What it does |
|---|---|
| `make startup` | Builds the board crate and links the startup program, `build/startup/startup-bcm2711-rpi4` |
| `make image` | Builds the boot image, `build/rpi4.ifs` |
| `make run` | Boots the image in QEMU, the console on the terminal |
| `make test` | Runs the unit tests, then boots the image in QEMU and checks it |
| `make qnx-reference` | Builds the same image with QNX's startup, `build/qnx-reference.ifs` |
| `make qnx-reference-run` | Boots it in QEMU |
| `make book` | Checks and builds both books into `build/book/` |
| `make abi-layout` | Regenerates the library's ABI layout table from the SDP's headers |

`STARTUP_OPTS` sets the startup's options in the image, for example
`make run STARTUP_OPTS="-vvv -D miniuart -u reg"` for more output.
`PROFILE=dev` builds the startup without optimizations.

## Building the startup

QNX's `mkifs` does not take an executable for startup: it takes a
*relocatable object* and links it itself, at the address where startup
lands in the image (the library's book,
[Chapter 2](../startup/02-image.html)). The Makefile makes one from the
board crate, which is built as a static library (Listing 2.1).

*Listing 2.1: Building the startup program (`Makefile`).*

```makefile
{{#include ../../../Makefile:make_startup}}
```

## Building the image

`images/rpi4.build` says what goes into the boot image. Listing 2.2 is its
bootstrap section: the startup, with options the Makefile fills in, and
the kernel.

*Listing 2.2: The image's bootstrap section (`images/rpi4.build`).*

```text
{{#include ../../../images/rpi4.build:bootstrap}}
```

The boot script then starts the serial driver for the mini-UART, the pipe
manager, and a shell; the files section lists the libraries and
utilities they need. mkifs looks for every file along `MKIFS_PATH`, which
the Makefile builds from three places, in order: the directory of the
startup it wants (ours, or none for the reference image), the BSP's
prebuilt binaries, the SDP's target tree. Since the first directory that
has a file wins, the same build file makes both images.

## Running in QEMU

QEMU's `raspi4b` machine models the BCM2711, but it does not run the
Raspberry Pi's firmware. A small firmware stub stands in for it (Chapter
3), and `tools/qemu/run.sh` starts QEMU with the stub, the image and the
Raspberry Pi's device tree (Listing 2.3).

*Listing 2.3: Starting QEMU (`tools/qemu/run.sh`).*

```bash
{{#include ../../../tools/qemu/run.sh:qemu_exec}}
```

The mini-UART, the console, is on the terminal; `Ctrl-A X` quits QEMU.
A boot with `-v` looks like this:

```text
RaspberryPi4B (revision 0xb03115), 4 CPUs at 700 MHz
RAM: 2048 MB, VideoCore 0x3c000000-0x40000000
MMU: TCR_EL1 0x14b5183519, L0 tables at 0xffffff8040200000
GICv2: 224 interrupts, SPIs to CPU interface 0
Loaded /proc/boot/procnto-smp-instr, entry 0xffffff8060072c80
System page at phys:0x9000 user:0xffffff8040209000 kern:0xffffff8040206000
Starting next program at 0xffffff8060072c80
Welcome to QNX on the Raspberry Pi 4
QNX 8.0.0 on RaspberryPi4B, 4 CPUs
#
```

## Testing

`make test` runs two kinds of test (Listing 2.4).

*Listing 2.4: The tests (`Makefile`).*

```makefile
{{#include ../../../Makefile:make_test}}
```

The **unit tests** run on the host: the library's (its binary interface
against QNX's headers, its parsers and builders; see the library's book,
[Chapter 15](../startup/15-testing.html)) and the board crate's (decoding
the board revision).

The **QEMU test**, `tools/qemu/test.py`, boots the image and checks the
boot script's banner, the shell prompt, that `pidin info` lists four
processors, that the kernel's idle threads run on all four, and that
`uname -m` reports the board. With `QEMU_TEST_IMAGE=build/qnx-reference.ifs`
it checks QNX's own startup the same way.

## The books

`make book` checks both books before it builds them. Each book has a
checker, `check.py`, which makes sure that every listing still names an
existing anchor in the source, that every anchor is used, that every
source file is in the book's source map, and that each book includes code
only from its own tree. This book's tree is the repository without the
library (Listing 2.5); the library's book's is the library's directory,
so that it can move with the library.

*Listing 2.5: This book's checker configuration (`docs/book/check.py`).*

```python
{{#include ../check.py:check_config}}
```

