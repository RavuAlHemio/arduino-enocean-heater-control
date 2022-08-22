#![no_main]
#![no_std]


mod atsam3x8e_ext;
mod click_spi;


use core::panic::PanicInfo;
use core::time::Duration;

use atsam3x8e::Peripherals;
use atsam3x8e::uart::mr::{CHMODE_A, PAR_A};
use cortex_m::Peripherals as CorePeripherals;
use cortex_m_rt::{entry, exception};

use crate::atsam3x8e_ext::nop;
use crate::atsam3x8e_ext::setup::system_init;
use crate::atsam3x8e_ext::tick::{delay, enable_tick_clock};


#[exception]
unsafe fn DefaultHandler(_: i16) {
}


#[panic_handler]
fn loopy_panic_handler(_: &PanicInfo) -> ! {
    loop {
    }
}


fn uart_send(peripherals: &mut Peripherals, buffer: &[u8]) {
    for b in buffer {
        while peripherals.UART.sr.read().txrdy().bit_is_clear() {
            // wait until transmission is ready
        }

        unsafe {
            peripherals.UART.thr.write_with_zero(|w| w
                .txchr().variant(*b)
            )
        };
    }

    while peripherals.UART.sr.read().txempty().bit_is_clear() {
        // wait until transmitter is empty
    }
}


#[entry]
fn main() -> ! {
    let mut core_peripherals = CorePeripherals::take().expect("failed to obtain core peripherals");
    let mut peripherals = Peripherals::take().expect("failed to obtain peripherals");

    // first things first: disable the watchdog
    peripherals.WDT.mr.modify(|_, w| w
        .wddis().set_bit()
    );

    // initialize system
    let mut clock = system_init(&mut peripherals);
    enable_tick_clock(&mut core_peripherals, clock.clock_speed / 1000);

    // PIOA PDR bits 8 and 9 to 1 = pins A8 and A9 are disabled on the PIO controller
    // => the peripheral (UART) may use them
    sam_pin!(disable_io, peripherals, PIOA, p8, p9);

    // PIOA ABSR bits 8 and 9 to 0 = pins A8 and A9 are used by peripheral A (UART)
    sam_pin!(peripheral_ab, peripherals, PIOA, p8, clear_bit, p9, clear_bit);

    // enable UART transmitter
    unsafe {
        peripherals.UART.cr.write_with_zero(|w| w
            .txen().set_bit()
        )
    };

    // set baud rate
    // baud = clockfreq/(16*cd)
    // cd = clockfreq/(16*baud)
    // cd = clockfreq/(16*115200)
    // assume clockfreq is 84MHz
    // cd = 84_000_000/(16*115200)
    // cd = 84_000_000/1_843_200
    // cd ~ 46
    peripherals.UART.brgr.write(|w| w
        .cd().variant(46)
    );

    // no parity bit, no test mode
    peripherals.UART.mr.write(|w| w
        .par().variant(PAR_A::NO)
        .chmode().variant(CHMODE_A::NORMAL)
    );

    uart_send(&mut peripherals, b"system and UART initialization complete\r\n");

    // reset the display

    // R/W pin (ADC0/PF0 = PA16) and Power Supply Enable pin (PB4/INT = PA27)
    // must be set to low
    sam_pin!(enable_io, peripherals, PIOA, p16, p27);
    sam_pin!(make_output, peripherals, PIOA, p16, p27);
    sam_pin!(set_low, peripherals, PIOA, p16, p27);

    // Reset pin is on PL0/RST = PC14; make it an output
    // Data/Command pin is on PE3/PWM = PC28; make it an output
    sam_pin!(enable_io, peripherals, PIOC, p14, p28);
    sam_pin!(make_output, peripherals, PIOC, p14, p28);
    // set Reset pin high
    sam_pin!(set_high, peripherals, PIOC, p14);
    // wait a bit
    delay(Duration::from_millis(100));
    // set Reset pin low (triggers reset)
    sam_pin!(set_low, peripherals, PIOC, p14);
    // wait a bit
    delay(Duration::from_millis(100));
    // set Reset pin and Power Supply Enable pin high
    sam_pin!(set_high, peripherals, PIOC, p14);
    sam_pin!(set_high, peripherals, PIOA, p27);

    // set up SPI
    click_spi::setup_pins_controller(&mut peripherals);

    // say it's a command (not data) = set PC28 low
    sam_pin!(set_low, peripherals, PIOC, p28);
    // wait a bit
    delay(Duration::from_millis(100));
    // bitbang 0xAF (display on)
    click_spi::cs1_low(&mut peripherals);
    delay(Duration::from_millis(100));
    click_spi::bitbang::<65536>(&mut peripherals, 0xAF);
    delay(Duration::from_millis(100));
    click_spi::cs1_high(&mut peripherals);

    // do nothing

    // PB27 = internal LED

    // PIOB OER bit 27 to 1 = pin B27 is now an output
    // (the rest remain inputs)
    sam_pin!(make_output, peripherals, PIOB, p27);

    // blinking!
    loop {
        // PIOB CODR bit 27 to 1 = pin B27 is driven down
        sam_pin!(set_low, peripherals, PIOB, p27);
        uart_send(&mut peripherals, b"-");

        delay(Duration::from_millis(1000));

        // PIOB SODR bit 27 to 1 = pin B27 is driven up
        sam_pin!(set_high, peripherals, PIOB, p27);
        uart_send(&mut peripherals, b"+");

        // wait a bit
        delay(Duration::from_millis(1000));
    }
}
