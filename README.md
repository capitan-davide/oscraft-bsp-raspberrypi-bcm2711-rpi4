# OsCraft BSP for the Raspberry Pi 4 (BCM2711)

A board support package for the Raspberry Pi 4, compatible with QNX OS
8.0, with a startup program written in Rust. QNX's unmodified kernel,
`procnto-smp-instr`, comes up on all four cores and runs the boot script
to a shell.

- `startup-bcm2711-rpi4/`: the board crate, which builds
  `startup-bcm2711-rpi4`. It uses the startup library,
  [oscraft-startup](https://github.com/capitan-davide/oscraft-startup), a
  Rust counterpart of QNX's `libstartup.a`, as a git dependency;
  `make OSCRAFT_STARTUP=<dir> ...` uses a local checkout of it instead.
- `images/rpi4.build`: the boot image.
- `tools/qemu/`: booting and testing the image in QEMU's `raspi4b`.
- `docs/book/`: the BSP's book.

## Building

You need the QNX SDP 8.0 with the Raspberry Pi 4 BSP package (from the
QNX Software Center), Rust (the toolchain in `rust-toolchain.toml`),
`clang` and `ld.lld`, QEMU 9.0 or later, and mdBook for the book. From
bash:

```text
source <sdp>/qnxsdp-env.sh
make image          # build/rpi4.ifs
make run            # boot it in QEMU; Ctrl-A X quits
make test           # unit tests, then a boot in QEMU on 4 CPUs
make qnx-reference  # the same image with QNX's own startup
make book           # the book, in build/book/
```

The book explains everything else. It is published at
<https://capitan-davide.github.io/oscraft-bsp-raspberrypi-bcm2711-rpi4/>,
and the startup library's at
<https://capitan-davide.github.io/oscraft-startup/>; `make book` builds
the BSP's locally, into `build/book/`.

## Status

Tested in QEMU, where the system page matches QNX's own startup's (the
book's "Matching QNX's Startup" chapter lists the small differences).
Not yet run on the board.

## For OsCraft

OsCraft's build takes this board from a checkout of this repository
(`make TARGET=rpi4 OSCRAFT_BSP=<dir>` in OsCraft's tree), which has the
shape every board out of OsCraft's tree has:

- `oscraft/board.build`: the board's part of OsCraft's image (the image's
  address, the startup, the board's modules), before OsCraft's
  `images/base.build`;
- `oscraft/qnx-board.build`: the board's part of the QNX personality's
  overlay (the console's driver and the session);
- `pcid/*.c`: the board's code in OsCraft's pcid module, built with
  FreeBSD's drivers for the board's PCIe bridge and mailbox
  (`bcm2711_pci.ko`).

The startup library is OsCraft's (`startup/` in its repository), a git
dependency pinned in `startup-bcm2711-rpi4/Cargo.toml`.
