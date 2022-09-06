/// A circular buffer of a preset capacity. If a value is appended to a full ring buffer, the oldest
/// element is replaced with the newest one.
#[derive(Clone, Copy, Debug)]
pub struct RingBuffer<T, const SIZE: usize> {
    buffer: [T; SIZE],
    first_set: usize,
    last_set_plus_one: usize,
}
impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Returns the capacity of this buffer, i.e. its length when it is entirely full.
    #[inline]
    pub fn capacity(&self) -> usize { SIZE }

    /// Returns the number of occupied elements within the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        if self.first_set > self.last_set_plus_one {
            // 0123456789
            // ####   ###
            // fs=7, lsp1=4, capa=10
            //
            // 0123456789
            //      #####
            // fs=5, lsp1=0, capa=10
            (self.capacity() - self.first_set) + self.last_set_plus_one
        } else {
            // 0123456789
            //   #####
            // fs=2, lsp1=7, capa=10
            //
            // 0123456789
            // ####
            // fs=0, lsp1=4, capa=10
            //
            // 0123456789
            // ##########
            // fs=0, lsp1=
            self.last_set_plus_one - self.first_set
        }
    }
}

impl<T: Default + Sized, const SIZE: usize> RingBuffer<T, SIZE> {
    /// Creates a new, empty ring buffer.
    pub const fn new() -> Self {
        Self {
            buffer: Self::make_buffer(),
            first_set: 0,
            last_set_plus_one: 0,
        }
    }

    /// Creates the inner buffer for this ring buffer.
    const fn make_buffer() -> [T; SIZE] {
        use core::mem::MaybeUninit;
        let mut data: [MaybeUninit<T>; SIZE] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        for elem in &mut data[..] {
            unsafe {
                core::ptr::write(elem.as_mut_ptr(), T::default());
            }
        }
        unsafe {
            core::mem::transmute(data)
        }
    }
}
