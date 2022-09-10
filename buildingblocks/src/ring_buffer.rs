use core::mem::MaybeUninit;


/// A circular buffer of a preset capacity. If a value is appended to a full ring buffer, depending
/// on the call used, either the oldest element is replaced with the newest one or the newest
/// element is rejected.
#[derive(Clone, Debug)]
pub struct RingBuffer<T: Copy + Default, const SIZE: usize> {
    buffer: [MaybeUninit<T>; SIZE],
    first_set: usize,
    count_set: usize,
}
impl<T: Copy + Default, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Returns the capacity of this buffer, i.e. its length when it is entirely full.
    #[inline]
    pub const fn capacity(&self) -> usize { SIZE }

    /// Returns the number of occupied elements within the buffer.
    #[inline]
    pub const fn len(&self) -> usize { self.count_set }

    /// Creates a new, empty ring buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [MaybeUninit::uninit(); SIZE],
            first_set: 0,
            count_set: 0,
        }
    }

    /// Attempts to add a value to the end of the buffer. Rejects the element if the buffer is full.
    ///
    /// Returns whether the value was added successfully.
    #[inline]
    pub fn push(&mut self, value: T) -> bool {
        if self.count_set == SIZE {
            false
        } else {
            let index = (self.first_set + self.count_set) % SIZE;
            self.buffer[index] = MaybeUninit::new(value);
            self.count_set += 1;
            true
        }
    }

    /// Attempts to add a value to the end of the buffer. Overwrites the oldest element if the
    /// buffer is full.
    #[inline]
    pub fn force_push(&mut self, value: T) {
        if self.count_set == SIZE {
            // this should ensure that the element is dropped correctly
            self.pop();
        }

        let index = (self.first_set + self.count_set) % SIZE;
        self.buffer[index] = MaybeUninit::new(value);
        self.count_set += 1;
    }

    /// Attempts to remove a value from the beginning of the buffer.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.count_set == 0 {
            None
        } else {
            let val = core::mem::replace(
                &mut self.buffer[self.first_set],
                MaybeUninit::uninit(),
            );
            self.first_set = (self.first_set + 1) % SIZE;
            self.count_set -= 1;
            Some(unsafe { val.assume_init() })
        }
    }

    /// Attempts to obtain the value at the given index without removing it from the buffer.
    #[inline]
    pub fn peek_at(&self, index: usize) -> Option<T> {
        if index >= self.count_set {
            None
        } else {
            let buffer_index = (self.first_set + index) % SIZE;
            Some(unsafe { self.buffer[buffer_index].assume_init() })
        }
    }

    /// Obtains an iterator that runs through the buffer and returns copies of each element.
    #[inline]
    pub fn iter(&self) -> Iter<T, SIZE> {
        Iter {
            ring_buffer: &self,
            index: 0,
        }
    }
}

pub struct Iter<'a, T: Copy + Default, const SIZE: usize> {
    ring_buffer: &'a RingBuffer<T, SIZE>,
    index: usize,
}
impl<'a, T: Copy + Default, const SIZE: usize> Iterator for Iter<'a, T, SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.ring_buffer.peek_at(self.index)?;
        self.index += 1;
        Some(val)
    }
}


#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn test_empty() {
        let mut rb: RingBuffer<u8, 32> = RingBuffer::new();
        assert_eq!(rb.capacity(), 32);
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.pop(), None);
        assert_eq!(rb.pop(), None);
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn test_push_pop() {
        let mut rb: RingBuffer<u8, 4> = RingBuffer::new();
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 0);

        assert_eq!(rb.push(b'1'), true);
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 1);

        assert_eq!(rb.push(b'2'), true);
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 2);

        assert_eq!(rb.push(b'3'), true);
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 3);

        assert_eq!(rb.push(b'4'), true);
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 4);

        // now the buffer is full

        assert_eq!(rb.push(b'5'), false);
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 4);

        // start popping

        assert_eq!(rb.pop(), Some(b'1'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 3);

        assert_eq!(rb.pop(), Some(b'2'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 2);

        assert_eq!(rb.pop(), Some(b'3'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 1);

        assert_eq!(rb.pop(), Some(b'4'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 0);

        assert_eq!(rb.pop(), None);
        assert_eq!(rb.pop(), None);
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn test_force_push_pop() {
        let mut rb: RingBuffer<u8, 4> = RingBuffer::new();
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 0);

        rb.force_push(b'1');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 1);

        rb.force_push(b'2');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 2);

        rb.force_push(b'3');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 3);

        rb.force_push(b'4');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 4);

        // now the buffer is full

        rb.force_push(b'5');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 4);

        // start popping

        assert_eq!(rb.pop(), Some(b'2'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 3);

        assert_eq!(rb.pop(), Some(b'3'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 2);

        assert_eq!(rb.pop(), Some(b'4'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 1);

        assert_eq!(rb.pop(), Some(b'5'));
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 0);

        assert_eq!(rb.pop(), None);
        assert_eq!(rb.pop(), None);
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn test_iterator() {
        let mut rb: RingBuffer<u8, 4> = RingBuffer::new();
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 0);

        rb.force_push(b'1');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 1);

        rb.force_push(b'2');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 2);

        rb.force_push(b'3');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 3);

        rb.force_push(b'4');
        assert_eq!(rb.capacity(), 4);
        assert_eq!(rb.len(), 4);

        {
            let mut iter = rb.iter();
            assert_eq!(iter.next(), Some(b'1'));
            assert_eq!(iter.next(), Some(b'2'));
            assert_eq!(iter.next(), Some(b'3'));
            assert_eq!(iter.next(), Some(b'4'));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        assert_eq!(rb.pop(), Some(b'1'));

        {
            let mut iter = rb.iter();
            assert_eq!(iter.next(), Some(b'2'));
            assert_eq!(iter.next(), Some(b'3'));
            assert_eq!(iter.next(), Some(b'4'));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        assert_eq!(rb.push(b'5'), true);

        {
            let mut iter = rb.iter();
            assert_eq!(iter.next(), Some(b'2'));
            assert_eq!(iter.next(), Some(b'3'));
            assert_eq!(iter.next(), Some(b'4'));
            assert_eq!(iter.next(), Some(b'5'));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        assert_eq!(rb.pop(), Some(b'2'));
        assert_eq!(rb.push(b'6'), true);

        {
            let mut iter = rb.iter();
            assert_eq!(iter.next(), Some(b'3'));
            assert_eq!(iter.next(), Some(b'4'));
            assert_eq!(iter.next(), Some(b'5'));
            assert_eq!(iter.next(), Some(b'6'));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        assert_eq!(rb.pop(), Some(b'3'));
        assert_eq!(rb.pop(), Some(b'4'));
        assert_eq!(rb.push(b'7'), true);
        assert_eq!(rb.push(b'8'), true);

        {
            let mut iter = rb.iter();
            assert_eq!(iter.next(), Some(b'5'));
            assert_eq!(iter.next(), Some(b'6'));
            assert_eq!(iter.next(), Some(b'7'));
            assert_eq!(iter.next(), Some(b'8'));
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }
    }
}
