/// An efficient storage of boolean values as bit flags.
pub(crate) struct BitField<const SIZE_BYTES: usize> {
    field: [u8; SIZE_BYTES],
}
impl<const SIZE_BYTES: usize> BitField<SIZE_BYTES> {
    #[inline]
    const fn byte_bit_index(index: usize) -> (usize, usize) {
        (index / 8, index % 8)
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
    pub fn size_bytes(&self) -> usize { SIZE_BYTES }

    /// The length of this bit field in bits.
    ///
    /// [`size_bytes`] can be called to obtain the size in bytes.
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
}


/// A bit field iterator.
pub(crate) struct BitFieldIterator<const SIZE_BYTES: usize> {
    field: BitField<SIZE_BYTES>,
    next_bit: usize,
}
impl<const SIZE_BYTES: usize> Iterator for BitFieldIterator<SIZE_BYTES> {
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
