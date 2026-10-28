//! The mini-UART (UART1), the console on GPIO 14/15.

use oscraft_startup::console::DebugDevice;

/// The mini-UART's registers, in the BCM2711's auxiliary peripherals.
const AUX_MU_IO: usize = 0xfe21_5040;
const AUX_MU_LSR: usize = 0xfe21_5054;
const LSR_TX_EMPTY: u32 = 1 << 5;

pub struct MiniUart;

pub static MINIUART: MiniUart = MiniUart;

impl DebugDevice for MiniUart {
    fn putc(&self, byte: u8) {
        // SAFETY: the mini-UART registers are device memory, mapped 1:1
        // while the MMU is off.
        unsafe {
            while core::ptr::read_volatile(AUX_MU_LSR as *const u32) & LSR_TX_EMPTY == 0 {}
            core::ptr::write_volatile(AUX_MU_IO as *mut u32, u32::from(byte));
        }
    }
}
