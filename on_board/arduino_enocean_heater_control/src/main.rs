#![no_main]
#![no_std]


mod click_spi;
mod display;
mod esp3_serial;
mod ring_buffer;
mod usart;


use core::panic::PanicInfo;
use core::time::Duration;

use atsam3x8e::Peripherals;
use atsam3x8e_ext::i2c_controller::{I2cController, Twi1I2cController};
use atsam3x8e_ext::sam_pin;
use atsam3x8e_ext::setup::system_init;
use atsam3x8e_ext::tick::{delay, enable_tick_clock};
use atsam3x8e_ext::uart;
use buildingblocks::bit_field;
use buildingblocks::crc8;
use buildingblocks::esp3::{CommandData, Esp3Packet, EventData};
use buildingblocks::max_array::MaxArray;
use cortex_m::Peripherals as CorePeripherals;
use cortex_m_rt::{entry, exception};

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


#[inline]
fn nibble_to_hex(byte: u8) -> u8 {
    match byte {
        0..=9 => byte + 0x30, // '0'
        10..=15 => byte - 10 + 0x41, // 'A'
        _ => 0x00,
    }
}

fn hex_dump<const N: usize>(bytes: &[u8], hex: &mut MaxArray<u8, N>) {
    for b in bytes {
        let push_res = hex.push(nibble_to_hex(*b >> 4));
        if push_res.is_err() {
            break;
        }
        let push_res = hex.push(nibble_to_hex(*b & 0x0F));
        if push_res.is_err() {
            break;
        }
    }
}

fn uart_send_hex_dump_outgoing(peripherals: &mut Peripherals, bytes: &[u8]) {
    let mut outgoing_hex: MaxArray<u8, {2*buildingblocks::esp3::MAX_ESP3_PACKET_LENGTH}> = MaxArray::new();
    hex_dump(bytes, &mut outgoing_hex);
    uart::send(peripherals, b"sending an ESP3 packet: ");
    uart::send(peripherals, outgoing_hex.as_slice());
    uart::send(peripherals, b"\r\n");
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

    // initialize UART
    uart::init(&mut peripherals);

    // try the I2C stuff

    // reset all mikroBUS devices
    sam_pin!(enable_io, peripherals, PIOC, p14, p15, p16);
    sam_pin!(make_output, peripherals, PIOC, p14, p15, p16);
    sam_pin!(set_high, peripherals, PIOC, p14, p15, p16);
    crate::delay(Duration::from_millis(10));
    sam_pin!(set_low, peripherals, PIOC, p14, p15, p16);
    crate::delay(Duration::from_millis(10));
    sam_pin!(set_high, peripherals, PIOC, p14, p15, p16);

    // enable display power
    uart::send(&mut peripherals, b"enabling display power\r\n");
    unsafe {
        peripherals.PIOA.oer.write_with_zero(|w| w
            .p28().set_bit()
        )
    };
    unsafe {
        peripherals.PIOA.sodr.write_with_zero(|w| w
            .p28().set_bit()
        )
    };

    // I2C controller setup
    uart::send(&mut peripherals, b"setting up I2C\r\n");
    Twi1I2cController::setup_pins(&mut peripherals);
    Twi1I2cController::enable_clock(&mut peripherals);

    Twi1I2cController::reset(&mut peripherals);
    Twi1I2cController::surrender_roles(&mut peripherals);
    Twi1I2cController::disable_dma(&mut peripherals);

    // SC18IS606 max speed is 400 kHz
    Twi1I2cController::set_speed(&mut peripherals, 400_000, clock.clock_speed);

    // ask our bridge chip to prepare its version info
    uart::send(&mut peripherals, b"asking for things to happen\r\n");
    Twi1I2cController::write(&mut peripherals, 0b0101_000, [0xFE]);

    // read it
    let mut buf = [0u8; 16];
    let mut buf_index = 0;
    uart::send(&mut peripherals, b"blammo?\r\n");
    Twi1I2cController::read(&mut peripherals, 0b0101_000, &mut buf);

    // find NUL byte
    let buf_len = buf.iter()
        .enumerate()
        .filter(|(_i, b)| **b == 0)
        .map(|(i, _b)| i)
        .nth(0)
        .unwrap_or(buf.len());
    uart::send(&mut peripherals, b"I2C chip version: ");
    uart::send(&mut peripherals, &buf[0..buf_len]);
    uart::send(&mut peripherals, b"\r\n");

    let mut clock_hex : MaxArray<u8, {4*2}> = MaxArray::new();
    hex_dump(&clock.clock_speed.to_be_bytes(), &mut clock_hex);
    uart::send(&mut peripherals, b"clock speed: 0x");
    uart::send(&mut peripherals, clock_hex.as_slice());
    uart::send(&mut peripherals, b"\r\n");
    uart::send(&mut peripherals, b"system and UART initialization complete\r\n");

    // enable RESET on the TCM515
    sam_pin!(make_output, peripherals, PIOC, p16);
    sam_pin!(set_low, peripherals, PIOC, p16);

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
    Usart3::set_receive_buffer_enabled(true);
    uart::send(&mut peripherals, b"RECEIVE BUFFER ENABLED\r\n");
    Usart3::set_receive_ready_interrupt(&mut peripherals, true);
    uart::send(&mut peripherals, b"RECEIVE-READY INTERRUPT ENABLED\r\n");
    Usart3::set_core_interrupt(true);
    uart::send(&mut peripherals, b"CORE USART3 INTERRUPT ENABLED\r\n");
    Usart3::set_rxtx_state(&mut peripherals, true, true);
    uart::send(&mut peripherals, b"TRANSMITTER AND RECEIVER ENABLED\r\n");

    // turn off the TCM515 reset pin
    sam_pin!(set_high, peripherals, PIOC, p16);

    /*
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
        uart::send(&mut peripherals, b"-");

        delay(Duration::from_millis(1000));

        // PIOB SODR bit 27 to 1 = pin B27 is driven up
        sam_pin!(set_high, peripherals, PIOB, p27);
        uart::send(&mut peripherals, b"+");

        // wait a bit
        delay(Duration::from_millis(1000));
    }
    */

    enum AwaitingWhat {
        Nothing,
        Version,
    }
    let mut awaiting = AwaitingWhat::Nothing;

    loop {
        // transfer from USART to ESP3 buffer
        if let Some(buf) = Usart3::take_receive_buffer() {
            for b in buf.iter() {
                esp3_serial::push_to_buffer(*b);
            }
        }

        // try taking a packet
        if let Some(packet) = esp3_serial::take_esp3_packet() {
            // hex-dump it
            let mut hex: MaxArray<u8, {2*buildingblocks::esp3::MAX_ESP3_PACKET_LENGTH}> = MaxArray::new();
            hex_dump(packet.as_slice(), &mut hex);

            // send the hex dump via UART
            uart::send(&mut peripherals, b"got an ESP3 packet: ");
            uart::send(&mut peripherals, hex.as_slice());
            uart::send(&mut peripherals, b"\r\n");

            // decode it
            let decoded_packet_opt = Esp3Packet::from_slice(packet.as_slice());
            if let Some(decoded_packet) = decoded_packet_opt {
                if let Esp3Packet::Event(event_packet) = decoded_packet {
                    if let EventData::CoReady { .. } = event_packet {
                        // send the version packet
                        let version_packet_opt = Esp3Packet::CommonCommand(CommandData::CoRdVersion)
                            .to_packet();
                        let version_packet = match &version_packet_opt {
                            Some(vp) => {
                                uart::send(&mut peripherals, b"VERSION PACKET PACKAGED\r\n");
                                vp
                            },
                            None => {
                                uart::send(&mut peripherals, b"OH NO\r\n");
                                loop {
                                    delay(Duration::from_secs(5));
                                }
                            }
                        };
                        uart_send_hex_dump_outgoing(&mut peripherals, version_packet.as_slice());
                        Usart3::transmit(&mut peripherals, version_packet.as_slice());
                        awaiting = AwaitingWhat::Version;
                    }
                } else if let Esp3Packet::Response { .. } = decoded_packet {
                    if let AwaitingWhat::Version = awaiting {
                        // version packet received
                        awaiting = AwaitingWhat::Nothing;

                        // switch to transparent mode
                        let pkt = Esp3Packet::CommonCommand(CommandData::CoWrTransparentMode {
                            enable: true.into(),
                        }).to_packet().unwrap();
                        uart::send(&mut peripherals, b"switching to transparent mode\r\n");
                        uart_send_hex_dump_outgoing(&mut peripherals, pkt.as_slice());
                        Usart3::transmit(&mut peripherals, pkt.as_slice());
                    }
                }
            }
        }

        // doze off for a bit
        delay(Duration::from_millis(10));
    }
}
