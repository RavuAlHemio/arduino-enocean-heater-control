//! Extensions to the [MaxArray] type.


use crate::max_array::MaxArray;


macro_rules! implement_signed {
    ($signed_name:ident, $signed_type:ty, $unsigned_name:ident, $unsigned_type:ty, $doc_string:expr) => {
        #[doc = $doc_string]
        #[inline]
        fn $signed_name(&mut self, value: $signed_type) -> Result<(), $signed_type> {
            self.$unsigned_name(value as $unsigned_type).map_err(|e| e as $signed_type)
        }
    }
}
macro_rules! implement_push {
    ($be_name:ident, $le_name:ident, $tp:ty) => {
        #[inline]
        fn $be_name(&mut self, value: $tp) -> Result<(), $tp> {
            if self.can_fit(::core::mem::size_of::<$tp>()) {
                for b in value.to_be_bytes() {
                    self.push(b).unwrap();
                }
                Ok(())
            } else {
                Err(value)
            }
        }

        #[inline]
        fn $le_name(&mut self, value: $tp) -> Result<(), $tp> {
            if self.can_fit(::core::mem::size_of::<$tp>()) {
                for b in value.to_le_bytes() {
                    self.push(b).unwrap();
                }
                Ok(())
            } else {
                Err(value)
            }
        }
    };
}

/// Extensions to [MaxArray<u8>] that allow pushing multi-byte integers.
pub trait MaxArrayPushIntExt {
    /// Pushes a value that can be converted into an unsigned 8-bit value.
    fn push_any<I: Into<u8>>(&mut self, value: I) -> Result<(), u8>;

    /// Pushes an unsigned 8-bit value.
    fn push_u8(&mut self, value: u8) -> Result<(), u8>;

    /// Pushes an unsigned 16-bit value in big-endian order.
    fn push_u16_be(&mut self, value: u16) -> Result<(), u16>;

    /// Pushes an unsigned 32-bit value in big-endian order.
    fn push_u32_be(&mut self, value: u32) -> Result<(), u32>;

    /// Pushes an unsigned 64-bit value in big-endian order.
    fn push_u64_be(&mut self, value: u64) -> Result<(), u64>;

    /// Pushes an unsigned 128-bit value in big-endian order.
    fn push_u128_be(&mut self, value: u128) -> Result<(), u128>;

    /// Pushes an unsigned 16-bit value in little-endian order.
    fn push_u16_le(&mut self, value: u16) -> Result<(), u16>;

    /// Pushes an unsigned 32-bit value in little-endian order.
    fn push_u32_le(&mut self, value: u32) -> Result<(), u32>;

    /// Pushes an unsigned 64-bit value in little-endian order.
    fn push_u64_le(&mut self, value: u64) -> Result<(), u64>;

    /// Pushes an unsigned 128-bit value in little-endian order.
    fn push_u128_le(&mut self, value: u128) -> Result<(), u128>;

    implement_signed!(push_i8, i8, push_u8, u8, "Pushes a signed 8-bit value.");
    implement_signed!(push_i16_be, i16, push_u16_be, u16, "Pushes a signed 16-bit value in big-endian order.");
    implement_signed!(push_i32_be, i32, push_u32_be, u32, "Pushes a signed 32-bit value in big-endian order.");
    implement_signed!(push_i64_be, i64, push_u64_be, u64, "Pushes a signed 64-bit value in big-endian order.");
    implement_signed!(push_i128_be, i128, push_u128_be, u128, "Pushes a signed 128-bit value in big-endian order.");
    implement_signed!(push_i16_le, i16, push_u16_le, u16, "Pushes a signed 16-bit value in little-endian order.");
    implement_signed!(push_i32_le, i32, push_u32_le, u32, "Pushes a signed 32-bit value in little-endian order.");
    implement_signed!(push_i64_le, i64, push_u64_le, u64, "Pushes a signed 64-bit value in little-endian order.");
    implement_signed!(push_i128_le, i128, push_u128_le, u128, "Pushes a signed 128-bit value in little-endian order.");
}

impl<const MAX_SIZE: usize> MaxArrayPushIntExt for MaxArray<u8, MAX_SIZE> {
    #[inline]
    fn push_any<I: Into<u8>>(&mut self, value: I) -> Result<(), u8> {
        self.push(value.into())
    }

    #[inline]
    fn push_u8(&mut self, value: u8) -> Result<(), u8> {
        if self.can_fit(::core::mem::size_of::<u8>()) {
            self.push(value).unwrap();
            Ok(())
        } else {
            Err(value)
        }
    }

    implement_push!(push_u16_be, push_u16_le, u16);
    implement_push!(push_u32_be, push_u32_le, u32);
    implement_push!(push_u64_be, push_u64_le, u64);
    implement_push!(push_u128_be, push_u128_le, u128);
}
