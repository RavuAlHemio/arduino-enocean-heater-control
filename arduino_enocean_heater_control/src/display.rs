use atsam3x8e::Peripherals;

use crate::sam_pin;
use crate::atsam3x8e_ext::multinop;
use crate::click_spi;


pub fn send_command(peripherals: &mut Peripherals, command: u8, args: &[u8]) {
    // D/C should be down by default

    click_spi::cs1_low(peripherals);
    multinop::<1024>();
    click_spi::bitbang::<1024>(peripherals, command);
    multinop::<1024>();
    click_spi::cs1_high(peripherals);

    if args.len() > 0 {
        multinop::<1024>();

        // D/C high (data)
        sam_pin!(set_high, peripherals, PIOC, p25);
        multinop::<1024>();

        for arg in args {
            click_spi::cs1_low(peripherals);
            multinop::<1024>();
            click_spi::bitbang::<1024>(peripherals, *arg);
            multinop::<1024>();
            click_spi::cs1_high(peripherals);
            multinop::<1024>();
        }

        // D/C low (command; default)
        sam_pin!(set_low, peripherals, PIOC, p25);
        multinop::<1024>();
    }
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

    // R/W is low (permanently), D/C is low (command), RST is high, EN is high
    sam_pin!(set_low, peripherals, PIOA, p16);
    sam_pin!(set_low, peripherals, PIOC, p25);
    sam_pin!(set_high, peripherals, PIOC, p14);
    sam_pin!(set_high, peripherals, PIOA, p28);

    // wait a bit while the power supply stabilizes
    multinop::<1024>();

    // bounce the RST pin (triggers the reset)
    sam_pin!(set_low, peripherals, PIOC, p14);
    multinop::<1024>();
    sam_pin!(set_high, peripherals, PIOC, p14);
    multinop::<1024>();

    // bitbang 0xAF (display on)
    send_command(peripherals, 0xAF, &[]);
}
