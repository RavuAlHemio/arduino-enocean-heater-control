//! SPI implementation for Mikroe Click boards mounted on the Arduino Mega Click Shield.
//!
//! The pinout is as follows:
//!
//! * Chip Select for board 1: PB14
//! * Chip Select for board 2: PC17
//! * Chip Select for board 3: PC18
//! * Serial Clock: PB21
//! * Controller In Peripheral Out: PC13
//! * Controller Out Peripheral In: PC12


use crate::Peripherals;


#[inline]
fn nop() {
    unsafe { core::arch::asm!("nop") };
}


fn setup_pins_controller(peripherals: &mut Peripherals) {
    // take over pins from peripherals
    unsafe {
        peripherals.PIOB.per.write_with_zero(|w| w
            .p14().set_bit()
            .p21().set_bit()
        )
    };
    unsafe {
        peripherals.PIOC.per.write_with_zero(|w| w
            .p12().set_bit()
            .p13().set_bit()
            .p17().set_bit()
            .p18().set_bit()
        )
    };

    // set pins as output...
    unsafe {
        peripherals.PIOB.oer.write_with_zero(|w| w
            .p14().set_bit()
            .p21().set_bit()
        )
    };
    unsafe {
        peripherals.PIOC.oer.write_with_zero(|w| w
            .p12().set_bit()
            .p17().set_bit()
            .p18().set_bit()
        )
    };

    // ... except CIPO (input)
    unsafe {
        peripherals.PIOC.odr.write_with_zero(|w| w
            .p13().set_bit()
        )
    };

    // enable pull-up resistor for CIPO
    // (this is the controller's job)
    unsafe {
        peripherals.PIOC.puer.write_with_zero(|w| w
            .p13().set_bit()
        )
    };

    // set clock and COPI down and CS up
    unsafe {
        peripherals.PIOB.sodr.write_with_zero(|w| w
            .p14().set_bit()
        )
    };
    unsafe {
        peripherals.PIOB.codr.write_with_zero(|w| w
            .p21().set_bit()
        )
    };
    unsafe {
        peripherals.PIOC.sodr.write_with_zero(|w| w
            .p17().set_bit()
            .p18().set_bit()
        )
    };
    unsafe {
        peripherals.PIOC.codr.write_with_zero(|w| w
            .p13().set_bit()
        )
    };

    // okay, we're ready
}


macro_rules! define_cs_functions {
    ($v:vis $low_name:ident, $high_name:ident, $pio:ident, $pin:ident) => {
        #[inline]
        $v fn $low_name(peripherals: &mut Peripherals) {
            unsafe {
                peripherals.$pio.odr.write_with_zero(|w| w
                    .$pin().set_bit()
                )
            };
        }

        #[inline]
        $v fn $high_name(peripherals: &mut Peripherals) {
            unsafe {
                peripherals.$pio.oer.write_with_zero(|w| w
                    .$pin().set_bit()
                )
            };
        }
    };
}

define_cs_functions!(pub cs1_low, cs1_high, PIOB, p14);
define_cs_functions!(pub cs2_low, cs2_high, PIOC, p17);
define_cs_functions!(pub cs3_low, cs3_high, PIOC, p18);
define_cs_functions!(clock_low, clock_high, PIOB, p21);
define_cs_functions!(copi_low, copi_high, PIOC, p12);

fn switch_to_cs1(peripherals: &mut Peripherals) {
    cs2_high(peripherals);
    cs3_high(peripherals);
    cs1_low(peripherals);
}
fn switch_to_cs2(peripherals: &mut Peripherals) {
    cs1_high(peripherals);
    cs3_high(peripherals);
    cs2_low(peripherals);
}
fn switch_to_cs3(peripherals: &mut Peripherals) {
    cs1_high(peripherals);
    cs2_high(peripherals);
    cs3_low(peripherals);
}

fn bitbang<const NOPCOUNT: usize>(peripherals: &mut Peripherals, write_byte: u8) -> u8 {
    let mut read_byte: u8 = 0;

    for i in 0..8 {
        // we send MSB first
        if write_byte & (1 << (7-i)) == 0 {
            // low
            copi_low(peripherals);
        } else {
            // high
            copi_high(peripherals);
        }

        // nop!
        for _ in 0..NOPCOUNT {
            nop();
        }

        // set clock up!
        clock_high(peripherals);

        // nop!
        for _ in 0..NOPCOUNT {
            nop();
        }

        // read a bit (MSB first)
        read_byte <<= 1;
        if peripherals.PIOC.pdsr.read().p13().bit_is_set() {
            read_byte |= 1;
        }
    }

    read_byte
}
