//! Semi-standard building blocks for embedded applications.


// Leave std activated when testing (on the host).
#![cfg_attr(not(test), no_std)]


pub mod bit_field;
pub mod crc8;
pub mod esp3;
pub mod max_array;
pub mod max_array_ext;
pub mod ring_buffer;
