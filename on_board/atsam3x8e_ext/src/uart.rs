//! Code for the built-in UART, communicating via pins PA8/RX0 and PA9/TX0 as well as the USB
//! connection.


use atsam3x8e::Peripherals;
use atsam3x8e::uart::mr::{CHMODE_A, PAR_A};

use crate::sam_pin;


/// Initialize the UART.
pub fn init(peripherals: &mut Peripherals) {
    // PIOA PDR bits 8 and 9 to 1 = pins A8 and A9 are disabled on the PIO controller
    // => the peripheral (UART) may use them
    sam_pin!(disable_io, peripherals, PIOA, p8, p9);

    // PIOA ABSR bits 8 and 9 to 0 = pins A8 and A9 are used by peripheral A (UART)
    sam_pin!(peripheral_ab, peripherals, PIOA, p8, clear_bit, p9, clear_bit);

    // feed clock to UART
    unsafe {
        peripherals.PMC.pmc_pcer0.write_with_zero(|w| w
            .pid8().set_bit() // UART
        )
    };

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
}

/// Sends a message via the UART.
pub fn send(peripherals: &mut Peripherals, buffer: &[u8]) {
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

/// Sends a message, stealing the UART for this purpose.
pub fn send_stolen(buffer: &[u8]) {
    let mut peripherals = unsafe { Peripherals::steal() };
    send(&mut peripherals, buffer);
}
