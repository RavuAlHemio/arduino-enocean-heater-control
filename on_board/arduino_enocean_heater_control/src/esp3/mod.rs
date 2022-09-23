//! An implementation of the EnOcean Serial Protocol 3 (ESP3).


pub(crate) mod response_data;
pub(crate) mod serial;


use bitflags::bitflags;
use buildingblocks::crc8::crc8_ccitt;
use buildingblocks::max_array::MaxArray;
use buildingblocks::max_array_ext::MaxArrayPushIntExt;


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
/// The data length field can store values of up to 0xFFFF, i.e. 65535. However, for pragmatic
/// reasons (RAM limitations), the maximum size can be reduced; longer packets are silently
/// discarded.
#[cfg(feature = "full_esp3_packet")]
const MAX_DATA_LENGTH: usize = 0xFFFF;

/// The maximum length of the (non-optional) data in an ESP3 packet.
///
/// The data length field can store values of up to 0xFFFF, i.e. 65535. However, for pragmatic
/// reasons (RAM limitations), the maximum size can be reduced; longer packets are silently
/// discarded.
#[cfg(not(feature = "full_esp3_packet"))]
const MAX_DATA_LENGTH: usize = 0x0FFF;


/// The maximum length of the optional data in an ESP3 packet.
///
/// The optional data length field can store values of up to 0xFF, i.e. 255.
const MAX_OPTIONAL_LENGTH: usize = 0xFF;


/// The (constant) length of an ESP3 packet footer.
///
/// The footer only contains the one-byte CRC8 value of the data and optional data.
const FOOTER_LENGTH: usize = 1;


/// The minimum theoretical length of an ESP3 packet.
///
/// The longest theoretical ESP3 packet consists of the following:
/// 1. The sync byte (always 0x55, 1 byte long)
/// 2. The data length (minimum 0x0000, 2 bytes long)
/// 3. The optional data length (minimum 0x00, 1 byte long)
/// 4. The packet type (1 byte long)
/// 5. The CRC8 checksum of the header (1 byte long)
/// 6. No data (0 bytes)
/// 7. No optional data (0 bytes)
/// 8. The CRC8 checksum of the data (1 byte long)
const MIN_ESP3_PACKET_LENGTH: usize =
    HEADER_LENGTH + FOOTER_LENGTH
;


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
pub const MAX_ESP3_PACKET_LENGTH: usize =
    HEADER_LENGTH + MAX_DATA_LENGTH + MAX_OPTIONAL_LENGTH + FOOTER_LENGTH
;


/// The byte used for synchronization.
const SYNC_BYTE: u8 = 0x55;


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
        response_data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
    },
    RadioSubTelegram {
        radio_telegram: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_sub_telegram_number: Option<u8>,
        opt_destination_id: Option<u32>,
        opt_dbm: Option<u8>,
        opt_security_level: Option<SecurityLevel>,
        opt_timestamp: Option<u16>,
        // 3 bytes for each subtelegram; opt length up to now is 9
        opt_sub_telegram_info: MaxArray<SubTelegramInfo, {(MAX_OPTIONAL_LENGTH - 9)/3}>,
    },
    Event(EventData),
    CommonCommand(CommandData),
    SmartAckCommand(SmartAckData),
    RemoteManCommand {
        function: u16,
        manufacturer: u16,
        message: MaxArray<u8, {MAX_DATA_LENGTH - 4}>,
        opt_destination_id: Option<u32>,
        opt_source_id: Option<u32>,
        opt_dbm: Option<u8>,
        opt_send_with_delay: Option<OneByteBoolean>,
    },
    RadioMessage {
        rorg: u8,
        data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
        opt_destination_id: Option<u32>,
        opt_source_id: Option<u32>,
        opt_dbm: Option<u8>,
        opt_security_level: Option<SecurityLevel>,
    },
    RadioErp2 {
        data: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_sub_telegram_number: Option<u8>,
        opt_dbm: Option<u8>,
        opt_security_level: Option<SecurityLevel>,
    },
    CommandAccepted {
        is_blocking: OneByteBoolean,
        estimated_time_ms: u16,
    },
    Radio802Dot15Dot4 {
        raw_data: MaxArray<u8, MAX_DATA_LENGTH>,
        opt_rssi: Option<u8>,
    },
    Command24(Command24Data),
    Unknown {
        packet_type: u8,
        data: MaxArray<u8, MAX_DATA_LENGTH>,
        optional_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
    },
}
impl Esp3Packet {
    pub fn packet_type(&self) -> u8 {
        match self {
            Self::RadioErp1 { .. } => 1,
            Self::Response { .. } => 2,
            Self::RadioSubTelegram { .. } => 3,
            Self::Event(_) => 4,
            Self::CommonCommand(_) => 5,
            Self::SmartAckCommand(_) => 6,
            Self::RemoteManCommand { .. } => 7,
            // 8 is undefined
            Self::RadioMessage { .. } => 9,
            Self::RadioErp2 { .. } => 10,
            // 11 is undefined
            Self::CommandAccepted { .. } => 12,
            // 13-15 are undefined
            Self::Radio802Dot15Dot4 { .. } => 16,
            Self::Command24 { .. } => 17,
            Self::Unknown { packet_type, .. } => *packet_type,
        }
    }

    pub fn to_packet(&self) -> Option<MaxArray<u8, MAX_ESP3_PACKET_LENGTH>> {
        let mut packet_data = self.to_packet_data();
        assert!(packet_data.len() <= MAX_DATA_LENGTH);
        let mut packet_optional = self.to_packet_optional()?;
        assert!(packet_optional.len() <= MAX_OPTIONAL_LENGTH);

        let mut buf = MaxArray::new();
        buf.push(SYNC_BYTE).unwrap();
        buf.push_u16_be(packet_data.len().try_into().unwrap()).unwrap();
        buf.push(packet_optional.len().try_into().unwrap()).unwrap();
        buf.push(self.packet_type()).unwrap();

        let crc_header = crc8_ccitt(&buf.as_slice()[1..5]);
        buf.push(crc_header).unwrap();

        while let Some(b) = packet_data.pop() {
            buf.push(b).unwrap();
        }
        while let Some(b) = packet_optional.pop() {
            buf.push(b).unwrap();
        }

        let crc_data = crc8_ccitt(&buf.as_slice()[5..]);
        buf.push(crc_data).unwrap();

        Some(buf)
    }

    pub fn to_packet_data(&self) -> MaxArray<u8, MAX_DATA_LENGTH> {
        match self {
            Self::RadioErp1 {
                radio_telegram,
                ..
            } => {
                radio_telegram.clone()
            },
            Self::Response {
                return_code,
                response_data,
            } => {
                let mut ret = MaxArray::new();
                ret.push_any(*return_code).unwrap();
                for b in response_data.iter() {
                    ret.push(*b).unwrap();
                }
                ret
            },
            Self::RadioSubTelegram {
                radio_telegram,
                ..
            } => {
                radio_telegram.clone()
            },
            Self::Event(event_data) => event_data.to_packet_data(),
            Self::CommonCommand(command_data) => command_data.to_packet_data(),
            Self::SmartAckCommand(smart_ack_data) => smart_ack_data.to_packet_data(),
            Self::RemoteManCommand {
                function,
                manufacturer,
                message,
                ..
            } => {
                let mut ret = MaxArray::new();
                ret.push_u16_be(*function).unwrap();
                ret.push_u16_be(*manufacturer).unwrap();
                for b in message.iter() {
                    ret.push(*b).unwrap();
                }
                ret
            },
            Self::RadioMessage {
                rorg,
                data,
                ..
            } => {
                let mut ret = MaxArray::new();
                ret.push(*rorg).unwrap();
                for b in data.iter() {
                    ret.push(*b).unwrap();
                }
                ret
            },
            Self::RadioErp2 {
                data,
                ..
            } => {
                data.clone()
            },
            Self::CommandAccepted {
                is_blocking,
                estimated_time_ms,
            } => {
                let mut ret = MaxArray::new();
                ret.push_any(*is_blocking).unwrap();
                ret.push_u16_be(*estimated_time_ms).unwrap();
                ret
            },
            Self::Radio802Dot15Dot4 {
                raw_data,
                ..
            } => {
                raw_data.clone()
            },
            Self::Command24(command_24_data) => command_24_data.to_packet_data(),
            Self::Unknown { data, .. } => data.clone(),
        }
    }

    pub fn to_packet_optional(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        match self {
            Self::RadioErp1 {
                opt_sub_telegram_number,
                opt_destination_id,
                opt_dbm,
                opt_security_level,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(sub_telegram_number) = opt_sub_telegram_number {
                    ret.push(*sub_telegram_number).unwrap();
                } else if opt_destination_id.is_some() || opt_dbm.is_some() || opt_security_level.is_some() {
                    // later fields are set but this one isn't
                    return None;
                }
                if let Some(destination_id) = opt_destination_id {
                    ret.push_u32_be(*destination_id).unwrap();
                } else if opt_dbm.is_some() || opt_security_level.is_some() {
                    return None;
                }
                if let Some(dbm) = opt_dbm {
                    ret.push(*dbm).unwrap();
                } else if opt_security_level.is_some() {
                    return None;
                }
                if let Some(security_level) = opt_security_level {
                    ret.push_any(*security_level).unwrap();
                }
                Some(ret)
            },
            Self::Response {
                ..
            } => {
                Some(MaxArray::new())
            },
            Self::RadioSubTelegram {
                opt_sub_telegram_number,
                opt_destination_id,
                opt_dbm,
                opt_security_level,
                opt_timestamp,
                opt_sub_telegram_info,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(sub_telegram_number) = opt_sub_telegram_number {
                    ret.push(*sub_telegram_number).unwrap();
                } else if opt_destination_id.is_some() || opt_dbm.is_some() || opt_security_level.is_some() || opt_timestamp.is_some() || opt_sub_telegram_info.len() > 0 {
                    // later fields are set but this one isn't
                    return None;
                }
                if let Some(destination_id) = opt_destination_id {
                    ret.push_u32_be(*destination_id).unwrap();
                } else if opt_dbm.is_some() || opt_security_level.is_some() || opt_timestamp.is_some() || opt_sub_telegram_info.len() > 0 {
                    return None;
                }
                if let Some(dbm) = opt_dbm {
                    ret.push(*dbm).unwrap();
                } else if opt_security_level.is_some() || opt_timestamp.is_some() || opt_sub_telegram_info.len() > 0 {
                    return None;
                }
                if let Some(security_level) = opt_security_level {
                    ret.push_any(*security_level).unwrap();
                } else if opt_timestamp.is_some() || opt_sub_telegram_info.len() > 0 {
                    return None;
                }
                if let Some(timestamp) = opt_timestamp {
                    ret.push_u16_be(*timestamp).unwrap();
                } else if opt_sub_telegram_info.len() > 0 {
                    return None;
                }
                for sub_telegram_info in opt_sub_telegram_info.iter() {
                    ret.push(sub_telegram_info.tick).unwrap();
                    ret.push(sub_telegram_info.dbm).unwrap();
                    ret.push(sub_telegram_info.status).unwrap();
                }
                Some(ret)
            },
            Self::Event(event_data) => event_data.to_packet_optional(),
            Self::CommonCommand(command_data) => command_data.to_packet_optional(),
            Self::SmartAckCommand(smart_ack_data) => smart_ack_data.to_packet_optional(),
            Self::RemoteManCommand {
                opt_destination_id,
                opt_source_id,
                opt_dbm,
                opt_send_with_delay,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(destination_id) = opt_destination_id {
                    ret.push_u32_be(*destination_id).unwrap();
                } else if opt_source_id.is_some() || opt_dbm.is_some() || opt_send_with_delay.is_some() {
                    return None;
                }
                if let Some(source_id) = opt_source_id {
                    ret.push_u32_be(*source_id).unwrap();
                } else if opt_dbm.is_some() || opt_send_with_delay.is_some() {
                    return None;
                }
                if let Some(dbm) = opt_dbm {
                    ret.push(*dbm).unwrap();
                } else if opt_send_with_delay.is_some() {
                    return None;
                }
                if let Some(send_with_delay) = opt_send_with_delay {
                    ret.push_any(*send_with_delay).unwrap();
                }
                Some(ret)
            },
            Self::RadioMessage {
                opt_destination_id,
                opt_source_id,
                opt_dbm,
                opt_security_level,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(destination_id) = opt_destination_id {
                    ret.push_u32_be(*destination_id).unwrap();
                } else if opt_source_id.is_some() || opt_dbm.is_some() || opt_security_level.is_some() {
                    return None;
                }
                if let Some(source_id) = opt_source_id {
                    ret.push_u32_be(*source_id).unwrap();
                } else if opt_dbm.is_some() || opt_security_level.is_some() {
                    return None;
                }
                if let Some(dbm) = opt_dbm {
                    ret.push(*dbm).unwrap();
                } else if opt_security_level.is_some() {
                    return None;
                }
                if let Some(security_level) = opt_security_level {
                    ret.push_any(*security_level).unwrap();
                }
                Some(ret)
            },
            Self::RadioErp2 {
                opt_sub_telegram_number,
                opt_dbm,
                opt_security_level,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(sub_telegram_number) = opt_sub_telegram_number {
                    ret.push(*sub_telegram_number).unwrap();
                } else if opt_dbm.is_some() || opt_security_level.is_some() {
                    return None;
                }
                if let Some(dbm) = opt_dbm {
                    ret.push(*dbm).unwrap();
                } else if opt_security_level.is_some() {
                    return None;
                }
                if let Some(security_level) = opt_security_level {
                    ret.push_any(*security_level).unwrap();
                }
                Some(ret)
            },
            Self::CommandAccepted { .. } => Some(MaxArray::new()),
            Self::Radio802Dot15Dot4 {
                opt_rssi,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(rssi) = opt_rssi {
                    ret.push_any(*rssi).unwrap();
                }
                Some(ret)
            },
            Self::Command24(command_24_data) => command_24_data.to_packet_optional(),
            Self::Unknown { optional_data, .. } => Some(optional_data.clone()),
        }
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        // check minimum length
        if bytes.len() < MIN_ESP3_PACKET_LENGTH {
            return None;
        }

        // check sync byte
        if bytes[0] != SYNC_BYTE {
            return None;
        }

        // extract lengths
        let data_length_u16 = (u16::from(bytes[1]) << 8) | u16::from(bytes[2]);
        let data_length: usize = data_length_u16.into();
        let opt_length_u8 = bytes[3];
        let opt_length: usize = opt_length_u8.into();

        // check total length
        let total_length = HEADER_LENGTH + data_length + opt_length + FOOTER_LENGTH;
        if bytes.len() != total_length {
            // FIXME: change from "not exactly total_length" to "at least total_length"?
            return None;
        }

        // check header CRC
        let crc_header = crc8_ccitt(&bytes[1..5]);
        if crc_header != bytes[5] {
            return None;
        }

        // check data CRC
        let crc_data = crc8_ccitt(&bytes[HEADER_LENGTH..HEADER_LENGTH+data_length+opt_length]);
        if crc_data != bytes[HEADER_LENGTH+data_length+opt_length] {
            return None;
        }

        let data_slice = &bytes[HEADER_LENGTH..HEADER_LENGTH+data_length];
        let optional_slice = &bytes[HEADER_LENGTH+data_length..HEADER_LENGTH+data_length+opt_length];

        let packet_type = bytes[4];
        match packet_type {
            1 => { // RadioErp1
                let radio_telegram = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );

                let opt_sub_telegram_number = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0]);
                let opt_destination_id = (optional_slice.len() >= 1 + 4)
                    .then(|| u32::from_be_bytes(optional_slice[1..1+4].try_into().unwrap()));
                let opt_dbm = (optional_slice.len() >= 1 + 4 + 1)
                    .then(|| optional_slice[1+4]);
                let opt_security_level = (optional_slice.len() >= 1 + 4 + 1 + 1)
                    .then(|| optional_slice[1+4+1].into());

                Some(Self::RadioErp1 {
                    radio_telegram,
                    opt_sub_telegram_number,
                    opt_destination_id,
                    opt_dbm,
                    opt_security_level,
                })
            },
            2 => { // Response
                if data_slice.len() < 1 {
                    return None;
                }

                let return_code = data_slice[0].into();
                let response_data = MaxArray::from_iter_or_panic(
                    data_slice[1..].iter().map(|b| *b).peekable()
                );

                Some(Self::Response {
                    return_code,
                    response_data,
                })
            },
            3 => { // RadioSubTelegram
                let radio_telegram = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );

                let opt_sub_telegram_number = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0]);
                let opt_destination_id = (optional_slice.len() >= 1 + 4)
                    .then(|| u32::from_be_bytes(optional_slice[1..1+4].try_into().unwrap()));
                let opt_dbm = (optional_slice.len() >= 1 + 4 + 1)
                    .then(|| optional_slice[1+4]);
                let opt_security_level = (optional_slice.len() >= 1 + 4 + 1 + 1)
                    .then(|| optional_slice[1+4+1].into());
                let opt_timestamp = (optional_slice.len() >= 1 + 4 + 1 + 1 + 2)
                    .then(|| u16::from_be_bytes(optional_slice[1+4+1+1..1+4+1+1+2].try_into().unwrap()));

                let mut i = 1 + 4 + 1 + 1 + 2;
                let mut opt_sub_telegram_info = MaxArray::new();
                while i + 3 <= optional_slice.len() {
                    let tick = optional_slice[i+0];
                    let dbm = optional_slice[i+1];
                    let status = optional_slice[i+2];
                    opt_sub_telegram_info
                        .push(SubTelegramInfo { tick, dbm, status })
                        .unwrap();

                    i += 3;
                }

                Some(Self::RadioSubTelegram {
                    radio_telegram,
                    opt_sub_telegram_number,
                    opt_destination_id,
                    opt_dbm,
                    opt_security_level,
                    opt_timestamp,
                    opt_sub_telegram_info,
                })
            },
            4 => { // Event
                if data_slice.len() < 1 {
                    return None;
                }

                let event_code = data_slice[0];
                EventData::from_data(event_code, &data_slice[1..], optional_slice)
                    .map(Self::Event)
            },
            5 => { // CommonCommand
                if data_slice.len() < 1 {
                    return None;
                }

                let command_code = data_slice[0];
                CommandData::from_data(command_code, &data_slice[1..], optional_slice)
                    .map(Self::CommonCommand)
            },
            6 => { // SmartAckCommand
                if data_slice.len() < 1 {
                    return None;
                }

                let command_code = data_slice[0];
                SmartAckData::from_data(command_code, &data_slice[1..], optional_slice)
                    .map(Self::SmartAckCommand)
            },
            7 => { // RemoteManCommand
                if data_slice.len() < 4 {
                    return None;
                }
                let function = u16::from_be_bytes(data_slice[0..2].try_into().unwrap());
                let manufacturer = u16::from_be_bytes(data_slice[2..4].try_into().unwrap());
                let message = MaxArray::from_iter_or_panic(
                    data_slice[4..].iter().map(|b| *b).peekable()
                );

                let opt_destination_id = (optional_slice.len() >= 4)
                    .then(|| u32::from_be_bytes(optional_slice[0..4].try_into().unwrap()));
                let opt_source_id = (optional_slice.len() >= 4 + 4)
                    .then(|| u32::from_be_bytes(optional_slice[4..4+4].try_into().unwrap()));
                let opt_dbm = (optional_slice.len() >= 4 + 4 + 1)
                    .then(|| optional_slice[4+4]);
                let opt_send_with_delay = (optional_slice.len() >= 4 + 4 + 1 + 1)
                    .then(|| optional_slice[4+4+1].into());

                Some(Self::RemoteManCommand {
                    function,
                    manufacturer,
                    message,
                    opt_destination_id,
                    opt_source_id,
                    opt_dbm,
                    opt_send_with_delay,
                })
            },
            9 => { // RadioMessage
                if data_slice.len() < 1 {
                    return None;
                }

                let rorg = data_slice[0];
                let data = MaxArray::from_iter_or_panic(
                    data_slice[1..].iter().map(|b| *b).peekable()
                );

                let opt_destination_id = (optional_slice.len() >= 4)
                    .then(|| u32::from_be_bytes(optional_slice[0..4].try_into().unwrap()));
                let opt_source_id = (optional_slice.len() >= 4 + 4)
                    .then(|| u32::from_be_bytes(optional_slice[4..4+4].try_into().unwrap()));
                let opt_dbm = (optional_slice.len() >= 4 + 4 + 1)
                    .then(|| optional_slice[4+4]);
                let opt_security_level = (optional_slice.len() >= 4 + 4 + 1 + 1)
                    .then(|| optional_slice[4+4+1].into());

                Some(Self::RadioMessage {
                    rorg,
                    data,
                    opt_destination_id,
                    opt_source_id,
                    opt_dbm,
                    opt_security_level,
                })
            },
            10 => { // RadioErp2
                let data = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );

                let opt_sub_telegram_number = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0]);
                let opt_dbm = (optional_slice.len() >= 1 + 1)
                    .then(|| optional_slice[1]);
                let opt_security_level = (optional_slice.len() >= 1 + 1 + 1)
                    .then(|| optional_slice[1+1].into());

                Some(Self::RadioErp2 {
                    data,
                    opt_sub_telegram_number,
                    opt_dbm,
                    opt_security_level,
                })
            },
            12 => { // CommandAccepted
                if data_slice.len() != 3 {
                    return None;
                }

                let is_blocking = data_slice[0].into();
                let estimated_time_ms = u16::from_be_bytes(data_slice[1..3].try_into().unwrap());

                Some(Self::CommandAccepted {
                    is_blocking,
                    estimated_time_ms,
                })
            },
            16 => { // Radio802Dot15Dot4
                let raw_data = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );

                let opt_rssi = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0]);

                Some(Self::Radio802Dot15Dot4 {
                    raw_data,
                    opt_rssi,
                })
            },
            17 => { // Command24
                if data_slice.len() < 1 {
                    return None;
                }

                let command_code = data_slice[0];
                Command24Data::from_data(command_code, &data_slice[1..], optional_slice)
                    .map(Self::Command24)
            },
            other => {
                let data = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );
                let optional_data = MaxArray::from_iter_or_panic(
                    optional_slice.iter().map(|b| *b).peekable()
                );

                Some(Self::Unknown {
                    packet_type: other,
                    data,
                    optional_data,
                })
            }
        }
    }
}


/// An ESP3 security level value.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum SecurityLevel {
    NoSecurity = 0x0,
    Obsolete = 0x1,
    Crypted = 0x2,
    Authenticated = 0x3,
    CryptedAuthenticated = 0x4,
    Other(u8),
}


/// The type of response.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
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
        data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
        optional_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
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

    pub fn to_packet_data(&self) -> MaxArray<u8, MAX_DATA_LENGTH> {
        let mut ret = MaxArray::new();

        let event_code = self.event_type();
        ret.push(event_code).unwrap();

        match self {
            Self::SmartAckReclaimNotSuccessful => {},
            Self::SmartAckConfirmLearn {
                postmaster_priority,
                manufacturer_id,
                eep,
                rssi,
                postmaster_candidate_id,
                smart_ack_client_id,
                hop_count,
            } => {
                ret.push(postmaster_priority.bits()).unwrap();
                ret.push_u16_be(*manufacturer_id).unwrap();

                // actually a 3-byte value; skip the MSB
                let eep_bytes: [u8; 4] = eep.to_be_bytes();
                ret.push(eep_bytes[1]).unwrap();
                ret.push(eep_bytes[2]).unwrap();
                ret.push(eep_bytes[3]).unwrap();

                ret.push(*rssi).unwrap();
                ret.push_u32_be(*postmaster_candidate_id).unwrap();
                ret.push_u32_be(*smart_ack_client_id).unwrap();
                ret.push(*hop_count).unwrap();
            },
            Self::SmartAckLearnAck {
                response_time,
                confirm_code,
            } => {
                ret.push_u16_be(*response_time).unwrap();
                ret.push_any(*confirm_code).unwrap();
            },
            Self::CoReady {
                wakeup_cause,
                ..
            } => {
                ret.push_any(*wakeup_cause).unwrap();
            },
            Self::CoEventSecureDevices {
                cause,
                device_id,
            } => {
                ret.push_any(*cause).unwrap();
                ret.push_u32_be(*device_id).unwrap();
            },
            Self::CoDutyCycleLimit {
                sending_possible,
            } => {
                ret.push_any(*sending_possible).unwrap();
            },
            Self::CoTransmitFailed {
                reason,
            } => {
                ret.push_any(*reason).unwrap();
            },
            Self::CoTxDone => {},
            Self::CoLearnModeDisabled => {},
            Self::Unknown {
                data,
                ..
            } => {
                for b in data.iter() {
                    ret.push(*b).unwrap();
                }
            },
        }

        ret
    }

    pub fn to_packet_optional(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        let mut ret = MaxArray::new();

        let event_code = self.event_type();
        ret.push(event_code).unwrap();

        match self {
            Self::SmartAckReclaimNotSuccessful => {},
            Self::SmartAckConfirmLearn { .. } => {},
            Self::SmartAckLearnAck { .. } => {},
            Self::CoReady {
                opt_security_mode,
                ..
            } => {
                if let Some(security_mode) = opt_security_mode {
                    ret.push_any(*security_mode).unwrap();
                }
            },
            Self::CoEventSecureDevices { .. } => {},
            Self::CoDutyCycleLimit { .. } => {},
            Self::CoTransmitFailed { .. } => {},
            Self::CoTxDone => {},
            Self::CoLearnModeDisabled => {},
            Self::Unknown {
                optional_data,
                ..
            } => {
                for b in optional_data.iter() {
                    ret.push(*b).unwrap();
                }
            },
        }

        Some(ret)
    }

    pub fn from_data(event_code: u8, data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        match event_code {
            1 => {
                if data_slice.len() != 0 {
                    return None;
                }
                Some(Self::SmartAckReclaimNotSuccessful)
            },
            2 => {
                if data_slice.len() != 16 {
                    return None;
                }

                let postmaster_priority = PostmasterPriority::from_bits_truncate(data_slice[0]);
                let manufacturer_id = u16::from_be_bytes(data_slice[1..3].try_into().unwrap());
                let eep =
                    u32::from(data_slice[3]) << 16
                    | u32::from(data_slice[4]) << 8
                    | u32::from(data_slice[5])
                ;
                let rssi = data_slice[6];
                let postmaster_candidate_id = u32::from_be_bytes(data_slice[7..11].try_into().unwrap());
                let smart_ack_client_id = u32::from_be_bytes(data_slice[11..15].try_into().unwrap());
                let hop_count = data_slice[15];

                Some(Self::SmartAckConfirmLearn {
                    postmaster_priority,
                    manufacturer_id,
                    eep,
                    rssi,
                    postmaster_candidate_id,
                    smart_ack_client_id,
                    hop_count,
                })
            },
            3 => {
                if data_slice.len() != 3 {
                    return None;
                }

                let response_time = u16::from_be_bytes(data_slice[0..2].try_into().unwrap());
                let confirm_code = data_slice[2].into();

                Some(Self::SmartAckLearnAck {
                    response_time,
                    confirm_code,
                })
            },
            4 => {
                if data_slice.len() != 1 {
                    return None;
                }

                let wakeup_cause = data_slice[0].into();

                let opt_security_mode = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                Some(Self::CoReady {
                    wakeup_cause,
                    opt_security_mode,
                })
            },
            5 => {
                if data_slice.len() != 5 {
                    return None;
                }

                let cause = data_slice[0].into();
                let device_id = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());

                Some(Self::CoEventSecureDevices {
                    cause,
                    device_id,
                })
            },
            6 => {
                if data_slice.len() != 1 {
                    return None;
                }

                let sending_possible = data_slice[0].into();

                Some(Self::CoDutyCycleLimit {
                    sending_possible,
                })
            },
            7 => {
                if data_slice.len() != 1 {
                    return None;
                }

                let reason = data_slice[0].into();

                Some(Self::CoTransmitFailed {
                    reason,
                })
            },
            8 => {
                if data_slice.len() != 0 {
                    return None;
                }
                Some(Self::CoTxDone)
            },
            9 => {
                if data_slice.len() != 0 {
                    return None;
                }
                Some(Self::CoLearnModeDisabled)
            },
            other => {
                Some(Self::Unknown {
                    code: other,
                    data: MaxArray::from_iter_or_panic(data_slice.iter().map(|b| *b).peekable()),
                    optional_data: MaxArray::from_iter_or_panic(optional_slice.iter().map(|b| *b).peekable()),
                })
            },
        }
    }
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
        data: MaxArray<u8, {MAX_DATA_LENGTH - (1 + 5)}>,
    },
    CoRdMem {
        memory_type: MemoryType,
        address: u32,
        length: u16,
    },
    CoRdMemAddress {
        area: AddressArea,
    },
    #[deprecated] CoRdSecurity,
    #[deprecated] CoWrSecurity {
        security_level: u8,
        key: u32,
        rolling_code: u32,
    },
    CoWrLearnMode {
        enable: OneByteBoolean,
        timeout: u32,
        opt_channel: Option<ChannelNumber>,
    },
    CoRdLearnMode,
    #[deprecated] CoWrSecureDeviceAdd {
        slf: u8,
        device_id: u32,
        private_key: [u32; 4],
        rolling_code: u32,
        opt_direction: Option<DirectionTable>,
        opt_is_ptm_sender: Option<OneByteBoolean>,
        opt_teach_info: Option<u8>,
    },
    CoWrSecureDeviceDel {
        device_id: u32,
        opt_direction: Option<DirectionTableBoth>,
    },
    #[deprecated] CoRdSecureDeviceByIndex {
        index: u8,
        opt_direction: Option<DirectionTable>,
    },
    CoWrMode {
        mode: TransceiverMode,
    },
    CoRdNumSecureDevices {
        opt_direction: Option<DirectionTableBoth>,
    },
    CoRdSecureDeviceById {
        device_id: u32,
        opt_direction: Option<DirectionTableMaintenance>,
    },
    CoWrSecureDeviceAddPsk {
        device_id: u32,
        psk: [u32; 4],
    },
    CoWrSecureDeviceSendTeachIn {
        device_id: u32,
        opt_teach_info: Option<u8>,
    },
    CoWrTemporaryRlcWindow {
        enable: OneByteBoolean,
        rlc_window: u32,
    },
    CoRdSecureDevicePsk {
        device_id: u32,
    },
    CoRdDutyCycleLimit,
    CoSetBaudRate {
        baud_rate: BaudRate,
    },
    CoGetFrequencyInfo,
    CoGetStepCode,
    CoWrReManCode {
        code: u32,
    },
    CoWrStartupDelay {
        delay: u8,
    },
    CoWrReManRepeating {
        repeat_re_man_telegrams: OneByteBoolean,
    },
    CoRdReManRepeating,
    CoSetNoiseThreshold {
        rssi_level: u8,
    },
    CoGetNoiseThreshold,
    CoWrRlcSavePeriod {
        save_period: u8,
    },
    CoWrRlcLegacyMode {
        enable: OneByteBoolean,
    },
    CoWrSecureDeviceV2Add {
        slf: u8,
        device_id: u32,
        private_key: [u32; 4],
        rolling_code: u32,
        teach_info: u8,
        opt_direction: Option<DirectionTable>,
    },
    CoRdSecureDeviceV2ByIndex {
        index: u8,
        opt_direction: Option<DirectionTable>,
    },
    CoWrRssiTestMode {
        enable: OneByteBoolean,
        timeout: u16,
    },
    CoRdRssiTestMode,
    CoWrSecureDeviceMaintenanceKey {
        device_id: u32,
        maintenance_key: [u32; 4],
        key_number: u8,
    },
    CoRdSecureDeviceMaintenanceKey {
        index: u8,
    },
    CoWrTransparentMode {
        enable: OneByteBoolean,
    },
    CoRdTransparentMode,
    CoWrTxOnlyMode {
        mode: TxOnlyMode,
    },
    CoRdTxOnlyMode,
    Unknown {
        code: u8,
        data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
        optional_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
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
            Self::CoWrIdBase { .. } => 7,
            Self::CoRdIdBase => 8,
            Self::CoWrRepeater { .. } => 9,
            Self::CoRdRepeater => 10,
            Self::CoWrFilterAdd { .. } => 11,
            Self::CoWrFilterDel { .. } => 12,
            Self::CoWrFilterDelAll => 13,
            Self::CoWrFilterEnable { .. } => 14,
            Self::CoRdFilter => 15,
            Self::CoWrWaitMaturity { .. } => 16,
            Self::CoWrSubTelegram { .. } => 17,
            Self::CoWrMem { .. } => 18,
            Self::CoRdMem { .. } => 19,
            Self::CoRdMemAddress { .. } => 20,
            #[allow(deprecated)] Self::CoRdSecurity => 21,
            #[allow(deprecated)] Self::CoWrSecurity { .. } => 22,
            Self::CoWrLearnMode { .. } => 23,
            Self::CoRdLearnMode => 24,
            #[allow(deprecated)] Self::CoWrSecureDeviceAdd { .. } => 25,
            Self::CoWrSecureDeviceDel { .. }    => 26,
            #[allow(deprecated)] Self::CoRdSecureDeviceByIndex { .. } => 27,
            Self::CoWrMode { .. } => 28,
            Self::CoRdNumSecureDevices { .. } => 29,
            Self::CoRdSecureDeviceById { .. } => 30,
            Self::CoWrSecureDeviceAddPsk { .. } => 31,
            Self::CoWrSecureDeviceSendTeachIn { .. } => 32,
            Self::CoWrTemporaryRlcWindow { .. } => 33,
            Self::CoRdSecureDevicePsk { .. } => 34,
            Self::CoRdDutyCycleLimit => 35,
            Self::CoSetBaudRate { .. } => 36,
            Self::CoGetFrequencyInfo => 37,
            // 38 = reserved
            Self::CoGetStepCode => 39,
            // 40-45 = reserved
            Self::CoWrReManCode { .. } => 46,
            Self::CoWrStartupDelay { .. } => 47,
            Self::CoWrReManRepeating { .. } => 48,
            Self::CoRdReManRepeating => 49,
            Self::CoSetNoiseThreshold { .. } => 50,
            Self::CoGetNoiseThreshold => 51,
            // 52-53 = reserved
            Self::CoWrRlcSavePeriod { .. } => 54,
            Self::CoWrRlcLegacyMode { .. } => 55,
            Self::CoWrSecureDeviceV2Add { .. }  => 56,
            Self::CoRdSecureDeviceV2ByIndex { .. } => 57,
            Self::CoWrRssiTestMode { .. } => 58,
            Self::CoRdRssiTestMode => 59,
            Self::CoWrSecureDeviceMaintenanceKey { .. } => 60,
            Self::CoRdSecureDeviceMaintenanceKey { .. } => 61,
            Self::CoWrTransparentMode { .. } => 62,
            Self::CoRdTransparentMode => 63,
            Self::CoWrTxOnlyMode { .. } => 64,
            Self::CoRdTxOnlyMode => 65,
            Self::Unknown { code, .. } => *code,
        }
    }

    pub fn to_packet_data(&self) -> MaxArray<u8, MAX_DATA_LENGTH> {
        let mut ret = MaxArray::new();

        let command_code = self.command_type();
        ret.push(command_code).unwrap();

        match self {
            Self::CoWrSleep {
                deep_sleep_period,
            } => {
                ret.push_u32_be(*deep_sleep_period).unwrap();
            },
            Self::CoWrReset => {},
            Self::CoRdVersion => {},
            Self::CoRdSysLog => {},
            Self::CoWrSysLog => {},
            Self::CoWrBiSt => {},
            Self::CoWrIdBase {
                base_id,
            } => {
                ret.push_u32_be(*base_id).unwrap();
            },
            Self::CoRdIdBase => {},
            Self::CoWrRepeater {
                enable,
                level,
            } => {
                ret.push_any(*enable).unwrap();
                ret.push_any(*level).unwrap();
            },
            Self::CoRdRepeater => {},
            Self::CoWrFilterAdd {
                criterion,
                value,
                action,
            } => {
                ret.push_any(*criterion).unwrap();
                ret.push_u32_be(*value).unwrap();
                ret.push_any(*action).unwrap();
            },
            Self::CoWrFilterDel {
                criterion,
                value,
                action,
            } => {
                ret.push_any(*criterion).unwrap();
                ret.push_u32_be(*value).unwrap();
                ret.push_any(*action).unwrap();
            },
            Self::CoWrFilterDelAll => {},
            Self::CoWrFilterEnable {
                enable,
                operator,
            } => {
                ret.push_any(*enable).unwrap();
                ret.push_any(*operator).unwrap();
            },
            Self::CoRdFilter => {},
            Self::CoWrWaitMaturity {
                wait_for_maturity,
            } => {
                ret.push_any(*wait_for_maturity).unwrap();
            },
            Self::CoWrSubTelegram {
                enable_subtelegram_info,
            } => {
                ret.push_any(*enable_subtelegram_info).unwrap();
            },
            Self::CoWrMem {
                memory_type,
                address,
                data,
            } => {
                ret.push_any(*memory_type).unwrap();
                ret.push_u32_be(*address).unwrap();
                for b in data.iter() {
                    ret.push(*b).unwrap();
                }
            },
            Self::CoRdMem {
                memory_type,
                address,
                length,
            } => {
                ret.push_any(*memory_type).unwrap();
                ret.push_u32_be(*address).unwrap();
                ret.push_u16_be(*length).unwrap();
            },
            Self::CoRdMemAddress {
                area,
            } => {
                ret.push_any(*area).unwrap();
            },
            #[allow(deprecated)] Self::CoRdSecurity => {},
            #[allow(deprecated)] Self::CoWrSecurity {
                security_level,
                key,
                rolling_code,
            } => {
                ret.push_any(*security_level).unwrap();
                ret.push_u32_be(*key).unwrap();
                ret.push_u32_be(*rolling_code).unwrap();
            },
            Self::CoWrLearnMode {
                enable,
                timeout,
                ..
            } => {
                ret.push_any(*enable).unwrap();
                ret.push_u32_be(*timeout).unwrap();
            },
            Self::CoRdLearnMode => {},
            #[allow(deprecated)] Self::CoWrSecureDeviceAdd {
                slf,
                device_id,
                private_key,
                rolling_code,
                ..
            } => {
                ret.push(*slf).unwrap();
                ret.push_u32_be(*device_id).unwrap();
                for private_key_chunk in private_key {
                    ret.push_u32_be(*private_key_chunk).unwrap();
                }

                // actually a 3-byte value; skip the MSB
                let rolling_code_bytes: [u8; 4] = rolling_code.to_be_bytes();
                ret.push(rolling_code_bytes[1]).unwrap();
                ret.push(rolling_code_bytes[2]).unwrap();
                ret.push(rolling_code_bytes[3]).unwrap();
            },
            Self::CoWrSecureDeviceDel {
                device_id,
                ..
            } => {
                ret.push_u32_be(*device_id).unwrap();
            },
            #[allow(deprecated)] Self::CoRdSecureDeviceByIndex {
                index,
                ..
            } => {
                ret.push(*index).unwrap();
            },
            Self::CoWrMode {
                mode,
            } => {
                ret.push_any(*mode).unwrap();
            },
            Self::CoRdNumSecureDevices { .. } => {},
            Self::CoRdSecureDeviceById {
                device_id,
                ..
            } => {
                ret.push_u32_be(*device_id).unwrap();
            },
            Self::CoWrSecureDeviceAddPsk {
                device_id,
                psk,
            } => {
                ret.push_u32_be(*device_id).unwrap();
                for psk_chunk in psk {
                    ret.push_u32_be(*psk_chunk).unwrap();
                }
            },
            Self::CoWrSecureDeviceSendTeachIn {
                device_id,
                ..
            } => {
                ret.push_u32_be(*device_id).unwrap();
            },
            Self::CoWrTemporaryRlcWindow {
                enable,
                rlc_window,
            } => {
                ret.push_any(*enable).unwrap();
                ret.push_u32_be(*rlc_window).unwrap();
            },
            Self::CoRdSecureDevicePsk {
                device_id,
            } => {
                ret.push_u32_be(*device_id).unwrap();
            },
            Self::CoRdDutyCycleLimit => {},
            Self::CoSetBaudRate {
                baud_rate,
            } => {
                ret.push_any(*baud_rate).unwrap();
            },
            Self::CoGetFrequencyInfo => {},
            Self::CoGetStepCode => {},
            Self::CoWrReManCode {
                code,
            } => {
                ret.push_u32_be(*code).unwrap();
            },
            Self::CoWrStartupDelay {
                delay,
            } => {
                ret.push(*delay).unwrap();
            },
            Self::CoWrReManRepeating {
                repeat_re_man_telegrams,
            } => {
                ret.push_any(*repeat_re_man_telegrams).unwrap();
            },
            Self::CoRdReManRepeating => {},
            Self::CoSetNoiseThreshold {
                rssi_level,
            } => {
                ret.push(*rssi_level).unwrap();
            },
            Self::CoGetNoiseThreshold => {},
            Self::CoWrRlcSavePeriod {
                save_period,
            } => {
                ret.push(*save_period).unwrap();
            },
            Self::CoWrRlcLegacyMode {
                enable,
            } => {
                ret.push_any(*enable).unwrap();
            },
            Self::CoWrSecureDeviceV2Add {
                slf,
                device_id,
                private_key,
                rolling_code,
                teach_info,
                ..
            } => {
                ret.push(*slf).unwrap();
                ret.push_u32_be(*device_id).unwrap();
                for private_key_chunk in private_key {
                    ret.push_u32_be(*private_key_chunk).unwrap();
                }
                ret.push_u32_be(*rolling_code).unwrap();
                ret.push(*teach_info).unwrap();
            },
            Self::CoRdSecureDeviceV2ByIndex {
                index,
                ..
            } => {
                ret.push(*index).unwrap();
            },
            Self::CoWrRssiTestMode {
                enable,
                timeout,
            } => {
                ret.push_any(*enable).unwrap();
                ret.push_u16_be(*timeout).unwrap();
            },
            Self::CoRdRssiTestMode => {},
            Self::CoWrSecureDeviceMaintenanceKey {
                device_id,
                maintenance_key,
                key_number,
            } => {
                ret.push_u32_be(*device_id).unwrap();
                for maintenance_key_chunk in maintenance_key {
                    ret.push_u32_be(*maintenance_key_chunk).unwrap();
                }
                ret.push(*key_number).unwrap();
            },
            Self::CoRdSecureDeviceMaintenanceKey {
                index,
            } => {
                ret.push(*index).unwrap();
            },
            Self::CoWrTransparentMode {
                enable,
            } => {
                ret.push_any(*enable).unwrap();
            },
            Self::CoRdTransparentMode => {},
            Self::CoWrTxOnlyMode {
                mode,
            } => {
                ret.push_any(*mode).unwrap();
            },
            Self::CoRdTxOnlyMode => {},
            Self::Unknown {
                data,
                ..
            } => {
                for b in data.iter() {
                    ret.push(*b).unwrap();
                }
            },
        }

        ret
    }

    pub fn to_packet_optional(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        match self {
            Self::CoWrSleep { .. } => Some(MaxArray::new()),
            Self::CoWrReset => Some(MaxArray::new()),
            Self::CoRdVersion => Some(MaxArray::new()),
            Self::CoRdSysLog => Some(MaxArray::new()),
            Self::CoWrSysLog => Some(MaxArray::new()),
            Self::CoWrBiSt => Some(MaxArray::new()),
            Self::CoWrIdBase { .. } => Some(MaxArray::new()),
            Self::CoRdIdBase => Some(MaxArray::new()),
            Self::CoWrRepeater { .. } => Some(MaxArray::new()),
            Self::CoRdRepeater => Some(MaxArray::new()),
            Self::CoWrFilterAdd { .. } => Some(MaxArray::new()),
            Self::CoWrFilterDel { .. } => Some(MaxArray::new()),
            Self::CoWrFilterDelAll => Some(MaxArray::new()),
            Self::CoWrFilterEnable { .. } => Some(MaxArray::new()),
            Self::CoRdFilter => Some(MaxArray::new()),
            Self::CoWrWaitMaturity { .. } => Some(MaxArray::new()),
            Self::CoWrSubTelegram { .. } => Some(MaxArray::new()),
            Self::CoWrMem { .. } => Some(MaxArray::new()),
            Self::CoRdMem { .. } => Some(MaxArray::new()),
            Self::CoRdMemAddress { .. } => Some(MaxArray::new()),
            #[allow(deprecated)] Self::CoRdSecurity => Some(MaxArray::new()),
            #[allow(deprecated)] Self::CoWrSecurity { .. } => Some(MaxArray::new()),
            Self::CoWrLearnMode {
                opt_channel,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(channel) = opt_channel {
                    ret.push_any(*channel).unwrap();
                }
                Some(ret)
            },
            Self::CoRdLearnMode => Some(MaxArray::new()),
            #[allow(deprecated)] Self::CoWrSecureDeviceAdd {
                opt_direction,
                opt_is_ptm_sender,
                opt_teach_info,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                } else if opt_is_ptm_sender.is_some() || opt_teach_info.is_some() {
                    return None;
                }
                if let Some(is_ptm_sender) = opt_is_ptm_sender {
                    ret.push_any(*is_ptm_sender).unwrap();
                } else if opt_teach_info.is_some() {
                    return None;
                }
                if let Some(teach_info) = opt_teach_info {
                    ret.push_any(*teach_info).unwrap();
                }
                Some(ret)
            },
            Self::CoWrSecureDeviceDel {
                opt_direction,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                }
                Some(ret)
            },
            #[allow(deprecated)] Self::CoRdSecureDeviceByIndex {
                opt_direction,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                }
                Some(ret)
            },
            Self::CoWrMode { .. } => Some(MaxArray::new()),
            Self::CoRdNumSecureDevices {
                opt_direction,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                }
                Some(ret)
            },
            Self::CoRdSecureDeviceById {
                opt_direction,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                }
                Some(ret)
            },
            Self::CoWrSecureDeviceAddPsk { .. } => Some(MaxArray::new()),
            Self::CoWrSecureDeviceSendTeachIn {
                opt_teach_info,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(teach_info) = opt_teach_info {
                    ret.push_any(*teach_info).unwrap();
                }
                Some(ret)
            },
            Self::CoWrTemporaryRlcWindow { .. } => Some(MaxArray::new()),
            Self::CoRdSecureDevicePsk { .. } => Some(MaxArray::new()),
            Self::CoRdDutyCycleLimit => Some(MaxArray::new()),
            Self::CoSetBaudRate { .. } => Some(MaxArray::new()),
            Self::CoGetFrequencyInfo => Some(MaxArray::new()),
            Self::CoGetStepCode => Some(MaxArray::new()),
            Self::CoWrReManCode { .. } => Some(MaxArray::new()),
            Self::CoWrStartupDelay { .. } => Some(MaxArray::new()),
            Self::CoWrReManRepeating { .. } => Some(MaxArray::new()),
            Self::CoRdReManRepeating => Some(MaxArray::new()),
            Self::CoSetNoiseThreshold { .. } => Some(MaxArray::new()),
            Self::CoGetNoiseThreshold => Some(MaxArray::new()),
            Self::CoWrRlcSavePeriod { .. } => Some(MaxArray::new()),
            Self::CoWrRlcLegacyMode { .. } => Some(MaxArray::new()),
            Self::CoWrSecureDeviceV2Add {
                opt_direction,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                }
                Some(ret)
            },
            Self::CoRdSecureDeviceV2ByIndex {
                opt_direction,
                ..
            } => {
                let mut ret = MaxArray::new();
                if let Some(direction) = opt_direction {
                    ret.push_any(*direction).unwrap();
                }
                Some(ret)
            },
            Self::CoWrRssiTestMode { .. } => Some(MaxArray::new()),
            Self::CoRdRssiTestMode => Some(MaxArray::new()),
            Self::CoWrSecureDeviceMaintenanceKey { .. } => Some(MaxArray::new()),
            Self::CoRdSecureDeviceMaintenanceKey { .. } => Some(MaxArray::new()),
            Self::CoWrTransparentMode { .. } => Some(MaxArray::new()),
            Self::CoRdTransparentMode => Some(MaxArray::new()),
            Self::CoWrTxOnlyMode { .. } => Some(MaxArray::new()),
            Self::CoRdTxOnlyMode => Some(MaxArray::new()),
            Self::Unknown {
                optional_data,
                ..
            } => {
                Some(optional_data.clone())
            },
        }
    }

    pub fn from_data(command_code: u8, data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        match command_code {
            2..=6|8|10|13|15|21|24|35|37|39|49|51|59|63|65 => {
                if data_slice.len() != 0 {
                    return None;
                }
                #[allow(deprecated)]
                Some(match command_code {
                    2 => Self::CoWrReset,
                    3 => Self::CoRdVersion,
                    4 => Self::CoRdSysLog,
                    5 => Self::CoWrSysLog,
                    6 => Self::CoWrBiSt,
                    8 => Self::CoRdIdBase,
                    10 => Self::CoRdRepeater,
                    13 => Self::CoWrFilterDelAll,
                    15 => Self::CoRdFilter,
                    21 => Self::CoRdSecurity,
                    24 => Self::CoRdLearnMode,
                    35 => Self::CoRdDutyCycleLimit,
                    37 => Self::CoGetFrequencyInfo,
                    39 => Self::CoGetStepCode,
                    49 => Self::CoRdReManRepeating,
                    51 => Self::CoGetNoiseThreshold,
                    59 => Self::CoRdRssiTestMode,
                    63 => Self::CoRdTransparentMode,
                    65 => Self::CoRdTxOnlyMode,
                    _ => unreachable!(),
                })
            },
            1 => { // CoWrSleep
                if data_slice.len() != 4 {
                    return None;
                }

                let deep_sleep_period = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                Some(Self::CoWrSleep {
                    deep_sleep_period,
                })
            },
            7 => { // CoWrIdBase
                if data_slice.len() != 4 {
                    return None;
                }

                let base_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                Some(Self::CoWrIdBase {
                    base_id,
                })
            },
            9 => { // CoWrRepeater
                if data_slice.len() != 2 {
                    return None;
                }

                let enable = data_slice[0].into();
                let level = data_slice[1].into();
                Some(Self::CoWrRepeater {
                    enable,
                    level,
                })
            },
            11 => { // CoWrFilterAdd
                if data_slice.len() != 6 {
                    return None;
                }

                let criterion = data_slice[0].into();
                let value = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let action = data_slice[5].into();
                Some(Self::CoWrFilterAdd {
                    criterion,
                    value,
                    action,
                })
            },
            12 => { // CoWrFilterDel
                if data_slice.len() != 6 {
                    return None;
                }

                let criterion = data_slice[0].into();
                let value = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let action = data_slice[5].into();
                Some(Self::CoWrFilterDel {
                    criterion,
                    value,
                    action,
                })
            },
            14 => { // CoWrFilterEnable
                if data_slice.len() != 2 {
                    return None;
                }

                let enable = data_slice[0].into();
                let operator = data_slice[1].into();
                Some(Self::CoWrFilterEnable {
                    enable,
                    operator,
                })
            },
            16 => { // CoWrWaitMaturity
                if data_slice.len() != 1 {
                    return None;
                }

                let wait_for_maturity = data_slice[0].into();
                Some(Self::CoWrWaitMaturity {
                    wait_for_maturity,
                })
            },
            17 => { // CoWrSubTelegram
                if data_slice.len() != 1 {
                    return None;
                }

                let enable_subtelegram_info = data_slice[0].into();
                Some(Self::CoWrSubTelegram {
                    enable_subtelegram_info,
                })
            },
            18 => { // CoWrMem
                if data_slice.len() < 5 {
                    return None;
                }

                let memory_type = data_slice[0].into();
                let address = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let data = MaxArray::from_iter_or_panic(
                    data_slice[5..].iter().map(|b| *b).peekable()
                );
                Some(Self::CoWrMem {
                    memory_type,
                    address,
                    data,
                })
            },
            19 => { // CoRdMem
                if data_slice.len() != 7 {
                    return None;
                }

                let memory_type = data_slice[0].into();
                let address = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let length = u16::from_be_bytes(data_slice[5..7].try_into().unwrap());
                Some(Self::CoRdMem {
                    memory_type,
                    address,
                    length,
                })
            },
            20 => { // CoRdMemAddress
                if data_slice.len() != 1 {
                    return None;
                }

                let area = data_slice[0].into();
                Some(Self::CoRdMemAddress {
                    area,
                })
            },
            22 => { // CoWrSecurity
                if data_slice.len() != 10 {
                    return None;
                }

                let security_level = data_slice[0].into();
                let key = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let rolling_code = u32::from_be_bytes(data_slice[5..9].try_into().unwrap());
                #[allow(deprecated)]
                Some(Self::CoWrSecurity {
                    security_level,
                    key,
                    rolling_code,
                })
            },
            23 => { // CoWrLearnMode
                if data_slice.len() != 5 {
                    return None;
                }

                let enable = data_slice[0].into();
                let timeout = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());

                let opt_channel = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());
                Some(Self::CoWrLearnMode {
                    enable,
                    timeout,
                    opt_channel,
                })
            },
            25 => { // CoWrSecureDeviceAdd
                if data_slice.len() != 24 {
                    return None;
                }

                let slf = data_slice[0].into();
                let device_id = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let private_key = [
                    u32::from_be_bytes(data_slice[5..9].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[9..13].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[13..17].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[17..21].try_into().unwrap()),
                ];

                // rolling code is only three bytes; decode manually
                let rolling_code =
                    (u32::from(data_slice[21]) << 16)
                    | (u32::from(data_slice[22]) << 8)
                    | u32::from(data_slice[23])
                ;

                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());
                let opt_is_ptm_sender = (optional_slice.len() >= 2)
                    .then(|| optional_slice[1].into());
                let opt_teach_info = (optional_slice.len() >= 3)
                    .then(|| optional_slice[2].into());

                #[allow(deprecated)]
                Some(Self::CoWrSecureDeviceAdd {
                    slf,
                    device_id,
                    private_key,
                    rolling_code,
                    opt_direction,
                    opt_is_ptm_sender,
                    opt_teach_info,
                })
            },
            26 => { // CoWrSecureDeviceDel
                if data_slice.len() != 4 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                #[allow(deprecated)]
                Some(Self::CoWrSecureDeviceDel {
                    device_id,
                    opt_direction,
                })
            },
            27 => { // CoRdSecureDeviceByIndex
                if data_slice.len() != 1 {
                    return None;
                }

                let index = data_slice[0];
                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                #[allow(deprecated)]
                Some(Self::CoRdSecureDeviceByIndex {
                    index,
                    opt_direction,
                })
            },
            28 => { // CoWrMode
                if data_slice.len() != 1 {
                    return None;
                }

                let mode = data_slice[0].into();
                Some(Self::CoWrMode {
                    mode,
                })
            },
            29 => { // CoRdNumSecureDevices
                if data_slice.len() != 0 {
                    return None;
                }

                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                Some(Self::CoRdNumSecureDevices {
                    opt_direction,
                })
            },
            30 => { // CoRdSecureDeviceById
                if data_slice.len() != 4 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                Some(Self::CoRdSecureDeviceById {
                    device_id,
                    opt_direction,
                })
            },
            31 => { // CoWrSecureDeviceAddPsk
                if data_slice.len() != 20 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let psk = [
                    u32::from_be_bytes(data_slice[4..8].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[8..12].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[12..16].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[16..20].try_into().unwrap()),
                ];

                Some(Self::CoWrSecureDeviceAddPsk {
                    device_id,
                    psk,
                })
            },
            32 => { // CoWrSecureDeviceSendTeachIn
                if data_slice.len() != 4 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let opt_teach_info = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                Some(Self::CoWrSecureDeviceSendTeachIn {
                    device_id,
                    opt_teach_info,
                })
            },
            33 => { // CoWrTemporaryRlcWindow
                if data_slice.len() != 5 {
                    return None;
                }

                let enable = data_slice[0].into();
                let rlc_window = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());

                Some(Self::CoWrTemporaryRlcWindow {
                    enable,
                    rlc_window,
                })
            },
            34 => { // CoRdSecureDevicePsk
                if data_slice.len() != 4 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                Some(Self::CoRdSecureDevicePsk {
                    device_id,
                })
            },
            36 => { // CoSetBaudRate
                if data_slice.len() != 1 {
                    return None;
                }

                let baud_rate = data_slice[0].into();
                Some(Self::CoSetBaudRate {
                    baud_rate,
                })
            },
            46 => { // CoWrReManCode
                if data_slice.len() != 4 {
                    return None;
                }

                let code = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                Some(Self::CoWrReManCode {
                    code,
                })
            },
            47 => { // CoWrStartupDelay
                if data_slice.len() != 1 {
                    return None;
                }

                let delay = data_slice[0];
                Some(Self::CoWrStartupDelay {
                    delay,
                })
            },
            48 => { // CoWrReManRepeating
                if data_slice.len() != 1 {
                    return None;
                }

                let repeat_re_man_telegrams = data_slice[0].into();
                Some(Self::CoWrReManRepeating {
                    repeat_re_man_telegrams,
                })
            },
            50 => { // CoSetNoiseThreshold
                if data_slice.len() != 1 {
                    return None;
                }

                let rssi_level = data_slice[0].into();
                Some(Self::CoSetNoiseThreshold {
                    rssi_level,
                })
            },
            54 => { // CoWrRlcSavePeriod
                if data_slice.len() != 1 {
                    return None;
                }

                let save_period = data_slice[0].into();
                Some(Self::CoWrRlcSavePeriod {
                    save_period,
                })
            },
            55 => { // CoWrRlcLegacyMode
                if data_slice.len() != 1 {
                    return None;
                }

                let enable = data_slice[0].into();
                Some(Self::CoWrRlcLegacyMode {
                    enable,
                })
            },
            56 => { // CoWrSecureDeviceV2Add
                if data_slice.len() != 26 {
                    return None;
                }

                let slf = data_slice[0].into();
                let device_id = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
                let private_key = [
                    u32::from_be_bytes(data_slice[5..9].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[9..13].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[13..17].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[17..21].try_into().unwrap()),
                ];
                let rolling_code = u32::from_be_bytes(data_slice[21..25].try_into().unwrap());
                let teach_info = data_slice[25];

                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                Some(Self::CoWrSecureDeviceV2Add {
                    slf,
                    device_id,
                    private_key,
                    rolling_code,
                    teach_info,
                    opt_direction,
                })
            },
            57 => { // CoRdSecureDeviceV2ByIndex
                if data_slice.len() != 1 {
                    return None;
                }

                let index = data_slice[0].into();
                let opt_direction = (optional_slice.len() >= 1)
                    .then(|| optional_slice[0].into());

                Some(Self::CoRdSecureDeviceV2ByIndex {
                    index,
                    opt_direction,
                })
            },
            58 => { // CoWrRssiTestMode
                if data_slice.len() != 3 {
                    return None;
                }

                let enable = data_slice[0].into();
                let timeout = u16::from_be_bytes(data_slice[1..3].try_into().unwrap());

                Some(Self::CoWrRssiTestMode {
                    enable,
                    timeout,
                })
            },
            60 => { // CoWrSecureDeviceMaintenanceKey
                if data_slice.len() != 21 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let maintenance_key = [
                    u32::from_be_bytes(data_slice[4..8].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[8..12].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[12..16].try_into().unwrap()),
                    u32::from_be_bytes(data_slice[16..20].try_into().unwrap()),
                ];
                let key_number = data_slice[20];

                Some(Self::CoWrSecureDeviceMaintenanceKey {
                    device_id,
                    maintenance_key,
                    key_number,
                })
            },
            61 => { // CoRdSecureDeviceMaintenanceKey
                if data_slice.len() != 1 {
                    return None;
                }

                let index = data_slice[0];
                Some(Self::CoRdSecureDeviceMaintenanceKey {
                    index,
                })
            },
            62 => { // CoWrTransparentMode
                if data_slice.len() != 1 {
                    return None;
                }

                let enable = data_slice[0].into();
                Some(Self::CoWrTransparentMode {
                    enable,
                })
            },
            64 => { // CoWrTxOnlyMode
                if data_slice.len() != 1 {
                    return None;
                }

                let mode = data_slice[0].into();
                Some(Self::CoWrTxOnlyMode {
                    mode,
                })
            },
            other => {
                Some(Self::Unknown {
                    code: other,
                    data: MaxArray::from_iter_or_panic(data_slice.iter().map(|b| *b).peekable()),
                    optional_data: MaxArray::from_iter_or_panic(optional_slice.iter().map(|b| *b).peekable()),
                })
            },
        }
    }
}

/// Data carried by a Smart Acknowledgement command.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SmartAckData {
    SaWrLearnMode {
        enable: OneByteBoolean,
        extended: ExtendedLearnMode,
        timeout: u32,
    },
    SaRdLearnMode,
    SaWrLearnConfirm {
        response_time: u16,
        confirm_code: LearnInOut,
        postmaster_candidate_id: u32,
        smart_ack_client_id: u32,
    },
    SaWrClientLearnRequest {
        manufacturer_id: u16,
        eep: u32,
    },
    SaWrReset {
        client_id: u32,
    },
    SaRdLearnedClients,
    SaWrReclaims {
        reclaim_count: u8,
    },
    SaWrPostmaster {
        mailbox_count: u8,
    },
    SaRdMailboxStatus {
        smart_ack_client_id: u32,
        controller_id: u32,
    },
    SaDelMailbox {
        device_id: u32,
        controller_id: u32,
    },
    Unknown {
        code: u8,
        data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
        optional_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
    },
}
impl SmartAckData {
    pub fn command_type(&self) -> u8 {
        match self {
            Self::SaWrLearnMode { .. } => 1,
            Self::SaRdLearnMode => 2,
            Self::SaWrLearnConfirm { .. } => 3,
            Self::SaWrClientLearnRequest { .. } => 4,
            Self::SaWrReset { .. } => 5,
            Self::SaRdLearnedClients => 6,
            Self::SaWrReclaims { .. } => 7,
            Self::SaWrPostmaster { .. } => 8,
            Self::SaRdMailboxStatus { .. } => 9,
            Self::SaDelMailbox { .. } => 10,
            Self::Unknown { code, .. } => *code,
        }
    }

    pub fn to_packet_data(&self) -> MaxArray<u8, MAX_DATA_LENGTH> {
        let mut ret = MaxArray::new();

        let command_type = self.command_type();
        ret.push(command_type).unwrap();

        match self {
            Self::SaWrLearnMode {
                enable,
                extended,
                timeout,
            } => {
                ret.push_any(*enable).unwrap();
                ret.push_any(*extended).unwrap();
                ret.push_u32_be(*timeout).unwrap();
            },
            Self::SaRdLearnMode => {},
            Self::SaWrLearnConfirm {
                response_time,
                confirm_code,
                postmaster_candidate_id,
                smart_ack_client_id,
            } => {
                ret.push_u16_be(*response_time).unwrap();
                ret.push_any(*confirm_code).unwrap();
                ret.push_u32_be(*postmaster_candidate_id).unwrap();
                ret.push_u32_be(*smart_ack_client_id).unwrap();
            },
            Self::SaWrClientLearnRequest {
                manufacturer_id,
                eep,
            } => {
                ret.push_u16_be(*manufacturer_id).unwrap();
                ret.push_u32_be(*eep).unwrap();
            },
            Self::SaWrReset {
                client_id,
            } => {
                ret.push_u32_be(*client_id).unwrap();
            },
            Self::SaRdLearnedClients => {},
            Self::SaWrReclaims {
                reclaim_count,
            } => {
                ret.push(*reclaim_count).unwrap();
            },
            Self::SaWrPostmaster {
                mailbox_count,
            } => {
                ret.push(*mailbox_count).unwrap();
            },
            Self::SaRdMailboxStatus {
                smart_ack_client_id,
                controller_id,
            } => {
                ret.push_u32_be(*smart_ack_client_id).unwrap();
                ret.push_u32_be(*controller_id).unwrap();
            },
            Self::SaDelMailbox {
                device_id,
                controller_id,
            } => {
                ret.push_u32_be(*device_id).unwrap();
                ret.push_u32_be(*controller_id).unwrap();
            },
            Self::Unknown {
                data,
                ..
            } => {
                for b in data.iter() {
                    ret.push(*b).unwrap();
                }
            },
        }

        ret
    }

    pub fn to_packet_optional(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        Some(MaxArray::new())
    }

    pub fn from_data(event_code: u8, data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        match event_code {
            2|6 => {
                if data_slice.len() != 0 {
                    return None;
                }

                Some(match event_code {
                    2 => Self::SaRdLearnMode,
                    6 => Self::SaRdLearnedClients,
                    _ => unreachable!(),
                })
            },
            1 => { // SaWrLearnMode
                if data_slice.len() != 6 {
                    return None;
                }

                let enable = data_slice[0].into();
                let extended = data_slice[1].into();
                let timeout = u32::from_be_bytes(data_slice[2..6].try_into().unwrap());
                Some(Self::SaWrLearnMode {
                    enable,
                    extended,
                    timeout,
                })
            },
            3 => { // SaWrLearnConfirm
                if data_slice.len() != 11 {
                    return None;
                }

                let response_time = u16::from_be_bytes(data_slice[0..2].try_into().unwrap());
                let confirm_code = data_slice[2].into();
                let postmaster_candidate_id = u32::from_be_bytes(data_slice[3..7].try_into().unwrap());
                let smart_ack_client_id = u32::from_be_bytes(data_slice[7..11].try_into().unwrap());
                Some(Self::SaWrLearnConfirm {
                    response_time,
                    confirm_code,
                    postmaster_candidate_id,
                    smart_ack_client_id,
                })
            },
            4 => { // SaWrClientLearnRequest
                if data_slice.len() != 5 {
                    return None;
                }

                let manufacturer_id = u16::from_be_bytes(data_slice[0..2].try_into().unwrap());
                let eep =
                    (u32::from(data_slice[2]) << 16)
                    | (u32::from(data_slice[3]) << 8)
                    | u32::from(data_slice[4])
                ;
                Some(Self::SaWrClientLearnRequest {
                    manufacturer_id,
                    eep,
                })
            },
            5 => { // SaWrReset
                if data_slice.len() != 4 {
                    return None;
                }

                let client_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                Some(Self::SaWrReset {
                    client_id,
                })
            },
            7 => { // SaWrReclaims
                if data_slice.len() != 1 {
                    return None;
                }

                let reclaim_count = data_slice[0];
                Some(Self::SaWrReclaims {
                    reclaim_count,
                })
            },
            8 => { // SaWrPostmaster
                if data_slice.len() != 1 {
                    return None;
                }

                let mailbox_count = data_slice[0];
                Some(Self::SaWrPostmaster {
                    mailbox_count,
                })
            },
            9 => { // SaRdMailboxStatus
                if data_slice.len() != 8 {
                    return None;
                }

                let smart_ack_client_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let controller_id = u32::from_be_bytes(data_slice[4..8].try_into().unwrap());
                Some(Self::SaRdMailboxStatus {
                    smart_ack_client_id,
                    controller_id,
                })
            },
            10 => { // SaDelMailbox
                if data_slice.len() != 8 {
                    return None;
                }

                let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
                let controller_id = u32::from_be_bytes(data_slice[4..8].try_into().unwrap());
                Some(Self::SaDelMailbox {
                    device_id,
                    controller_id,
                })
            },
            other => {
                let data = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );
                let optional_data = MaxArray::from_iter_or_panic(
                    optional_slice.iter().map(|b| *b).peekable()
                );

                Some(Self::Unknown {
                    code: other,
                    data,
                    optional_data,
                })
            },
        }
    }
}

/// Data carried by a 2.4 GHz command.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Command24Data {
    SetChannel {
        channel: u8,
    },
    ReadChannel,
    Unknown {
        code: u8,
        data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
        optional_data: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
    },
}
impl Command24Data {
    pub fn command_type(&self) -> u8 {
        match self {
            Self::SetChannel { .. } => 1,
            Self::ReadChannel => 2,
            Self::Unknown { code, .. } => *code,
        }
    }

    pub fn to_packet_data(&self) -> MaxArray<u8, MAX_DATA_LENGTH> {
        let mut ret = MaxArray::new();

        let command_type = self.command_type();
        ret.push(command_type).unwrap();

        match self {
            Self::SetChannel {
                channel,
            } => {
                ret.push(*channel).unwrap();
            },
            Self::ReadChannel => {},
            Self::Unknown { data, .. } => {
                for b in data.iter() {
                    ret.push(*b).unwrap();
                }
            },
        }

        ret
    }

    pub fn to_packet_optional(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        Some(MaxArray::new())
    }

    pub fn from_data(event_code: u8, data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        match event_code {
            1 => {
                if data_slice.len() != 1 {
                    return None;
                }

                let channel = data_slice[0];
                Some(Self::SetChannel {
                    channel,
                })
            },
            2 => {
                if data_slice.len() != 0 {
                    return None;
                }
                Some(Self::ReadChannel)
            },
            other => {
                let data = MaxArray::from_iter_or_panic(
                    data_slice.iter().map(|b| *b).peekable()
                );
                let optional_data = MaxArray::from_iter_or_panic(
                    optional_slice.iter().map(|b| *b).peekable()
                );

                Some(Self::Unknown {
                    code: other,
                    data,
                    optional_data,
                })
            },
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
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
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
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
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
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum SecurityMode {
    Standard = 0x00,
    Extended = 0x01,
    Other(u8),
}

/// The cause for the secure device event.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
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
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
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
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum TransmissionFailureReason {
    CsmaFailed = 0x00,
    NotAcknowledged = 0x01,
    Other(u8),
}

/// The mode in which to enable the internal repeater.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum RepeaterEnable {
    Off = 0x00,
    On = 0x01,
    Selective = 0x02,
    Other(u8),
}

/// The level with which to repeat datagrams.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
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

/// The channel which to target.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum ChannelNumber {
    Absolute(u8),
    RelativePrevious = 0xFE,
    RelativeNext = 0xFF,
}

/// The mode in which to operate the transceiver.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum TransceiverMode {
    Compatible = 0x00,
    Advanced = 0x01,
    Other(u8),
}

/// The direction table on which to operate.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum DirectionTable {
    Inbound = 0x00,
    Outbound = 0x01,
    OutboundBroadcast = 0x02,
    Other(u8),
}

/// The direction table on which to operate, for commands that allow specifying "both".
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum DirectionTableBoth {
    Inbound = 0x00,
    Outbound = 0x01,
    OutboundBroadcast = 0x02,
    Both = 0x03,
    Other(u8),
}

/// The direction table on which to operate, for commands that allow specifying the maintenance
/// link.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum DirectionTableMaintenance {
    Inbound = 0x00,
    Outbound = 0x01,
    OutboundBroadcast = 0x02,
    MaintenanceLink = 0x03,
    Other(u8),
}

/// The baud rate with which the controller should communicate via UART.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum BaudRate {
    Baud57600 = 0x00,
    Baud115200 = 0x01,
    Baud230400 = 0x02,
    Baud460800 = 0x03,
    Other(u8),
}

/// The radio frequency on which the controller communicates.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum Frequency {
    Mhz315 = 0x00,
    Mhz868Point3 = 0x01,
    Mhz902Point875 = 0x02,
    Mhz925 = 0x03,
    Mhz928 = 0x04,
    Ghz2Point4 = 0x20,
    Other(u8),
}

/// The radio protocol with which the controller communicates.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum Protocol {
    Erp1 = 0x00,
    Erp2 = 0x01,
    Ieee802Dot15Dot4 = 0x10,
    LongRange = 0x30,
    Other(u8),
}

/// A type of transmit-only mode.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum TxOnlyMode {
    Off = 0x00,
    On = 0x01,
    OnWithAutoSleep = 0x02,
    Other(u8),
}

/// The criterion on which to filter messages.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum FilterCriterion {
    SourceAddress = 0x00,
    TelegramType = 0x01,
    MinimumSignalStrength = 0x02,
    DestinationAddress = 0x03,
    Other(u8),
}


/// The action to perform on messages that match the filter criterion.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum FilterAction {
    Drop = 0x00,
    Forward = 0x80,
    DoNotRepeat = 0x40,
    Repeat = 0xC0,
    Other(u8),
}

/// The operator used to unify multiple filters.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum FilterOperator {
    Or = 0x00,
    And = 0x01,
    ReceiveOrRepeatAnd = 0x08,
    ReceiveAndRepeatOr = 0x09,
    Other(u8),
}

/// The type of memory being accessed.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum MemoryType {
    Flash = 0x00,
    Ram0 = 0x01,
    DataRam = 0x02,
    IDataRam = 0x03,
    XDataRam = 0x04,
    Eeprom = 0x05,
    Other(u8),
}

/// The address area being queried.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum AddressArea {
    Config = 0,
    SmartAckTable = 1,
    SystemErrorLog = 2,
    Other(u8),
}


/// An extended learn mode.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum ExtendedLearnMode {
    Simple = 0,
    Advanced = 1,
    AdvancedSelectRepeater = 2,
    Other(u8),
}

/// Learn In/Out mode.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum LearnInOut {
    LearnIn = 0x00,
    LearnOut = 0x20,
    Other(u8),
}

/// Information about a learned client.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LearnedClient {
    pub smart_ack_client_id: u32,
    pub controller_id: u32,
    pub mailbox_index: u8,
}

/// The status of a mailbox.
#[derive(Clone, Copy, Debug)]
#[from_to_repr::from_to_other(base_type = u8, derive_compare = "as_int")]
pub enum MailboxStatus {
    Empty = 0,
    Full = 1,
    DoesNotExist = 2,
    Other(u8),
}
