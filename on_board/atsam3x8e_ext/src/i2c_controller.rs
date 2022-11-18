//! Implementation of I<sup>2</sup>C controller (master) operations.


use atsam3x8e::Peripherals;

use crate::sam_pin;


/// Operations common to all I<sup>2</sup>C controllers.
pub trait I2cController {
    /// Enables the clock to be passed to the I<sup>2</sup>C controller peripheral.
    fn enable_clock(peripherals: &mut Peripherals);

    /// Disables the clock being passed to the I<sup>2</sup>C controller peripheral.
    fn disable_clock(peripherals: &mut Peripherals);

    /// Hand over the relevant pins to the I<sup>2</sup>C controller peripheral.
    fn setup_pins(peripherals: &mut Peripherals);

    /// Obtains the register block of the I<sup>2</sup>C controller peripheral.
    ///
    /// Note that this might require casting a different register block to the TWI0 register block;
    /// this should not be an issue in practice as the register blocks all have the same layout.
    fn get_register_block(peripherals: &mut Peripherals) -> &atsam3x8e::twi0::RegisterBlock;

    /// Resets the I<sup>2</sup>C controller peripheral.
    fn reset(peripherals: &mut Peripherals) {
        unsafe {
            Self::get_register_block(peripherals)
                .cr.write_with_zero(|w| w
                    .swrst().set_bit()
                )
        }
    }

    /// Sets the speed for the I<sup>2</sup>C communication.
    fn set_speed(peripherals: &mut Peripherals, i2c_speed: u32, clock_speed: u32) {
        let mut delay_value = clock_speed / i2c_speed - 4;
        let mut power = 0;
        while delay_value > 0xFF {
            delay_value /= 2;
            power += 1;
        }

        Self::get_register_block(peripherals)
            .cwgr.modify(|_, w| w
                .cldiv().variant(delay_value as u8)
                .chdiv().variant(delay_value as u8)
                .ckdiv().variant(power)
            );
    }

    /// Grab the controller role.
    fn become_controller(peripherals: &mut Peripherals) {
        unsafe {
            Self::get_register_block(peripherals)
                .cr.write_with_zero(|w| w
                    .msen().set_bit()
                    .svdis().set_bit()
                )
        };
    }

    /// Write data to an address via I<sup>2</sup>C.
    fn write(peripherals: &mut Peripherals, address: u8, data: &[u8]) {
        let twi = Self::get_register_block(peripherals);

        unsafe {
            twi.mmr.modify(|_, w| w
                .dadr().variant(address)
                .mread().clear_bit()
                .iadrsz().none()
            )
        };

        // wait until the TWI controller is ready to take the first byte
        while twi.sr.read().txrdy().bit_is_clear() {
        }

        for (i, b) in data.into_iter().enumerate() {
            // feed the byte to the TWI controller for sending
            twi.thr.write(|w| w
                .txdata().variant(*b)
            );

            if i == data.len() - 1 {
                // we are sending the last byte; tell the peripheral that we will be stopping now
                unsafe {
                    twi.cr.write_with_zero(|w| w
                        .stop().set_bit()
                    )
                }
            }

            // wait until the TWI controller has "taken" the byte
            while twi.sr.read().txrdy().bit_is_clear() {
            }
        }

        // wait until the TWI controller is fully done sending
        while twi.sr.read().txcomp().bit_is_clear() {
        }
    }

    /// Read data from an address via I<sup>2</sup>C.
    fn read<F: FnMut(u8) -> bool>(peripherals: &mut Peripherals, address: u8, mut handle_byte: F) {
        let twi = Self::get_register_block(peripherals);

        unsafe {
            twi.mmr.modify(|_, w| w
                .dadr().variant(address)
                .mread().set_bit()
                .iadrsz().none()
            )
        };

        // give me what you got
        unsafe {
            twi.cr.write_with_zero(|w| w
                .start().set_bit()
            )
        };

        // wait until a byte has been received
        while twi.sr.read().rxrdy().bit_is_clear() {
        }
        let received_byte = twi.rhr.read().rxdata().bits();
        let keep_going = handle_byte(received_byte);
        if !keep_going {
            // signal that this is the last byte we want
            unsafe {
                twi.cr.write_with_zero(|w| w
                    .stop().set_bit()
                )
            };
        }

        // wait until the TWI controller is fully done receiving
        while twi.sr.read().txcomp().bit_is_clear() {
        }
    }
}

/// An I<sup>2</sup>C controller on Two-Wire Interface 0 (peripheral 22).
///
/// Pinout:
/// 
/// | role  | pin  | peripheral |
/// | ----- | ---- | ---------- |
/// | clock | PA18 | A          |
/// | data  | PA17 | A          |
pub struct Twi0I2cController;
impl I2cController for Twi0I2cController {
    fn enable_clock(peripherals: &mut Peripherals) {
        unsafe {
            peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
                .pid22().set_bit()
            )
        };
    }

    fn disable_clock(peripherals: &mut Peripherals) {
        unsafe {
            peripherals.PMC.pmc_pcdr0.write_with_zero(|w| w
                .pid22().set_bit()
            )
        };
    }

    fn setup_pins(peripherals: &mut Peripherals) {
        sam_pin!(disable_io, peripherals, PIOA, p17, p18);
        sam_pin!(peripheral_ab, peripherals, PIOA, p17, clear_bit, p18, clear_bit);
    }

    fn get_register_block(peripherals: &mut Peripherals) -> &atsam3x8e::twi0::RegisterBlock {
        unsafe {
            &*(atsam3x8e::TWI0::ptr() as *const atsam3x8e::twi0::RegisterBlock)
        }
    }
}

/// An I<sup>2</sup>C controller on Two-Wire Interface 1 (peripheral 23).
///
/// Pinout:
/// 
/// | role  | pin  | peripheral |
/// | ----- | ---- | ---------- |
/// | clock | PB13 | A          |
/// | data  | PB12 | A          |
pub struct Twi1I2cController;
impl I2cController for Twi1I2cController {
    fn enable_clock(peripherals: &mut Peripherals) {
        unsafe {
            peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
                .pid23().set_bit()
            )
        };
    }

    fn disable_clock(peripherals: &mut Peripherals) {
        unsafe {
            peripherals.PMC.pmc_pcdr0.write_with_zero(|w| w
                .pid23().set_bit()
            )
        };
    }

    fn setup_pins(peripherals: &mut Peripherals) {
        sam_pin!(disable_io, peripherals, PIOB, p12, p13);
        sam_pin!(peripheral_ab, peripherals, PIOB, p12, clear_bit, p13, clear_bit);
    }

    fn get_register_block(peripherals: &mut Peripherals) -> &atsam3x8e::twi0::RegisterBlock {
        unsafe {
            &*(atsam3x8e::TWI1::ptr() as *const atsam3x8e::twi0::RegisterBlock)
        }
    }
}
