//! EnOcean Radio Protocol data as it appears in an ESP3 packet.
//!
//! The structure of the contained data is not regulated in the EnOcean specification but is managed
//! by the EnOcean Alliance, a group of vendors with a common goal of increasing interoperability.
//! The most helpful document describing the common packet types is _EnOcean Equipment Profiles_.


use crate::esp3::MAX_DATA_LENGTH;
use crate::max_array::MaxArray;


/// The maximum length of variable data in a VLD ESP3 packet.
///
/// Specified in _EnOcean Equipment Profiles_ (section 3.1.3).
pub const MAXIMUM_VLD_DATA_LENGTH: usize = 14;

/// The maximum length of variable data in an MSC ESP3 packet.
///
/// Technically 12.5 bytes (100 bits), but we round this up for at-rest use cases.
///
/// Specified in _EnOcean Equipment Profiles_ (section 3.1.3).
pub const MAXIMUM_MSC_DATA_LENGTH: usize = 13;


/// EnOcean Radio Protocol data in an ESP3 packet.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErpData {
    /// RPS (0xF6)
    RepeatedSwitch(RepeatedSwitchTelegram),

    /// 1BS (0xD5)
    OneByte(OneByteTelegram),

    /// 4BS (0xA5)
    FourByte(FourByteTelegram),

    /// VLD (0xD2)
    VariableLength(VariableLengthTelegram),

    // the following telegrams might be implemented eventually
    // but are currently considered out-of-scope

    /*
    /// MSC (0xD1)
    ManufacturerSpecific(ManufacturerSpecificTelegram),

    /// ADT (0xA6)
    AddressingDestination(AddressingDestinationTelegram),

    /// SM_LRN_REQ (0xC6)
    SmartAckLearnRequest(SmartAckLearnRequestTelegram),

    /// SM_LRN_ANS (0xC7)
    SmartAckLearnAnswer(SmartAckLearnAnswerTelegram),

    /// SM_REC (0xA7)
    SmartAckReclaim(SmartAckReclaimTelegram),

    /// SYS_EX (0xC5)
    RemoteManagement(RemoteManagementTelegram),

    /// SEC (0x30)
    Secure(SecureTelegram),

    /// SEC_ENCAPS (0x31)
    SecureEncapsulated(SecureEncapsulatedTelegram),

    /// SEC_MAN (0x34)
    MaintenanceSecurity(MaintenanceSecurityTelegram),

    /// SIGNAL (0xD0)
    Signal(SignalTelegram),

    /// UTE (0xD4)
    UniversalTeachIn(UniversalTeachInTelegram),
    */

    /// Another type of telegram.
    Other {
        rorg: u8,
        data: MaxArray<u8, {MAX_DATA_LENGTH-1}>,
    },
}
impl ErpData {
    /// Returns the RORG value for this ERP telegram.
    pub fn rorg_value(&self) -> u8 {
        match self {
            Self::RepeatedSwitch(_) => 0xF6,
            Self::OneByte(_) => 0xD5,
            Self::FourByte(_) => 0xA5,
            Self::VariableLength(_) => 0xD2,
            Self::Other { rorg, .. } => *rorg,
        }
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 1 {
            None
        } else {
            Self::from_rorg_and_data(bytes[0], &bytes[1..])
        }
    }

    pub fn from_rorg_and_data(rorg: u8, data_bytes: &[u8]) -> Option<Self> {
        // match the RORG value
        match rorg {
            0xF6 => RepeatedSwitchTelegram::from_slice(&data_bytes)
                .map(Self::RepeatedSwitch),
            0xD5 => OneByteTelegram::from_slice(&data_bytes)
                .map(Self::OneByte),
            0xA5 => FourByteTelegram::from_slice(&data_bytes)
                .map(Self::FourByte),
            0xD2 => VariableLengthTelegram::from_slice(&data_bytes)
                .map(Self::VariableLength),
            other => Some(Self::Other {
                rorg: other,
                data: MaxArray::from_iter_or_panic(
                    data_bytes.iter().map(|b| *b).peekable(),
                ),
            }),
        }
    }
}


/// A trait for ERP telegrams that have a status byte.
pub trait ErpStatusByte {
    /// The value of the status byte contained in this ERP telegram.
    fn status_byte(&self) -> u8;

    /// Indicates whether this telegram has been or should be repeated.
    fn repeater_count(&self) -> RepeaterCount {
        (self.status_byte() & 0b1111).into()
    }

    /// Indicates whether the CRC8 algorithm is used for checksum calculation instead of a simple
    /// sum modulo 255.
    ///
    /// The CRC8 polynomial in use is CRC8-CCITT.
    fn uses_crc8(&self) -> bool {
        (self.status_byte() & (1 << 7)) != 0
    }
}


/// Repeated Switch Telegram (RPS, 0xF6)
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepeatedSwitchTelegram {
    pub data: u8,
    pub sender: u32,
    pub status: u8,
}
impl RepeatedSwitchTelegram {
    /// `true` if this is an N-message (normal message); `false` if this is a U-message (unassigned
    /// message).
    pub fn is_normal(&self) -> bool {
        (self.status & (1 << 4)) != 0
    }

    /// `true` if this message is from a type-2 module; `false` if this message is from a type-1
    /// module.
    ///
    /// Type 1 generally corresponds to PTM1xx; type 2 to PTM2xx.
    pub fn is_type2(&self) -> bool {
        (self.status & (1 << 5)) != 0
    }

    /// Attempts to assemble this telegram from the given slice.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 6 {
            return None;
        }

        Some(Self {
            data: bytes[0],
            sender: u32::from_be_bytes(bytes[1..5].try_into().unwrap()),
            status: bytes[5],
        })
    }
}
impl ErpStatusByte for RepeatedSwitchTelegram {
    fn status_byte(&self) -> u8 { self.status }
}

/// One-Byte Telegram (1BS, 0xD5)
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OneByteTelegram {
    pub data: u8,
    pub sender: u32,
    pub status: u8,
}
impl OneByteTelegram {
    /// Indicates whether this packet is a teach-in packet (i.e., the LRN bit is set).
    pub fn is_teach_in(&self) -> bool {
        (self.data & (1 << 3)) == 0
    }

    /// Attempts to assemble this telegram from the given slice.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 6 {
            return None;
        }

        Some(Self {
            data: bytes[0],
            sender: u32::from_be_bytes(bytes[1..5].try_into().unwrap()),
            status: bytes[5],
        })
    }
}
impl ErpStatusByte for RepeatedSwitchTelegram {
    fn status_byte(&self) -> u8 { self.status }
}

/// Four-Byte Telegram (4BS, 0xA5)
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FourByteTelegram {
    pub data: u32,
    pub sender: u32,
    pub status: u8,
}
impl FourByteTelegram {
    /// Indicates whether this packet is a teach-in packet (i.e., the LRN bit is set).
    pub fn is_teach_in(&self) -> bool {
        (self.data & (1 << 3)) == 0
    }

    /// Indicates whether this packet is a teach-in packet and contains added information.
    pub fn is_detailed_teach_in(&self) -> bool {
        self.is_teach_in() && (self.data & (1 << 7)) == 0
    }

    /// Returns details about the device if this is a teach-in telegram and contains this
    /// information.
    pub fn teach_in_details(&self) -> Option<FourByteTeachInDetails> {
        if !self.is_detailed_teach_in() {
            None
        } else {
            let func_value = (self.data >> 26) & 0b111111;
            let type_value = (self.data >> 19) & 0b1111111;
            let manufacturer_id = (self.data >> 8) & 0b111_11111111;
            Some(FourByteTeachInDetails {
                func_value: func_value.try_into().unwrap(),
                type_value: type_value.try_into().unwrap(),
                manufacturer_id: manufacturer_id.try_into().unwrap(),
            })
        }
    }

    /// Returns the teach-in flags for this telegram.
    pub fn teach_in_flags(&self) -> Option<FourByteTeachInFlags> {
        if !self.is_detailed_teach_in() {
            None
        } else {
            Some(FourByteTeachInFlags {
                is_response: (self.data & (1 << 4)) != 0,
                sender_id_is_stored: (self.data & (1 << 5)) != 0,
                eep_is_supported: (self.data & (1 << 6)) != 0,
            })
        }
    }

    /// Attempts to assemble this telegram from the given slice.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 9 {
            return None;
        }

        Some(Self {
            data: u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            sender: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            status: bytes[8],
        })
    }
}
impl ErpStatusByte for RepeatedSwitchTelegram {
    fn status_byte(&self) -> u8 { self.status }
}

/// Variable-Length Data Telegram (VLD, 0xD2)
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariableLengthTelegram {
    pub data: MaxArray<u8, MAXIMUM_VLD_DATA_LENGTH>,
    pub sender: u32,
    pub status: u8,
}
impl VariableLengthTelegram {
    /// Attempts to assemble this telegram from the given slice.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        // at least one byte is required
        if bytes.len() < 6 || bytes.len() > 5 + MAXIMUM_VLD_DATA_LENGTH {
            return None;
        }

        Some(Self {
            data: MaxArray::from_iter_or_panic(
                bytes[0..bytes.len()-5].iter().map(|b| *b).peekable()
            ),
            sender: u32::from_be_bytes(bytes[bytes.len()-5..bytes.len()-1].try_into().unwrap()),
            status: bytes[bytes.len()-1],
        })
    }
}
impl ErpStatusByte for RepeatedSwitchTelegram {
    fn status_byte(&self) -> u8 { self.status }
}


/// The repeater count values in the status byte.
///
/// These are specified in EnOcean's ERP1 documentation (section 5.2.3).
#[derive(Copy, Clone, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum RepeaterCount {
    /// This is the packet.
    OriginalSender = 0b0000,

    /// This is a copy of the packet.
    RepeatedOnce = 0b0001,

    /// This is a copy of a copy of the packet.
    RepeatedTwice = 0b0010,

    /// This packet is not to be repeated under any circumstances.
    DoNotRepeat = 0b1111,

    Other(u8),
}

/// Detailed information about a device transmitted as part of some four-byte teach-in telegrams.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FourByteTeachInDetails {
    /// The function value (second byte of the EEP profile number).
    ///
    /// The first byte of the EEP profile number is the RORG value of the telegram.
    pub func_value: u8,

    /// The type value (third byte of the EEP profile number).
    pub type_value: u8,

    /// The ID of the manufacturer of the device.
    pub manufacturer_id: u16,
}

/// The teach-in flags relevant to bidirectional teach-in using four-byte telegrams.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FourByteTeachInFlags {
    /// `false` if this is a query, `true` if this is a response. (LRN Status)
    pub is_response: bool,

    /// `false` if the sender ID is deleted or not stored, `true` if it is stored. (LRN Result)
    pub sender_id_is_stored: bool,

    /// `false` if EEP is not supported, `true` if it is. (EEP Result)
    pub eep_is_supported: bool,
}
