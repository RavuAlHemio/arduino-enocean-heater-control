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
        ret.push_any(self.enabled).unwrap();
        ret.push_any(self.level).unwrap();
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

        let mut filters = MaxArray::new();
        let mut i = 0;
        while i < data_slice.len() {
            let criterion = data_slice[i].into();
            let value = u32::from_be_bytes(data_slice[i+1..i+1+4].try_into().unwrap());

            filters.push(FilterEntry {
                criterion,
                value,
            }).unwrap();
            i += 5;
        }

        Some(Self {
            filters,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        for filter in self.filters.iter() {
            ret.push_any(filter.criterion).unwrap();
            ret.push_u32_be(filter.value).unwrap();
        }
        ret
    }
}

/// Response data to CommandData::CoRdMem.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdMem {
    // max data length minus 1 byte of return code
    pub data: MaxArray<u8, {MAX_DATA_LENGTH-1}>,
}
impl ResponseData for CoRdMem {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        let data = MaxArray::from_iter_or_panic(
            data_slice.iter().map(|b| *b).peekable()
        );
        Some(Self {
            data,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        self.data.clone()
    }
}

/// Response data to CommandData::CoRdMemAddress.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdMemAddress {
    pub memory_type: MemoryType,
    pub address: u32,
    pub length: u32,
}
impl ResponseData for CoRdMemAddress {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 9 {
            return None;
        }

        let memory_type = data_slice[0].into();
        let address = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
        let length = u32::from_be_bytes(data_slice[5..9].try_into().unwrap());
        Some(Self {
            memory_type,
            address,
            length,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.memory_type).unwrap();
        ret.push_u32_be(self.address).unwrap();
        ret.push_u32_be(self.length).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdSecurity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[deprecated]
pub struct CoRdSecurity {
    pub security_level: u8,
    pub key: u32,
    pub rolling_code: u32,
}
#[allow(deprecated)]
impl ResponseData for CoRdSecurity {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 9 {
            return None;
        }

        let security_level = data_slice[0].into();
        let key = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
        let rolling_code = u32::from_be_bytes(data_slice[5..9].try_into().unwrap());
        Some(Self {
            security_level,
            key,
            rolling_code,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.security_level).unwrap();
        ret.push_u32_be(self.key).unwrap();
        ret.push_u32_be(self.rolling_code).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdLearnMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdLearnMode {
    pub enabled: OneByteBoolean,
    pub opt_channel: Option<ChannelNumber>,
}
impl ResponseData for CoRdLearnMode {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let enabled = data_slice[0].into();
        let opt_channel = (optional_slice.len() >= 1)
            .then(|| optional_slice[0].into());
        Some(Self {
            enabled,
            opt_channel,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.enabled).unwrap();
        ret
    }

    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        let mut ret = MaxArray::new();
        if let Some(channel) = self.opt_channel {
            ret.push_any(channel).unwrap();
        }
        Some(ret)
    }
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
#[allow(deprecated)]
impl ResponseData for CoRdSecureDeviceByIndex {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 5 {
            return None;
        }

        let slf = data_slice[0];
        let device_id = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());

        let opt_private_key = (optional_slice.len() >= 16)
            .then(|| [
                u32::from_be_bytes(optional_slice[0..4].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[4..8].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[8..12].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[12..16].try_into().unwrap()),
            ]);

        // rolling code is only 3 bytes; deserialize manually
        let opt_rolling_code = (optional_slice.len() >= 19)
            .then(||
                (u32::from(optional_slice[16]) << 16)
                | (u32::from(optional_slice[17]) << 8)
                | u32::from(optional_slice[18])
            );

        let opt_psk = (optional_slice.len() >= 35)
            .then(|| [
                u32::from_be_bytes(optional_slice[19..23].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[23..27].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[27..31].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[31..35].try_into().unwrap()),
            ]);
        let opt_teach_info = (optional_slice.len() >= 36)
            .then(|| optional_slice[35]);

        Some(Self {
            slf,
            device_id,
            opt_private_key,
            opt_rolling_code,
            opt_psk,
            opt_teach_info,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.slf).unwrap();
        ret.push_u32_be(self.device_id).unwrap();
        ret
    }

    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        let mut ret = MaxArray::new();

        if let Some(private_key) = self.opt_private_key {
            ret.push_u32_be(private_key[0]).unwrap();
            ret.push_u32_be(private_key[1]).unwrap();
            ret.push_u32_be(private_key[2]).unwrap();
            ret.push_u32_be(private_key[3]).unwrap();
        } else if self.opt_rolling_code.is_some() || self.opt_psk.is_some() || self.opt_teach_info.is_some() {
            // later optional field is Some(_) but this one is None
            return None;
        }
        if let Some(rolling_code) = self.opt_rolling_code {
            // rolling code is only 3 bytes; serialize manually
            ret.push(((rolling_code >> 16) & 0xFF).try_into().unwrap()).unwrap();
            ret.push(((rolling_code >> 8) & 0xFF).try_into().unwrap()).unwrap();
            ret.push((rolling_code & 0xFF).try_into().unwrap()).unwrap();
        } else if self.opt_psk.is_some() || self.opt_teach_info.is_some() {
            return None;
        }
        if let Some(psk) = self.opt_psk {
            ret.push_u32_be(psk[0]).unwrap();
            ret.push_u32_be(psk[1]).unwrap();
            ret.push_u32_be(psk[2]).unwrap();
            ret.push_u32_be(psk[3]).unwrap();
        } else if self.opt_teach_info.is_some() {
            return None;
        }
        if let Some(teach_info) = self.opt_teach_info {
            ret.push(teach_info).unwrap();
        }

        Some(ret)
    }
}

/// Response data to CommandData::CoRdNumSecureDevices.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdNumSecureDevices {
    pub number: u8,
}
impl ResponseData for CoRdNumSecureDevices {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let number = data_slice[0];
        Some(Self {
            number,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.number).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdSecureDeviceById.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSecureDeviceById {
    pub slf: u8,
    pub opt_index: Option<u8>,
}
impl ResponseData for CoRdSecureDeviceById {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let slf = data_slice[0];

        let opt_index = (optional_slice.len() >= 1)
            .then(|| optional_slice[0]);

        Some(Self {
            slf,
            opt_index,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.slf).unwrap();
        ret
    }

    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        let mut ret = MaxArray::new();

        if let Some(index) = self.opt_index {
            ret.push(index).unwrap();
        }

        Some(ret)
    }
}

/// Response data to CommandData::CoRdSecureDevicePsk.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdSecureDevicePsk {
    pub psk: [u32; 4],
}
impl ResponseData for CoRdSecureDevicePsk {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 16 {
            return None;
        }

        let psk = [
            u32::from_be_bytes(data_slice[0..4].try_into().unwrap()),
            u32::from_be_bytes(data_slice[4..8].try_into().unwrap()),
            u32::from_be_bytes(data_slice[8..12].try_into().unwrap()),
            u32::from_be_bytes(data_slice[12..16].try_into().unwrap()),
        ];

        Some(Self {
            psk,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_u32_be(self.psk[0]).unwrap();
        ret.push_u32_be(self.psk[1]).unwrap();
        ret.push_u32_be(self.psk[2]).unwrap();
        ret.push_u32_be(self.psk[3]).unwrap();
        ret
    }
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
impl ResponseData for CoRdDutyCycleLimit {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 7 {
            return None;
        }

        let available = data_slice[0];
        let slots = data_slice[1];
        let slot_period = u16::from_be_bytes(data_slice[2..4].try_into().unwrap());
        let actual_slot_left = u16::from_be_bytes(data_slice[4..6].try_into().unwrap());
        let load_after_actual = data_slice[6];

        Some(Self {
            available,
            slots,
            slot_period,
            actual_slot_left,
            load_after_actual,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.available).unwrap();
        ret.push(self.slots).unwrap();
        ret.push_u16_be(self.slot_period).unwrap();
        ret.push_u16_be(self.actual_slot_left).unwrap();
        ret.push(self.load_after_actual).unwrap();
        ret
    }
}

/// Response data to CommandData::CoGetFrequencyInfo.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoGetFrequencyInfo {
    pub frequency: Frequency,
    pub protocol: Protocol,
}
impl ResponseData for CoGetFrequencyInfo {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 2 {
            return None;
        }

        let frequency = data_slice[0].into();
        let protocol = data_slice[1].into();

        Some(Self {
            frequency,
            protocol,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.frequency).unwrap();
        ret.push_any(self.protocol).unwrap();
        ret
    }
}

/// Response data to CommandData::CoGetStepCode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoGetStepCode {
    pub step_code: u8,
    pub status_code: u8,
}
impl ResponseData for CoGetStepCode {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 2 {
            return None;
        }

        let step_code = data_slice[0];
        let status_code = data_slice[1];

        Some(Self {
            step_code,
            status_code,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.step_code).unwrap();
        ret.push(self.status_code).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdReManRepeating.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdReManRepeating {
    pub re_man_telegrams_repeated: OneByteBoolean,
}
impl ResponseData for CoRdReManRepeating {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let re_man_telegrams_repeated = data_slice[0].into();

        Some(Self {
            re_man_telegrams_repeated,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.re_man_telegrams_repeated).unwrap();
        ret
    }
}

/// Response data to CommandData::CoGetNoiseThreshold.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoGetNoiseThreshold {
    pub rssi_level: u8,
}
impl ResponseData for CoGetNoiseThreshold {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let rssi_level = data_slice[0];

        Some(Self {
            rssi_level,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.rssi_level).unwrap();
        ret
    }
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
impl ResponseData for CoRdSecureDeviceV2ByIndex {
    fn from_data(data_slice: &[u8], optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 26 {
            return None;
        }

        let slf = data_slice[0];
        let device_id = u32::from_be_bytes(data_slice[1..5].try_into().unwrap());
        let private_key = [
            u32::from_be_bytes(data_slice[5..9].try_into().unwrap()),
            u32::from_be_bytes(data_slice[9..13].try_into().unwrap()),
            u32::from_be_bytes(data_slice[13..17].try_into().unwrap()),
            u32::from_be_bytes(data_slice[17..21].try_into().unwrap()),
        ];
        let rolling_code = u32::from_be_bytes(data_slice[21..25].try_into().unwrap());
        let teach_info = data_slice[25];

        let opt_psk = (optional_slice.len() >= 16)
            .then(|| [
                u32::from_be_bytes(optional_slice[0..4].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[4..8].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[8..12].try_into().unwrap()),
                u32::from_be_bytes(optional_slice[12..16].try_into().unwrap()),
            ]);

        Some(Self {
            slf,
            device_id,
            private_key,
            rolling_code,
            teach_info,
            opt_psk,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.slf).unwrap();
        ret.push_u32_be(self.device_id).unwrap();
        ret.push_u32_be(self.private_key[0]).unwrap();
        ret.push_u32_be(self.private_key[1]).unwrap();
        ret.push_u32_be(self.private_key[2]).unwrap();
        ret.push_u32_be(self.private_key[3]).unwrap();
        ret.push_u32_be(self.rolling_code).unwrap();
        ret.push(self.teach_info).unwrap();
        ret
    }

    fn to_optional_data(&self) -> Option<MaxArray<u8, MAX_OPTIONAL_LENGTH>> {
        let mut ret = MaxArray::new();
        if let Some(psk) = self.opt_psk {
            ret.push_u32_be(psk[0]).unwrap();
            ret.push_u32_be(psk[1]).unwrap();
            ret.push_u32_be(psk[2]).unwrap();
            ret.push_u32_be(psk[3]).unwrap();
        }
        Some(ret)
    }
}

/// Response data to CommandData::CoRdRssiTestMode and CommandData::CoRdTransparentMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OneByteBooleanResponseData {
    pub enabled: OneByteBoolean,
}
impl ResponseData for OneByteBooleanResponseData {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let enabled = data_slice[0].into();
        Some(Self {
            enabled,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.enabled).unwrap();
        ret
    }
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
impl ResponseData for CoRdSecureDeviceMaintenanceKey {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 29 {
            return None;
        }

        let device_id = u32::from_be_bytes(data_slice[0..4].try_into().unwrap());
        let private_key = [
            u32::from_be_bytes(data_slice[4..8].try_into().unwrap()),
            u32::from_be_bytes(data_slice[8..12].try_into().unwrap()),
            u32::from_be_bytes(data_slice[12..16].try_into().unwrap()),
            u32::from_be_bytes(data_slice[16..20].try_into().unwrap()),
        ];
        let key_number = data_slice[20];
        let inbound_rolling_code = u32::from_be_bytes(data_slice[21..25].try_into().unwrap());
        let outbound_rolling_code = u32::from_be_bytes(data_slice[25..29].try_into().unwrap());

        Some(Self {
            device_id,
            private_key,
            key_number,
            inbound_rolling_code,
            outbound_rolling_code,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_u32_be(self.device_id).unwrap();
        ret.push_u32_be(self.private_key[0]).unwrap();
        ret.push_u32_be(self.private_key[1]).unwrap();
        ret.push_u32_be(self.private_key[2]).unwrap();
        ret.push_u32_be(self.private_key[3]).unwrap();
        ret.push(self.key_number).unwrap();
        ret.push_u32_be(self.inbound_rolling_code).unwrap();
        ret.push_u32_be(self.outbound_rolling_code).unwrap();
        ret
    }
}

/// Response data to CommandData::CoRdTxOnlyMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoRdTxOnlyMode {
    pub mode: TxOnlyMode,
}
impl ResponseData for CoRdTxOnlyMode {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let mode = data_slice[0].into();
        Some(Self {
            mode,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.mode).unwrap();
        ret
    }
}

/// Response data to SmartAckData::SaRdLearnMode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SaRdLearnMode {
    pub enabled: OneByteBoolean,
    pub extended: ExtendedLearnMode,
}
impl ResponseData for SaRdLearnMode {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 2 {
            return None;
        }

        let enabled = data_slice[0].into();
        let extended = data_slice[1].into();
        Some(Self {
            enabled,
            extended,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.enabled).unwrap();
        ret.push_any(self.enabled).unwrap();
        ret
    }
}

/// Response data to SmartAckData::SaRdLearnedClients.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SaRdLearnedClients {
    // max data minus one byte of return code
    // 9 bytes per learned client
    pub learned_clients: MaxArray<LearnedClient, {(MAX_DATA_LENGTH - 1)/9}>,
}
impl ResponseData for SaRdLearnedClients {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() % 9 != 0 {
            return None;
        }

        let mut learned_clients = MaxArray::new();
        let mut i = 0;
        while i < data_slice.len() {
            let smart_ack_client_id = u32::from_be_bytes(data_slice[i..i+4].try_into().unwrap());
            let controller_id = u32::from_be_bytes(data_slice[i+4..i+8].try_into().unwrap());
            let mailbox_index = data_slice[i+8];
            learned_clients.push(LearnedClient {
                smart_ack_client_id,
                controller_id,
                mailbox_index,
            }).unwrap();
            i += 9;
        }

        Some(Self {
            learned_clients,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        for learned_client in self.learned_clients.iter() {
            ret.push_u32_be(learned_client.smart_ack_client_id).unwrap();
            ret.push_u32_be(learned_client.controller_id).unwrap();
            ret.push(learned_client.mailbox_index).unwrap();
        }
        ret
    }
}

/// Response data to SmartAckData::SaRdMailboxStatus.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SaRdMailboxStatus {
    pub status: MailboxStatus,
}
impl ResponseData for SaRdMailboxStatus {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let status = data_slice[0].into();
        Some(Self {
            status,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push_any(self.status).unwrap();
        ret
    }
}

/// Response data to Command24Data::ReadChannel.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct R802RdChannel {
    pub channel: u8,
}
impl ResponseData for R802RdChannel {
    fn from_data(data_slice: &[u8], _optional_slice: &[u8]) -> Option<Self> {
        if data_slice.len() != 1 {
            return None;
        }

        let channel = data_slice[0];
        Some(Self {
            channel,
        })
    }

    fn to_data(&self) -> MaxArray<u8, {MAX_DATA_LENGTH-1}> {
        let mut ret = MaxArray::new();
        ret.push(self.channel).unwrap();
        ret
    }
}
