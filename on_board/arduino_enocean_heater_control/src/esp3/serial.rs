//! Code related to EnOcean serial I/O.


use buildingblocks::crc8::crc8_ccitt;
use buildingblocks::max_array::MaxArray;

use crate::esp3::{FOOTER_LENGTH, HEADER_LENGTH, MAX_ESP3_PACKET_LENGTH, SYNC_BYTE};
use crate::ring_buffer::CriticalRingBuffer;


/// The ring buffer used for decoding incoming ESP3 packets.
static ESP3_BUFFER: CriticalRingBuffer<u8, MAX_ESP3_PACKET_LENGTH> = CriticalRingBuffer::new();


/// Pushes a byte into the ESP3 ring buffer.
pub fn push_to_buffer(b: u8) {
    ESP3_BUFFER.push(b);
}


/// Takes bytes from the ESP3 ring buffer until a valid ESP3 packet is decoded, then returns its
/// bytes.
pub fn take_esp3_packet() -> Option<MaxArray<u8, MAX_ESP3_PACKET_LENGTH>> {
    // loop to get an actual packet
    let total_length = loop {
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
        let calculated_crc = crc8_ccitt(&possible_header[1..HEADER_LENGTH-1]);
        if calculated_crc == possible_header[HEADER_LENGTH-1] {
            // yes -- it's a packet!

            // have we already collected all of it?
            let data_length = u16::from_be_bytes(possible_header[1..3].try_into().unwrap());
            let opt_length = possible_header[3];
            // header, data, opt data, data CRC8
            let total_length = HEADER_LENGTH + usize::from(data_length) + usize::from(opt_length) + FOOTER_LENGTH;
            if ESP3_BUFFER.max_size() < total_length {
                // our buffer isn't large enough for this huge packet anyway
                // pop the sync byte and search for the next one
                ESP3_BUFFER.pop();
                continue;
            }
            if ESP3_BUFFER.len() < total_length {
                // nope, we still need a few more bytes
                return None;
            }

            // alright, keep processing it!
            break total_length;
        }

        // no, it isn't a valid packet...
        crate::uart::send_stolen(b"invalid packet :-(\r\n");
        // pop the sync byte and search for the next one
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
