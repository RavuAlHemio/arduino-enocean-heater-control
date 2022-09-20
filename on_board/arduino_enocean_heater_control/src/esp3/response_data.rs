//! Definitions of the structures of ESP3 response data.


use buildingblocks::max_array::MaxArray;
use buildingblocks::max_array_ext::MaxArrayPushIntExt;

use crate::esp3::{
    ChannelNumber, ExtendedLearnMode, FilterEntry, Frequency, LearnedClient, MailboxStatus,
    MAX_DATA_LENGTH, MAX_OPTIONAL_LENGTH, MemoryType, OneByteBoolean, Protocol, RepeaterEnable,
    RepeaterLevel, TxOnlyMode,
};


/// Trait to be implemented by response data.
pub trait ResponseData {
    /// Attempts to deserialize a response packet from the given data and optional data slices.
    ///
    /// Note that the passed data slice must not contain the response code in the first byte.
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> where Self : Sized;

    /// Attempts to serialize this response packet into data bytes.
    ///
    /// The maximum data length is reduced by 1 to make space for the response code byte.
    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}>;

    /// Attempts to serialize this response packet into optional data bytes.
    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        // no optional data by default
        Some(MaxArray::new())
    }
}


/// Response data to CommandData::CoRdVersion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdVersion {
    pub app_version: u32,
    pub api_version: u32,
    pub chip_id: u32,
    pub chip_version: u32,
    pub app_description: [u8; 16],
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

        let mut app_description = [0; 16];
        app_description.copy_from_slice(&data_slice[16..32]);

        Some(Self {
            app_version,
            api_version,
            chip_id,
            chip_version,
            app_description,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_u32_be(self.app_version).unwrap();
        ret.push_u32_be(self.api_version).unwrap();
        ret.push_u32_be(self.chip_id).unwrap();
        ret.push_u32_be(self.chip_version).unwrap();
        for b in self.app_description {
            ret.push(b).unwrap();
        }
        ret
    }
}

/// Response data to CommandData::CoRdSysLog.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSysLog {
    pub api_log_entries: MaxArray<u8, {MAX_DATA_LENGTH-1}>,
    pub app_log_entries: MaxArray<u8, MAX_OPTIONAL_LENGTH>,
}
impl ResponseData for CoRdSysLog {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        let mut api_log_entries = MaxArray::new();
        for b in data_slice {
            api_log_entries.push(*b).unwrap();
        }

        let mut app_log_entries = MaxArray::new();
        for b in optional_slice {
            app_log_entries.push(*b).unwrap();
        }

        Some(Self {
            api_log_entries,
            app_log_entries,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        self.api_log_entries.clone()
    }

    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        Some(self.app_log_entries.clone())
    }
}

/// Response data to CommandData::CoWrBiSt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoWrBiSt {
    pub bist_result: u8,
}
impl ResponseData for CoWrBiSt {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let bist_result = data_slice[0];
        Some(Self {
            bist_result,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.bist_result).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdIdBase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdIdBase {
    pub base_id: u32,
    pub opt_remaining_write_cycles: Option<u8>,
}
impl ResponseData for CoRdIdBase {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 4 {
            return None;
        }

        let base_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
        let opt_remaining_write_cycles = (optional_slice.len() >= 1)
            .then(|| optional_slice[0]);

        Some(Self {
            base_id,
            opt_remaining_write_cycles,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_u32_be(self.base_id).unwrap();
        ret
    }

    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        let mut ret = MaxArray::new();
        if let Some(remaining_write_cycles) = self.opt_remaining_write_cycles {
            ret.push(remaining_write_cycles).unwrap();
        }
        Some(ret)
    }
}

/// Response data to CommandData::CoRdRepeater.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdRepeater {
    pub enabled: RepeaterEnable,
    pub level: RepeaterLevel,
}
impl ResponseData for CoRdRepeater {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 2 {
            return None;
        }

        let enabled = data_slice[0].into();
        let level = data_slice[1].into();
        Some(Self {
            enabled,
            level,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.enabled.into()).unwrap();
        ret.push(self.level.into()).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdFilter.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdFilter {
    // max data length minus 1 byte of return code
    // 5 bytes per filter
    pub filters: MaxArray<FilterEntry, {(MAX_DATA_LENGTH - 1)/5}>,
}
impl ResponseData for CoRdFilter {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() % 5 != 0 {
            return None;
        }

        let filters = todo!();

        Some(Self {
            filters,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        todo!();
        ret
    }
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
