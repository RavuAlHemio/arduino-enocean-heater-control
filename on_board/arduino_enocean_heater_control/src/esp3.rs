//! An implementation of the EnOcean Serial Protocol 3 (ESP3).


use buildingblocks::crc8::crc8_ccitt;
use buildingblocks::max_array::MaxArray;

use crate::ring_buffer::CriticalRingBuffer;


/// The (constant) length of an ESP3 packet header.
///
/// The header consists of:
/// 1. The sync byte (always 0x55, 1 byte long)
/// 2. The data length (maximum 0xFFFF, 2 bytes long)
/// 3. The optional data length (maximum 0xFF, 1 byte long)
/// 4. The packet type (1 byte long)
/// 5. The CRC8 checksum of the header (1 byte long)
const HEADER_LENGTH: usize = 1 + 2 + 1 + 1 + 1;


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
const MAX_ESP3_PACKET_LENGTH: usize = HEADER_LENGTH + 0xFFFF + 0xFF + 1;


/// The byte used for synchronization.
const SYNC_BYTE: u8 = 0x55;


/// The ring buffer used for decoding incoming ESP3 packets.
static ESP3_BUFFER: CriticalRingBuffer<u8, MAX_ESP3_PACKET_LENGTH> = CriticalRingBuffer::new();


/// Takes bytes from the ESP3 ring buffer until a valid ESP3 packet is decoded, then returns its
/// bytes.
pub fn take_esp3_packet() -> Option<MaxArray<u8, MAX_ESP3_PACKET_LENGTH>> {
    // loop to get an actual packet
    let (data_length, opt_length, total_length) = loop {
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
        let mut possible_header = [0u8; 6];
        ESP3_BUFFER.peek_fill(&mut possible_header);

        // does the CRC8 match?
        let calculated_crc = crc8_ccitt(&possible_header[0..5]);
        if calculated_crc == possible_header[5] {
            // yes -- it's a packet!

            // have we already collected all of it?
            let data_length = u16::from_be_bytes(possible_header[1..3].try_into().unwrap());
            let opt_length = possible_header[3];
            // header, data, opt data, data CRC8
            let total_length = HEADER_LENGTH + usize::from(data_length) + usize::from(opt_length) + 1;
            if ESP3_BUFFER.len() < total_length {
                // nope, we still need a few more bytes
                return None;
            }

            // alright, keep processing it!
            break (data_length, opt_length, total_length);
        }

        // no, it isn't a valid packet... pop the sync byte and search for the next one
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
