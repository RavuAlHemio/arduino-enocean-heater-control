//! Code for the built-in USARTs, communicating via the following pins:
//!
//! | USART | RXD      | TXD      | SCK      | CTS      | RTS      |
//! |-------|----------|----------|----------|----------|----------|
//! | 0     | PA10 (A) | PA11 (A) | PA17 (B) | PB26 (A) | PB25 (A) |
//! | 1     | PA12 (A) | PA13 (A) | PA16 (A) | PA15 (A) | PA14 (A) |
//! | 2     | PB21 (A) | PB20 (A) | PB24 (A) | PB23 (A) | PB22 (A) |
//! | 3     | PD5 (B)  | PD4 (B)  | PE15 (B) | PF4 (A)  | PF5 (A)  |


use atsam3x8e::{Interrupt, interrupt, Peripherals};
use atsam3x8e::usart0::RegisterBlock as UsartRegisterBlock;
use atsam3x8e_ext::sam_pin;
use atsam3x8e_ext::setup::SystemCoreClock;
use atsam3x8e_ext::uart;
use buildingblocks::max_array::MaxArray;
use buildingblocks::ring_buffer::RingBuffer;
use cortex_m::interrupt as cortex_interrupt;
use cortex_m::peripheral::NVIC;


const RING_BUFFER_SIZE: usize = 512;


static mut USART0_BUFFER: Option<RingBuffer<u8, RING_BUFFER_SIZE>> = None;
static mut USART1_BUFFER: Option<RingBuffer<u8, RING_BUFFER_SIZE>> = None;
static mut USART2_BUFFER: Option<RingBuffer<u8, RING_BUFFER_SIZE>> = None;
static mut USART3_BUFFER: Option<RingBuffer<u8, RING_BUFFER_SIZE>> = None;


pub trait Usart {
    /// Configures the relevant pins on the Parallel I/O controller for use with the USART.
    fn set_pins(peripherals: &mut Peripherals);

    /// Returns the peripheral ID for this USART.
    fn peripheral_id() -> u8;

    /// Returns the address of the USART register block in memory.
    fn register_address() -> usize;

    /// Returns a mutable reference to the ring buffer for this USART.
    unsafe fn get_buffer_reference() -> &'static mut Option<RingBuffer<u8, RING_BUFFER_SIZE>>;

    /// Returns the interrupt number for this USART.
    fn interrupt_number() -> Interrupt;

    /// Returns a reference to the USART register block.
    #[inline]
    fn register_block(peripherals: &mut Peripherals) -> &mut UsartRegisterBlock {
        // unused but necessary for safety reasons
        let _ = peripherals;
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
        Self::register_block(peripherals)
            .ptcr.write(|w| w
                .rxtdis().set_bit()
                .txtdis().set_bit()
            )
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
                    if tx_enable {
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
        let mut clock_divider_hex: MaxArray<u8, {2*2}> = MaxArray::new();
        crate::hex_dump(&clock_divider.to_be_bytes(), &mut clock_divider_hex);
        uart::send(peripherals, b"USART clock divider: 0x");
        uart::send(peripherals, clock_divider_hex.as_slice());
        uart::send(peripherals, b"\r\n");

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

    /// Enable or disable all interrupts for this USART in the system core.
    fn set_core_interrupt(enabled: bool) {
        if enabled {
            unsafe { NVIC::unmask(Self::interrupt_number()) };
        } else {
            NVIC::mask(Self::interrupt_number());
        }
    }

    /// Enable or disable the interrupt for this USART that a byte is ready to be received.
    fn set_receive_ready_interrupt(peripherals: &mut Peripherals, enabled: bool) {
        if enabled {
            unsafe {
                Self::register_block(peripherals).ier().write_with_zero(|w| w
                    .rxrdy().set_bit()
                )
            };
        } else {
            unsafe {
                Self::register_block(peripherals).idr().write_with_zero(|w| w
                    .rxrdy().set_bit()
                )
            };
        }
    }

    /// Sends the given data.
    fn transmit(peripherals: &mut Peripherals, data: &[u8]) {
        uart::send(peripherals, b"PREPARING FOR TRANSMISSION\r\n");
        for b in data {
            while Self::register_block(peripherals).csr().read().txrdy().bit_is_clear() {
                // transmitter is not ready; wait...
            }
            uart::send(peripherals, b"b");
            unsafe {
                Self::register_block(peripherals)
                    .thr.write_with_zero(|w| w
                        .txchr().variant((*b).into())
                        .txsynh().clear_bit()
                    )
            };
        }
        uart::send(peripherals, b"\r\nFLUSHING\r\n");
        while Self::register_block(peripherals).csr().read().txempty().bit_is_clear() {
            // transmitter is not empty; wait...
        }
        uart::send(peripherals, b"SENT\r\n");
    }

    /// Receives enough data to fill the buffer.
    fn receive_exact(peripherals: &mut Peripherals, buffer: &mut [u8]) {
        let mut i = 0;
        uart::send(peripherals, b"RECEIVING\r\n");
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
        uart::send(peripherals, b"RECEIVED\r\n");
    }

    /// Sets whether the receive buffer for this USART is enabled. Also clears the buffer.
    fn set_receive_buffer_enabled(enabled: bool) {
        let ring_buffer_opt = unsafe { Self::get_buffer_reference() };
        if enabled {
            *ring_buffer_opt = Some(RingBuffer::new());
        } else {
            *ring_buffer_opt = None;
        }
    }

    /// Clears out the receive buffer for this USART and returns its contents.
    fn take_receive_buffer() -> Option<MaxArray<u8, RING_BUFFER_SIZE>> {
        let ring_buffer = unsafe { Self::get_buffer_reference() }
            .as_mut()?;
        let mut buf = MaxArray::new();
        while buf.len() < buf.max_size() {
            let byte_opt = cortex_interrupt::free(|_| ring_buffer.pop());
            let byte = match byte_opt {
                Some(b) => b,
                None => break,
            };
            buf.push(byte).unwrap();
        }
        Some(buf)
    }

    fn handle_interrupt() {
        let mut peripherals = unsafe { Peripherals::steal() };
        let reg_block = Self::register_block(&mut peripherals);

        // do we even have a buffer?
        let ring_buffer = match unsafe { Self::get_buffer_reference() } {
            Some(b) => b,
            None => return,
        };

        // do we even have a byte waiting?
        while reg_block.csr().read().rxrdy().bit_is_set() {
            // read the byte
            let byte = (reg_block.rhr.read().rxchr().bits() & 0xFF) as u8;

            // toss it in
            cortex_interrupt::free(|_| ring_buffer.push(byte));
        }

        if reg_block.csr().read().ovre().bit_is_set() {
            uart::send_stolen(b"OVERRUN\r\n");
            unsafe {
                reg_block.cr().write_with_zero(|w| { w
                    .rststa().set_bit()
                })
            };
        }
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
    #[inline] fn interrupt_number() -> Interrupt { Interrupt::USART0 }
    #[inline] unsafe fn get_buffer_reference() -> &'static mut Option<RingBuffer<u8, RING_BUFFER_SIZE>> { &mut USART0_BUFFER }
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
    #[inline] fn interrupt_number() -> Interrupt { Interrupt::USART1 }
    #[inline] unsafe fn get_buffer_reference() -> &'static mut Option<RingBuffer<u8, RING_BUFFER_SIZE>> { &mut USART1_BUFFER }
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
    #[inline] fn interrupt_number() -> Interrupt { Interrupt::USART2 }
    #[inline] unsafe fn get_buffer_reference() -> &'static mut Option<RingBuffer<u8, RING_BUFFER_SIZE>> { &mut USART2_BUFFER }
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
    #[inline] fn register_address() -> usize { 0x400A_4000 }
    #[inline] fn interrupt_number() -> Interrupt { Interrupt::USART3 }
    #[inline] unsafe fn get_buffer_reference() -> &'static mut Option<RingBuffer<u8, RING_BUFFER_SIZE>> { &mut USART3_BUFFER }
}


// and now, the interrupt handlers
#[interrupt] fn USART0() { Usart0::handle_interrupt() }
#[interrupt] fn USART1() { Usart1::handle_interrupt() }
#[interrupt] fn USART2() { Usart2::handle_interrupt() }
#[interrupt] fn USART3() { Usart3::handle_interrupt() }
