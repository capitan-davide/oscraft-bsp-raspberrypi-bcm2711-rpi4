//! QNX OS 8.0 startup for the Raspberry Pi 4 (BCM2711).

#![no_std]

mod miniuart;

use oscraft_startup::cmdline::Opt;
use oscraft_startup::syspage::{hwinfo, strings};
use oscraft_startup::{Startup, console};

/// Options of this board, besides the common ones: `-W` enables the
/// watchdog.
const BOARD_OPTIONS: &str = "W:";

/// The GIC-400's distributor and CPU interface.
const GICD: u64 = 0xff84_1000;
const GICC: u64 = 0xff84_2000;
/// The virtual timer's PPI: the system tick.
const TIMER_PPI: i32 = 27;

fn main(s: &mut Startup) -> ! {
    console::set(&miniuart::MINIUART);
    for opt in s.options(BOARD_OPTIONS) {
        match opt {
            Opt::Flag('W', _) => {}
            other => s.handle_common_option(&other),
        }
    }
    // TODO: RAM from the VideoCore mailbox.
    s.add_ram(0, 0x8000_0000);
    s.init_mmu();
    oscraft_startup::arch::aarch64::gic_v2::init(s, GICD, GICC);
    s.init_qtime(TIMER_PPI);
    s.init_boot_cpu(700);

    let uart = s.map_device(miniuart::BASE, 0x100);
    let sp = &mut s.syspage;
    sp.callout.debug[0].display_char = sp.callout(&miniuart::DISPLAY_CHAR, &[uart]);
    sp.callout.debug[0].poll_key = sp.callout(&miniuart::POLL_KEY, &[uart]);
    sp.callout.debug[0].break_detect = sp.callout(&miniuart::BREAK_DETECT, &[]);
    sp.add_typed_string(strings::CS_MACHINE, "RaspberryPi4B");
    hwinfo::add_default(&mut sp.hwinfo, &mut sp.strings);
    sp.hwinfo.finish();
    s.boot()
}

oscraft_startup::entry!(main);
