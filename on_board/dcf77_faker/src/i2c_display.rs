use core::time::Duration;

use atsam3x8e::Peripherals;
use atsam3x8e_ext::sam_pin;
use atsam3x8e_ext::tick::delay;


const LONG_DELAY: Duration = Duration::from_micros(2_160);
const SHORT_DELAY: Duration = Duration::from_nanos(52_600);


/// Common trait for I2C character-based liquid crystal displays consisting of:
///
/// * PCF8574 I2C-to-GPIO chip
/// * HD44780 LCD controller
///
/// The following PCF8574-to-HD44780 pinout is assumed:
///
/// | PCF8574 | HD44780     |
/// | ------- | ----------- |
/// | P7      | D7          |
/// | P6      | D6          |
/// | P5      | D5          |
/// | P4      | D4          |
/// | P3      | (backlight) |
/// | P2      | E           |
/// | P1      | R/~W        |
/// | P0      | RS          |
pub trait I2cDisplay {
    /// Obtains the register block used to control the relevant TWI controller.
    ///
    /// It might be necessary to cast the pointer to `atsam3x8e::twi0::RegisterBlock` if a different
    /// TWI controller than TWI0 is being targeted. Since the structure of each register block is
    /// the same, this should not be an issue in practice.
    fn get_twi_registers(peripherals: &mut Peripherals) -> &mut atsam3x8e::twi0::RegisterBlock;

    /// Enables the system clock being transferred to the relevant TWI controller.
    fn enable_system_clock_to_twi(peripherals: &mut Peripherals);

    /// Transfers the I/O pins to the TWI controller.
    fn grab_io_pins(peripherals: &mut Peripherals);

    /// Obtains the address of the display on the I2C bus.
    fn display_address(&self) -> u8;

    /// Whether the user wants the backlight of the display turned on.
    fn wants_backlight(&self) -> bool;

    /// Initializes the Two-Wire Interface controller.
    fn setup_twi(peripherals: &mut Peripherals) {
        let mut twi = Self::get_twi_registers(peripherals);

        // set up clock

        // SAM3X8E datasheet says:
        // T = ((C_DIV * 2**CKDIV) + 4) * TMCK (where TMCK = 1/MCK)
        // let's transform that formula
        // T = ((C_DIV * 2**CKDIV) + 4) / MCK
        // T * MCK = (C_DIV * 2**CKDIV) + 4
        // T * MCK - 4 = (C_DIV * 2**CKDIV)

        // PCF8574 datasheet says "min. 4.7µs LOW, min. 4µs HIGH"
        // let's simply take 4.7µs as the value for both
        // 4.7µs = 4.7 * 10**-6 s = 4.7 / 1_000_000 s = 47 / 10_000_000 s
        let mut full_divisor: u32 = atsam3x8e_ext::setup::CHIP_FREQ_CPU_MAX * 47 / 10_000_000 - 4;
        let mut power_of_two: u8 = 0;
        while full_divisor > 255 {
            full_divisor /= 2;
            power_of_two += 1;
        }
        let full_divisor_u8: u8 = full_divisor.try_into().unwrap();

        twi.cwgr.modify(|_, w| w
            .cldiv().variant(full_divisor_u8)
            .chdiv().variant(full_divisor_u8)
            .ckdiv().variant(power_of_two)
        );

        // disable slave mode, enable master mode
        unsafe {
            twi.cr.write_with_zero(|w| w
                .svdis().set_bit()
                .msen().set_bit()
            )
        };
    }

    /// Performs low-level transmission of the given data to the current target address.
    fn low_level_transmit(peripherals: &mut Peripherals, data: &[u8]) {
        let mut twi = Self::get_twi_registers(peripherals);

        // wait until we are ready to transfer
        while twi.sr.read().txrdy().bit_is_clear() {
        }

        // send off the data
        for b in data {
            // send a byte
            twi.thr.write(|w| w
                .txdata().variant(*b)
            );

            // wait until the byte has been transferred
            while twi.sr.read().txrdy().bit_is_clear() {
            }
        }

        // send stop signal
        unsafe {
            twi.cr.write_with_zero(|w| w
                .stop().set_bit()
            )
        };

        // wait until the transmission has been completed
        while twi.sr.read().txcomp().bit_is_clear() {
        }
    }

    /// Sets the target address to which to transfer data.
    fn set_target(&self, peripherals: &mut Peripherals) {
        let mut twi = Self::get_twi_registers(peripherals);

        // focus our attention on the display
        twi.mmr.modify(|_, w| w
            .iadrsz().none() // the address only has the regular 7 bits
            .mread().clear_bit() // we will be writing, not reading
            .dadr().variant(self.display_address() & 0b0111_1111)
        );
    }

    /// Transmits a nibble (4 bits) of data.
    fn transmit_nibble(&self, peripherals: &mut Peripherals, nibble: u8, rs: bool) {
        // pin mapping (bits 7 to 0):
        // D7, D6, D5, D4, BL, E, RW, RS
        // BL = backlight
        // E = "read the data now" (we pulse this for a bit)
        // RW = Read=1, Write=0 (always 0 for transmissions)
        // RS = Register Select (0 for command, 1 for data)

        // prepare the byte to transmit, with E low
        let backlight_flag = if self.wants_backlight() { 0b0000_1000 } else { 0b0000_0000 };
        let rs_flag = if rs { 0b0000_0001 } else { 0b0000_0000 };
        let mut transmit_me = (nibble << 4) | backlight_flag | rs_flag;

        // send (with E low)
        Self::low_level_transmit(peripherals, &[transmit_me]);
        delay(Duration::from_nanos(500));

        // pull E high
        transmit_me |= 0b0000_0100;

        // send (with E high)
        Self::low_level_transmit(peripherals, &[transmit_me]);
        delay(Duration::from_nanos(500));

        // pull E low
        transmit_me &= 0b1111_1011;

        // send (with E low)
        Self::low_level_transmit(peripherals, &[transmit_me]);
        delay(Duration::from_nanos(500));
    }

    fn transmit_byte(&self, peripherals: &mut Peripherals, byte: u8, rs: bool) {
        // in 4-bit mode, the upper nibble is transmitted first

        // transmit the upper nibble
        self.transmit_nibble(peripherals, byte >> 4, rs);

        // transmit the lower nibble
        self.transmit_nibble(peripherals, byte & 0xF, rs);
    }

    fn short_delay() {
        delay(SHORT_DELAY);
    }

    fn long_delay() {
        delay(LONG_DELAY);
    }
}


/// I2C LCD on Two-Wire Interface 0.
pub struct I2cDisplayTwi0 {
    display_address: u8,
    wants_backlight: bool,
}
impl I2cDisplayTwi0 {
    pub const fn new(
        display_address: u8,
        wants_backlight: bool,
    ) -> Self {
        Self {
            display_address,
            wants_backlight,
        }
    }
}
impl I2cDisplay for I2cDisplayTwi0 {
    #[inline]
    fn get_twi_registers(peripherals: &mut Peripherals) -> &mut atsam3x8e::twi0::RegisterBlock {
        let ptr = atsam3x8e::TWI0::PTR as *mut atsam3x8e::twi0::RegisterBlock;
        unsafe { &mut *ptr }
    }

    fn enable_system_clock_to_twi(peripherals: &mut Peripherals) {
        unsafe {
            peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
                .pid22().set_bit()
            )
        };
    }

    fn grab_io_pins(peripherals: &mut Peripherals) {
        // TWI0: SCL = PA18 peripheral A, SDA = PA17 peripheral A
        sam_pin!(disable_io, peripherals, PIOA, p17, p18);
        sam_pin!(make_output, peripherals, PIOA, p17, p18);
        sam_pin!(peripheral_ab, peripherals, PIOA, p17, clear_bit, p18, clear_bit);
    }

    #[inline] fn display_address(&self) -> u8 { self.display_address }
    #[inline] fn wants_backlight(&self) -> bool { self.wants_backlight }
}

/// I2C LCD on Two-Wire Interface 1.
pub struct I2cDisplayTwi1 {
    display_address: u8,
    wants_backlight: bool,
}
impl I2cDisplayTwi1 {
    pub const fn new(
        display_address: u8,
        wants_backlight: bool,
    ) -> Self {
        Self {
            display_address,
            wants_backlight,
        }
    }
}
impl I2cDisplay for I2cDisplayTwi1 {
    #[inline]
    fn get_twi_registers(peripherals: &mut Peripherals) -> &mut atsam3x8e::twi0::RegisterBlock {
        let ptr = atsam3x8e::TWI1::PTR as *mut atsam3x8e::twi0::RegisterBlock;
        unsafe { &mut *ptr }
    }

    fn enable_system_clock_to_twi(peripherals: &mut Peripherals) {
        unsafe {
            peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
                .pid23().set_bit()
            )
        };
    }

    fn grab_io_pins(peripherals: &mut Peripherals) {
        // TWI1: SCL = PB13 peripheral A, SDA = PB12 peripheral A
        sam_pin!(disable_io, peripherals, PIOB, p12, p13);
        sam_pin!(make_output, peripherals, PIOB, p12, p13);
        sam_pin!(peripheral_ab, peripherals, PIOB, p12, clear_bit, p13, clear_bit);
    }

    #[inline] fn display_address(&self) -> u8 { self.display_address }
    #[inline] fn wants_backlight(&self) -> bool { self.wants_backlight }
}
