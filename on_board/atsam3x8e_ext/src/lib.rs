#![no_std]


pub mod i2c_controller;
pub mod pin;
pub mod setup;
pub mod tick;
pub mod uart;

#[inline]
pub fn nop() {
    unsafe { core::arch::asm!("nop") };
}

#[inline]
pub fn multinop<const COUNT: usize>() {
    for _ in 0..COUNT {
        nop();
    }
}
