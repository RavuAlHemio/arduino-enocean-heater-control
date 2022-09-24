use std::env;

use buildingblocks::esp3::Esp3Packet;


fn unhex_nibble(b: u8) -> Option<u8> {
    if b >= b'0' && b <= b'9' {
        Some(b - b'0')
    } else if b >= b'A' && b <= b'F' {
        Some(b + 10 - b'A')
    } else if b >= b'a' && b <= b'f' {
        Some(b + 10 - b'a')
    } else {
        None
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: deesp3 PACKETHEXDUMP");
        return;
    }

    let hex_packet: Vec<u8> = args[1].bytes().collect();
    if hex_packet.len() % 2 != 0 {
        eprintln!("packet hexdump contains half a byte");
        return;
    }

    let mut buf = Vec::with_capacity(hex_packet.len()/2);
    for i in 0..hex_packet.len()/2 {
        let top_nibble = unhex_nibble(hex_packet[2*i])
            .expect("invalid hex digit");
        let bottom_nibble = unhex_nibble(hex_packet[2*i+1])
            .expect("invalid hex digit");
        let b = (top_nibble << 4) | bottom_nibble;
        buf.push(b);
    }

    // attempt to decode this
    let pkt = Esp3Packet::from_slice(&buf);
    println!("{:#?}", pkt);
}
