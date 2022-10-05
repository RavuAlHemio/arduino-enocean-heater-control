//! Code for the PSP27801 OLED display combined with the SSD1351 display controller.

mod settings;


use core::time::Duration;

use atsam3x8e::Peripherals;
use atsam3x8e_ext::{multinop, sam_pin};
use atsam3x8e_ext::tick::delay;

use crate::bit_field::BitField;
use crate::click_spi;


/// The width of the display RAM in pixels.
const RAM_WIDTH: usize = 128;

/// The height of the display RAM in pixels.
const RAM_HEIGHT: usize = 128;

/// The width of the actual display in pixels.
const DISPLAY_WIDTH: usize = 96;

/// The height of the actual display in pixels.
const DISPLAY_HEIGHT: usize = 96;

/// The horizontal offset of the window into display RAM that the display actually displays.
const DISPLAY_OFFSET_X: usize = (RAM_WIDTH - DISPLAY_WIDTH)/2;

/// The vertical offset of the window into display RAM that the display actually displays.
const DISPLAY_OFFSET_Y: usize = 0;

/// The number of times to issue a NOP command between changing SPI pin values.
const MULTINOP_COUNT: usize = 1;

/// The alphabet to use to paint letters on the display.
const ALPHABET: BitField<1152> = BitField::from_bytes(*include_bytes!("../../data/alphabet.bin"));

/// The width of each character in the alphabet.
const LETTER_WIDTH: usize = 8;

/// The height of each character in the alphabet.
const LETTER_HEIGHT: usize = 12;

/// The maximum amount of letters per line, as allowed by display memory.
const MAX_LETTERS_PER_ROW: usize = DISPLAY_WIDTH / LETTER_WIDTH;

/// Number of bytes per pixel.
const COLOR_DEPTH: usize = 2;


#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DisplayCommand<'a> {
    SetColumnAddress { first: u8, last: u8 },
    SetRowAddress { first: u8, last: u8 },
    WriteRam(&'a [u8]),
    ReadRam(&'a mut [u8]),
    SetRemap {
        vertical_increment: bool,
        reverse_columns: bool,
        reverse_colors: bool,
        reverse_com: bool,
        com_split: bool,
        color_depth: settings::ColorDepth,
    },
    SetDisplayStartLine(u8),
    SetDisplayOffset(u8),
    SetDisplayMode(settings::DisplayMode),
    FunctionSelection {
        internal_vdd: bool,
        interface: settings::DisplayInterface,
    },
    NopAD,
    SetSleepMode(bool),
    NopB0,
    SetPeriods {
        reset: settings::ResetPeriod,
        precharge: settings::PrechargePeriod,
    },
    SetDisplayEnhancement(bool),
    SetFrontClock {
        divider: settings::Divider,
        oscillator: u8,
    },
    SetGpio {
        gpio0: settings::GpioState,
        gpio1: settings::GpioState,
    },
    SetSecondPrechargePeriod(settings::SecondPrechargePeriod),
    SetGrayscaleTable([u8; 63]),
    ResetGrayscaleTable,
    SetPrechargeVoltage(settings::PrechargeVoltage),
    SetComDeselectVoltage(settings::ComDeselectVoltage),
    SetContrastPerColor { red_contrast: u8, green_contrast: u8, blue_contrast: u8 },
    SetGeneralContrast(settings::GeneralContrast),
    SetMuxRatio(u8),
    NopD1,
    NopE3,
    SetProtectionLevel(settings::ProtectionLevel),
    SetHorizontalScroll {
        speed_and_direction: u8,
        start_row: u8,
        row_count: u8,
        time_interval: settings::ScrollTimeInterval,
    },
    StopHorizontalScroll,
    StartHorizontalScroll,
}
impl<'a> DisplayCommand<'a> {
    pub fn code(&self) -> u8 {
        match self {
            Self::SetColumnAddress { .. } => 0x15,
            Self::SetRowAddress { .. } => 0x75,
            Self::WriteRam(_) => 0x5C,
            Self::ReadRam(_) => 0x5D,
            Self::SetRemap { .. } => 0xA0,
            Self::SetDisplayStartLine(_) => 0xA1,
            Self::SetDisplayOffset(_) => 0xA2,
            Self::SetDisplayMode(mode) => match mode {
                settings::DisplayMode::AllOff => 0xA4,
                settings::DisplayMode::AllOn => 0xA5,
                settings::DisplayMode::Normal => 0xA6,
                settings::DisplayMode::Inverse => 0xA7,
            },
            Self::FunctionSelection { .. } => 0xAB,
            Self::NopAD => 0xAD,
            Self::SetSleepMode(sleep_on) => if *sleep_on { 0xAE } else { 0xAF },
            Self::NopB0 => 0xB0,
            Self::SetPeriods { .. } => 0xB1,
            Self::SetDisplayEnhancement(_) => 0xB2,
            Self::SetFrontClock { .. } => 0xB3,
            Self::SetGpio { .. } => 0xB5,
            Self::SetSecondPrechargePeriod(_) => 0xB6,
            Self::SetGrayscaleTable(_) => 0xB8,
            Self::ResetGrayscaleTable => 0xB9,
            Self::SetPrechargeVoltage(_) => 0xBB,
            Self::SetComDeselectVoltage(_) => 0xBE,
            Self::SetContrastPerColor { .. } => 0xC1,
            Self::SetGeneralContrast(_) => 0xC7,
            Self::SetMuxRatio(_) => 0xCA,
            Self::NopD1 => 0xD1,
            Self::NopE3 => 0xE3,
            Self::SetProtectionLevel(_) => 0xFD,
            Self::SetHorizontalScroll { .. } => 0x96,
            Self::StopHorizontalScroll => 0x9E,
            Self::StartHorizontalScroll => 0x9F,
        }
    }

    pub fn encode_data<'b, 'c>(&'a self, buffer: &'b mut [u8]) -> &'c [u8]
        where
            'a: 'c,
            'b: 'c {
        match self {
            Self::SetColumnAddress { first, last } => {
                assert!(buffer.len() >= 2);
                buffer[0] = *first;
                buffer[1] = *last;
                &buffer[0..2]
            },
            Self::SetRowAddress { first, last } => {
                assert!(buffer.len() >= 2);
                buffer[0] = *first;
                buffer[1] = *last;
                &buffer[0..2]
            },
            Self::WriteRam(data) => *data,
            Self::ReadRam(data) => &data,
            Self::SetRemap {
                vertical_increment,
                reverse_columns,
                reverse_colors,
                reverse_com,
                com_split,
                color_depth,
            } => {
                assert!(buffer.len() >= 1);
                buffer[0] = 0b0000_0000;
                if *vertical_increment {
                    buffer[0] |= 0b0000_0001;
                }
                if *reverse_columns {
                    buffer[0] |= 0b0000_0010;
                }
                if *reverse_colors {
                    buffer[0] |= 0b0000_0100;
                }
                if *reverse_com {
                    buffer[0] |= 0b0001_0000;
                }
                if *com_split {
                    buffer[0] |= 0b0010_0000;
                }
                buffer[0] |= u8::from(*color_depth) << 6;
                &buffer[0..1]
            },
            Self::SetDisplayStartLine(line) => {
                assert!(buffer.len() >= 1);
                buffer[0] = *line;
                &buffer[0..1]
            },
            Self::SetDisplayOffset(offset) => {
                assert!(buffer.len() >= 1);
                buffer[0] = *offset;
                &buffer[0..1]
            },
            Self::SetDisplayMode(_mode) => &buffer[0..0],
            Self::FunctionSelection { internal_vdd, interface } => {
                assert!(buffer.len() >= 1);
                buffer[0] = 0x00;
                if *internal_vdd {
                    buffer[0] |= 0b0000_0001;
                }
                buffer[0] |= u8::from(*interface) << 6;
                &buffer[0..1]
            },
            Self::NopAD => &buffer[0..0],
            Self::SetSleepMode(_) => &buffer[0..0],
            Self::NopB0 => &buffer[0..0],
            Self::SetPeriods { reset, precharge } => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*reset).into();
                buffer[0] |= u8::from(*precharge) << 4;
                &buffer[0..1]
            },
            Self::SetDisplayEnhancement(enabled) => {
                assert!(buffer.len() >= 3);
                buffer[0] = if *enabled { 0xA4 } else { 0x00 };
                buffer[1] = 0x00;
                buffer[2] = 0x00;
                &buffer[0..3]
            },
            Self::SetFrontClock { divider, oscillator } => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*divider).into();
                buffer[0] |= u8::from(*oscillator) << 4;
                &buffer[0..1]
            },
            Self::SetGpio { gpio0, gpio1 } => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*gpio0).into();
                buffer[0] |= u8::from(*gpio1) << 2;
                &buffer[0..1]
            },
            Self::SetSecondPrechargePeriod(spp) => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*spp).into();
                &buffer[0..1]
            },
            Self::SetGrayscaleTable(table) => {
                table
            },
            Self::ResetGrayscaleTable => &buffer[0..0],
            Self::SetPrechargeVoltage(pcv) => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*pcv).into();
                &buffer[0..1]
            },
            Self::SetComDeselectVoltage(cdv) => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*cdv).into();
                &buffer[0..1]
            },
            Self::SetContrastPerColor { red_contrast, green_contrast, blue_contrast } => {
                assert!(buffer.len() >= 3);
                buffer[0] = *red_contrast;
                buffer[1] = *green_contrast;
                buffer[2] = *blue_contrast;
                &buffer[0..3]
            },
            Self::SetGeneralContrast(contrast) => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*contrast).into();
                &buffer[0..1]
            },
            Self::SetMuxRatio(ratio) => {
                assert!(buffer.len() >= 1);
                buffer[0] = *ratio;
                &buffer[0..1]
            },
            Self::NopD1 => &buffer[0..0],
            Self::NopE3 => &buffer[0..0],
            Self::SetProtectionLevel(protection) => {
                assert!(buffer.len() >= 1);
                buffer[0] = (*protection).into();
                &buffer[0..1]
            },
            Self::SetHorizontalScroll {
                speed_and_direction,
                start_row,
                row_count,
                time_interval,
            } => {
                assert!(buffer.len() >= 5);
                buffer[0] = *speed_and_direction;
                buffer[1] = *start_row;
                buffer[2] = *row_count;
                buffer[3] = 0x00;
                buffer[4] = (*time_interval).into();
                &buffer[0..5]
            },
            Self::StopHorizontalScroll => &buffer[0..0],
            Self::StartHorizontalScroll => &buffer[0..0],
        }
    }
}


pub fn send_low_level_command<A: IntoIterator<Item = u8>>(peripherals: &mut Peripherals, command: u8, args: A) {
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

    for arg in args {
        click_spi::bitbang::<MULTINOP_COUNT>(peripherals, arg);
        multinop::<MULTINOP_COUNT>();
    }

    // deselect device
    click_spi::cs1_high(peripherals);
    multinop::<MULTINOP_COUNT>();
}


pub fn send_command(peripherals: &mut Peripherals, command: DisplayCommand) {
    let command_code = command.code();
    let mut buf = [0u8; 6];
    let command_data = command.encode_data(&mut buf);

    send_low_level_command(peripherals, command_code, command_data.iter().map(|b| *b));
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
    set_default_dimensions(peripherals);

    // set display to black
    send_low_level_command(peripherals, 0x5C, (0..DISPLAY_WIDTH*DISPLAY_HEIGHT*COLOR_DEPTH).map(|_| 0x00));

    // turn on display
    send_command(peripherals, DisplayCommand::SetSleepMode(false));
}

fn str_to_char_indexes(text: &str) -> ([usize; MAX_LETTERS_PER_ROW], usize) {
    let mut ret = [0usize; MAX_LETTERS_PER_ROW];
    for (i, c) in text.chars().enumerate() {
        if i >= MAX_LETTERS_PER_ROW {
            break;
        }

        if c == '\u{B0}' {
            // degree sign (special case)
            ret[i] = 0x5F;
        } else if c >= '\u{20}' && c <= '\u{7E}' {
            ret[i] = ((c as u8) - 0x0020).into();
        } else {
            // fold out-of-range characters to the question mark
            ret[i] = 0x003F - 0x0020;
        }
    }
    (ret, text.chars().count().min(MAX_LETTERS_PER_ROW))
}

fn char_by_index(index: usize) -> [[bool; LETTER_WIDTH]; LETTER_HEIGHT] {
    let mut ret = [[false; LETTER_WIDTH]; LETTER_HEIGHT];
    let mut i = index * LETTER_WIDTH * LETTER_HEIGHT;
    for row in 0..LETTER_HEIGHT {
        for col in 0..LETTER_WIDTH {
            if ALPHABET.is_bit_set(i) {
                ret[row][col] = true;
            }
            i += 1;
        }
    }
    ret
}

fn set_default_dimensions(peripherals: &mut Peripherals) {
    send_command(peripherals, DisplayCommand::SetColumnAddress {
        first: DISPLAY_OFFSET_X.try_into().unwrap(),
        last: (DISPLAY_OFFSET_X + DISPLAY_WIDTH - 1).try_into().unwrap(),
    });
    send_command(peripherals, DisplayCommand::SetRowAddress {
        first: DISPLAY_OFFSET_Y.try_into().unwrap(),
        last: (DISPLAY_OFFSET_Y + DISPLAY_HEIGHT - 1).try_into().unwrap(),
    });
}

pub fn write_line(
    peripherals: &mut Peripherals,
    x: u32, y: u32,
    fg_color: [u8; COLOR_DEPTH], bg_color: [u8; COLOR_DEPTH],
    text: &str,
) {
    let (char_indexes, char_count) = str_to_char_indexes(text);

    if usize::try_from(y).unwrap() > DISPLAY_HEIGHT {
        return;
    }

    for i in 0..char_count {
        let first_col: u32 = x + u32::try_from(i * LETTER_WIDTH).unwrap();
        let first_col_usize: usize = first_col.try_into().unwrap();
        if first_col_usize > DISPLAY_WIDTH {
            break;
        }

        let character = char_by_index(char_indexes[i]);

        send_command(peripherals, DisplayCommand::SetColumnAddress {
            first: (DISPLAY_OFFSET_X + first_col_usize).try_into().unwrap(),
            last: (DISPLAY_OFFSET_X + first_col_usize + LETTER_WIDTH - 1).try_into().unwrap(),
        });
        send_command(peripherals, DisplayCommand::SetRowAddress {
            first: (DISPLAY_OFFSET_Y + usize::try_from(y).unwrap()).try_into().unwrap(),
            last: (DISPLAY_OFFSET_Y + usize::try_from(y).unwrap() + LETTER_HEIGHT - 1).try_into().unwrap(),
        });

        // write image data
        let character_colors = character.iter().flat_map(|row|
            row.iter().flat_map(|pixel|
                if *pixel { fg_color } else { bg_color }
            )
        );
        send_low_level_command(peripherals, 0x5C, character_colors);
    }

    set_default_dimensions(peripherals);
}
