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


use crate::{Peripherals, sam_pin};
use crate::atsam3x8e_ext::nop;


/// Set up the SPI pins, assuming the role of controller (as opposed to peripheral).
pub fn setup_pins_controller(peripherals: &mut Peripherals) {
    // take over pins from peripherals
    sam_pin!(enable_io, peripherals, PIOB, p14, p21);
    sam_pin!(enable_io, peripherals, PIOC, p12, p13, p17, p18);

    // set pins as output...
    sam_pin!(make_output, peripherals, PIOB, p14, p21);
    sam_pin!(make_output, peripherals, PIOC, p12, p17, p18);

    // ... except CIPO (input)
    sam_pin!(make_input, peripherals, PIOC, p13);

    // enable pull-up resistor for CIPO
    // (this is the controller's job)
    sam_pin!(enable_pullup, peripherals, PIOC, p13);

    // set clock and COPI down
    sam_pin!(set_low, peripherals, PIOB, p21);
    sam_pin!(set_low, peripherals, PIOC, p12);

    // set all CS pins up
    sam_pin!(set_high, peripherals, PIOB, p14);
    sam_pin!(set_high, peripherals, PIOC, p17, p18);

    // okay, we're ready
}


macro_rules! define_cs_functions {
    ($v:vis $low_name:ident, $high_name:ident, $pio:ident, $pin:ident) => {
        #[inline]
        $v fn $low_name(peripherals: &mut Peripherals) {
            sam_pin!(set_low, peripherals, $pio, $pin);
        }

        #[inline]
        $v fn $high_name(peripherals: &mut Peripherals) {
            sam_pin!(set_high, peripherals, $pio, $pin);
        }
    };
}

define_cs_functions!(pub cs1_low, cs1_high, PIOB, p14);
define_cs_functions!(pub cs2_low, cs2_high, PIOC, p17);
define_cs_functions!(pub cs3_low, cs3_high, PIOC, p18);
define_cs_functions!(clock_low, clock_high, PIOB, p21);
define_cs_functions!(copi_low, copi_high, PIOC, p12);

/// Sends eight bits worth of information and reads back eight bits worth of information from the
/// SPI bus.
///
/// Assumes the output byte is sent most-significant-bit first and the input byte is received
/// most-significant-bit first.
pub fn bitbang<const NOPCOUNT: usize>(peripherals: &mut Peripherals, write_byte: u8) -> u8 {
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
        if sam_pin!(is_up, peripherals, PIOC, p13) {
            read_byte |= 1;
        }

        // set clock low!
        clock_low(peripherals);

        // nop!
        for _ in 0..NOPCOUNT {
            nop();
        }
    }

    read_byte
}
