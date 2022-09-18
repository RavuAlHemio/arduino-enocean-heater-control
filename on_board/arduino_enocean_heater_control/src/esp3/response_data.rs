//! Definitions of the structures of ESP3 response data.


use buildingblocks::max_array::MaxArray;

use crate::esp3::{
    ChannelNumber, FilterEntry, Frequency, MAX_DATA_LENGTH, MAX_OPTIONAL_LENGTH, MemoryType,
    OneByteBoolean, Protocol, RepeaterEnable, RepeaterLevel, TxOnlyMode,
};


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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdSysLog {
    pub api_log_entries: MaxArray<u8, MAX_DATA_LENGTH>,
    pub app_log_entries: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
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
    pub enabled: RepeaterEnable,
    pub level: RepeaterLevel,
}

/// Response data to CommandData::CoRdFilter.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdFilter {
    // max data length minus 1 byte of return code
    // 5 bytes per filter
    pub filters: MaxArray<FilterEntry, {(MAX_DATA_LENGTH - 1)/5}>,
}

/// Response data to CommandData::CoRdMem.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdMem {
    // max data length minus 1 byte of return code
    pub data: MaxArray<u8, {MAX_DATA_LENGTH - 1}>,
}

/// Response data to CommandData::CoRdMemAddress.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdMemAddress {
    pub memory_type: MemoryType,
    pub address: u32,
    pub length: u32,
}

/// Response data to CommandData::CoRdSecurity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[deprecated]
pub struct ResponseDataCoRdSecurity {
    pub security_level: u8,
    pub key: u32,
    pub rolling_code: u32,
}

/// Response data to CommandData::CoRdLearnMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdLearnMode {
    pub enabled: OneByteBoolean,
    pub opt_channel: Option<ChannelNumber>,
}

/// Response data to CommandData::CoRdSecureDeviceByIndex.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[deprecated]
pub struct ResponseDataCoRdSecureDeviceByIndex {
    pub slf: u8,
    pub device_id: u32,
    pub opt_private_key: Option<[u32; 4]>,
    pub opt_rolling_code: Option<u32>,
    pub opt_psk: Option<[u32; 4]>,
    pub opt_teach_info: Option<u8>,
}

/// Response data to CommandData::CoRdNumSecureDevices.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdNumSecureDevices {
    pub number: u8,
}

/// Response data to CommandData::CoRdSecureDeviceById.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdSecureDeviceById {
    pub slf: u8,
    pub opt_index: Option<u8>,
}

/// Response data to CommandData::CoRdSecureDevicePsk.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdSecureDevicePsk {
    pub psk: [u32; 4],
}

/// Response data to CommandData::CoRdDutyCycleLimit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdDutyCycleLimit {
    pub available: u8,
    pub slots: u8,
    pub slot_period: u16,
    pub actual_slot_left: u16,
    pub load_after_actual: u8,
}

/// Response data to CommandData::CoGetFrequencyInfo.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoGetFrequencyInfo {
    pub frequency: Frequency,
    pub protocol: Protocol,
}

/// Response data to CommandData::CoGetStepCode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoGetStepCode {
    pub step_code: u8,
    pub status_code: u8,
}

/// Response data to CommandData::CoRdReManRepeating.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdReManRepeating {
    pub re_man_telegrams_repeated: OneByteBoolean,
}

/// Response data to CommandData::CoGetNoiseThreshold.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoGetNoiseThreshold {
    pub rssi_level: u8,
}

/// Response data to CommandData::CoRdSecureDeviceV2ByIndex.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResponseDataCoRdSecureDeviceV2ByIndex {
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
pub struct ResponseDataCoRdSecureDeviceMaintenanceKey {
    pub device_id: u32,
    pub private_key: [u32; 4],
    pub key_number: u8,
    pub inbound_rolling_code: u32,
    pub outbound_rolling_code: u32,
}

/// Response data to CommandData::CoRdTxOnlyMode.
pub struct ResponseDataCoRdTxOnlyMode {
    pub mode: TxOnlyMode,
}
