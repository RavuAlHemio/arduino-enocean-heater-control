pub(crate) mod setup;
pub(crate) mod tick;

#[inline]
pub(crate) fn nop() {
    unsafe { core::arch::asm!("nop") };
}
