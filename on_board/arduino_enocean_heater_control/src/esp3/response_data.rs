//! Definitions of the structures of ESP3 response data.


use buildingblocks::max_array::MaxArray;

use crate::esp3::{
    ChannelNumber, ExtendedLearnMode, FilterEntry, Frequency, LearnedClient, MailboxStatus,
    MAX_DATA_LENGTH, MAX_OPTIONAL_LENGTH, MemoryType, OneByteBoolean, Protocol, RepeaterEnable,
    RepeaterLevel, TxOnlyMode,
};


/// Trait to be implemented by response data.
pub trait ResponseData {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> where Self : Sized;
}


/// Response data to CommandData::CoRdVersion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdVersion {
    pub app_version: u32,
    pub api_version: u32,
    pub chip_id: u32,
    pub chip_version: u32,
    pub app_description: [char; 16],
}
impl ResponseData for CoRdVersion {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 32 {
            return None;
        }

        let app_version = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
        let api_version = u32::from_be_bytes(data_slice[4..8].try_into().unwrap());
        let chip_id = u32::from_be_bytes(data_slice[8..12].try_into().unwrap());
        let chip_version = u32::from_be_bytes(data_slice[12..16].try_into().unwrap());

        let mut app_description = [0 as char; 16];
        for (ad, d) in app_description.iter_mut().zip(data_slice[16..32].iter()) {
            *ad = *d as char;
        }

        Some(Self {
            app_version,
            api_version,
            chip_id,
            chip_version,
            app_description,
        })
    }
}

/// Response data to CommandData::CoRdSysLog.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSysLog {
    pub api_log_entries: MaxArray<u8, MAX_DATA_LENGTH>,
    pub app_log_entries: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
}

/// Response data to CommandData::CoWrBiSt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoWrBiSt {
    pub bist_result: u8,
}

/// Response data to CommandData::CoRdIdBase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdIdBase {
    pub base_id: u32,
    pub opt_remaining_write_cycles: Option<u8>,
}

/// Response data to CommandData::CoRdRepeater.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdRepeater {
    pub enabled: RepeaterEnable,
    pub level: RepeaterLevel,
}

/// Response data to CommandData::CoRdFilter.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdFilter {
    // max data length minus 1 byte of return code
    // 5 bytes per filter
    pub filters: MaxArray<FilterEntry, {(MAX_DATA_LENGTH - 1)/5}>,
}

/// Response data to CommandData::CoRdMem.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdMem {
    // max data length minus 1 byte of return code
    pub data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
}

/// Response data to CommandData::CoRdMemAddress.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdMemAddress {
    pub memory_type: MemoryType,
    pub address: u32,
    pub length: u32,
}

/// Response data to CommandData::CoRdSecurity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[deprecated]
pub struct CoRdSecurity {
    pub security_level: u8,
    pub key: u32,
    pub rolling_code: u32,
}

/// Response data to CommandData::CoRdLearnMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdLearnMode {
    pub enabled: OneByteBoolean,
    pub opt_channel: Option<ChannelNumber>,
}

/// Response data to CommandData::CoRdSecureDeviceByIndex.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[deprecated]
pub struct CoRdSecureDeviceByIndex {
    pub slf: u8,
    pub device_id: u32,
    pub opt_private_key: Option<[u32; 4]>,
    pub opt_rolling_code: Option<u32>,
    pub opt_psk: Option<[u32; 4]>,
    pub opt_teach_info: Option<u8>,
}

/// Response data to CommandData::CoRdNumSecureDevices.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdNumSecureDevices {
    pub number: u8,
}

/// Response data to CommandData::CoRdSecureDeviceById.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSecureDeviceById {
    pub slf: u8,
    pub opt_index: Option<u8>,
}

/// Response data to CommandData::CoRdSecureDevicePsk.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSecureDevicePsk {
    pub psk: [u32; 4],
}

/// Response data to CommandData::CoRdDutyCycleLimit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdDutyCycleLimit {
    pub available: u8,
    pub slots: u8,
    pub slot_period: u16,
    pub actual_slot_left: u16,
    pub load_after_actual: u8,
}

/// Response data to CommandData::CoGetFrequencyInfo.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoGetFrequencyInfo {
    pub frequency: Frequency,
    pub protocol: Protocol,
}

/// Response data to CommandData::CoGetStepCode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoGetStepCode {
    pub step_code: u8,
    pub status_code: u8,
}

/// Response data to CommandData::CoRdReManRepeating.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdReManRepeating {
    pub re_man_telegrams_repeated: OneByteBoolean,
}

/// Response data to CommandData::CoGetNoiseThreshold.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoGetNoiseThreshold {
    pub rssi_level: u8,
}

/// Response data to CommandData::CoRdSecureDeviceV2ByIndex.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSecureDeviceV2ByIndex {
    pub slf: u8,
    pub device_id: u32,
    pub private_key: [u32; 4],
    pub rolling_code: u32,
    pub teach_info: u8,
    pub opt_psk: Option<[u32; 4]>,
}

/// Response data to CommandData::CoRdRssiTestMode and CommandData::CoRdTransparentMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OneByteBooleanResponseData {
    pub enabled: OneByteBoolean,
}

/// Response data to CommandData::CoRdSecureDeviceMaintenanceKey.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSecureDeviceMaintenanceKey {
    pub device_id: u32,
    pub private_key: [u32; 4],
    pub key_number: u8,
    pub inbound_rolling_code: u32,
    pub outbound_rolling_code: u32,
}

/// Response data to CommandData::CoRdTxOnlyMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdTxOnlyMode {
    pub mode: TxOnlyMode,
}

/// Response data to SmartAckData::SaRdLearnMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SaRdLearnMode {
    pub enabled: OneByteBoolean,
    pub extended: ExtendedLearnMode,
}

/// Response data to SmartAckData::SaRdLearnedClients.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SaRdLearnedClients {
    // max data minus one byte of return code
    // 9 bytes per learned client
    pub learned_clients: MaxArray<LearnedClient, {(MAX_DATA_LENGTH - 1)/9}>,
}

/// Response data to SmartAckData::SaRdMailboxStatus.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SaRdMailboxStatus {
    pub status: MailboxStatus,
}

/// Response data to Command24Data::ReadChannel.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct R802RdChannel {
    pub channel: u8,
}
