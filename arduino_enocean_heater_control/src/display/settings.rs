use from_to_repr::FromToRepr;


#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ColorDepth {
    Colors65k = 0b00,
    Colors262k = 0b10,
    Colors262kFormat2 = 0b11,
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DisplayMode {
    AllOff = 0b00,
    AllOn = 0b01,
    Normal = 0b10,
    Inverse = 0b11,
}
impl Default for DisplayMode {
    fn default() -> Self { Self::Normal }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DisplayInterface {
    Parallel8Bit = 0b00,
    Parallel16Bit = 0b01,
    Parallel18Bit = 0b11,
}
impl Default for DisplayInterface {
    fn default() -> Self { Self::Parallel8Bit }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ResetPeriod {
    Period5 = 2,
    Period7 = 3,
    Period9 = 4,
    Period11 = 5,
    Period13 = 6,
    Period15 = 7,
    Period17 = 8,
    Period19 = 9,
    Period21 = 10,
    Period23 = 11,
    Period25 = 12,
    Period27 = 13,
    Period29 = 14,
    Period31 = 15,
}
impl Default for ResetPeriod {
    fn default() -> Self { Self::Period5 }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PrechargePeriod {
    Period3 = 3,
    Period4 = 4,
    Period5 = 5,
    Period6 = 6,
    Period7 = 7,
    Period8 = 8,
    Period9 = 9,
    Period10 = 10,
    Period11 = 11,
    Period12 = 12,
    Period13 = 13,
    Period14 = 14,
    Period15 = 15,
}
impl Default for PrechargePeriod {
    fn default() -> Self { Self::Period8 }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Divider {
    DivideBy1    = 0b0000,
    DivideBy2    = 0b0001,
    DivideBy4    = 0b0010,
    DivideBy8    = 0b0011,
    DivideBy16   = 0b0100,
    DivideBy32   = 0b0101,
    DivideBy64   = 0b0110,
    DivideBy128  = 0b0111,
    DivideBy256  = 0b1000,
    DivideBy512  = 0b1001,
    DivideBy1024 = 0b1010,
}
impl Default for Divider {
    fn default() -> Self { Self::DivideBy2 }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GpioState {
    HiZInputDisabled = 0b00,
    HiZInputEnabled = 0b01,
    OutputLow = 0b10,
    OutputHigh = 0b11,
}
impl Default for GpioState {
    fn default() -> Self { Self::OutputLow }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SecondPrechargePeriod {
    Period1 = 1,
    Period2 = 2,
    Period3 = 3,
    Period4 = 4,
    Period5 = 5,
    Period6 = 6,
    Period7 = 7,
    Period8 = 8,
    Period9 = 9,
    Period10 = 10,
    Period11 = 11,
    Period12 = 12,
    Period13 = 13,
    Period14 = 14,
    Period15 = 15,
}
impl Default for SecondPrechargePeriod {
    fn default() -> Self { Self::Period8 }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PrechargeVoltage {
    VccTimes1By5 = 0x00,
    VccTimes33By155 = 0x01,
    VccTimes7By31 = 0x02,
    VccTimes37By155 = 0x03,
    VccTimes39By155 = 0x04,
    VccTimes41By155 = 0x05,
    VccTimes43By155 = 0x06,
    VccTimes9By31 = 0x07,
    VccTimes47By155 = 0x08,
    VccTimes49By155 = 0x09,
    VccTimes51By155 = 0x0A,
    VccTimes53By155 = 0x0B,
    VccTimes11By31 = 0x0C,
    VccTimes57By155 = 0x0D,
    VccTimes59By155 = 0x0E,
    VccTimes61By155 = 0x0F,
    VccTimes63By155 = 0x10,
    VccTimes13By31 = 0x11,
    VccTimes67By155 = 0x12,
    VccTimes69By155 = 0x13,
    VccTimes71By155 = 0x14,
    VccTimes73By155 = 0x15,
    VccTimes15By31 = 0x16,
    VccTimes77By155 = 0x17,
    VccTimes79By155 = 0x18,
    VccTimes81By155 = 0x19,
    VccTimes83By155 = 0x1A,
    VccTimes17By31 = 0x1B,
    VccTimes87By155 = 0x1C,
    VccTimes89By155 = 0x1D,
    VccTimes91By155 = 0x1E,
    VccTimes3By5 = 0x1F,
}
impl Default for PrechargeVoltage {
    fn default() -> Self { Self::VccTimes77By155 }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ComDeselectVoltage {
    VccTimes0Point72 = 0b000,
    VccTimes0Point74 = 0b001,
    VccTimes0Point76 = 0b010,
    VccTimes0Point78 = 0b011,
    VccTimes0Point80 = 0b100,
    VccTimes0Point82 = 0b101,
    VccTimes0Point84 = 0b110,
    VccTimes0Point86 = 0b111,
}
impl Default for ComDeselectVoltage {
    fn default() -> Self { Self::VccTimes0Point82 }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GeneralContrast {
    Contrast1By16 = 0b0000,
    Contrast2By16 = 0b0001,
    Contrast3By16 = 0b0010,
    Contrast4By16 = 0b0011,
    Contrast5By16 = 0b0100,
    Contrast6By16 = 0b0101,
    Contrast7By16 = 0b0110,
    Contrast8By16 = 0b0111,
    Contrast9By16 = 0b1000,
    Contrast10By16 = 0b1001,
    Contrast11By16 = 0b1010,
    Contrast12By16 = 0b1011,
    Contrast13By16 = 0b1100,
    Contrast14By16 = 0b1101,
    Contrast15By16 = 0b1110,
    FullContrast = 0b1111,
}
impl Default for GeneralContrast {
    fn default() -> Self { Self::FullContrast }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ProtectionLevel {
    UnlockCommands = 0x12,
    LockCommands = 0x16,
    LockAdvancedCommands = 0xB0,
    UnlockAdvancedCommands = 0xB1,
}
impl Default for ProtectionLevel {
    fn default() -> Self { Self::UnlockAdvancedCommands }
}

#[derive(Clone, Copy, Debug, Eq, FromToRepr, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ScrollTimeInterval {
    TestMode = 0b00,
    Normal = 0b01,
    Slow = 0b10,
    Slowest = 0b11,
}
