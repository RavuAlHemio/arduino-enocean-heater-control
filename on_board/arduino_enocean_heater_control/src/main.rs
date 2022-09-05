#![no_main]
#![no_std]


mod atsam3x8e_ext;
mod click_spi;
mod display;
mod uart;
mod usart;


use core::panic::PanicInfo;
use core::time::Duration;

use atsam3x8e::Peripherals;
use buildingblocks::bit_field;
use buildingblocks::crc8;
use cortex_m::Peripherals as CorePeripherals;
use cortex_m_rt::{entry, exception};

use crate::atsam3x8e_ext::nop;
use crate::atsam3x8e_ext::setup::system_init;
use crate::atsam3x8e_ext::tick::{delay, enable_tick_clock};
use crate::display::DisplayCommand;
use crate::usart::{Usart, Usart3};


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

#[inline]
fn nibble_to_hex(byte: u8) -> u8 {
    match byte {
        0..=9 => byte + 0x30, // '0'
        10..=15 => byte - 10 + 0x41, // 'A'
        _ => 0x00,
    }
}

fn hex_dump(bytes: &[u8], hex: &mut [u8]) -> usize {
    let mut i = 0;
    for b in bytes {
        if i >= hex.len() {
            break;
        }
        hex[i] = nibble_to_hex(*b >> 4);
        i += 1;

        if i >= hex.len() {
            break;
        }
        hex[i] = nibble_to_hex(*b & 0x0F);
        i += 1;
    }
    i
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

    uart::init(&mut peripherals);
    let mut clock_hex = [0u8; 4*2];
    hex_dump(&clock.clock_speed.to_be_bytes(), &mut clock_hex);
    uart::send(&mut peripherals, b"clock speed: 0x");
    uart::send(&mut peripherals, &clock_hex);
    uart::send(&mut peripherals, b"\r\n");
    uart::send(&mut peripherals, b"system and UART initialization complete\r\n");

    // set up SPI
    click_spi::setup_pins_controller(&mut peripherals);

    // disable RESET on the TCM515
    sam_pin!(make_output, peripherals, PIOC, p16);
    sam_pin!(set_low, peripherals, PIOC, p16);
    delay(Duration::from_millis(1000));
    sam_pin!(set_high, peripherals, PIOC, p16);

    // set up the connection to the TCM515
    Usart3::set_pins(&mut peripherals);
    uart::send(&mut peripherals, b"PINS SET\r\n");
    Usart3::enable_clock(&mut peripherals);
    uart::send(&mut peripherals, b"CLOCK ENABLED\r\n");
    Usart3::disable_pdc(&mut peripherals);
    uart::send(&mut peripherals, b"PDC disabled\r\n");
    Usart3::reset_and_disable(&mut peripherals);
    uart::send(&mut peripherals, b"USART RESET\r\n");
    Usart3::set_standard_params(&mut peripherals);
    uart::send(&mut peripherals, b"STANDARD PARAMS SET\r\n");
    Usart3::set_baud_rate(&clock, &mut peripherals, 57600);
    uart::send(&mut peripherals, b"BAUD RATE SET\r\n");
    Usart3::disable_interrupts(&mut peripherals);
    uart::send(&mut peripherals, b"INTERRUPTS DISABLED\r\n");
    Usart3::set_rxtx_state(&mut peripherals, true, true);
    uart::send(&mut peripherals, b"TRANSMITTER AND RECEIVER ENABLED\r\n");

    // wait five seconds
    delay(Duration::from_secs(5));

    // prepare a packet (CO_RD_VERSION)
    let mut read_version_packet = [
        0x55, // sync byte
        0x00, 0x01, // data length
        0x00, // optional length
        0x05, // packet type (COMMON_COMMAND)
        0x00, // space for the header CRC
        0x03, // command code (CO_RD_VERSION)
        0x00, // space for the data CRC
    ];
    read_version_packet[5] = crc8::crc8_ccitt(&read_version_packet[1..5]);
    read_version_packet[7] = crc8::crc8_ccitt(&read_version_packet[6..7]);

    // toss it over
    Usart3::transmit(&mut peripherals, &read_version_packet);

    // read the response
    let mut b = [0u8];
    Usart3::receive_exact(&mut peripherals, &mut b);
    let mut hex = [0u8; 2];
    hex_dump(&b, &mut hex);
    uart::send(&mut peripherals, &hex);

    // initialize the display
    display::init_display(&mut peripherals);

    // write some stuff
    display::write_line(
        &mut peripherals,
        0, 0,
        [0xFF, 0xFF],
        [0x00, 0x00],
        "GOOD MORNING",
    );

    // give it a few seconds
    delay(Duration::from_secs(5));

    // turn off the display
    display::send_command(&mut peripherals, DisplayCommand::SetSleepMode(true));

    // read buttons (TODO)

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
