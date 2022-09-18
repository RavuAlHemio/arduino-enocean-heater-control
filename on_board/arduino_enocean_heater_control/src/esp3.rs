//! An implementation of the EnOcean Serial Protocol 3 (ESP3).


use bitflags::bitflags;
use buildingblocks::crc8::crc8_ccitt;
use buildingblocks::max_array::MaxArray;

use crate::ring_buffer::CriticalRingBuffer;


/// The (constant) length of an ESP3 packet header.
///
/// The header consists of:
/// 1. The sync byte (always 0x55, 1 byte long)
/// 2. The data length (maximum 0xFFFF, 2 bytes long)
/// 3. The optional data length (maximum 0xFF, 1 byte long)
/// 4. The packet type (1 byte long)
/// 5. The CRC8 checksum of the header (1 byte long)
const HEADER_LENGTH: usize = 1 + 2 + 1 + 1 + 1;


/// The maximum length of the (non-optional) data in an ESP3 packet.
///
/// The data length field can store values of up to 0xFFFF, i.e. 65535.
const MAX_DATA_LENGTH: usize = 0xFFFF;


/// The maximum length of the optional data in an ESP3 packet.
///
/// The optional data length field can store values of up to 0xFF, i.e. 255.
const MAX_OPTIONAL_LENGTH: usize = 0xFF;


/// The (constant) length of an ESP3 packet footer.
///
/// The footer only contains the one-byte CRC8 value of the data and optional data.
const FOOTER_LENGTH: usize = 1;


/// The maximum theoretical length of an ESP3 packet.
///
/// The longest theoretical ESP3 packet consists of the following:
/// 1. The sync byte (always 0x55, 1 byte long)
/// 2. The data length (maximum 0xFFFF, 2 bytes long)
/// 3. The optional data length (maximum 0xFF, 1 byte long)
/// 4. The packet type (1 byte long)
/// 5. The CRC8 checksum of the header (1 byte long)
/// 6. The data (65535 bytes long according to the size of the data length field)
/// 7. The optional data (255 bytes according to the size of the optional data length field)
/// 8. The CRC8 checksum of the data (1 byte long)
const MAX_ESP3_PACKET_LENGTH: usize =
    HEADER_LENGTH + MAX_DATA_LENGTH + MAX_OPTIONAL_LENGTH + FOOTER_LENGTH
;


/// The byte used for synchronization.
const SYNC_BYTE: u8 = 0x55;


/// The ring buffer used for decoding incoming ESP3 packets.
static ESP3_BUFFER: CriticalRingBuffer<u8, MAX_ESP3_PACKET_LENGTH> = CriticalRingBuffer::new();


/// Takes bytes from the ESP3 ring buffer until a valid ESP3 packet is decoded, then returns its
/// bytes.
pub fn take_esp3_packet() -> Option<MaxArray<u8, MAX_ESP3_PACKET_LENGTH>> {
    // loop to get an actual packet
    let (data_length, opt_length, total_length) = loop {
        // loop to fast-forward to sync byte communication
        loop {
            // toss out bytes until we get a potential sync byte
            match ESP3_BUFFER.peek_at(0) {
                None => return None,
                Some(SYNC_BYTE) => break,
                Some(_) => {
                    ESP3_BUFFER.pop();
                    continue;
                },
            }
        }

        // we need at least the size of a zero-data packet
        if ESP3_BUFFER.len() < 7 {
            return None;
        }

        // peek at the header
        let mut possible_header = [0u8; HEADER_LENGTH];
        ESP3_BUFFER.peek_fill(&mut possible_header);

        // does the CRC8 match?
        let calculated_crc = crc8_ccitt(&possible_header[0..HEADER_LENGTH-1]);
        if calculated_crc == possible_header[HEADER_LENGTH-1] {
            // yes -- it's a packet!

            // have we already collected all of it?
            let data_length = u16::from_be_bytes(possible_header[1..3].try_into().unwrap());
            let opt_length = possible_header[3];
            // header, data, opt data, data CRC8
            let total_length = HEADER_LENGTH + usize::from(data_length) + usize::from(opt_length) + FOOTER_LENGTH;
            if ESP3_BUFFER.len() < total_length {
                // nope, we still need a few more bytes
                return None;
            }

            // alright, keep processing it!
            break (data_length, opt_length, total_length);
        }

        // no, it isn't a valid packet... pop the sync byte and search for the next one
        ESP3_BUFFER.pop();
    };

    // take out the packet
    let mut packet = MaxArray::new();
    while packet.len() < total_length {
        packet.push(ESP3_BUFFER.pop().expect("failed to pop from ESP3 buffer"))
            .expect("failed to push to ESP3 packet");
    }

    // check data CRC8
    let data_crc8 = packet.as_slice()[total_length-1];
    let data_crc8_calc = crc8_ccitt(&packet.as_slice()[HEADER_LENGTH..total_length-1]);
    if data_crc8 == data_crc8_calc {
        Some(packet)
    } else {
        None
    }
}


/// The contents of an ESP3 data packet.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Esp3Packet {
    RadioErp1 {
        radio_telegram: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_sub_telegram_number: Option<u8>,
        opt_destination_id: Option<u32>,
        opt_dbm: Option<u8>,
        opt_security_level: Option<SecurityLevel>,
    },
    Response {
        return_code: ReturnCode,
        response_data: MaxArray<u8, MAX_DATA_LENGTH>,
    },
    RadioSubTelegram {
        radio_telegram: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_sub_telegram_number: Option<u8>,
        opt_destination_id: Option<u32>,
        opt_dbm: Option<u8>,
        opt_security_level: Option<SecurityLevel>,
        opt_timestamp: u16,
        // 3 bytes for each subtelegram; opt length up to now is 9 => up to 82 subtelegrams
        opt_sub_telegram_info: MaxArray<SubTelegramInfo, 82>,
    },
    Event(EventData),
    CommonCommand(CommandData),
}
impl Esp3Packet {
    pub fn packet_type(&self) -> u8 {
        match self {
            Self::RadioErp1 { .. } => 0x01,
            Self::Response { .. } => 0x02,
            Self::RadioSubTelegram { .. } => 0x03,
            Self::Event(_) => 0x04,
            Self::CommonCommand(_) => 0x05,
        }
    }
}


/// An ESP3 security level value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum SecurityLevel {
    NotProcessed = 0x0,
    Obsolete = 0x1,
    Decrypted = 0x2,
    Authenticated = 0x3,
    DecryptedAuthenticated = 0x4,
    Other(u8),
}


/// The type of response.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum ReturnCode {
    Ok = 0x00,
    Error = 0x01,
    NotSupported = 0x02,
    WrongParam = 0x03,
    OperationDenied = 0x04,
    LockSet = 0x05,
    BufferTooSmall = 0x06,
    NoFreeBuffer = 0x07,
    MemoryError = 0x82,
    BaseIdOutOfRange = 0x90,
    BaseIdMaxReached = 0x91,
    Other(u8),
}

/// Information about a single sub-telegram.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubTelegramInfo {
    pub tick: u8,
    pub dbm: u8,
    pub status: u8,
}

/// An EnOcean event that may occur.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EventData {
    SmartAckReclaimNotSuccessful,
    SmartAckConfirmLearn {
        postmaster_priority: PostmasterPriority,
        manufacturer_id: u16,
        eep: u32,
        rssi: u8,
        postmaster_candidate_id: u32,
        smart_ack_client_id: u32,
        hop_count: u8,
    },
    SmartAckLearnAck {
        response_time: u16,
        confirm_code: SmartAckConfirmCode,
    },
    CoReady {
        wakeup_cause: WakeupCause,
        opt_security_mode: Option<SecurityMode>,
    },
    CoEventSecureDevices {
        cause: SecureDeviceEventCause,
        device_id: u32,
    },
    CoDutyCycleLimit {
        sending_possible: OneByteBoolean,
    },
    CoTransmitFailed {
        reason: TransmissionFailureReason,
    },
    CoTxDone,
    CoLearnModeDisabled,
    Unknown {
        code: u8,
        data: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
    },
}
impl EventData {
    pub fn event_type(&self) -> u8 {
        match self {
            Self::SmartAckReclaimNotSuccessful => 0x01,
            Self::SmartAckConfirmLearn { .. } => 0x02,
            Self::SmartAckLearnAck { .. } => 0x03,
            Self::CoReady { .. } => 0x04,
            Self::CoEventSecureDevices { .. } => 0x05,
            Self::CoDutyCycleLimit { .. } => 0x06,
            Self::CoTransmitFailed { .. } => 0x07,
            Self::CoTxDone => 0x08,
            Self::CoLearnModeDisabled => 0x09,
            Self::Unknown { code, .. } => *code,
        }
    }

    pub fn expects_response(&self) -> bool {
        match self {
            Self::SmartAckReclaimNotSuccessful => true,
            Self::SmartAckConfirmLearn { .. } => true,
            _ => false,
        }
    }
}

bitflags! {
    pub struct PostmasterPriority : u8 {
        const LOCAL = 0b0000_0001;
        const GOOD_RSSI = 0b0000_0010;
        const MAILBOX_PLACE = 0b0000_0100;
        const ALREADY_POSTMASTER = 0b0000_1000;
    }
}

/// A Smart Acknowledgement confirmation code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum SmartAckConfirmCode {
    LearnIn = 0x00,
    DiscardEep = 0x11,
    DiscardNoPlaceMb = 0x12,
    DiscardNoPlaceSensor = 0x13,
    DiscardRssi = 0x14,
    LearnOut = 0x20,
    Other(u8),
}

/// The reason for the EnOcean controller being activated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum WakeupCause {
    VoltageSupplyDrop = 0x00,
    ResetPin = 0x01,
    Watchdog = 0x02,
    Flywheel = 0x03,
    ParityError = 0x04,
    HardwareParityError = 0x05,
    PageFault = 0x06,
    WakeUpPin0 = 0x07,
    WakeUpPin1 = 0x08,
    UnknownSource = 0x09,
    Uart = 0x10,
    Other(u8),
}

/// The security mode in which the controller is operating.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum SecurityMode {
    Standard = 0x00,
    Extended = 0x01,
    Other(u8),
}

/// The cause for the secure device event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum SecureDeviceEventCause {
    SecureLinkTableFull = 0x00,
    // 0x01 is reserved
    WrongPrivKeyResync = 0x02,
    WrongCmacCountHit = 0x03,
    TelegramCorrupted = 0x04,
    PskUnset = 0x05,
    TeachInWithoutPsk = 0x06,
    CmacOrRlc = 0x07,
    InsecureTelegramSecureDevice = 0x08,
    TeachInSuccess = 0x09,
    ValidRlcSync = 0x0A,
    Other(u8),
}

/// A boolean value stored in one byte.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum OneByteBoolean {
    Yes = 0x00,
    No = 0x01,
    Other(u8),
}
impl From<bool> for OneByteBoolean {
    fn from(v: bool) -> Self {
        if v { Self::Yes } else { Self::No }
    }
}

/// The reason for a transmission failure.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum TransmissionFailureReason {
    CsmaFailed = 0x00,
    NotAcknowledged = 0x01,
    Other(u8),
}

/// An EnOcean common command.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CommandData {
    CoWrSleep {
        deep_sleep_period: u32,
    },
    CoWrReset,
    CoRdVersion,
    CoRdSysLog,
    CoWrSysLog,
    CoWrBiSt,
    CoWrIdBase {
        base_id: u32,
    },
    CoRdIdBase,
    CoWrRepeater {
        enable: RepeaterEnable,
        level: RepeaterLevel,
    },
    CoRdRepeater,
    CoWrFilterAdd {
        criterion: FilterCriterion,
        value: u32,
        action: FilterAction,
    },
    CoWrFilterDel {
        criterion: FilterCriterion,
        value: u32,
        action: FilterAction,
    },
    CoWrFilterDelAll,
    CoWrFilterEnable {
        enable: OneByteBoolean,
        operator: FilterOperator,
    },
    CoRdFilter,
    CoWrWaitMaturity {
        wait_for_maturity: OneByteBoolean,
    },
    CoWrSubTelegram {
        enable_subtelegram_info: OneByteBoolean,
    },
    CoWrMem {
        memory_type: MemoryType,
        address: u32,
        // max data minus (one byte command plus five fixed bytes)
        data: MaxArray<u8; 0xFFFF - (1 + 5)>,
    },
    CoRdMem {
        memory_type: MemoryType,
        address: u32,
        length: u16,
    },
    CoRdMemAddress,
    CoRdSecurity,
    CoWrSecurity,
    CoWrLearnMode,
    CoRdLearnMode,
    CoWrSecureDeviceAdd,
    CoWrSecureDeviceDel,
    CoRdSecureDeviceByIndex,
    CoWrMode,
    CoRdNumSecureDevices,
    CoRdSecureDeviceById,
    CoWrSecureDeviceAddPsk,
    CoWrSecureDeviceSendTeachIn,
    CoWrTemporaryRlcWindow,
    CoRdSecureDevicePsk,
    CoRdDutyCycleLimit,
    CoSetBaudRate,
    CoGetFrequencyInfo,
    CoGetStepCode,
    CoWrReManCode,
    CoWrStartupDelay,
    CoWrReManRepeating,
    CoRdReManRepeating,
    CoSetNoiseThreshold,
    CoGetNoiseThreshold,
    CoWrRlcSavePeriod,
    CoWrRlcLegacyMode,
    CoWrSecureDeviceV2Add,
    CoRdSecureDeviceV2ByIndex,
    CoWrRssiTestMode,
    CoRdRssiTestMode,
    CoWrSecureDeviceMaintenanceKey,
    CoRdSecureDeviceMaintenanceKey,
    CoWrTransparentMode,
    CoRdTransparentMode,
    CoWrTxOnlyMode,
    CoRdTxOnlyMode,
    Unknown {
        code: u8,
        data: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
    },
}
impl CommandData {
    pub fn command_type(&self) -> u8 {
        match self {
            Self::CoWrSleep { .. } => 1,
            Self::CoWrReset => 2,
            Self::CoRdVersion => 3,
            Self::CoRdSysLog => 4,
            Self::CoWrSysLog => 5,
            Self::CoWrBiSt => 6,
            Self::CoWrIdBase => 7,
            Self::CoRdIdBase => 8,
            Self::CoWrRepeater => 9,
            Self::CoRdRepeater => 10,
            Self::CoWrFilterAdd => 11,
            Self::CoWrFilterDel => 12,
            Self::CoWrFilterDelAll => 13,
            Self::CoWrFilterEnable => 14,
            Self::CoRdFilter => 15,
            Self::CoWrWaitMaturity => 16,
            Self::CoWrSubTelegram => 17,
            Self::CoWrMem => 18,
            Self::CoRdMem => 19,
            Self::CoRdMemAddress => 20,
            Self::CoRdSecurity => 21,
            Self::CoWrSecurity => 22,
            Self::CoWrLearnMode => 23,
            Self::CoRdLearnMode => 24,
            Self::CoWrSecureDeviceAdd => 25,
            Self::CoWrSecureDeviceDel => 26,
            Self::CoRdSecureDeviceByIndex => 27,
            Self::CoWrMode => 28,
            Self::CoRdNumSecureDevices => 29,
            Self::CoRdSecureDeviceById => 30,
            Self::CoWrSecureDeviceAddPsk => 31,
            Self::CoWrSecureDeviceSendTeachIn => 32,
            Self::CoWrTemporaryRlcWindow => 33,
            Self::CoRdSecureDevicePsk => 34,
            Self::CoRdDutyCycleLimit => 35,
            Self::CoSetBaudRate => 36,
            Self::CoGetFrequencyInfo => 37,
            // 38 = reserved
            Self::CoGetStepCode => 39,
            // 40-45 = reserved
            Self::CoWrReManCode => 46,
            Self::CoWrStartupDelay => 47,
            Self::CoWrReManRepeating => 48,
            Self::CoRdReManRepeating => 49,
            Self::CoSetNoiseThreshold => 50,
            Self::CoGetNoiseThreshold => 51,
            // 52-53 = reserved
            Self::CoWrRlcSavePeriod => 54,
            Self::CoWrRlcLegacyMode => 55,
            Self::CoWrSecureDeviceV2Add => 56,
            Self::CoRdSecureDeviceV2ByIndex => 57,
            Self::CoWrRssiTestMode => 58,
            Self::CoRdRssiTestMode => 59,
            Self::CoWrSecureDeviceMaintenanceKey => 60,
            Self::CoRdSecureDeviceMaintenanceKey => 61,
            Self::CoWrTransparentMode => 62,
            Self::CoRdTransparentMode => 63,
            Self::CoWrTxOnlyMode => 64,
            Self::CoRdTxOnlyMode => 65,
            Self::Unknown { code, .. } => *code,
        }
    }
}

/// Response data to CommandData::CoRdVersion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdVersion {
    pub app_version: u32,
    pub api_version: u32,
    pub chip_id: u32,
    pub chip_version: u32,
    pub app_description: [char; 16],
}

/// Response data to CommandData::CoRdSysLog.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdSysLog {
    pub api_log_entries: MaxArray<u8; MAX_DATA_LENGTH>,
    pub app_log_entries: MaxArray<u8; MAX_OPTIONAL_LENGTH>,
}

/// Response data to CommandData::CoWrBiSt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoWrBiSt {
    pub bist_result: u8,
}

/// Response data to CommandData::CoRdIdBase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdIdBase {
    pub base_id: u32,
    pub opt_remaining_write_cycles: Option<u8>,
}

/// Response data to CommandData::CoRdRepeater.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdRepeater {
    pub enable: RepeaterEnable,
    pub level: RepeaterLevel,
}

/// Response data to CommandData::CoRdFilter.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdFilter {
    // max data length minus 1 byte of return code
    // 5 bytes per filter
    pub filters: MaxArray<FilterEntry, (MAX_DATA_LENGTH - 1)/5>,
}

/// Response data to CommandData::CoRdMem.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdMem {
    // max data length minus 1 byte of return code
    pub data: MaxArray<u8, MAX_DATA_LENGTH - 1>,
}

/// The mode in which to enable the internal repeater.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum RepeaterEnable {
    Off = 0x00,
    On = 0x01,
    Selective = 0x02,
    Other(u8),
}

/// The level with which to repeat datagrams.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[from_to_repr::from_to_other(base_type = u8)]
pub enum RepeaterLevel {
    Off = 0x00,
    OneLevel = 0x01,
    TwoLevel = 0x02,
    Other(u8),
}

/// A single entry in the filter list read back from the controller.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FilterEntry {
    pub criterion: FilterCriterion,
    pub value: u32,
}