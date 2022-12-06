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

    /// Disables receives and transmits going through the PDC (Peripheral DMA Controller).
    fn disable_dma(peripherals: &mut Peripherals) {
        unsafe {
            Self::get_register_block(peripherals)
                .ptcr.write_with_zero(|w| w
                    .rxtdis().set_bit()
                    .txtdis().set_bit()
                )
        }
    }

    /// Outputs the current TWI state to UART.
    fn output_state_to_uart(twi: &atsam3x8e::twi0::RegisterBlock) {
        let state = twi.sr.read().bits();
        crate::uart::send_stolen(b"TWI state:");
        if state & (1 << 0) != 0 { crate::uart::send_stolen(b" TXCOMP"); }
        if state & (1 << 1) != 0 { crate::uart::send_stolen(b" RXRDY"); }
        if state & (1 << 2) != 0 { crate::uart::send_stolen(b" TXRDY"); }
        if state & (1 << 3) != 0 { crate::uart::send_stolen(b" SVREAD"); }
        if state & (1 << 4) != 0 { crate::uart::send_stolen(b" SVACC"); }
        if state & (1 << 5) != 0 { crate::uart::send_stolen(b" GACC"); }
        if state & (1 << 6) != 0 { crate::uart::send_stolen(b" OVRE"); }
        // 7 is unused
        if state & (1 << 8) != 0 { crate::uart::send_stolen(b" NACK"); }
        if state & (1 << 9) != 0 { crate::uart::send_stolen(b" Arbalest"); }
        if state & (1 << 10) != 0 { crate::uart::send_stolen(b" SCLWS"); }
        if state & (1 << 11) != 0 { crate::uart::send_stolen(b" EOSACC"); }
        if state & (1 << 12) != 0 { crate::uart::send_stolen(b" ENDRX"); }
        if state & (1 << 13) != 0 { crate::uart::send_stolen(b" ENDTX"); }
        if state & (1 << 14) != 0 { crate::uart::send_stolen(b" RXBUFF"); }
        if state & (1 << 15) != 0 { crate::uart::send_stolen(b" TXBUFE"); }
        crate::uart::send_stolen(b"\r\n");
    }

    /// Sets the speed for the I<sup>2</sup>C communication.
    fn set_speed(peripherals: &mut Peripherals, i2c_speed: u32, clock_speed: u32) {
        // standard I2C speed values like "100 kHz" describe one LOW + one HIGH clock period
        // => multiply i2c_speed by 2 to obtain the value for a single period

        let mut delay_value = clock_speed / (i2c_speed*2) - 4;
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

    /// Surrender both the controller and the peripheral role.
    fn surrender_roles(peripherals: &mut Peripherals) {
        let twi = Self::get_register_block(peripherals);
        unsafe {
            twi.cr.write_with_zero(|w| w
                .msdis().set_bit()
                .svdis().set_bit()
            )
        };
    }

    /// Write data to an address via I<sup>2</sup>C.
    fn write<D: IntoIterator<Item = u8>>(peripherals: &mut Peripherals, address: u8, data: D) {
        let twi = Self::get_register_block(peripherals);

        // wait until the TWI controller is ready to switch modes
        while twi.sr.read().txcomp().bit_is_clear() {
        }

        unsafe {
            twi.cr.write_with_zero(|w| w
                .msen().set_bit()
                .svdis().set_bit()
            )
        };

        unsafe {
            twi.mmr.modify(|_, w| w
                .dadr().variant(address)
                .mread().clear_bit()
                .iadrsz().none()
            )
        };

        let mut data_peek = data.into_iter().peekable();
        while let Some(b) = data_peek.next() {
            // feed the byte to the TWI controller for sending
            twi.thr.write(|w| w
                .txdata().variant(b)
            );

            if data_peek.peek().is_none() {
                // we have enqueued the last byte; tell the peripheral that we will be stopping now
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
    fn read(peripherals: &mut Peripherals, address: u8, buffer: &mut [u8]) {
        let twi = Self::get_register_block(peripherals);

        // wait until the TWI controller is ready to switch modes
        while twi.sr.read().txcomp().bit_is_clear() {
        }

        unsafe {
            twi.cr.write_with_zero(|w| w
                .msen().set_bit()
                .svdis().set_bit()
            )
        };

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

        let mut buffer_index = 0;
        while buffer_index < buffer.len() {
            // last byte to read?
            if buffer_index == buffer.len() - 1 {
                // yes
                unsafe {
                    twi.cr.write_with_zero(|w| w
                        .stop().set_bit()
                    )
                };
            }

            // wait until a byte has been received
            while twi.sr.read().rxrdy().bit_is_clear() {
            }
            let received_byte = twi.rhr.read().rxdata().bits();
            buffer[buffer_index] = received_byte;
            buffer_index += 1;
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
        // stop clock
        unsafe {
            peripherals.PMC.pmc_pcdr0.write_with_zero(|w| w
                .pid22().set_bit()
            )
        };

        // no division (supported by CAN only on the SAM3X8E)
        unsafe {
            peripherals.PMC.pmc_pcr.write_with_zero(|w| w
                .pid().variant(22)
                .cmd().set_bit() // write
                .div().periph_div_mck() // no division (divide by 1)
            )
        };

        // start clock
        unsafe {
            peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
                .pid22().set_bit()
            )
        };

        // wait for the clock to start
        while peripherals.PMC.pmc_pcsr0.read().pid22().bit_is_clear() {
        }
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
        // stop clock
        unsafe {
            peripherals.PMC.pmc_pcdr0.write_with_zero(|w| w
                .pid23().set_bit()
            )
        };

        // no division (supported by CAN only on the SAM3X8E)
        unsafe {
            peripherals.PMC.pmc_pcr.write_with_zero(|w| w
                .pid().variant(23)
                .cmd().set_bit() // write
                .div().periph_div_mck() // no division (divide by 1)
            )
        };

        // start clock
        unsafe {
            peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
                .pid23().set_bit()
            )
        };

        // wait for the clock to start
        while peripherals.PMC.pmc_pcsr0.read().pid23().bit_is_clear() {
        }
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
