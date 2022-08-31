//! Code for the built-in USARTs, communicating via the following pins:
//!
//! | USART | RXD      | TXD      | SCK      | CTS      | RTS      |
//! |-------|----------|----------|----------|----------|----------|
//! | 0     | PA10 (A) | PA11 (A) | PA17 (B) | PB26 (A) | PB25 (A) |
//! | 1     | PA12 (A) | PA13 (A) | PA16 (A) | PA15 (A) | PA14 (A) |
//! | 2     | PB21 (A) | PB20 (A) | PB24 (A) | PB23 (A) | PB22 (A) |
//! | 3     | PD5 (B)  | PD4 (B)  | PE15 (B) | PF4 (A)  | PF5 (A)  |


use atsam3x8e::Peripherals;
use atsam3x8e::usart0::RegisterBlock as UsartRegisterBlock;

use crate::sam_pin;
use crate::atsam3x8e_ext::setup::SystemCoreClock;


pub trait Usart {
    /// Configures the relevant pins on the Parallel I/O controller for use with the USART.
    fn set_pins(peripherals: &mut Peripherals);

    /// Returns the peripheral ID for this USART.
    fn peripheral_id() -> u8;

    /// Returns the address of the USART register block in memory.
    fn register_address() -> usize;

    /// Returns a reference to the USART register block.
    #[inline]
    fn register_block(peripherals: &mut Peripherals) -> &mut UsartRegisterBlock {
        unsafe { &mut *(Self::register_address() as *mut UsartRegisterBlock) }
    }

    /// Enables the clock for this USART.
    fn enable_clock(peripherals: &mut Peripherals) {
        let mut peripheral_id = Self::peripheral_id();

        if peripheral_id >= 64 {
            return;
        }

        if peripheral_id >= 32 {
            peripheral_id -= 32;
            if peripherals.PMC.pmc_pcsr1.read().bits() & (1 << peripheral_id) == 0 {
                unsafe {
                    peripherals.PMC.pmc_pcer1.write_with_zero(|w|
                        w.bits(1 << peripheral_id)
                    )
                };
            }
        } else {
            if peripherals.PMC.pmc_pcsr0.read().bits() & (1 << peripheral_id) == 0 {
                unsafe {
                    peripherals.PMC.pmc_pcer0.write_with_zero(|w|
                        w.bits(1 << peripheral_id)
                    )
                };
            }
        }
    }

    /// Disables the clock for this USART.
    fn disable_clock(peripherals: &mut Peripherals) {
        let mut peripheral_id = Self::peripheral_id();

        if peripheral_id >= 64 {
            return;
        }

        if peripheral_id >= 32 {
            peripheral_id -= 32;
            if peripherals.PMC.pmc_pcsr1.read().bits() & (1 << peripheral_id) != 0 {
                unsafe {
                    peripherals.PMC.pmc_pcdr1.write_with_zero(|w|
                        w.bits(1 << peripheral_id)
                    )
                };
            }
        } else {
            if peripherals.PMC.pmc_pcsr0.read().bits() & (1 << peripheral_id) != 0 {
                unsafe {
                    peripherals.PMC.pmc_pcdr0.write_with_zero(|w|
                        w.bits(1 << peripheral_id)
                    )
                };
            }
        }
    }

    /// Disables the Peripheral DMA Channels.
    fn disable_pdc(peripherals: &mut Peripherals) {
        // disable PDC channels
        unsafe {
            Self::register_block(peripherals)
                .ptcr.write(|w| w
                    .rxtdis().set_bit()
                    .txtdis().set_bit()
                )
        };
    }

    /// Resets and disables the USART transmitter and receiver.
    fn reset_and_disable(peripherals: &mut Peripherals) {
        unsafe {
            Self::register_block(peripherals)
                .cr().write_with_zero(|w| w
                    .rsttx().set_bit()
                    .rstrx().set_bit()
                    .txdis().set_bit()
                    .rxdis().set_bit()
                )
        };
    }

    /// Resets the USART receiver.
    fn reset_receiver(peripherals: &mut Peripherals) {
        unsafe {
            Self::register_block(peripherals)
                .cr().write_with_zero(|w| w
                    .rstrx().set_bit()
                )
        };
    }

    /// Sets the state of the USART transmitter or receiver.
    fn set_rxtx_state(peripherals: &mut Peripherals, tx_enable: bool, rx_enable: bool) {
        unsafe {
            Self::register_block(peripherals)
                .cr().write_with_zero(|w| {
                    let w_tx = if tx_enable {
                        w.txen().set_bit()
                    } else {
                        w.txdis().set_bit()
        };
                    if rx_enable {
                        w.rxen().set_bit()
                    } else {
                        w.rxdis().set_bit()
    }
            })
        };
    }

    /// Sets standard parameters on the USART.
    fn set_standard_params(peripherals: &mut Peripherals) {
        // 8N1
        unsafe {
            Self::register_block(peripherals)
                .mr().write_with_zero(|w| w
                    .usart_mode().normal()
                    .usclks().mck() // master clock at full speed
                    .chmode().normal()
                    .chrl()._8_bit()
                    .par().no()
                    .nbstop()._1_bit()
                )
        };
    }

    /// Sets the baud rate.
    fn set_baud_rate(clock: &SystemCoreClock, peripherals: &mut Peripherals, bps: u32) {
        // baud_rate = selected_clock/(16*cd)
        // cd = selected_clock/(16*baud_rate)

        let clock_divider_u32 = (clock.clock_speed / 16) / bps;
        let clock_divider: u16 = clock_divider_u32.try_into().unwrap();
        let mut clock_divider_hex = [0u8; 2*2];
        crate::hex_dump(&clock_divider.to_be_bytes(), &mut clock_divider_hex);
        crate::uart::send(peripherals, b"USART clock divider: 0x");
        crate::uart::send(peripherals, &clock_divider_hex);
        crate::uart::send(peripherals, b"\r\n");

        unsafe {
            Self::register_block(peripherals)
                .brgr.write_with_zero(|w| w
                    .cd().variant(clock_divider)
                    .fp().variant(0)
                )
        };
    }

    /// Disable all USART-related interrupts.
    fn disable_interrupts(peripherals: &mut Peripherals) {
        unsafe {
            Self::register_block(peripherals)
                .idr().write_with_zero(|w| w
                    .bits(0xFFFFFFFF)
                )
        };
    }

    /// Sends the given data.
    fn transmit(peripherals: &mut Peripherals, data: &[u8]) {
        crate::uart::send(peripherals, b"PREPARING FOR TRANSMISSION\r\n");
        for b in data {
            while Self::register_block(peripherals).csr().read().txrdy().bit_is_clear() {
                // transmitter is not ready; wait...
            }
            crate::uart::send(peripherals, b"b");
            unsafe {
                Self::register_block(peripherals)
                    .thr.write_with_zero(|w| w
                        .txchr().variant((*b).into())
                        .txsynh().clear_bit()
                    )
            };
        }
        crate::uart::send(peripherals, b"\r\nFLUSHING\r\n");
        while Self::register_block(peripherals).csr().read().txempty().bit_is_clear() {
            // transmitter is not empty; wait...
        }
        crate::uart::send(peripherals, b"SENT\r\n");
    }

    /// Receives enough data to fill the buffer.
    fn receive_exact(peripherals: &mut Peripherals, buffer: &mut [u8]) {
        let mut i = 0;
        crate::uart::send(peripherals, b"RECEIVING\r\n");
        while i < buffer.len() {
            while Self::register_block(peripherals).csr().read().rxrdy().bit_is_clear() {
                // receiver is not ready; wait...
            }
            buffer[i] = (Self::register_block(peripherals).rhr.read().rxchr().bits() & 0x00FF) as u8;
            i += 1;

            // TODO: handle errors
            unsafe {
                Self::register_block(peripherals)
                    .cr().write_with_zero(|w| w
                        .rststa().set_bit()
                    )
            };
        }
        crate::uart::send(peripherals, b"RECEIVED\r\n");
    }
}


pub struct Usart0;
impl Usart for Usart0 {
    fn set_pins(peripherals: &mut Peripherals) {
        // disable regular I/O on the USART0 pins
        sam_pin!(disable_io, peripherals, PIOA, p10, p11, p17);
        sam_pin!(disable_io, peripherals, PIOB, p25, p26);

        // choose the correct peripheral for each USART0 pin
        sam_pin!(
            peripheral_ab, peripherals, PIOA
            , p10, clear_bit
            , p11, clear_bit
            , p17, set_bit
        );
        sam_pin!(
            peripheral_ab, peripherals, PIOB
            , p25, clear_bit
            , p26, clear_bit
        );
    }

    #[inline] fn peripheral_id() -> u8 { 17 }
    #[inline] fn register_address() -> usize { 0x4009_8000 }
}


pub struct Usart1;
impl Usart for Usart1 {
    fn set_pins(peripherals: &mut Peripherals) {
        sam_pin!(disable_io, peripherals, PIOA, p12, p13, p14, p15, p16);

        sam_pin!(
            peripheral_ab, peripherals, PIOA
            , p12, clear_bit
            , p13, clear_bit
            , p14, clear_bit
            , p15, clear_bit
            , p16, clear_bit
        );
    }

    #[inline] fn peripheral_id() -> u8 { 18 }
    #[inline] fn register_address() -> usize { 0x4009_C000 }
}


pub struct Usart2;
impl Usart for Usart2 {
    fn set_pins(peripherals: &mut Peripherals) {
        sam_pin!(disable_io, peripherals, PIOB, p20, p21, p22, p23, p24);

        sam_pin!(
            peripheral_ab, peripherals, PIOB
            , p20, clear_bit
            , p21, clear_bit
            , p22, clear_bit
            , p23, clear_bit
            , p24, clear_bit
        );
    }

    #[inline] fn peripheral_id() -> u8 { 19 }
    #[inline] fn register_address() -> usize { 0x400A_0000 }
}


pub struct Usart3;
impl Usart for Usart3 {
    fn set_pins(peripherals: &mut Peripherals) {
        sam_pin!(disable_io, peripherals, PIOD, p4, p5);

        // SCK, CTS and RTS are not available on USART3 on the Arduino Due version of the SAM3X
        // (parallel controllers PIOE/PIOF do not exist)

        sam_pin!(
            peripheral_ab, peripherals, PIOD
            , p4, set_bit
            , p5, set_bit
        );
    }

    #[inline] fn peripheral_id() -> u8 { 20 }
    #[inline] fn register_address() -> usize  { 0x400A_4000 }
}
