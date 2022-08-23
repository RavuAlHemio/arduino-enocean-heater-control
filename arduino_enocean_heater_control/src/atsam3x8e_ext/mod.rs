#[macro_use] pub(crate) mod pin;
pub(crate) mod setup;
pub(crate) mod tick;

#[inline]
pub(crate) fn nop() {
    unsafe { core::arch::asm!("nop") };
}

#[inline]
pub(crate) fn multinop<const COUNT: usize>() {
    for _ in 0..COUNT {
        nop();
    }
}
