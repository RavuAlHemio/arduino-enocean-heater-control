const CRC8_CCITT_LUT: [u8; 256] = make_crc8_ccitt_lut();


const fn make_crc8_ccitt_lut() -> [u8; 256] {
    let mut ret = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        ret[i] = crc8_ccitt_byte(i as u8);
        i += 1;
    }
    ret
}

const fn crc8_ccitt_byte(b: u8) -> u8 {
    // (x^8) + x^2 + x + 1 =~= (1) 0000 0111

    let mut crc: u8 = b;
    let mut i = 8;
    while i > 0 {
        // check the top bit
        if crc & 0b1000_0000 != 0 {
            crc = (crc << 1) ^ 0b0000_0111;
        } else {
            crc = crc << 1;
        }
        i -= 1;
    }
    crc
}

pub fn crc8_ccitt(bytes: &[u8]) -> u8 {
    let mut crc: u8 = 0x00;
    for b in bytes {
        crc = CRC8_CCITT_LUT[usize::from(crc ^ *b)];
    }
    crc
}
