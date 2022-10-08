//! EnOcean Radio Protocol data as it appears in an ESP3 packet.
//!
//! The structure of the contained data is not regulated in the EnOcean specification but is managed
//! by the EnOcean Alliance, a group of vendors with a common goal of increasing interoperability.
//! The most helpful document describing the common packet types is _EnOcean Equipment Profiles_.


use crate::esp3::{MAX_DATA_LENGTH, OneByteBoolean};
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

/// The maximum length of variable data in a SIG ESP3 packet.
///
/// Specified in _Signal Telegram_ (section 2).
pub const MAXIMUM_SIG_DATA_LENGTH: usize = 13;


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

    /// SIGNAL (0xD0)
    Signal(SignalTelegram),

    /// UTE (0xD4)
    UniversalTeachIn(UniversalTeachInTelegram),

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
            Self::Signal(_) => 0xD0,
            Self::UniversalTeachIn(_) => 0xD4,
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
            0xD0 => SignalTelegram::from_slice(&data_bytes)
                .map(Self::Signal),
            0xD4 => UniversalTeachInTelegram::from_slice(&data_bytes)
                .map(Self::UniversalTeachIn),
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
impl ErpStatusByte for OneByteTelegram {
    fn status_byte(&self) -> u8 { self.status }
}

/// Four-Byte Telegram (4BS, 0xA5)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
impl ErpStatusByte for FourByteTelegram {
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
impl ErpStatusByte for VariableLengthTelegram {
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


/// Signal telegram (SIG, 0xD0)
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignalTelegram {
    pub data: SignalData,
    pub sender: u32,
    pub status: u8,
}
impl SignalTelegram {
    /// Attempts to assemble this telegram from the given slice.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 6 || bytes.len() > 6 + MAXIMUM_SIG_DATA_LENGTH {
            return None;
        }

        let message_id = bytes[0];
        let signal_data = SignalData::from_id_and_slice(message_id, &bytes[1..bytes.len()-5])?;
        let sender = u32::from_be_bytes(bytes[bytes.len()-5..bytes.len()-1].try_into().unwrap());
        let status = bytes[bytes.len()-1];

        Some(Self {
            data: signal_data,
            sender,
            status,
        })
    }
}
impl ErpStatusByte for SignalTelegram {
    fn status_byte(&self) -> u8 { self.status }
}


/// Data of a signal telegram.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SignalData {
    SmartAckMailboxEmpty,
    SmartAckMailboxDoesNotExist,
    SmartAckReset,
    TriggerStatusMessage(StatusType),
    LastUnicastAcknowledge,
    EnergyPercentage(u8),
    Revision {
        software_version: u32,
        hardware_version: u32,
    },
    Heartbeat,
    RxWindowOpen,
    RxChannelQuality {
        request_sender: u32,
        worst_dbm: i8,
        best_dbm: i8,
        subtelegram_count: u8,
        max_repeater_level: u8,
    },
    DutyCycleAvailable(OneByteBoolean),
    ConfigChanged,
    HarvesterEnergyQuality(HarvesterEnergyQuality),
    TxOff,
    TxOn,
    BackupBatteryStatus(BackupBatteryStatus),
    LearnModeStatus {
        link_table_full: bool,
        teach_request_message_reception_enabled: bool,
        learn_mode_type: LearnModeType,
        teach_result: TeachResult,
        remaining_timeout_10s: u8,
        device_id: u32,
        device_eep: [u8; 3],
    },
    ProductId {
        manufacturer_id: u16,
        product_reference: u32,
    },
    Other {
        code: u8,
        data: MaxArray<u8, MAXIMUM_SIG_DATA_LENGTH>,
    }
}
impl SignalData {
    /// Assembles signal data from a message ID and a slice of optional data.
    pub fn from_id_and_slice(message_id: u8, signal_data: &[u8]) -> Option<Self> {
        match message_id {
            0x01 => Some(Self::SmartAckMailboxEmpty),
            0x02 => Some(Self::SmartAckMailboxDoesNotExist),
            0x03 => Some(Self::SmartAckReset),
            0x04 => {
                if signal_data.len() != 1 {
                    None
                } else {
                    Some(Self::TriggerStatusMessage(signal_data[0].into()))
                }
            },
            0x05 => Some(Self::LastUnicastAcknowledge),
            0x06 => {
                if signal_data.len() != 1 {
                    None
                } else {
                    Some(Self::EnergyPercentage(signal_data[0]))
                }
            },
            0x07 => {
                if signal_data.len() != 8 {
                    None
                } else {
                    Some(Self::Revision {
                        software_version: u32::from_be_bytes(signal_data[0..4].try_into().unwrap()),
                        hardware_version: u32::from_be_bytes(signal_data[4..8].try_into().unwrap()),
                    })
                }
            },
            0x08 => Some(Self::Heartbeat),
            0x09 => Some(Self::RxWindowOpen),
            0x0A => {
                if signal_data.len() != 7 {
                    None
                } else {
                    let request_sender = u32::from_be_bytes(signal_data[0..4].try_into().unwrap());
                    let worst_dbm = -(signal_data[4] as i8);
                    let best_dbm = -(signal_data[5] as i8);
                    let subtelegram_count = signal_data[6] >> 4;
                    let max_repeater_level = signal_data[6] & 0xF;

                    Some(Self::RxChannelQuality {
                        request_sender,
                        worst_dbm,
                        best_dbm,
                        subtelegram_count,
                        max_repeater_level,
                    })
                }
            },
            0x0B => {
                if signal_data.len() != 1 {
                    None
                } else {
                    Some(Self::DutyCycleAvailable(signal_data[0].into()))
                }
            },
            0x0C => Some(Self::ConfigChanged),
            0x0D => {
                if signal_data.len() != 1 {
                    None
                } else {
                    Some(Self::HarvesterEnergyQuality(signal_data[0].into()))
                }
            },
            0x0E => Some(Self::TxOff),
            0x0F => Some(Self::TxOn),
            0x10 => {
                if signal_data.len() != 1 {
                    None
                } else {
                    Some(Self::BackupBatteryStatus(signal_data[0].into()))
                }
            },
            0x11 => {
                if signal_data.len() != 9 {
                    None
                } else {
                    let link_table_full = (signal_data[0] & 0b1000_0000) != 0;
                    let teach_request_message_reception_enabled = (signal_data[0] & 0b0100_0000) != 0;
                    let learn_mode_type = ((signal_data[0] & 0b0011_0000) >> 4).into();
                    let teach_result = (signal_data[0] & 0b0000_1111).into();
                    let remaining_timeout_10s = signal_data[1];
                    let device_id = u32::from_be_bytes(signal_data[2..6].try_into().unwrap());
                    let device_eep = [
                        signal_data[6],
                        signal_data[7],
                        signal_data[8],
                    ];

                    Some(Self::LearnModeStatus {
                        link_table_full,
                        teach_request_message_reception_enabled,
                        learn_mode_type,
                        teach_result,
                        remaining_timeout_10s,
                        device_id,
                        device_eep,
                    })
                }
            },
            0x12 => {
                if signal_data.len() != 6 {
                    None
                } else {
                    let manufacturer_id = u16::from_be_bytes(signal_data[0..2].try_into().unwrap());
                    let product_reference = u32::from_be_bytes(signal_data[2..6].try_into().unwrap());
                    Some(Self::ProductId {
                        manufacturer_id,
                        product_reference,
                    })
                }
            },
            other => {
                if signal_data.len() > MAXIMUM_SIG_DATA_LENGTH {
                    None
                } else {
                    let data = MaxArray::from_iter_or_panic(
                        signal_data.iter().map(|b| *b).peekable()
                    );
                    Some(Self::Other {
                        code: other,
                        data,
                    })
                }
            },
        }
    }
}


/// A status message that may be requested.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum StatusType {
    EepStatus = 0x00,
    EnergyPercentage = 0x01,
    DeviceRevision = 0x02,
    RxLevelOfThisRequest = 0x03,
    CurrentHarvestedEnergy = 0x04,
    Other(u8),
}

/// The quality of the energy provided by the energy harvester.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum HarvesterEnergyQuality {
    VeryGood = 0x00,
    Good = 0x01,
    Average = 0x02,
    Bad = 0x03,
    VeryBad = 0x04,
    Other(u8),
}

/// The status of a backup battery.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BackupBatteryStatus {
    Percentage(u8),
    Reserved(u8),
    NoBattery,
}
impl From<u8> for BackupBatteryStatus {
    fn from(val: u8) -> Self {
        match val {
            0..=100 => Self::Percentage(val),
            101..=254 => Self::Reserved(val),
            255 => Self::NoBattery,
        }
    }
}
impl From<BackupBatteryStatus> for u8 {
    fn from(bbs: BackupBatteryStatus) -> Self {
        match bbs {
            BackupBatteryStatus::Percentage(v) => v,
            BackupBatteryStatus::Reserved(v) => v,
            BackupBatteryStatus::NoBattery => 255,
        }
    }
}

/// The type of learn mode used by the device.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum LearnModeType {
    Standard = 0b00,
    Extended1 = 0b01,
    Extended2 = 0b10,
    NotApplicable = 0b11,
    Other(u8),
}

/// The result of the teach-in process.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum TeachResult {
    TeachInSuccess = 0x0,
    TeachInFailedUnsupportedEep = 0x1,
    TeachInFailedTooManyDevicesOfEep = 0x2,
    TeachInFailedTooManyDevices = 0x3,
    TeachOutSuccess = 0x4,
    TeachOutFailedUnknownDeviceId = 0x5,
    Other(u8),
    NotApplicable = 0xF,
}

/// Universal Teach-In telegram (UTE, 0xD4)
///
/// Defined in _EnOcean Equipment Profiles_.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UniversalTeachInTelegram {
    pub data: UteData,
    pub sender: u32,
    pub status: u8,
}
impl UniversalTeachInTelegram {
    /// Attempts to assemble this telegram from the given slice.
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 12 {
            return None;
        }

        let data = UteData::from_slice(&bytes[0..7])?;
        Some(Self {
            data,
            sender: u32::from_be_bytes(bytes[7..11].try_into().unwrap()),
            status: bytes[11],
        })
    }
}
impl ErpStatusByte for UniversalTeachInTelegram {
    fn status_byte(&self) -> u8 { self.status }
}

/// Data in a Universal Teach-In (UTE) telegram.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UteData {
    Request(UteRequest),
    Response(UteResponse),
    Other {
        bidirectional_eep: bool,
        expects_teach_in_response: bool,
        status: u8,
        command: u8,
        data: [u8; 6],
    },
}
impl UteData {
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 7 {
            return None;
        }

        // pick through the first byte
        let bidirectional_eep = (bytes[0] & 0b1000_0000) != 0;
        let expects_teach_in_response = (bytes[0] & 0b0100_0000) != 0;
        let status = (bytes[0] & 0b0011_0000) >> 4;
        let command = bytes[0] & 0b0000_1111;

        match command {
            0x00 => UteRequest::from_ute_data(bidirectional_eep, expects_teach_in_response, status, &bytes[1..7])
                .map(Self::Request),
            0x01 => UteResponse::from_ute_data(bidirectional_eep, expects_teach_in_response, status, &bytes[1..7])
                .map(Self::Response),
            other => Some(Self::Other {
                bidirectional_eep,
                expects_teach_in_response,
                status,
                command: other,
                data: bytes[1..7].try_into().unwrap(),
            }),
        }
    }
}


/// A Universal Teach-In (UTE) request.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UteRequest {
    pub bidirectional_eep: bool,
    pub expects_teach_in_response: bool,
    pub request_type: UteRequestType,
    pub teach_in_channel: u8,
    pub manufacturer: u16,
    pub eep_type: u8,
    pub eep_func: u8,
    pub eep_rorg: u8,
}
impl UteRequest {
    pub fn from_ute_data(bidirectional_eep: bool, expects_teach_in_response: bool, status: u8, data: &[u8]) -> Option<Self> {
        if data.len() != 6 {
            return None;
        }

        let request_type = status.into();
        let teach_in_channel = data[0];
        let manufacturer =
            (u16::from(data[2]) << 8)
            | u16::from(data[1]);
        let eep_type = data[3];
        let eep_func = data[4];
        let eep_rorg = data[5];

        Some(Self {
            bidirectional_eep,
            expects_teach_in_response,
            request_type,
            teach_in_channel,
            manufacturer,
            eep_type,
            eep_func,
            eep_rorg,
        })
    }
}

/// The type of Universal Teach-In (UTE) request.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum UteRequestType {
    TeachIn = 0b00,
    TeachInDeletion = 0b01,
    TeachInOrDeletion = 0b10,
    Other(u8),
}

/// A Universal Teach-In (UTE) response.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UteResponse {
    pub bidirectional_eep: bool,
    pub response_type: UteResponseType,
    pub teach_in_channel: u8,
    pub manufacturer: u16,
    pub eep_type: u8,
    pub eep_func: u8,
    pub eep_rorg: u8,
}
impl UteResponse {
    pub fn from_ute_data(bidirectional_eep: bool, _expects_teach_in_response: bool, status: u8, data: &[u8]) -> Option<Self> {
        if data.len() != 6 {
            return None;
        }

        let response_type = status.into();
        let teach_in_channel = data[0];
        let manufacturer =
            (u16::from(data[2]) << 8)
            | u16::from(data[1]);
        let eep_type = data[3];
        let eep_func = data[4];
        let eep_rorg = data[5];

        Some(Self {
            bidirectional_eep,
            response_type,
            teach_in_channel,
            manufacturer,
            eep_type,
            eep_func,
            eep_rorg,
        })
    }
}

/// The type of Universal Teach-In (UTE) response.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum UteResponseType {
    NotAccepted = 0b00,
    TeachInSuccess = 0b01,
    DeletionSuccess = 0b10,
    UnsupportedEep = 0b11,
    Other(u8),
}
