//! Functions to calculate 8-bit cyclic redundancy checks (CRCs).


/// A precomputed look-up table for values of CRC8-CCITT, the 8-bit CCITT (ITU-T I.432.1) CRC
/// variant.
const CRC8_CCITT_LUT: [u8; 256] = make_crc8_ccitt_lut();


/// Precomputes a CRC8-CCITT look-up table.
const fn make_crc8_ccitt_lut() -> [u8; 256] {
    let mut ret = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        ret[i] = crc8_ccitt_byte(i as u8);
        i += 1;
    }
    ret
}

/// Calculates the CRC8-CCITT checksum for a single byte.
const fn crc8_ccitt_byte(b: u8) -> u8 {
    let mut crc: u8 = b;
    let mut i = 8;
    while i > 0 {
        // check the top bit
        if crc & 0b1000_0000 != 0 {
            // (x^8) + x^2 + x + 1 =~= (1) 0000 0111
            crc = (crc << 1) ^ 0b0000_0111;
        } else {
            crc = crc << 1;
        }
        i -= 1;
    }
    crc
}

/// Calculates the CRC-CCITT checksum for a slice of bytes.
///
/// Uses the lookup table to speed up calculations.
pub fn crc8_ccitt(bytes: &[u8]) -> u8 {
    let mut crc: u8 = 0x00;
    for b in bytes {
        crc = CRC8_CCITT_LUT[usize::from(crc ^ *b)];
    }
    crc
}


#[cfg(test)]
mod tests {
    use super::crc8_ccitt;

    #[test]
    fn test_crc8_ccitt() {
        assert_eq!(crc8_ccitt(&[]), 0);
        assert_eq!(crc8_ccitt(&[0x00]), 0);
        assert_eq!(crc8_ccitt(&[0x00, 0x00]), 0);

        assert_eq!(crc8_ccitt(b"hello, world\n"), 0x93); // spelling/capitalization lifted from K&R
        assert_eq!(crc8_ccitt(b"All human beings are born free and equal in dignity and rights."), 0x3e);

        assert_eq!(crc8_ccitt(&[0x00, 0x02, 0x01, 0x04]), 0xDF);
        assert_eq!(crc8_ccitt(&[0x04, 0x01, 0x00]), 0xBE);
    }
}
