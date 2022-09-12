use std::fmt;


/// An efficient storage of boolean values as bit flags.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct DynamicBitField {
    field: Vec<u8>,
    size_bits: usize,
}
impl DynamicBitField {
    pub fn new() -> Self {
        Self {
            field: Vec::new(),
            size_bits: 0,
        }
    }

    pub fn with_capacity_bytes(reserve_bytes: usize) -> Self {
        Self {
            field: vec![0u8; reserve_bytes],
            size_bits: 0,
        }
    }

    #[inline]
    const fn byte_bit_index(index: usize) -> (usize, usize) {
        (index / 8, index % 8)
    }

    /// Sets the bit in the bit field, i.e. sets it to 1.
    pub fn set_bit(&mut self, index: usize) {
        let (byte_index, bit_index) = Self::byte_bit_index(index);
        while byte_index >= self.field.len() {
            self.field.push(0x00);
        }
        self.field[byte_index] |= 1 << bit_index;
        if self.size_bits <= index {
            self.size_bits = index + 1;
        }
    }

    /// Clears the bit in the bit field, i.e. sets it to 0.
    #[inline]
    pub fn clear_bit(&mut self, index: usize) {
        let (byte_index, bit_index) = Self::byte_bit_index(index);
        while byte_index >= self.field.len() {
            self.field.push(0x00);
        }
        self.field[byte_index] &= (1 << bit_index) ^ 0b1111_1111;
        if self.size_bits <= index {
            self.size_bits = index + 1;
        }
    }

    /// Returns whether the given bit is set, i.e. equals 1.
    #[inline]
    pub fn is_bit_set(&self, index: usize) -> bool {
        if index >= self.size_bits {
            return false;
        }
        let (byte_index, bit_index) = Self::byte_bit_index(index);
        self.field[byte_index] & (1 << bit_index) != 0
    }

    /// The size of this bit field in bytes.
    ///
    /// [`len_bits`] can be called to obtain the length in bits.
    #[inline]
    pub fn size_bytes(&self) -> usize { self.field.len() }

    /// The length of this bit field in bits.
    ///
    /// [`size_bytes`] can be called to obtain the size in bytes.
    #[inline]
    pub fn len_bits(&self) -> usize { self.size_bits }

    /// Fills this bit field with the values of the given bytes.
    ///
    /// If `bytes` is shorter than this bit field, the remaining bits are left untouched. If `bytes`
    /// is longer, the extraneous bytes are ignored.
    pub fn fill_with_bytes(&mut self, bytes: &[u8]) {
        let size = self.size_bytes().min(bytes.len());
        for i in 0..size {
            self.field[i] = bytes[i];
        }
    }

    /// Fills this bit field with the values of the given bits.
    ///
    /// If `bits` is shorter than this bit field, the remaining bits are left untouched. If `bits`
    /// is longer, the extraneous bits are ignored.
    pub fn fill_with_bits(&mut self, bits: &[bool]) {
        let size = self.len_bits().min(bits.len());
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

    /// Appends a single bit to this bit field.
    pub fn push(&mut self, new_bit: bool) {
        if new_bit {
            self.set_bit(self.size_bits)
        } else {
            self.clear_bit(self.size_bits)
        }
    }

    pub fn iter(&self) -> DynamicBitFieldIterator {
        DynamicBitFieldIterator {
            field: self,
            next_bit: 0,
        }
    }
}
impl fmt::Display for DynamicBitField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.iter() {
            if b {
                write!(f, "1")?;
            } else {
                write!(f, "0")?;
            }
        }
        Ok(())
    }
}


/// A bit field iterator.
pub(crate) struct DynamicBitFieldIterator<'a> {
    field: &'a DynamicBitField,
    next_bit: usize,
}
impl<'a> Iterator for DynamicBitFieldIterator<'a> {
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
