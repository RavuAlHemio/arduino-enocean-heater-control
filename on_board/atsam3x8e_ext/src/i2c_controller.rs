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
    fn get_register_block(peripherals: &mut Peripherals) -> *const atsam3x8e::twi0::RegisterBlock;

    /// Resets the I<sup>2</sup>C controller peripheral.
    fn reset(peripherals: &mut Peripherals) {
        unsafe {
            Self::get_register_block(peripherals)
                .cr.write_with_zero(|w| w
                    .swrst().set_bit()
                )
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

    fn get_register_block(peripherals: &mut Peripherals) -> *const atsam3x8e::twi0::RegisterBlock {
        atsam3x8e::TWI0::ptr() as *const atsam3x8e::twi0::RegisterBlock
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

    fn get_register_block(peripherals: &mut Peripherals) -> *const atsam3x8e::twi0::RegisterBlock {
        atsam3x8e::TWI1::ptr() as *const atsam3x8e::twi0::RegisterBlock
    }
}
