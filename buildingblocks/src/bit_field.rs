//! Provides efficient storage of boolean values as bit flags.


/// A fixed-size, efficient storage of boolean values as bit flags.
///
/// To simplify runtime computation, the bits are packed into bytes such that the boolean value with
/// the lowest index is stored in the LSB, and the bytes are in ascending order. This leads to the
/// following order:
///
/// ```text
/// | field[0]        | field[1]        |
/// | v--MSB   LSB--v | v--MSB   LSB--v |
/// | 7 6 5 4 3 2 1 0 | F E D C B A 9 8 |
/// ```
///
/// The bit field only stores multiples of 8 bits. If storage of a "lopsided" number of bits is
/// required, the actual length of the bit field must be stored externally.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BitField<const SIZE_BYTES: usize> {
    field: [u8; SIZE_BYTES],
}
impl<const SIZE_BYTES: usize> BitField<SIZE_BYTES> {
    /// Returns the byte and bit index within the bit field for the given boolean value index.
    #[inline]
    const fn byte_bit_index(index: usize) -> (usize, usize) {
        (index / 8, index % 8)
    }

    /// Wraps the given byte array as a bit field.
    pub const fn from_bytes(bytes: [u8; SIZE_BYTES]) -> Self {
        Self {
            field: bytes,
        }
    }

    /// Sets the bit in the bit field, i.e. sets it to 1.
    #[inline]
    pub fn set_bit(&mut self, index: usize) {
        let (byte_index, bit_index) = Self::byte_bit_index(index);
        self.field[byte_index] |= 1 << bit_index;
    }

    /// Clears the bit in the bit field, i.e. sets it to 0.
    #[inline]
    pub fn clear_bit(&mut self, index: usize) {
        let (byte_index, bit_index) = Self::byte_bit_index(index);
        self.field[byte_index] &= (1 << bit_index) ^ 0b1111_1111;
    }

    /// Returns whether the given bit is set, i.e. equals 1.
    #[inline]
    pub fn is_bit_set(&self, index: usize) -> bool {
        let (byte_index, bit_index) = Self::byte_bit_index(index);
        self.field[byte_index] & (1 << bit_index) != 0
    }

    /// The size of this bit field in bytes.
    ///
    /// [`len_bits`] can be called to obtain the length in bits.
    #[inline]
    pub fn size_bytes(&self) -> usize { SIZE_BYTES }

    /// The length of this bit field in bits.
    ///
    /// [`size_bytes`] can be called to obtain the size in bytes.
    #[inline]
    pub fn len_bits(&self) -> usize { SIZE_BYTES * 8 }

    /// Fills this bit field with the values of the given bytes.
    ///
    /// If `bytes` is shorter than this bit field, the remaining bits are left untouched. If `bytes`
    /// is longer, the extraneous bytes are ignored.
    pub fn fill_with_bytes(&mut self, bytes: &[u8]) {
        let size = SIZE_BYTES.min(bytes.len());
        for i in 0..size {
            self.field[i] = bytes[i];
        }
    }

    /// Fills this bit field with the values of the given bits.
    ///
    /// If `bits` is shorter than this bit field, the remaining bits are left untouched. If `bits`
    /// is longer, the extraneous bits are ignored.
    pub fn fill_with_bits(&mut self, bits: &[bool]) {
        let size = (SIZE_BYTES * 8).min(bits.len());
        for i in 0..size {
            if bits[i] {
                self.set_bit(i);
            } else {
                self.clear_bit(i);
            }
        }
    }

    /// Return a reference the array of bytes backing this bit field.
    pub fn as_bytes(&self) -> &[u8] {
        &self.field
    }

    /// Return an iterator iterating over each bit of the bit field.
    pub fn bit_iter(&self) -> BitFieldIterator<SIZE_BYTES> {
        BitFieldIterator {
            field: &self,
            next_bit: 0,
        }
    }
}
impl<const SIZE_BYTES: usize> Default for BitField<SIZE_BYTES> {
    fn default() -> Self {
        Self {
            field: [0; SIZE_BYTES],
        }
    }
}


/// Iterates over each bit in a bit field.
pub struct BitFieldIterator<'a, const SIZE_BYTES: usize> {
    field: &'a BitField<SIZE_BYTES>,
    next_bit: usize,
}
impl<'a, const SIZE_BYTES: usize> Iterator for BitFieldIterator<'a, SIZE_BYTES> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_bit >= self.field.len_bits() {
            None
        } else {
            let ret = self.field.is_bit_set(self.next_bit);
            self.next_bit += 1;
            Some(ret)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::BitField;

    #[test]
    fn test_empty_bit_field() {
        let mut empty = BitField::from_bytes([]);
        assert_eq!(empty.len_bits(), 0);
        assert_eq!(empty.size_bytes(), 0);
        assert_eq!(empty.as_bytes(), &[]);

        // filling beyond the length does nothing
        empty.fill_with_bits(&[true, true, true]);
        assert_eq!(empty.len_bits(), 0);
        assert_eq!(empty.size_bytes(), 0);
        assert_eq!(empty.as_bytes(), &[]);

        empty.fill_with_bytes(&[0xFF, 0xFF, 0xFF]);
        assert_eq!(empty.len_bits(), 0);
        assert_eq!(empty.size_bytes(), 0);
        assert_eq!(empty.as_bytes(), &[]);
    }

    #[test]
    fn test_single_byte_bit_field() {
        let mut one_byte = BitField::from_bytes([0x00]);
        assert_eq!(one_byte.len_bits(), 8);
        assert_eq!(one_byte.size_bytes(), 1);
        assert_eq!(one_byte.as_bytes(), &[0x00]);

        one_byte.fill_with_bits(&[true, true, true]);
        assert_eq!(one_byte.len_bits(), 8);
        assert_eq!(one_byte.size_bytes(), 1);
        assert_eq!(one_byte.as_bytes(), &[0x07]);
        assert_eq!(one_byte.is_bit_set(0), true);
        assert_eq!(one_byte.is_bit_set(1), true);
        assert_eq!(one_byte.is_bit_set(2), true);
        assert_eq!(one_byte.is_bit_set(3), false);
        assert_eq!(one_byte.is_bit_set(4), false);
        assert_eq!(one_byte.is_bit_set(5), false);
        assert_eq!(one_byte.is_bit_set(6), false);
        assert_eq!(one_byte.is_bit_set(7), false);
        let one_byte_vec: Vec<bool> = one_byte.bit_iter().collect();
        assert_eq!(
            &one_byte_vec,
            &[true, true, true, false, false, false, false, false],
        );

        one_byte.fill_with_bytes(&[0xFF, 0xFF, 0xFF]);
        assert_eq!(one_byte.len_bits(), 8);
        assert_eq!(one_byte.size_bytes(), 1);
        assert_eq!(one_byte.as_bytes(), &[0xFF]);
        assert_eq!(one_byte.is_bit_set(0), true);
        assert_eq!(one_byte.is_bit_set(1), true);
        assert_eq!(one_byte.is_bit_set(2), true);
        assert_eq!(one_byte.is_bit_set(3), true);
        assert_eq!(one_byte.is_bit_set(4), true);
        assert_eq!(one_byte.is_bit_set(5), true);
        assert_eq!(one_byte.is_bit_set(6), true);
        assert_eq!(one_byte.is_bit_set(7), true);
        let one_byte_vec: Vec<bool> = one_byte.bit_iter().collect();
        assert_eq!(
            &one_byte_vec,
            &[true, true, true, true, true, true, true, true],
        );

        one_byte.clear_bit(0);
        assert_eq!(one_byte.len_bits(), 8);
        assert_eq!(one_byte.size_bytes(), 1);
        assert_eq!(one_byte.as_bytes(), &[0xFE]);
        assert_eq!(one_byte.is_bit_set(0), false);
        assert_eq!(one_byte.is_bit_set(1), true);
        assert_eq!(one_byte.is_bit_set(2), true);
        assert_eq!(one_byte.is_bit_set(3), true);
        assert_eq!(one_byte.is_bit_set(4), true);
        assert_eq!(one_byte.is_bit_set(5), true);
        assert_eq!(one_byte.is_bit_set(6), true);
        assert_eq!(one_byte.is_bit_set(7), true);
        let one_byte_vec: Vec<bool> = one_byte.bit_iter().collect();
        assert_eq!(
            &one_byte_vec,
            &[false, true, true, true, true, true, true, true],
        );

        one_byte.clear_bit(5);
        assert_eq!(one_byte.len_bits(), 8);
        assert_eq!(one_byte.size_bytes(), 1);
        assert_eq!(one_byte.as_bytes(), &[0xDE]);
        assert_eq!(one_byte.is_bit_set(0), false);
        assert_eq!(one_byte.is_bit_set(1), true);
        assert_eq!(one_byte.is_bit_set(2), true);
        assert_eq!(one_byte.is_bit_set(3), true);
        assert_eq!(one_byte.is_bit_set(4), true);
        assert_eq!(one_byte.is_bit_set(5), false);
        assert_eq!(one_byte.is_bit_set(6), true);
        assert_eq!(one_byte.is_bit_set(7), true);
        let one_byte_vec: Vec<bool> = one_byte.bit_iter().collect();
        assert_eq!(
            &one_byte_vec,
            &[false, true, true, true, true, false, true, true],
        );
    }

    #[test]
    fn test_two_byte_bit_field() {
        let mut two_bytes = BitField::from_bytes([0x00, 0x00]);
        assert_eq!(two_bytes.len_bits(), 16);
        assert_eq!(two_bytes.size_bytes(), 2);
        assert_eq!(two_bytes.as_bytes(), &[0x00, 0x00]);
        assert_eq!(two_bytes.is_bit_set(0), false);
        assert_eq!(two_bytes.is_bit_set(1), false);
        assert_eq!(two_bytes.is_bit_set(2), false);
        assert_eq!(two_bytes.is_bit_set(3), false);
        assert_eq!(two_bytes.is_bit_set(4), false);
        assert_eq!(two_bytes.is_bit_set(5), false);
        assert_eq!(two_bytes.is_bit_set(6), false);
        assert_eq!(two_bytes.is_bit_set(7), false);
        assert_eq!(two_bytes.is_bit_set(8), false);
        assert_eq!(two_bytes.is_bit_set(9), false);
        assert_eq!(two_bytes.is_bit_set(10), false);
        assert_eq!(two_bytes.is_bit_set(11), false);
        assert_eq!(two_bytes.is_bit_set(12), false);
        assert_eq!(two_bytes.is_bit_set(13), false);
        assert_eq!(two_bytes.is_bit_set(14), false);
        assert_eq!(two_bytes.is_bit_set(15), false);
        let two_byte_vec: Vec<bool> = two_bytes.bit_iter().collect();
        assert_eq!(
            &two_byte_vec,
            &[
                false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false,
            ],
        );

        two_bytes.fill_with_bits(&[true, true, true]);
        assert_eq!(two_bytes.len_bits(), 16);
        assert_eq!(two_bytes.size_bytes(), 2);
        assert_eq!(two_bytes.as_bytes(), &[0x07, 0x00]);
        assert_eq!(two_bytes.is_bit_set(0), true);
        assert_eq!(two_bytes.is_bit_set(1), true);
        assert_eq!(two_bytes.is_bit_set(2), true);
        assert_eq!(two_bytes.is_bit_set(3), false);
        assert_eq!(two_bytes.is_bit_set(4), false);
        assert_eq!(two_bytes.is_bit_set(5), false);
        assert_eq!(two_bytes.is_bit_set(6), false);
        assert_eq!(two_bytes.is_bit_set(7), false);
        assert_eq!(two_bytes.is_bit_set(8), false);
        assert_eq!(two_bytes.is_bit_set(9), false);
        assert_eq!(two_bytes.is_bit_set(10), false);
        assert_eq!(two_bytes.is_bit_set(11), false);
        assert_eq!(two_bytes.is_bit_set(12), false);
        assert_eq!(two_bytes.is_bit_set(13), false);
        assert_eq!(two_bytes.is_bit_set(14), false);
        assert_eq!(two_bytes.is_bit_set(15), false);
        let two_byte_vec: Vec<bool> = two_bytes.bit_iter().collect();
        assert_eq!(
            &two_byte_vec,
            &[
                true, true, true, false, false, false, false, false,
                false, false, false, false, false, false, false, false,
            ],
        );

        two_bytes.fill_with_bytes(&[0xFF, 0xFF, 0xFF]);
        assert_eq!(two_bytes.len_bits(), 16);
        assert_eq!(two_bytes.size_bytes(), 2);
        assert_eq!(two_bytes.as_bytes(), &[0xFF, 0xFF]);
        assert_eq!(two_bytes.is_bit_set(0), true);
        assert_eq!(two_bytes.is_bit_set(1), true);
        assert_eq!(two_bytes.is_bit_set(2), true);
        assert_eq!(two_bytes.is_bit_set(3), true);
        assert_eq!(two_bytes.is_bit_set(4), true);
        assert_eq!(two_bytes.is_bit_set(5), true);
        assert_eq!(two_bytes.is_bit_set(6), true);
        assert_eq!(two_bytes.is_bit_set(7), true);
        assert_eq!(two_bytes.is_bit_set(8), true);
        assert_eq!(two_bytes.is_bit_set(9), true);
        assert_eq!(two_bytes.is_bit_set(10), true);
        assert_eq!(two_bytes.is_bit_set(11), true);
        assert_eq!(two_bytes.is_bit_set(12), true);
        assert_eq!(two_bytes.is_bit_set(13), true);
        assert_eq!(two_bytes.is_bit_set(14), true);
        assert_eq!(two_bytes.is_bit_set(15), true);
        let two_byte_vec: Vec<bool> = two_bytes.bit_iter().collect();
        assert_eq!(
            &two_byte_vec,
            &[
                true, true, true, true, true, true, true, true,
                true, true, true, true, true, true, true, true,
            ],
        );

        two_bytes.clear_bit(0);
        assert_eq!(two_bytes.len_bits(), 16);
        assert_eq!(two_bytes.size_bytes(), 2);
        assert_eq!(two_bytes.as_bytes(), &[0xFE, 0xFF]);
        assert_eq!(two_bytes.is_bit_set(0), false);
        assert_eq!(two_bytes.is_bit_set(1), true);
        assert_eq!(two_bytes.is_bit_set(2), true);
        assert_eq!(two_bytes.is_bit_set(3), true);
        assert_eq!(two_bytes.is_bit_set(4), true);
        assert_eq!(two_bytes.is_bit_set(5), true);
        assert_eq!(two_bytes.is_bit_set(6), true);
        assert_eq!(two_bytes.is_bit_set(7), true);
        assert_eq!(two_bytes.is_bit_set(8), true);
        assert_eq!(two_bytes.is_bit_set(9), true);
        assert_eq!(two_bytes.is_bit_set(10), true);
        assert_eq!(two_bytes.is_bit_set(11), true);
        assert_eq!(two_bytes.is_bit_set(12), true);
        assert_eq!(two_bytes.is_bit_set(13), true);
        assert_eq!(two_bytes.is_bit_set(14), true);
        assert_eq!(two_bytes.is_bit_set(15), true);
        let two_byte_vec: Vec<bool> = two_bytes.bit_iter().collect();
        assert_eq!(
            &two_byte_vec,
            &[
                false, true, true, true, true, true, true, true,
                true, true, true, true, true, true, true, true,
            ],
        );

        two_bytes.clear_bit(5);
        assert_eq!(two_bytes.len_bits(), 16);
        assert_eq!(two_bytes.size_bytes(), 2);
        assert_eq!(two_bytes.as_bytes(), &[0xDE, 0xFF]);
        assert_eq!(two_bytes.is_bit_set(0), false);
        assert_eq!(two_bytes.is_bit_set(1), true);
        assert_eq!(two_bytes.is_bit_set(2), true);
        assert_eq!(two_bytes.is_bit_set(3), true);
        assert_eq!(two_bytes.is_bit_set(4), true);
        assert_eq!(two_bytes.is_bit_set(5), false);
        assert_eq!(two_bytes.is_bit_set(6), true);
        assert_eq!(two_bytes.is_bit_set(7), true);
        assert_eq!(two_bytes.is_bit_set(8), true);
        assert_eq!(two_bytes.is_bit_set(9), true);
        assert_eq!(two_bytes.is_bit_set(10), true);
        assert_eq!(two_bytes.is_bit_set(11), true);
        assert_eq!(two_bytes.is_bit_set(12), true);
        assert_eq!(two_bytes.is_bit_set(13), true);
        assert_eq!(two_bytes.is_bit_set(14), true);
        assert_eq!(two_bytes.is_bit_set(15), true);
        let two_byte_vec: Vec<bool> = two_bytes.bit_iter().collect();
        assert_eq!(
            &two_byte_vec,
            &[
                false, true, true, true, true, false, true, true,
                true, true, true, true, true, true, true, true,
            ],
        );

        two_bytes.clear_bit(14);
        assert_eq!(two_bytes.len_bits(), 16);
        assert_eq!(two_bytes.size_bytes(), 2);
        assert_eq!(two_bytes.as_bytes(), &[0xDE, 0xBF]);
        assert_eq!(two_bytes.is_bit_set(0), false);
        assert_eq!(two_bytes.is_bit_set(1), true);
        assert_eq!(two_bytes.is_bit_set(2), true);
        assert_eq!(two_bytes.is_bit_set(3), true);
        assert_eq!(two_bytes.is_bit_set(4), true);
        assert_eq!(two_bytes.is_bit_set(5), false);
        assert_eq!(two_bytes.is_bit_set(6), true);
        assert_eq!(two_bytes.is_bit_set(7), true);
        assert_eq!(two_bytes.is_bit_set(8), true);
        assert_eq!(two_bytes.is_bit_set(9), true);
        assert_eq!(two_bytes.is_bit_set(10), true);
        assert_eq!(two_bytes.is_bit_set(11), true);
        assert_eq!(two_bytes.is_bit_set(12), true);
        assert_eq!(two_bytes.is_bit_set(13), true);
        assert_eq!(two_bytes.is_bit_set(14), false);
        assert_eq!(two_bytes.is_bit_set(15), true);
        let two_byte_vec: Vec<bool> = two_bytes.bit_iter().collect();
        assert_eq!(
            &two_byte_vec,
            &[
                false, true, true, true, true, false, true, true,
                true, true, true, true, true, true, false, true,
            ],
        );
    }
}
