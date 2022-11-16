use std::ffi::{OsStr, OsString};
use std::io;
use std::mem::{replace, size_of_val};
use std::os::windows::ffi::OsStrExt;

use c2rust_bitfields::BitfieldStruct;
use clap::Parser;
use windows::core::PCWSTR;
use windows::Win32::Devices::Communication::{DCB, GetCommState, SetCommState};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::SystemServices::{GENERIC_READ, GENERIC_WRITE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ACCESS_FLAGS, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_NONE, OPEN_EXISTING,
};


#[derive(Parser)]
struct Opts {
    pub serial_port: OsString,

    #[arg(short, long)]
    pub disable_dtr: bool,
}

#[derive(BitfieldStruct)]
struct DcbBitfield {
    #[bitfield(name = "binary", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "parity", ty = "bool", bits = "1..=1")]
    #[bitfield(name = "outx_cts_flow", ty = "bool", bits = "2..=2")]
    #[bitfield(name = "outx_dsr_flow", ty = "bool", bits = "3..=3")]
    #[bitfield(name = "dtr_control", ty = "u8", bits = "4..=5")]
    #[bitfield(name = "dsr_sensitivity", ty = "bool", bits = "6..=6")]
    #[bitfield(name = "tx_continue_on_xoff", ty = "bool", bits = "7..=7")]
    #[bitfield(name = "out_x", ty = "bool", bits = "8..=8")]
    #[bitfield(name = "in_x", ty = "bool", bits = "9..=9")]
    #[bitfield(name = "error_char", ty = "bool", bits = "10..=10")]
    #[bitfield(name = "null", ty = "bool", bits = "11..=11")]
    #[bitfield(name = "rts_control", ty = "u8", bits = "12..=13")]
    #[bitfield(name = "abort_on_error", ty = "bool", bits = "14..=14")]
    field: [u8; 4],
}


fn to_null_term_wide_string(s: &OsStr) -> Vec<u16> {
    let mut ret: Vec<u16> = s.encode_wide()
        .collect();
    ret.push(0x0000);
    ret
}


struct ClosableHandle(pub HANDLE);
impl ClosableHandle {
    /// Frees the inner handle from the ClosableHandle and replaces it with an invalid one.
    ///
    /// The inner handle will no longer be closed when the ClosableHandle is dropped.
    pub fn take(&mut self) -> HANDLE {
        replace(&mut self.0, HANDLE(0))
    }
}
impl Drop for ClosableHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { CloseHandle(self.0) };
            self.0 = HANDLE(0);
        }
    }
}


fn main() {
    let opts = Opts::parse();

    let serial_port_ws = to_null_term_wide_string(&opts.serial_port);

    let file_handle_raw = unsafe {
        CreateFileW(
            PCWSTR(serial_port_ws.as_ptr()),
            FILE_ACCESS_FLAGS(GENERIC_READ | GENERIC_WRITE),
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES::default(),
            None,
        )
    }
        .expect("failed to open COM port");
    let file_handle = ClosableHandle(file_handle_raw);

    let mut dcb = DCB::default();
    dcb.DCBlength = size_of_val(&dcb).try_into().unwrap();

    let state_got = unsafe { GetCommState(file_handle.0, &mut dcb) }
        .as_bool();
    if !state_got {
        let port_state_error = io::Error::last_os_error();
        drop(file_handle);
        panic!("failed to obtain port state: {}", port_state_error);
    }

    let mut bitfield = DcbBitfield {
        field: dcb._bitfield.to_ne_bytes(),
    };

    if opts.disable_dtr {
        bitfield.set_dtr_control(0);
        dcb._bitfield = u32::from_ne_bytes(bitfield.field);

        let state_set = unsafe { SetCommState(file_handle.0, &dcb) }
            .as_bool();
        if !state_set {
            let port_state_error = io::Error::last_os_error();
            drop(file_handle);
            panic!("failed to set port state: {}", port_state_error);
        }
    } else {
        // print current state
        println!("        baud rate: {:10}", dcb.BaudRate);
        println!("           binary: {}", bitfield.binary());
        println!("           parity: {}", bitfield.parity());
        println!("    outx CTS flow: {}", bitfield.outx_cts_flow());
        println!("    outx DSR flow: {}", bitfield.outx_dsr_flow());
        println!("      DTR control: {}", bitfield.dtr_control());
        println!("  DSR sensitivity: {}", bitfield.dsr_sensitivity());
        println!("      continue Tx: {}", bitfield.tx_continue_on_xoff());
        println!("          on XOFF");
        println!("outgoing XON/XOFF: {}", bitfield.out_x());
        println!("incoming XON/XOFF: {}", bitfield.in_x());
        println!("       error char: {}", bitfield.error_char());
        println!("             null: {}", bitfield.null());
        println!("      RTS control: {}", bitfield.rts_control());
        println!("   abort on error: {}", bitfield.abort_on_error());
        println!("        XON limit: {:5}", dcb.XonLim);
        println!("       XOFF limit: {:5}", dcb.XoffLim);
        println!("        byte size: {:3}", dcb.ByteSize);
        println!("           parity: {:?}", dcb.Parity);
        println!("        stop bits: {:?}", dcb.StopBits);
        println!("         XON char: 0x{:02X}", dcb.XonChar.0);
        println!("        XOFF char: 0x{:02X}", dcb.XoffChar.0);
        println!("       error char: 0x{:02X}", dcb.ErrorChar.0);
        println!("         EOF char: 0x{:02X}", dcb.EofChar.0);
        println!("       event char: 0x{:02X}", dcb.EvtChar.0);
    }

    drop(file_handle);
}
