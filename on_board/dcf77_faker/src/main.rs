#![no_main]
#![no_std]


mod i2c_display;


use core::panic::PanicInfo;
use core::time::Duration;

use atsam3x8e::{Interrupt, interrupt, Peripherals};
use atsam3x8e::tc0::wave_eq_1_cmr0_wave_eq_1 as tc0cmr0;
use atsam3x8e::tc1::wave_eq_1_cmr0_wave_eq_1 as tc1cmr0;
use atsam3x8e_ext::sam_pin;
use atsam3x8e_ext::setup::{CHIP_FREQ_CPU_MAX, system_init};
use atsam3x8e_ext::tick::delay;
use atsam3x8e_ext::uart;
use buildingblocks::bit_field_from_bool;
use buildingblocks::bit_field::BitField;
use cortex_m::Peripherals as CorePeripherals;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::{entry, exception};
use vcell::VolatileCell;

use crate::i2c_display::{I2cDisplay, I2cDisplayTwi0, I2cDisplayTwi1};


// most accurate timer is MCLK/2 = 42 MHz = 42_000_000 Hz
// carrier signal is 77.5 kHz = 77_500 Hz
// => one period is 541 + 29/31 counter values
// however, the oscilloscope says we're more like 74 kHz, so tune this
//const PERIOD_WHOLE: u32 = 541;
const PERIOD_WHOLE: u32 = 518;
const PERIOD_NUMER: i32 = 0;
const PERIOD_DENOM: i32 = 31;


static mut DCF77: Dcf77 = Dcf77::new();
static mut DCF77_DATA: BitField<8> = BitField::from_bytes([0u8; 8]);
static mut CURRENT_IS_DATA: bool = false;
static mut CURRENT_PERIOD_WHOLE: u32 = PERIOD_WHOLE;
static mut CURRENT_NUMER: i32 = 0;
static mut DISPLAY: I2cDisplayTwi1 = I2cDisplayTwi1::new(
    0b0100_111, // PCF8574 is always 0b0100xxx; we didn't change the jumpers from 0b111
    true,
);
static mut UPDATE_TIME: VolatileCell<bool> = VolatileCell::new(true);


#[panic_handler]
fn loopy_panic_handler(_: &PanicInfo) -> ! {
    loop {
    }
}

#[exception]
unsafe fn DefaultHandler(_: i16) {
}


#[inline]
fn update_timer0_duty_cycle(peripherals: &mut Peripherals) {
    let duty_cycle_value = unsafe {
        if CURRENT_IS_DATA {
            // "data" is transmitted using a duty cycle of 1/44 of a period
            //sam_pin!(set_high, peripherals, PIOB, p27);
            CURRENT_PERIOD_WHOLE / 44
        } else {
            // "no data" is transmitted using a duty cycle of half a period
            //sam_pin!(set_low, peripherals, PIOB, p27);
            CURRENT_PERIOD_WHOLE / 2
        }
    };
    peripherals.TC0.ra0.write(|w| w.ra().variant(duty_cycle_value));
}


#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    // first things first: disable the watchdog
    peripherals.WDT.mr.modify(|_, w| w
        .wddis().set_bit()
    );

    // set up the clocks
    let mut core_peripherals = CorePeripherals::take().unwrap();
    let clock = system_init(&mut peripherals);

    // enable the tick clock and give us 3s to connect the debugger
    atsam3x8e_ext::tick::enable_tick_clock(&mut core_peripherals, clock.clock_speed / 1000);
    //atsam3x8e_ext::tick::delay(core::time::Duration::from_secs(3));

    // initialize UART
    uart::init(&mut peripherals);

    unsafe {
        DCF77.set_date(Dcf77Date { day_of_month: 10, day_of_week: 2, month: 4, year_of_century: 90 });
        DCF77.set_hours(10);
        DCF77.set_minutes(40);
        DCF77.set_winter_time(false);
        DCF77.set_summer_time(true);
        DCF77_DATA = DCF77.get_storage_copy();
    }

    // give pin to timer
    // TIOA0 = PB25 (= Arduino Due: D2) peripheral B
    sam_pin!(disable_io, peripherals, PIOB, p25);
    sam_pin!(make_output, peripherals, PIOB, p25);
    sam_pin!(peripheral_ab, peripherals, PIOB, p25, set_bit);

    // set LED pin as output
    sam_pin!(enable_io, peripherals, PIOB, p27);
    sam_pin!(make_output, peripherals, PIOB, p27);
    sam_pin!(set_low, peripherals, PIOB, p27);

    // feed clock to a few peripherals
    unsafe {
        peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
            .pid12().set_bit() // PIOB
            .pid27().set_bit() // timer 0
            .pid30().set_bit() // timer 3
        )
    };

    // wait 50ms to ensure display chip is ready
    delay(Duration::from_millis(50));

    I2cDisplayTwi1::grab_io_pins(&mut peripherals);
    I2cDisplayTwi1::enable_system_clock_to_twi(&mut peripherals);
    I2cDisplayTwi1::setup_twi(&mut peripherals);

    // set display to 8-bit mode
    // send the same nibble three times so that we take care of all situations:
    // * 8-bit mode (reads 0011_0000, sets to 8 bit)
    // * 4-bit mode, start of a byte (reads 0011 & 0011, sets to 8 bit, reads 0011_0000, sets to 8 bit)
    // * 4-bit mode, middle of a byte (reads 0011, executes something, then reads 0011 & 0011, sets to 8 bit)
    unsafe { &DISPLAY }.set_target(&mut peripherals);
    unsafe { &DISPLAY }.transmit_nibble(&mut peripherals, 0b0011, false);
    I2cDisplayTwi1::long_delay();
    unsafe { &DISPLAY }.transmit_nibble(&mut peripherals, 0b0011, false);
    I2cDisplayTwi1::short_delay();
    unsafe { &DISPLAY }.transmit_nibble(&mut peripherals, 0b0011, false);
    I2cDisplayTwi1::short_delay();

    // set display to 4-bit mode
    unsafe { &DISPLAY }.transmit_nibble(&mut peripherals, 0b0010, false);
    I2cDisplayTwi1::short_delay();
    unsafe { &DISPLAY }.transmit_byte(&mut peripherals, 0b0010_1000, false);
    I2cDisplayTwi1::short_delay();

    // disable display
    unsafe { &DISPLAY }.transmit_byte(&mut peripherals, 0b0000_1000, false);
    I2cDisplayTwi1::short_delay();

    // clear display and go home
    unsafe { &DISPLAY }.transmit_byte(&mut peripherals, 0b0000_0001, false);
    I2cDisplayTwi1::long_delay();

    // increment but don't shift
    unsafe { &DISPLAY }.transmit_byte(&mut peripherals, 0b0000_0110, false);
    I2cDisplayTwi1::short_delay();

    // enable display
    unsafe { &DISPLAY }.transmit_byte(&mut peripherals, 0b0000_1100, false);
    I2cDisplayTwi1::short_delay();

    // write a bunch of characters
    for b in b"DCF77 Faker" {
        unsafe { &DISPLAY }.transmit_byte(&mut peripherals, *b, true);
        I2cDisplayTwi1::short_delay();
    }

    // set up timer counter for PWM (timer 0)
    // disable and re-enable the clock first
    unsafe {
        peripherals.TC0.ccr0.write_with_zero(|w| w
            .clkdis().set_bit()
        )
    };
    unsafe {
        peripherals.TC0.ccr0.write_with_zero(|w| w
            .clken().set_bit()
        )
    };
    // disable write protection
    peripherals.TC0.wpmr.write(|w| w
        .wpkey().variant(atsam3x8e::tc0::wpmr::WPKEY_A::PASSWD)
        .wpen().clear_bit()
    );
    // read the status register, which clears it
    peripherals.TC0.sr0.read().bits();
    // set up everything
    peripherals.TC0.wave_eq_1_cmr0_wave_eq_1().modify(|_, w| w
        .tcclks().variant(tc0cmr0::TCCLKS_A::TIMER_CLOCK1) // MCLK/2
        .clki().clear_bit() // increment on rising clock edge
        .burst().variant(tc0cmr0::BURST_A::NONE) // clock not gated by external signal
        .wave().set_bit() // wave mode
        .wavsel().variant(tc0cmr0::WAVSEL_A::UP_RC) // count up and reset on hitting RC
        .aswtrg().variant(tc0cmr0::ASWTRG_A::SET) // when triggered by software (we do this on every overflow), turn on power of TIOA0
        .acpa().variant(tc0cmr0::ACPA_A::CLEAR) // once we hit RA ("on" period is over), turn off power on TIOA0
        .acpc().variant(tc0cmr0::ACPC_A::SET) // enable TIOA0 once we hit RC
        .aeevt().variant(tc0cmr0::AEEVT_A::NONE) // do nothing with TIOA0 if we receive an external event
        .bcpb().variant(tc0cmr0::BCPB_A::NONE) // do nothing with TIOB0 when we hit RB
        .bcpc().variant(tc0cmr0::BCPC_A::NONE) // do nothing with TIOB0 when we hit RC
        .beevt().variant(tc0cmr0::BEEVT_A::NONE) // do nothing with TIOB0 if we receive an external event
        .bswtrg().variant(tc0cmr0::BSWTRG_A::NONE) // do nothing with TIOB0 if we receive a software trigger
        .cpcstop().clear_bit() // don't stop clock when we hit RC
        .cpcdis().clear_bit() // don't disable clock when we hit RC
        .eevt().variant(tc0cmr0::EEVT_A::XC0) // external event is XC0 (not TIOB)
        .eevtedg().variant(tc0cmr0::EEVTEDG_A::NONE) // detect no external event edge
        .enetrg().clear_bit() // no trigger by external event
    );

    // => set RA to on duration and RC to on+off duration
    peripherals.TC0.rc0.write(|w| w.rc().variant(PERIOD_WHOLE));
    unsafe { CURRENT_IS_DATA = false };
    update_timer0_duty_cycle(&mut peripherals);
    unsafe {
        peripherals.TC0.ier0.write_with_zero(|w| w
            .cpcs().set_bit() // interrupt when counter overflows RC
        )
    };

    // set up timer counter for sender on/off (timer 3; 10 times per second)
    const TEN_HZ_COUNTER_MCLK_BY_128: u32 = (CHIP_FREQ_CPU_MAX/128) / 10;
    unsafe {
        peripherals.TC1.ccr0.write_with_zero(|w| w
            .clkdis().set_bit()
        )
    };
    unsafe {
        peripherals.TC1.ccr0.write_with_zero(|w| w
            .clken().set_bit()
        )
    };
    peripherals.TC1.wave_eq_1_cmr0_wave_eq_1().modify(|_, w| w
        .tcclks().variant(tc1cmr0::TCCLKS_A::TIMER_CLOCK4) // MCLK/128
        .wave().set_bit() // wave mode
        .wavsel().variant(tc1cmr0::WAVSEL_A::UP) // count up (we reset by triggering in the interrupt handler)
        .cpcstop().clear_bit() // don't stop clock when we hit RC
        .cpcdis().clear_bit() // don't disable clock when we hit RC
    );
    peripherals.TC1.ra0.write(|w| w.ra().variant(0));
    peripherals.TC1.rc0.write(|w| w.rc().variant(TEN_HZ_COUNTER_MCLK_BY_128));
    unsafe {
        peripherals.TC1.ier0.write_with_zero(|w| w
            .cpcs().set_bit() // interrupt when counter overflows RC
        )
    };

    // enable system-level interrupts for timers 0 and 3
    unsafe { NVIC::unmask(Interrupt::TC0) };
    unsafe { NVIC::unmask(Interrupt::TC3) };

    // increase TC0 priority value
    // (reduces its priority and allows it to be preempted by TC3)
    unsafe { core_peripherals.NVIC.set_priority(Interrupt::TC0, 1 << 4) };

    // enable and start both timers
    trigger_timer0(&mut peripherals);
    trigger_timer3(&mut peripherals);

    loop {
        cortex_m::asm::wfi();

        let should_update_time = unsafe { UPDATE_TIME.get() };
        if should_update_time {
            unsafe { UPDATE_TIME.set(false) };

            // go to address 20
            unsafe { &DISPLAY }.transmit_byte(&mut peripherals, 0b1000_0000 | 20, false);

            // write the new time
            let date = unsafe { DCF77.get_date() };
            let day_buf = u8_to_dec(date.day_of_month);
            let mon_buf = u8_to_dec(date.month);
            let year_buf = u8_to_dec(date.year_of_century);
            let hour_buf = u8_to_dec(unsafe { DCF77.get_hours() });
            let minute_buf = u8_to_dec(unsafe { DCF77.get_minutes() });
            let all_buf = [
                day_buf[0], day_buf[1],
                b'.',
                mon_buf[0], mon_buf[1],
                b'.',
                year_buf[0], year_buf[1],
                b' ',
                hour_buf[0], hour_buf[1],
                b':',
                minute_buf[0], minute_buf[1],
            ];
            for b in all_buf {
                unsafe { &DISPLAY }.transmit_byte(&mut peripherals, b, true);
                I2cDisplayTwi1::short_delay();
            }
        }
    }
}

#[inline]
fn nibble_to_ascii_hex(number: u8) -> u8 {
    if number < 0xA {
        b'0' + number
    } else if number < 0x10 {
        b'A' - 10 + number
    } else {
        0x00
    }
}
fn u8_to_hex(number: u8) -> [u8; 2] {
    let upper = nibble_to_ascii_hex(number >> 4);
    let lower = nibble_to_ascii_hex(number & 0xF);
    [upper, lower]
}
fn u8_to_dec(number: u8) -> [u8; 2] {
    let upper = number / 10;
    let lower = number % 10;
    [nibble_to_ascii_hex(upper), nibble_to_ascii_hex(lower)]
}

#[inline]
fn trigger_timer0(peripherals: &mut Peripherals) {
    unsafe {
        peripherals.TC0.ccr0.write_with_zero(|w| w
            .swtrg().set_bit()
        )
    };
}

#[inline]
fn trigger_timer3(peripherals: &mut Peripherals) {
    unsafe {
        peripherals.TC1.ccr0.write_with_zero(|w| w
            .swtrg().set_bit()
        )
    };
}


#[interrupt]
fn TC0() {
    // timer 0 ticked over
    // adjust fractional part of duty cycle
    let mut stolen_peripherals = unsafe { Peripherals::steal() };
    //sam_pin!(set_high, stolen_peripherals, PIOB, p27);

    // read-and-clear status register
    stolen_peripherals.TC0.sr0.read().bits();

    unsafe { CURRENT_NUMER += PERIOD_NUMER };
    if unsafe { CURRENT_NUMER < PERIOD_DENOM } {
        if unsafe { CURRENT_PERIOD_WHOLE != PERIOD_WHOLE - 1 } {
            // compensate downward
            unsafe { CURRENT_PERIOD_WHOLE = PERIOD_WHOLE - 1 };
            update_timer0_duty_cycle(&mut stolen_peripherals);
        }
    } else {
        // compensate upward
        unsafe { CURRENT_NUMER -= PERIOD_DENOM };
        unsafe { CURRENT_PERIOD_WHOLE = PERIOD_WHOLE };
        update_timer0_duty_cycle(&mut stolen_peripherals);
    }

    trigger_timer0(&mut stolen_peripherals);
}

#[interrupt]
fn TC3() {
    // 0.1s elapsed
    static mut BIT_POS: usize = 0;
    static mut WITHIN_SECOND: u8 = 0;

    let mut stolen_peripherals = unsafe { Peripherals::steal() };

    *WITHIN_SECOND += 1;
    if *WITHIN_SECOND >= 10 {
        *WITHIN_SECOND = 0;
        *BIT_POS += 1;
        if *BIT_POS >= 60 {
            // increment
            unsafe { DCF77.increment() };
            unsafe { DCF77_DATA = DCF77.get_storage_copy() };

            // update the display next time around
            unsafe { UPDATE_TIME.set(true) };

            // repeat from the beginning
            *BIT_POS = 0;
        }
    }

    // a 0 bit is transmitted using 0.1s of "data" followed by 0.9s of "no data"
    // a 1 bit is transmitted using 0.2s of "data" followed by 0.8s of "no data"
    //sam_pin!(set_low, stolen_peripherals, PIOB, p27);
    sam_pin!(set_high, stolen_peripherals, PIOB, p27);

    // read-and-clear status register
    stolen_peripherals.TC1.sr0.read().bits();

    /*
    let hexy = u8_to_hex(*BIT_POS as u8);
    let buf = [b'0', b'x', hexy[0], hexy[1], b'\r', b'\n'];
    uart::send(&mut stolen_peripherals, &buf);
    */

    if *WITHIN_SECOND == 0 {
        // start of a new second -- activate the "data" duty cycle
        // except: shut up completely if we are at second 59
        let is_data = *BIT_POS != 59;
        unsafe { CURRENT_IS_DATA = is_data };
        update_timer0_duty_cycle(&mut stolen_peripherals);
    } else if *WITHIN_SECOND == 1 && unsafe { !DCF77_DATA.is_bit_set(*BIT_POS) } {
        // bit is 0 and we are at n+0.1s -- switch to "no data" duty cycle
        unsafe { CURRENT_IS_DATA = false };
        update_timer0_duty_cycle(&mut stolen_peripherals);
    } else if *WITHIN_SECOND == 2 && unsafe { DCF77_DATA.is_bit_set(*BIT_POS) } {
        // bit is 1 and we are at n+0.2s -- switch to "no data" duty cycle
        unsafe { CURRENT_IS_DATA = false };
        update_timer0_duty_cycle(&mut stolen_peripherals);
    }

    // change nothing in all the remaining cases

    // re-trigger timer 3
    trigger_timer3(&mut stolen_peripherals);
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
    pub const fn new() -> Self {
        let storage = bit_field_from_bool![
            // BCD encoding means: 1, 2, 4, 8, 10, 20, 40, 80, ...

            false, // start of minute is always 0

            // 13 bits of civil warning and weather
            false, false, false, false, false, false, false, false,
            false, false, false, false, false,

            false, // abnormal transmitter operation
            false, // on the next hour, a summer<->winter time changeover will happen
            false, // summer time is in effect
            true, // winter time is in effect
            false, // on the next hour, a leap second will be introduced

            true, // start of time data is always 1

            // BCD encoding of minutes (7 bits) + even parity (xor sum)
            false, false, false, false, false, false, false, false,

            // BCD encoding of hours (6 bits) + even parity (xor sum)
            false, false, false, false, false, false, false, false,

            // BCD encoding of day-of-month (6 bits)
            true, false, false, false, false, false,

            // BCD encoding of day-of-week (3 bits; Unix Epoch was a Thursday)
            false, false, true,

            // BCD encoding of month (5 bits)
            true, false, false, false, false,

            // BCD encoding of year within century (8 bits; 70 = 10 + 20 + 40)
            false, false, false, false, true, true, true, false,

            // even parity over the previous fields (xor sum)
            false,

            // final byte of silence (pad with false)
            false,

            // 4 bits of padding to 64 bits = 8 bytes
            false, false, false, false,
        ];
        Dcf77 { storage }
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

    pub fn increment(&mut self) {
        let mut new_minutes = self.get_minutes() + 1;
        if new_minutes == 60 {
            new_minutes = 0;

            let mut new_hours = self.get_hours() + 1;
            if new_hours == 24 {
                new_hours = 0;
            }

            // don't bother with the date
            self.set_hours(new_hours);
        }
        self.set_minutes(new_minutes);
    }

    pub fn get_storage_copy(&self) -> BitField<8> {
        self.storage.clone()
    }
}
