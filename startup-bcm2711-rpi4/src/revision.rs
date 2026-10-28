//! The board revision code: which Raspberry Pi this is.
//!
//! New-style codes pack the board's description into bits:
//! `NOQuuuWuFMMMCCCCPPPPTTTTTTTTRRRR`, where MMM is the memory size, CCCC
//! the manufacturer, PPPP the processor and TTTTTTTT the board type.

fn field(rev: u32, shift: u32, mask: u32) -> usize {
    ((rev >> shift) & mask) as usize
}

/// The RAM size, in MB.
pub fn memory_mb(rev: u32) -> u64 {
    const SIZES: [u64; 6] = [256, 512, 1024, 2048, 4096, 8192];
    SIZES.get(field(rev, 20, 0x7)).copied().unwrap_or(0)
}

pub fn manufacturer(rev: u32) -> &'static str {
    const NAMES: [&str; 6] = [
        "Sony UK",
        "Egoman",
        "Embest",
        "Sony Japan",
        "Embest",
        "Stadium",
    ];
    NAMES.get(field(rev, 16, 0xf)).copied().unwrap_or("Unknown")
}

pub fn soc(rev: u32) -> &'static str {
    const NAMES: [&str; 4] = ["BCM2835", "BCM2836", "BCM2837", "BCM2711"];
    NAMES.get(field(rev, 12, 0xf)).copied().unwrap_or("Unknown")
}

pub fn board(rev: u32) -> &'static str {
    match field(rev, 4, 0xff) {
        0x11 => "RaspberryPi4B",
        0x13 => "RaspberryPi400",
        0x14 => "RaspberryPiCM4",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_raspberry_pi_4() {
        // QEMU's raspi4b: a 2 GB Pi 4 B made by Sony UK.
        let rev = 0x00b0_3115;
        assert_eq!(memory_mb(rev), 2048);
        assert_eq!(manufacturer(rev), "Sony UK");
        assert_eq!(soc(rev), "BCM2711");
        assert_eq!(board(rev), "RaspberryPi4B");
        // An 8 GB Pi 4 B.
        assert_eq!(memory_mb(0x00d0_3114), 8192);
    }
}
