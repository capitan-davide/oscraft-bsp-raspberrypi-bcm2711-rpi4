//! QNX OS 8.0 startup for the Raspberry Pi 4 (BCM2711).

#![no_std]

mod miniuart;

use oscraft_startup::cmdline::Opt;
use oscraft_startup::{Startup, console, kprintln};

/// Options of this board, besides the common ones: `-W` enables the
/// watchdog.
const BOARD_OPTIONS: &str = "W:";

fn main(s: &mut Startup) -> ! {
    console::set(&miniuart::MINIUART);
    for opt in s.options(BOARD_OPTIONS) {
        match opt {
            Opt::Flag('W', _) => {}
            other => s.handle_common_option(&other),
        }
    }
    kprintln!("startup-bcm2711-rpi4: debug level {}", s.debug_level);
    kprintln!("startup header at {:#x}: {:#x?}", s.shdr_paddr, s.shdr);
    kprintln!("argv {:?}", s.cmdline.argv.as_slice());
    kprintln!("x0-x3 {:#x?}", s.boot_regs);
    oscraft_startup::crash!("not implemented yet");
}

oscraft_startup::entry!(main);
