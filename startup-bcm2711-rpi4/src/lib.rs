//! QNX OS 8.0 startup for the Raspberry Pi 4 (BCM2711).
//!
//! The board's part of startup: what the BCM2711 and the Raspberry Pi
//! firmware are, on top of the generic `oscraft-startup` library.

#![cfg_attr(not(test), no_std)]

#[cfg(target_arch = "aarch64")]
mod board;
#[cfg(target_arch = "aarch64")]
mod hwinfo;
#[cfg(target_arch = "aarch64")]
mod mbox;
#[cfg(target_arch = "aarch64")]
mod miniuart;
#[cfg(target_arch = "aarch64")]
mod pm;
mod revision;
