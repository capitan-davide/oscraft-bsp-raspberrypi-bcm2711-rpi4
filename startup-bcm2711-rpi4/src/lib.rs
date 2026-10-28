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
    kprintln!(
        "startup-bcm2711-rpi4: EL{}, debug level {}, argv {:?}",
        oscraft_startup::arch::aarch64::current_el(),
        s.debug_level,
        s.cmdline.argv.as_slice()
    );
    oscraft_startup::crash!("not implemented yet");
}

oscraft_startup::entry!(main);
