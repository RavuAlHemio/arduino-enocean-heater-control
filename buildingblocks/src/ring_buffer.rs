use core::mem::MaybeUninit;


/// A circular buffer of a preset capacity. If a value is appended to a full ring buffer, the oldest
/// element is replaced with the newest one.
#[derive(Clone, Copy, Debug)]
pub struct RingBuffer<T, const SIZE: usize> {
    buffer: [MaybeUninit<T>; SIZE],
    first_set: usize,
    count_set: usize,
}
impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Returns the capacity of this buffer, i.e. its length when it is entirely full.
    #[inline]
    pub const fn capacity(&self) -> usize { SIZE }

    /// Returns the number of occupied elements within the buffer.
    #[inline]
    pub const fn len(&self) -> usize { count_set }

    /// Creates a new, empty ring buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [MaybeUninit::uninit(); SIZE],
            first_set: 0,
            count_set: 0,
        }
    }

    /// Attempts to add a value to the end of the buffer.
    ///
    /// Returns whether the value was added successfully.
    #[inline]
    pub const fn push(&mut self, value: T) -> bool {
        if self.count_set == SIZE {
            false
        } else {
            let index = (self.first_set + self.count_set) % SIZE;
            self.buffer[index] = MaybeUninit::new(value);
            self.count_set += 1;
            true
        }
    }

    /// Attempts to remove a value from the beginning of the buffer.
    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        if self.count_set == 0 {
            None
        } else {
            let val = core::mem::replace(
                &mut self.buffer[self.first_set],
                MaybeUninit::uninit(),
            );
            self.first_set = (self.first_set + 1) % SIZE;
            self.count_set -= 1;
        }
    }
}

struct Iter<'a, T, SIZE> {
    ring_buffer: &'a RingBuffer<T, SIZE>,
    index: usize,
}
impl Iterator for Iter<'a, T, SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn test_empty() {
        let mut rb: RingBuffer<u8, 32> = RingBuffer::new();
        todo!();
    }
}
