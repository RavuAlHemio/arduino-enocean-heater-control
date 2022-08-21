use core::cell::UnsafeCell;
use core::time::Duration;

use cortex_m::Peripherals;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;


pub struct VolatileStorage<T: Copy> {
    value: UnsafeCell<T>,
}
impl<T: Copy> VolatileStorage<T> {
    pub const fn new(val: T) -> Self {
        Self {
            value: UnsafeCell::new(val),
        }
    }

    pub fn get(&self) -> T {
        unsafe {
            core::ptr::read_volatile(self.value.get())
        }
    }

    pub fn set(&self, new_value: T) {
        unsafe {
            core::ptr::write_volatile(self.value.get(), new_value)
        }
    }
}
unsafe impl<T: Copy> Sync for VolatileStorage<T> {
    // damned lies; don't care
}


pub(crate) static TICK_CLOCK: VolatileStorage<u32> = VolatileStorage::new(0);


#[exception]
unsafe fn SysTick() {
    TICK_CLOCK.set(TICK_CLOCK.get().wrapping_add(1))
}

pub(crate) fn enable_tick_clock(core_peripherals: &mut Peripherals, frequency: u32) {
    const SYST_CSR_ENABLE_ENABLED: u32 = 1 << 0;
    const SYST_CSR_TICKINT_ENABLED: u32 = 1 << 1;
    const SYST_CSR_CLKSOURCE_MCK: u32 = 1 << 2;

    unsafe {
        core_peripherals.SYST.rvr.write(frequency)
    };
    unsafe {
        core_peripherals.SYST.csr.write(
            SYST_CSR_ENABLE_ENABLED
            | SYST_CSR_TICKINT_ENABLED
            | SYST_CSR_CLKSOURCE_MCK
        )
    };
}

#[inline]
pub(crate) fn delay(duration: Duration) {
    let ms_u128 = duration.as_millis();
    let ms = if ms_u128 > u32::MAX.into() {
        u32::MAX
    } else {
        ms_u128 as u32
    };

    let start = TICK_CLOCK.get();
    while TICK_CLOCK.get() < start + ms {
        // nop
    }
}
