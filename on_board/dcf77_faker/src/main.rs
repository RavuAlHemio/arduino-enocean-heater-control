#![no_main]
#![no_std]


use core::panic::PanicInfo;

use atsam3x8e::{Interrupt, interrupt, Peripherals};
use atsam3x8e::tc0::wave_eq_1_cmr0_wave_eq_1 as tc0cmr0;
use atsam3x8e::tc0::wave_eq_1_cmr1_wave_eq_1 as tc0cmr1;
use atsam3x8e_ext::sam_pin;
use atsam3x8e_ext::setup::{CHIP_FREQ_CPU_MAX, system_init};
use buildingblocks::bit_field::BitField;
use cortex_m::Peripherals as CorePeripherals;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::{entry, exception};


// most accurate timer is MCLK/2 = 42 MHz = 42_000_000 Hz
// carrier signal is 77.5 kHz = 77_500 Hz
// => one period is 541 + 29/31 counter values
const PERIOD_WHOLE: u32 = 541;
const PERIOD_NUMER: i32 = 29;
const PERIOD_DENOM: i32 = 31;


static mut DCF77_DATA: BitField<8> = BitField::from_bytes([0u8; 8]);
static mut CURRENT_IS_DATA: bool = false;
static mut CURRENT_PERIOD_WHOLE: u32 = PERIOD_WHOLE;
static mut CURRENT_NUMER: i32 = 0;


#[panic_handler]
fn loopy_panic_handler(_: &PanicInfo) -> ! {
    loop {
    }
}

#[exception]
unsafe fn DefaultHandler(_: i16) {
}


#[inline]
fn update_duty_cycle(peripherals: &mut Peripherals) {
    let duty_cycle_value = unsafe {
        if CURRENT_IS_DATA {
            // "data" is transmitted using a duty cycle of 1/44 of a period
            CURRENT_PERIOD_WHOLE / 44
        } else {
            // "no data" is transmitted using a duty cycle of half a period
            CURRENT_PERIOD_WHOLE / 2
        }
    };
    peripherals.TC0.rc0.write(|w| w.rc().variant(duty_cycle_value));
}


#[entry]
fn main() -> ! {
    let mut core_peripherals = CorePeripherals::take().unwrap();
    let mut peripherals = Peripherals::take().unwrap();
    let _clock = system_init(&mut peripherals);

    let mut dcf = Dcf77::new();
    dcf.set_date(Dcf77Date { day_of_month: 10, day_of_week: 2, month: 4, year_of_century: 90 });
    dcf.set_winter_time(false);
    dcf.set_summer_time(true);
    unsafe { DCF77_DATA = dcf.get_storage_copy() };

    // give pin to timer
    // TIOA0 = PB25 (= Arduino Due: D2) peripheral B
    sam_pin!(disable_io, peripherals, PIOB, p25);
    sam_pin!(make_output, peripherals, PIOB, p25);
    sam_pin!(peripheral_ab, peripherals, PIOB, p25, set_bit);

    // set up timer counter for PWM (TC0)
    peripherals.TC0.wave_eq_1_cmr0_wave_eq_1().modify(|_, w| w
        .tcclks().variant(tc0cmr0::TCCLKS_A::TIMER_CLOCK1) // MCLK/2
        .wave().set_bit() // wave mode
        .wavsel().variant(tc0cmr0::WAVSEL_A::UP_RC) // count up until RC, then wrap around to 0
        .acpa().variant(tc0cmr0::ACPA_A::CLEAR) // once we hit RA ("on" period is over), turn off power
        .acpc().variant(tc0cmr0::ACPC_A::SET) // once we hit RC (one cycle of "on" followed by "off" is over), turn on power
    );

    // => set RC to on duration and RA to on+off duration
    peripherals.TC0.ra0.write(|w| w.ra().variant(PERIOD_WHOLE));
    unsafe { CURRENT_IS_DATA = false };
    update_duty_cycle(&mut peripherals);
    unsafe {
        peripherals.TC0.ier0.write_with_zero(|w| w
            .cpcs().set_bit() // interrupt when counter overflows RC
        )
    };

    // set up timer counter for sender on/off (10 times per second)
    const TEN_HZ_COUNTER_MCLK_BY_128: u32 = (CHIP_FREQ_CPU_MAX/128) / 10;
    peripherals.TC0.wave_eq_1_cmr1_wave_eq_1().modify(|_, w| w
        .tcclks().variant(tc0cmr1::TCCLKS_A::TIMER_CLOCK4) // MCLK/128
        .wave().set_bit() // wave mode
        .wavsel().variant(tc0cmr1::WAVSEL_A::UP_RC) // count up until RC, then wrap around to 0
    );
    peripherals.TC0.rc1.write(|w| w.rc().variant(TEN_HZ_COUNTER_MCLK_BY_128));
    unsafe {
        peripherals.TC0.ier1.write_with_zero(|w| w
            .cpcs().set_bit() // interrupt when counter overflows RC
        )
    };

    // enable system-level interrupts for timers 0 and 1 (= TC0 timers 0 and 1)
    unsafe { NVIC::unmask(Interrupt::TC0) };
    unsafe { NVIC::unmask(Interrupt::TC1) };

    // start both timers
    unsafe {
        peripherals.TC0.ccr0.write_with_zero(|w| w
            .clken().set_bit()
        )
    };
    unsafe {
        peripherals.TC0.ccr1.write_with_zero(|w| w
            .clken().set_bit()
        )
    };

    loop {
    }
}

#[interrupt]
fn TC0() {
    // timer 0 ticked over
    // adjust fractional part of duty cycle
    let mut stolen_peripherals = unsafe { Peripherals::steal() };
    unsafe { CURRENT_NUMER += PERIOD_NUMER };
    if unsafe { CURRENT_NUMER >= PERIOD_DENOM } {
        // perform compensation
        unsafe { CURRENT_PERIOD_WHOLE = PERIOD_WHOLE + 1 };
        unsafe { CURRENT_NUMER -= PERIOD_DENOM };
        update_duty_cycle(&mut stolen_peripherals);
    } else {
        // no compensation is required
        if unsafe { CURRENT_PERIOD_WHOLE == PERIOD_WHOLE + 1 } {
            // but it's active; deactivate it
            unsafe { CURRENT_PERIOD_WHOLE = PERIOD_WHOLE };
            update_duty_cycle(&mut stolen_peripherals);
        }
    }
}

#[interrupt]
fn TC1() {
    // 0.1s elapsed

    static mut BIT_POS: usize = 0;
    static mut WITHIN_SECOND: u32 = 0;

    *WITHIN_SECOND += 1;
    if *WITHIN_SECOND == 10 {
        *WITHIN_SECOND = 0;
        *BIT_POS += 1;
        if *BIT_POS == 60 {
            // repeat from the beginning
            *BIT_POS = 0;
        }
    }

    // a 0 bit is transmitted using 0.1s of "data" followed by 0.9s of "no data"
    // a 1 bit is transmitted using 0.2s of "data" followed by 0.8s of "no data"
    let mut stolen_peripherals = unsafe { Peripherals::steal() };
    if *WITHIN_SECOND == 0 {
        // start of a new second -- activate the "data" duty cycle
        unsafe { CURRENT_IS_DATA = true };
        update_duty_cycle(&mut stolen_peripherals);
    } else if *WITHIN_SECOND == 1 && unsafe { !DCF77_DATA.is_bit_set(*BIT_POS) } {
        // bit is 0 and we are at n+0.1s -- switch to "no data" duty cycle
        unsafe { CURRENT_IS_DATA = false };
        update_duty_cycle(&mut stolen_peripherals);
    } else if *WITHIN_SECOND == 2 && unsafe { DCF77_DATA.is_bit_set(*BIT_POS) } {
        // bit is 1 and we are at n+0.2s -- switch to "no data" duty cycle
        unsafe { CURRENT_IS_DATA = false };
        update_duty_cycle(&mut stolen_peripherals);
    }

    // change nothing in all the remaining cases
}


macro_rules! single_bit_op {
    ($bit_index:expr, $get_name:ident, $set_name:ident) => {
        #[inline]
        pub fn $get_name(&self) -> bool {
            self.storage.is_bit_set($bit_index)
        }

        #[inline]
        pub fn $set_name(&mut self, new_value: bool) {
            if new_value {
                self.storage.set_bit($bit_index);
            } else {
                self.storage.clear_bit($bit_index);
            }
        }
    };
}
macro_rules! bcd_bit {
    ($self:expr, $bit_index:expr, $variable:expr, $value:expr $(, $parity_value:expr)?) => {
        if $variable >= $value {
            $variable -= $value;
            $self.storage.set_bit($bit_index);
            $(
                $parity_value = !$parity_value;
            )?
        } else {
            $self.storage.clear_bit($bit_index);
        }
    };
}

struct Dcf77Date {
    pub day_of_month: u8,
    pub day_of_week: u8,
    pub month: u8,
    pub year_of_century: u8,
}

struct Dcf77 {
    storage: BitField<8>,
}
impl Dcf77 {
    pub fn new() -> Self {
        let mut ret = Self {
            storage: BitField::from_bytes([0; 8]),
        };
        ret.set_date(Dcf77Date { day_of_month: 1, day_of_week: 4, month: 1, year_of_century: 70 });
        ret.set_winter_time(true);
        ret
    }

    single_bit_op!(15, is_abnormal_operation, set_abnormal_operation);
    single_bit_op!(16, is_time_switchover_next_hour, set_time_switchover_next_hour);
    single_bit_op!(17, is_summer_time, set_summer_time);
    single_bit_op!(18, is_winter_time, set_winter_time);
    single_bit_op!(19, is_leap_second, set_leap_second);

    pub fn get_minutes(&self) -> u8 {
        let mut minutes = 0;
        if self.storage.is_bit_set(21) { minutes += 1; }
        if self.storage.is_bit_set(22) { minutes += 2; }
        if self.storage.is_bit_set(23) { minutes += 4; }
        if self.storage.is_bit_set(24) { minutes += 8; }
        if self.storage.is_bit_set(25) { minutes += 10; }
        if self.storage.is_bit_set(26) { minutes += 20; }
        if self.storage.is_bit_set(27) { minutes += 40; }
        minutes
    }

    pub fn set_minutes(&mut self, mut minutes: u8) {
        let mut parity = false;
        assert!(minutes <= 59);
        bcd_bit!(self, 27, minutes, 40, parity);
        bcd_bit!(self, 26, minutes, 20, parity);
        bcd_bit!(self, 25, minutes, 10, parity);
        bcd_bit!(self, 24, minutes, 8, parity);
        bcd_bit!(self, 23, minutes, 4, parity);
        bcd_bit!(self, 22, minutes, 2, parity);
        bcd_bit!(self, 21, minutes, 1, parity);
        if parity {
            self.storage.set_bit(28);
        } else {
            self.storage.clear_bit(28);
        }
    }

    pub fn get_hours(&self) -> u8 {
        let mut hours = 0;
        if self.storage.is_bit_set(29) { hours += 1; }
        if self.storage.is_bit_set(30) { hours += 2; }
        if self.storage.is_bit_set(31) { hours += 4; }
        if self.storage.is_bit_set(32) { hours += 8; }
        if self.storage.is_bit_set(33) { hours += 10; }
        if self.storage.is_bit_set(34) { hours += 20; }
        hours
    }

    pub fn set_hours(&mut self, mut hours: u8) {
        let mut parity = false;
        assert!(hours <= 23);
        bcd_bit!(self, 34, hours, 20, parity);
        bcd_bit!(self, 33, hours, 10, parity);
        bcd_bit!(self, 32, hours, 8, parity);
        bcd_bit!(self, 31, hours, 4, parity);
        bcd_bit!(self, 30, hours, 2, parity);
        bcd_bit!(self, 29, hours, 1, parity);
        if parity {
            self.storage.set_bit(35);
        } else {
            self.storage.clear_bit(35);
        }
    }

    pub fn get_date(&self) -> Dcf77Date {
        let mut day = 0;
        if self.storage.is_bit_set(36) { day += 1; }
        if self.storage.is_bit_set(37) { day += 2; }
        if self.storage.is_bit_set(38) { day += 4; }
        if self.storage.is_bit_set(39) { day += 8; }
        if self.storage.is_bit_set(40) { day += 10; }
        if self.storage.is_bit_set(41) { day += 20; }

        let mut dow = 0;
        if self.storage.is_bit_set(42) { dow += 1; }
        if self.storage.is_bit_set(43) { dow += 2; }
        if self.storage.is_bit_set(44) { dow += 4; }

        let mut month = 0;
        if self.storage.is_bit_set(45) { month += 1; }
        if self.storage.is_bit_set(46) { month += 2; }
        if self.storage.is_bit_set(47) { month += 4; }
        if self.storage.is_bit_set(48) { month += 8; }
        if self.storage.is_bit_set(49) { month += 10; }

        let mut year = 0;
        if self.storage.is_bit_set(50) { year += 1; }
        if self.storage.is_bit_set(51) { year += 2; }
        if self.storage.is_bit_set(52) { year += 4; }
        if self.storage.is_bit_set(53) { year += 8; }
        if self.storage.is_bit_set(54) { year += 10; }
        if self.storage.is_bit_set(55) { year += 20; }
        if self.storage.is_bit_set(56) { year += 40; }
        if self.storage.is_bit_set(57) { year += 80; }

        Dcf77Date { day_of_month: day, day_of_week: dow, month, year_of_century: year }
    }

    pub fn set_date(&mut self, mut date: Dcf77Date) {
        let mut parity = false;

        assert!(date.day_of_month >= 1 && date.day_of_month <= 31);
        bcd_bit!(self, 41, date.day_of_month, 20, parity);
        bcd_bit!(self, 40, date.day_of_month, 10, parity);
        bcd_bit!(self, 39, date.day_of_month, 8, parity);
        bcd_bit!(self, 38, date.day_of_month, 4, parity);
        bcd_bit!(self, 37, date.day_of_month, 2, parity);
        bcd_bit!(self, 36, date.day_of_month, 1, parity);

        assert!(date.day_of_week >= 1 && date.day_of_week <= 7);
        bcd_bit!(self, 44, date.day_of_week, 4, parity);
        bcd_bit!(self, 43, date.day_of_week, 2, parity);
        bcd_bit!(self, 42, date.day_of_week, 1, parity);

        assert!(date.month >= 1 && date.month <= 12);
        bcd_bit!(self, 49, date.month, 10, parity);
        bcd_bit!(self, 48, date.month, 8, parity);
        bcd_bit!(self, 47, date.month, 4, parity);
        bcd_bit!(self, 46, date.month, 2, parity);
        bcd_bit!(self, 45, date.month, 1, parity);

        assert!(date.year_of_century <= 99);
        bcd_bit!(self, 57, date.year_of_century, 80, parity);
        bcd_bit!(self, 56, date.year_of_century, 40, parity);
        bcd_bit!(self, 55, date.year_of_century, 20, parity);
        bcd_bit!(self, 54, date.year_of_century, 10, parity);
        bcd_bit!(self, 53, date.year_of_century, 8, parity);
        bcd_bit!(self, 52, date.year_of_century, 4, parity);
        bcd_bit!(self, 51, date.year_of_century, 2, parity);
        bcd_bit!(self, 50, date.year_of_century, 1, parity);

        if parity {
            self.storage.set_bit(58);
        } else {
            self.storage.clear_bit(58);
        }
    }

    pub fn get_storage_copy(&self) -> BitField<8> {
        self.storage.clone()
    }
}
