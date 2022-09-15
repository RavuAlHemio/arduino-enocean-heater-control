use core::mem::{MaybeUninit, replace};


/// A variable-length array of constant (upper-bound) size.
pub struct MaxArray<T, const MAX_SIZE: usize> {
    array: [MaybeUninit<T>; MAX_SIZE],
    length: usize,
}
impl<T, const MAX_SIZE: usize> MaxArray<T, MAX_SIZE> {
    pub const fn new() -> Self {
        let buf = unsafe {
            MaybeUninit::<[MaybeUninit<T>; MAX_SIZE]>::uninit().assume_init()
        };
        Self {
            array: buf,
            length: 0,
        }
    }

    #[inline] pub const fn len(&self) -> usize { self.length }
    #[inline] pub const fn max_size(&self) -> usize { MAX_SIZE }

    pub fn push(&mut self, val: T) -> Result<(), T> {
        if self.length < self.array.len() {
            self.array[self.length] = MaybeUninit::new(val);
            self.length += 1;
            Ok(())
        } else {
            Err(val)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            self.length -= 1;
            let elem = replace(&mut self.array[self.length], MaybeUninit::uninit());
            Some(unsafe { elem.assume_init() })
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<T, MAX_SIZE> {
        Iter {
            max_slice: self,
            next_index: 0,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let array_ptr = &self.array[0..self.length] as *const [MaybeUninit<T>] as *const [T];
            &*array_ptr
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let array_ptr = &mut self.array[0..self.length] as *mut [MaybeUninit<T>] as *mut [T];
            &mut *array_ptr
        }
    }

    pub fn fill_from<I: Iterator<Item = T>>(&mut self, mut iterator: I) {
        let old_length = self.len();

        self.length = 0;
        while self.length < self.max_size() {
            let new_value = match iterator.next() {
                Some(nv) => nv,
                None => break,
            };
            let old_value = replace(
                &mut self.array[self.length],
                MaybeUninit::new(new_value),
            );
            if self.length < old_length {
                // old_value is initialized
                // transmogrify it using assume_init() to make sure that it is dropped
                unsafe { old_value.assume_init() };
            }
            self.length += 1;
        }

        // drop the remaining values if the old length was greater than the new one
        for i in self.length..old_length {
            let old_value = replace(
                &mut self.array[i],
                MaybeUninit::uninit(),
            );
            unsafe { old_value.assume_init() };
        }
    }
}
impl<T: Clone, const MAX_SIZE: usize> MaxArray<T, MAX_SIZE> {
    pub fn copy_into(&self, slice: &mut [T]) {
        let out_length = slice.len().min(self.len());
        for i in 0..out_length {
            slice[i] = unsafe { self.array[i].assume_init_ref() }.clone();
        }
    }
}
impl<T, const SIZE: usize> Drop for MaxArray<T, SIZE> {
    fn drop(&mut self) {
        // take out the values from the occupied indexes and call assume_init on them
        // this ensures that they are dropped
        for i in 0..self.len() {
            let uninit_val = replace(
                &mut self.array[i],
                MaybeUninit::uninit(),
            );
            unsafe { uninit_val.assume_init() };
        }
    }
}

pub struct Iter<'a, T, const MAX_SIZE: usize> {
    max_slice: &'a MaxArray<T, MAX_SIZE>,
    next_index: usize,
}
impl<'a, T, const MAX_SIZE: usize> Iterator for Iter<'a, T, MAX_SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.max_slice.len() {
            let uninit_item = &self.max_slice.array[self.next_index];
            let item = unsafe { uninit_item.assume_init_ref() };
            self.next_index += 1;
            Some(item)
        } else {
            None
        }
    }
}
