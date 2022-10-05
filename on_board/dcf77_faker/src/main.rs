#![no_main]
#![no_std]


use core::panic::PanicInfo;

use atsam3x8e::Peripherals;
use atsam3x8e::tc0::wave_eq_1_cmr0_wave_eq_1::{ACPA_A, ACPC_A, TCCLKS_A, WAVSEL_A};
use atsam3x8e_ext::sam_pin;
use atsam3x8e_ext::setup::system_init;
use buildingblocks::bit_field::BitField;
use cortex_m_rt::{entry, exception};


#[panic_handler]
fn loopy_panic_handler(_: &PanicInfo) -> ! {
    loop {
    }
}

#[exception]
unsafe fn DefaultHandler(_: i16) {
}


#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let _clock = system_init(&mut peripherals);

    // give pin to timer
    // TIOA0 = PB25 (= Arduino Due: D2) peripheral B
    sam_pin!(disable_io, peripherals, PIOB, p25);
    sam_pin!(make_output, peripherals, PIOB, p25);
    sam_pin!(peripheral_ab, peripherals, PIOB, p25, set_bit);

    // set up timer counter for PWM (TC0)
    // most accurate timer is MCLK/2 = 42 MHz = 42_000_000 Hz
    // carrier signal is 77.5 kHz = 77_500 Hz
    // => one period is 541 + 29/31 counter values
    const PERIOD_WHOLE: u32 = 541;
    const PERIOD_NUMER: u32 = 29;
    const PERIOD_DENOM: u32 = 31;

    // "data" is transmitted using a duty cycle of 1/44 of a period
    // "no data" is transmitted using a duty cycle of half a period
    const DUTY_CYCLE_DATA: u32 = PERIOD_WHOLE / 44;
    const DUTY_CYCLE_NO_DATA: u32 = PERIOD_WHOLE / 2;

    // a 0 bit is transmitted using 0.1s of "data" followed by 0.9s of "no data"
    // a 1 bit is transmitted using 0.2s of "data" followed by 0.8s of "no data"

    peripherals.TC0.wave_eq_1_cmr0_wave_eq_1().modify(|_, w| w
        .tcclks().variant(TCCLKS_A::TIMER_CLOCK1) // MCLK/2
        .wave().set_bit() // wave mode
        .wavsel().variant(WAVSEL_A::UP_RC) // count up until RC, then wrap around to 0
        .acpa().variant(ACPA_A::CLEAR) // once we hit RA ("on" period is over), turn off power
        .acpc().variant(ACPC_A::SET) // once we hit RC (one cycle of "on" followed by "off" is over), turn on power
    );

    // => set RC to on duration and RA to on+off duration
    peripherals.TC0.ra0.write(|w| w.ra().variant(541));
    peripherals.TC0.rc0.write(|w| w.rc().variant(270));

    // set up timer counter for sender on/off (10 times per second)
    todo!();

    loop {
    }
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
}
