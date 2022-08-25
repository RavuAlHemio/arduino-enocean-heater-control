use core::time::Duration;

use atsam3x8e::Peripherals;

use crate::{delay, sam_pin};
use crate::atsam3x8e_ext::multinop;
use crate::click_spi;


const MULTINOP_COUNT: usize = 128;


pub fn send_command(peripherals: &mut Peripherals, command: u8, args: &[u8]) {
    // select device
    click_spi::cs1_low(peripherals);
    multinop::<MULTINOP_COUNT>();

    // set D/C low for command
    sam_pin!(set_low, peripherals, PIOC, p25);
    multinop::<MULTINOP_COUNT>();

    // send command
    click_spi::bitbang::<MULTINOP_COUNT>(peripherals, command);
    multinop::<MULTINOP_COUNT>();

    // set D/C high for data (default)
    sam_pin!(set_high, peripherals, PIOC, p25);
    multinop::<MULTINOP_COUNT>();

    if args.len() > 0 {
        for arg in args {
            click_spi::bitbang::<MULTINOP_COUNT>(peripherals, *arg);
            multinop::<MULTINOP_COUNT>();
        }
    }

    // deselect device
    click_spi::cs1_high(peripherals);
    multinop::<MULTINOP_COUNT>();
}


pub fn init_display(peripherals: &mut Peripherals) {
    // pinout at Mikrobus slot 1 on Arduino Mega Shield on Arduino Due:
    // PA16 = R/W = read/write (tie low; we're using the 4-wire SPI interface)
    // PC14 = RST = reset
    // PC25 = D/C = data/command (high is data, low is command)
    // PA28 = EN  = set high for power supply enable
    //              (connected to TI power chip, not display controller)
    //
    // the following pins are managed by the SPI module, not by us:
    // PB14 = CS  = chip select slot 1 (SPI set-low-to-talk-to-me)
    // PB21 = SCK = SPI clock (bus)
    // PC13 = SDO/CIPO = SPI peripheral to controller (bus)
    // PC12 = SDI/COPI = SPI controller to peripheral (bus)

    // configure pin modes
    sam_pin!(enable_io, peripherals, PIOA, p16, p28);
    sam_pin!(enable_io, peripherals, PIOC, p14, p25);
    sam_pin!(make_output, peripherals, PIOA, p16, p28);
    sam_pin!(make_output, peripherals, PIOC, p14, p25);

    // R/W is low (permanently), D/C is high (data), RST is high, EN starts out low
    sam_pin!(set_low, peripherals, PIOA, p16, p28);
    sam_pin!(set_low, peripherals, PIOC, p14);
    sam_pin!(set_high, peripherals, PIOC, p25);

    // wait a bit
    delay(Duration::from_millis(1));

    // set EN high (turn display power on)
    sam_pin!(set_high, peripherals, PIOA, p28);

    // wait a bit while the power supply stabilizes
    delay(Duration::from_millis(1));

    // bounce the RST pin (triggers the reset)
    sam_pin!(set_high, peripherals, PIOC, p14);
    delay(Duration::from_millis(1));
    sam_pin!(set_low, peripherals, PIOC, p14);
    delay(Duration::from_millis(1));
    sam_pin!(set_high, peripherals, PIOC, p14);
    delay(Duration::from_millis(100));

    // the display only has 96x96 pixels while RAM is 128x128
    // columns are centered, rows are top-aligned
    send_command(peripherals, 0x15, &[16, 111]);
    delay(Duration::from_millis(1000));
    send_command(peripherals, 0x75, &[0, 95]);
    delay(Duration::from_millis(1000));

    // turn on display
    send_command(peripherals, 0xAF, &[]);
    delay(Duration::from_millis(1000));

    // fill display with data
    send_command(peripherals, 0x5C, include_bytes!("picture.data"));
    delay(Duration::from_millis(1000));
}
