use buildingblocks::esp3::Esp3Packet;
use buildingblocks::esp3::eep;
use buildingblocks::esp3::erp::ErpData;
use clap::Parser;


#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    pub eep: Option<String>,

    pub packet_hex_dump: String,
}


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

fn unhex_byte(b: &[u8]) -> u8 {
    let top_nibble = unhex_nibble(b[0])
        .expect("invalid hex digit");
    let bottom_nibble = unhex_nibble(b[1])
        .expect("invalid hex digit");
    (top_nibble << 4) | bottom_nibble
}


fn main() {
    let args = Args::parse();

    let rorg_func_type_opt: Option<(u8, u8, u8)> = if let Some(eep) = &args.eep {
        if eep.len() != 8 {
            panic!("EEP {:?} must be in the format \"AA-BB-CC\"", eep);
        }
        let eep_bytes: Vec<u8> = eep.bytes().collect();
        if eep_bytes[2] != b'-' || eep_bytes[5] != b'-' {
            panic!("EEP {:?} must be in the format \"AA-BB-CC\"", eep);
        }
        let rorg = unhex_byte(&eep_bytes[0..2]);
        let func = unhex_byte(&eep_bytes[3..5]);
        let tp = unhex_byte(&eep_bytes[6..8]);
        Some((rorg, func, tp))
    } else {
        None
    };

    let hex_packet: Vec<u8> = args.packet_hex_dump.bytes().collect();
    if hex_packet.len() % 2 != 0 {
        eprintln!("packet hexdump contains half a byte");
        return;
    }

    let mut buf = Vec::with_capacity(hex_packet.len()/2);
    for i in 0..hex_packet.len()/2 {
        buf.push(unhex_byte(&hex_packet[2*i..2*i+2]));
    }

    // attempt to decode this
    let pkt_opt = Esp3Packet::from_slice(&buf);
    println!("{:#?}", pkt_opt);

    if let Some(pkt) = pkt_opt {
        if let Esp3Packet::RadioErp1 { radio_telegram, .. } = pkt {
            // attempt to decode this message
            let message_opt = ErpData::from_slice(radio_telegram.as_slice());
            match message_opt {
                Some(msg) => {
                    println!("decoded radio message: {:#?}", msg);

                    if let Some((rorg, func, tp)) = rorg_func_type_opt {
                        // decode further, using EEP
                        match msg {
                            ErpData::RepeatedSwitch(rst) => {
                                let reversed_bytes = [rst.data];
                                let decoded_opt = eep::Eep::from_reversed_bytes(rorg, func, tp, &reversed_bytes);
                                if let Some(decoded) = decoded_opt {
                                    println!("decoded EEP: {:#?}", decoded);
                                } else {
                                    println!("failed to decode EEP");
                                }
                            },
                            ErpData::OneByte(ob) => {
                                let reversed_bytes = [ob.data];
                                let decoded_opt = eep::Eep::from_reversed_bytes(rorg, func, tp, &reversed_bytes);
                                if let Some(decoded) = decoded_opt {
                                    println!("decoded EEP: {:#?}", decoded);
                                } else {
                                    println!("failed to decode EEP");
                                }
                            },
                            ErpData::FourByte(fb) => {
                                let reversed_bytes: [u8; 4] = fb.data.to_le_bytes();
                                let decoded_opt = eep::Eep::from_reversed_bytes(rorg, func, tp, &reversed_bytes);
                                if let Some(decoded) = decoded_opt {
                                    println!("decoded EEP: {:#?}", decoded);
                                } else {
                                    println!("failed to decode EEP");
                                }
                            },
                            ErpData::VariableLength(fb) => {
                                let mut reversed_bytes: Vec<u8> = Vec::with_capacity(fb.data.len());
                                reversed_bytes.extend(fb.data.iter().map(|b| *b));
                                let decoded_opt = eep::Eep::from_reversed_bytes(rorg, func, tp, &reversed_bytes);
                                if let Some(decoded) = decoded_opt {
                                    println!("decoded EEP: {:#?}", decoded);
                                } else {
                                    println!("failed to decode EEP");
                                }
                            },
                            _ => {},
                        }
                    }
                },
                None => println!("failed to decode radio message"),
            }
        }
    }
}
