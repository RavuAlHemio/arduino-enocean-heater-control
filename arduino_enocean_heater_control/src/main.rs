#![no_main]
#![no_std]


use core::panic::PanicInfo;

use cortex_m_rt::{entry, exception};


#[exception]
unsafe fn DefaultHandler(_: i16) {
}


#[panic_handler]
fn loopy_panic_handler(_: &PanicInfo) -> ! {
    loop {
    }
}


#[entry]
fn main() -> ! {
    loop {
    }
}
