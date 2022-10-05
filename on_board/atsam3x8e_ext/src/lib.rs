#![no_std]


pub mod pin;
pub mod setup;
pub mod tick;

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
