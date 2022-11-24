use std::io;
use std::mem::size_of_val;
use std::ptr::null_mut;

use windows::w;
use windows::core::{GUID, PCWSTR, PWSTR};
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_Device_ID_List_SizeW, CM_Get_Device_ID_ListW, CM_Get_Device_Interface_List_SizeW,
    CM_Get_Device_Interface_ListW, CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
    CM_GETIDLIST_FILTER_ENUMERATOR, CM_GETIDLIST_FILTER_PRESENT, CM_Open_Class_KeyW,
    CM_OPEN_CLASS_KEY_INTERFACE, CR_SUCCESS, RegDisposition_OpenExisting,
};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, KEY_ENUMERATE_SUB_KEYS, KEY_QUERY_VALUE, RegCloseKey, RegEnumKeyExW, RegOpenKeyExW,
    RegQueryInfoKeyW,
};


const BLACK_MAGIC_TRACE_CAPTURE_HARDWARE_ID: PCWSTR = w!("USB\\VID_1D50&PID_6018&MI_05");
const ZADIG_TEMPORARY_DEVICE_CLASS_GUID: GUID = GUID::from_u128(0xD7703CA2_E823_4929_92F6_A43BA6A71CB2);


struct RegKeyHandle(pub HKEY);
impl Drop for RegKeyHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { RegCloseKey(self.0) };
            self.0 = HKEY::default();
        }
    }
}


fn guid_to_string(guid: &GUID) -> String {
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid.data1, guid.data2, guid.data3,
        guid.data4[0], guid.data4[1], guid.data4[2], guid.data4[3],
        guid.data4[4], guid.data4[5], guid.data4[6], guid.data4[7],
    )
}


fn string_to_utf16_nul(s: &str) -> Vec<u16> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}


/// Decodes a sequence of wide-character strings.
///
/// Strings are delimited with a NUL wide-character. The end of the sequence is marked with a
/// zero-length string that is not considered part of the sequence.
fn decode_wide_string_sequence(seq: &[u16]) -> Vec<String> {
    let mut ret = Vec::new();

    let mut cur_seq = seq;
    while cur_seq.len() > 0 {
        // find ending NUL
        let nul_offset = cur_seq.iter()
            .enumerate()
            .filter(|(_i, w)| **w == 0x0000)
            .map(|(i, _w)| i)
            .nth(0)
            .unwrap_or(cur_seq.len());

        // slice the string
        let cur_slice = &cur_seq[0..nul_offset];
        if cur_slice.len() == 0 {
            // end of list
            break;
        }

        let string = match String::from_utf16(cur_slice) {
            Ok(s) => s,
            Err(e) => panic!("invalid wide string sequence {:?}: {}", cur_slice, e),
        };
        ret.push(string);

        // next slice!
        cur_seq = &cur_seq[nul_offset+1..];
    }

    ret
}


/// Enumerates the subkeys of a registry key and runs a function on each of them.
fn enumerate_registry_subkeys<F: FnMut(&str) -> Result<(), io::Error>>(key: HKEY, mut handle: F) -> Result<(), io::Error> {
    let mut subkey_index = 0;
    loop {
        let mut subkey_name_buf = [0u16; 256];
        let mut subkey_name_len: u32 = subkey_name_buf.len().try_into().unwrap();
        let ret = unsafe {
            RegEnumKeyExW(
                key,
                subkey_index,
                PWSTR(subkey_name_buf.as_mut_ptr()),
                &mut subkey_name_len,
                None,
                PWSTR(null_mut()),
                None,
                None,
            )
        };
        if ret == ERROR_NO_MORE_ITEMS {
            break;
        } else if ret != ERROR_SUCCESS {
            eprintln!(
                "failed to enumerate subkey at index {}: {:?}",
                subkey_index, ret.ok().unwrap_err(),
            );
            return Err(io::ErrorKind::Other.into());
        }

        todo!();
    }

    Ok(())
}


/// Attempts to find the interface class GUID for the probe trace capture endpoint.
fn find_interface_class_guid(device_id: &str) -> Result<GUID, io::Error> {
    // scan HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceClasses to find interface class
    // don't hard-code this registry path; let Windows tell us
    let mut iface_classes_hkey_raw = HKEY::default();
    let ret = unsafe {
        CM_Open_Class_KeyW(
            None,
            None,
            KEY_ENUMERATE_SUB_KEYS.0,
            RegDisposition_OpenExisting,
            &mut iface_classes_hkey_raw,
            CM_OPEN_CLASS_KEY_INTERFACE,
        )
    };
    if ret != CR_SUCCESS {
        eprintln!("failed to open device interface class key: {:?}", ret);
        return Err(io::ErrorKind::Other.into());
    }
    let iface_classes_hkey = RegKeyHandle(iface_classes_hkey_raw);

    // run through each interface class
    let mut iface_class_index = 0;
    loop {
        let mut iface_class_key_name_buf = [0u16; 256];
        let mut iface_class_key_name_len: u32 = iface_class_key_name_buf.len().try_into().unwrap();
        let ret = unsafe {
            RegEnumKeyExW(
                iface_classes_hkey.0,
                iface_class_index,
                PWSTR(iface_class_key_name_buf.as_mut_ptr()),
                &mut iface_class_key_name_len,
                None,
                PWSTR(null_mut()),
                None,
                None,
            )
        };
        if ret == ERROR_NO_MORE_ITEMS {
            break;
        } else if ret != ERROR_SUCCESS {
            eprintln!(
                "failed to enumerate device interface class key at index {}: {:?}",
                iface_class_index, ret.ok().unwrap_err(),
            );
            return Err(io::ErrorKind::Other.into());
        }

        // open the interface class subkey
        let mut iface_class_hkey_raw = HKEY::default();
        let ret = unsafe {
            RegOpenKeyExW(
                iface_classes_hkey.0,
                PCWSTR(iface_class_key_name_buf.as_ptr()),
                0,
                (KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE).0,
                &mut iface_class_hkey_raw,
            )
        };
        if ret != ERROR_SUCCESS {
            let subkey_name = unsafe {
                PCWSTR(iface_class_key_name_buf).to_string()
            };
            eprintln!(
                "failed to open interface class subkey {:?}: {:?}",
                subkey_name, ret,
            );
            interface_class_index += 1;
            continue;
        }

        // enumerate the devices in the interface class subkey
        let mut device_index = 0;
        loop {

        }

        interface_class_index += 1;
    }

    // TODO: RegEnumKeyExW on hkey
    // gives us subkeys named "{01234567-89AB-CDEF-0123-456789ABCDEF}"
    // which are the GUIDs of the interface classes

    // TODO: RegEnumKeyExW on each interface class GUID key
    // don't attempt to process the key names (weird escaping is going on there)
    // value named "DeviceInstance" tells us the instance path of the device
    // that corresponds to this interface class
    // if its prefix is (BLACK_MAGIC_TRACE_CAPTURE_HARDWARE_ID + "\\"),
    // we have found the correct interface class GUID
}


fn main() -> Result<(), io::Error> {
    let mut chars_required: u32 = 0;
    let ret = unsafe {
        CM_Get_Device_ID_List_SizeW(
            &mut chars_required,
            BLACK_MAGIC_TRACE_CAPTURE_HARDWARE_ID,
            CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT,
        )
    };
    if ret != CR_SUCCESS {
        eprintln!("failed to obtain device list size: {:?}", ret);
        return Err(io::ErrorKind::Other.into());
    }

    let chars_required_usize: usize = chars_required.try_into().unwrap();
    let mut device_ids_chars = vec![0u16; chars_required_usize];
    let ret = unsafe {
        CM_Get_Device_ID_ListW(
            BLACK_MAGIC_TRACE_CAPTURE_HARDWARE_ID,
            device_ids_chars.as_mut_slice(),
            CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT,
        )
    };
    if ret != CR_SUCCESS {
        eprintln!("failed to obtain device list: {:?}", ret);
        return Err(io::ErrorKind::Other.into());
    }

    let mut device_ids = decode_wide_string_sequence(&device_ids_chars);
    if device_ids.len() == 0 {
        eprintln!("no Black Magic Trace Capture devices found");
        return Err(io::ErrorKind::Other.into());
    }

    for device_id in device_ids {
        let device_id_pcwstr = string_to_utf16_nul(&device_id);

        // TODO: use the interface class GUID in this call
        // instead of ZADIG_TEMPORARY_DEVICE_CLASS_GUID:
        let mut chars_required = 0;
        let ret = unsafe {
            CM_Get_Device_Interface_List_SizeW(
                &mut chars_required,
                &ZADIG_TEMPORARY_DEVICE_CLASS_GUID,
                PCWSTR(device_id_pcwstr.as_ptr()),
                CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
            )
        };
        if ret != CR_SUCCESS {
            eprintln!("failed to obtain device interface list size for {:?}: {:?}", device_id, ret);
            return Err(io::ErrorKind::Other.into());
        }

        let chars_required_usize: usize = chars_required.try_into().unwrap();
        let mut interfaces_chars = vec![0u16; chars_required_usize];
        let ret = unsafe {
            CM_Get_Device_Interface_ListW(
                &ZADIG_TEMPORARY_DEVICE_CLASS_GUID,
                PCWSTR(device_id_pcwstr.as_ptr()),
                interfaces_chars.as_mut_slice(),
                CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
            )
        };
        if ret != CR_SUCCESS {
            eprintln!("failed to obtain device interface list for {:?}: {:?}", device_id, ret);
            return Err(io::ErrorKind::Other.into());
        }

        let interfaces = decode_wide_string_sequence(&interfaces_chars);
        for interface in interfaces {
            eprintln!("interface: {:?}", interface);
        }
    }

    Ok(())
}
