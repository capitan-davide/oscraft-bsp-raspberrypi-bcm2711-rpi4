//! QNX OS 8.0 startup for the Raspberry Pi 4 (BCM2711).

#![no_std]

use oscraft_startup::Board;

/// The mini-UART (UART1), the console on GPIO 14/15.
const AUX_MU_IO: usize = 0xfe21_5040;
const AUX_MU_LSR: usize = 0xfe21_5054;
const LSR_TX_EMPTY: u32 = 1 << 5;

struct Rpi4;

impl Board for Rpi4 {
    fn debug_putc(&self, byte: u8) {
        // SAFETY: the mini-UART registers are device memory, mapped 1:1
        // while the MMU is off.
        unsafe {
            while core::ptr::read_volatile(AUX_MU_LSR as *const u32) & LSR_TX_EMPTY == 0 {}
            core::ptr::write_volatile(AUX_MU_IO as *mut u32, u32::from(byte));
        }
    }
}

fn main() -> ! {
    for &b in b"startup-bcm2711-rpi4: hello\r\n" {
        Rpi4.debug_putc(b);
    }
    loop {
        core::hint::spin_loop();
    }
}

oscraft_startup::entry!(main);
