use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::iter::Peekable;
use core::mem::{MaybeUninit, replace};


/// A variable-length array of constant (upper-bound) size.
pub struct MaxArray<T, const MAX_SIZE: usize> {
    array: [MaybeUninit<T>; MAX_SIZE],
    length: usize,
}
impl<T, const MAX_SIZE: usize> MaxArray<T, MAX_SIZE> {
    /// Creates a new, empty `MaxArray`.
    pub const fn new() -> Self {
        let buf = unsafe {
            MaybeUninit::<[MaybeUninit<T>; MAX_SIZE]>::uninit().assume_init()
        };
        Self {
            array: buf,
            length: 0,
        }
    }

    /// Creates a new `MaxArray` from all the elements in the peekable iterator. Panics if not all
    /// elements from the iterator fit into the `MaxArray`.
    pub fn from_iter_or_panic<I: Iterator<Item = T>>(iterator: Peekable<I>) -> Self {
        let mut ret = Self::new();
        if !ret.fill_from(iterator) {
            panic!("iterator not emptied when filling MaxArray");
        }
        ret
    }

    /// The number of elements currently contained in the array.
    #[inline] pub const fn len(&self) -> usize { self.length }

    /// The maximum number of elements that can be stored in this array.
    #[inline] pub const fn max_size(&self) -> usize { MAX_SIZE }

    /// Appends a single element to the end of the array. Returns `Ok(())` if there was enough space
    /// for the element and `Err(val)` (where `val` is the element to be appended) if not.
    pub fn push(&mut self, val: T) -> Result<(), T> {
        if self.length < self.array.len() {
            self.array[self.length] = MaybeUninit::new(val);
            self.length += 1;
            Ok(())
        } else {
            Err(val)
        }
    }

    /// Removes a single item from the end of the array. Returns `Some(element)` if the array
    /// contained at least one element and `None` if it was empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            self.length -= 1;
            let elem = replace(&mut self.array[self.length], MaybeUninit::uninit());
            Some(unsafe { elem.assume_init() })
        } else {
            None
        }
    }

    /// Returns an iterator that iterates over references to the elements currently contained in the
    /// MaxArray.
    pub fn iter(&self) -> Iter<T, MAX_SIZE> {
        Iter {
            max_slice: self,
            next_index: 0,
        }
    }

    /// Returns an immutable slice of the part of the array that is currently occupied.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let array_ptr = &self.array[0..self.length] as *const [MaybeUninit<T>] as *const [T];
            &*array_ptr
        }
    }

    /// Returns a mutable slice of the part of the array that is currently occupied.
    ///
    /// Note that it is not possible to change the length of the MaxArray through this slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let array_ptr = &mut self.array[0..self.length] as *mut [MaybeUninit<T>] as *mut [T];
            &mut *array_ptr
        }
    }

    /// Returns whether the array can fit the given number of elements.
    #[inline]
    pub fn can_fit(&self, element_count: usize) -> bool {
        element_count <= self.max_size() - self.len()
    }

    /// Replaces the elements in the MaxArray by those returned by the provided peekable iterator.
    /// Returns whether the iterator was fully emptied by this operation.
    pub fn fill_from<I: Iterator<Item = T>>(&mut self, mut iterator: Peekable<I>) -> bool {
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

        // return whether we managed to fill the buffer
        // (the iterator is empty if we did)
        iterator.peek().is_none()
    }
}
impl<T: Clone, const MAX_SIZE: usize> MaxArray<T, MAX_SIZE> {
    /// Stores clones of the elements from this MaxArray into the given slice.
    ///
    /// Returns the number of elements stored in the slice (the length of the MaxArray or the
    /// length of the slice, whichever is smaller).
    pub fn copy_into(&self, slice: &mut [T]) -> usize {
        let out_length = slice.len().min(self.len());
        for i in 0..out_length {
            slice[i] = unsafe { self.array[i].assume_init_ref() }.clone();
        }
        out_length
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
impl<T: Copy, const MAX_SIZE: usize> Clone for MaxArray<T, MAX_SIZE> {
    fn clone(&self) -> Self {
        Self { array: self.array.clone(), length: self.length.clone() }
    }
}
impl<T: PartialEq, const MAX_SIZE: usize> PartialEq for MaxArray<T, MAX_SIZE> {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }

        for (s, o) in self.array.iter().zip(other.array.iter()).take(self.length) {
            let s_init = unsafe { s.assume_init_ref() };
            let o_init = unsafe { o.assume_init_ref() };
            if s_init != o_init {
                return false;
            }
        }

        true
    }
}
impl<T: Eq, const MAX_SIZE: usize> Eq for MaxArray<T, MAX_SIZE> {
}
impl<T: PartialOrd, const MAX_SIZE: usize> PartialOrd for MaxArray<T, MAX_SIZE> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let min_length = self.length.min(other.length);
        for (s, o) in self.array.iter().zip(other.array.iter()).take(min_length) {
            let s_init = unsafe { s.assume_init_ref() };
            let o_init = unsafe { o.assume_init_ref() };
            match s_init.partial_cmp(o_init) {
                Some(Ordering::Equal) => {}, // try the next pair
                other => return other,
            }
        }

        // all values up to the common length are equal; compare lengths
        Some(self.length.cmp(&other.length))
    }
}
impl<T: Ord, const MAX_SIZE: usize> Ord for MaxArray<T, MAX_SIZE> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl<T: Hash, const MAX_SIZE: usize> Hash for MaxArray<T, MAX_SIZE> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for item in self.array.iter().take(self.length) {
            let item_init = unsafe { item.assume_init_ref() };
            item_init.hash(state);
        }
    }
}
impl<T: fmt::Debug, const MAX_SIZE: usize> fmt::Debug for MaxArray<T, MAX_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaxArray")
            .field("array", &MaxArrayFormatter(&self))
            .field("length", &self.length)
            .finish()
    }
}

struct MaxArrayFormatter<'a, T: fmt::Debug, const MAX_SIZE: usize>(&'a MaxArray<T, MAX_SIZE>);
impl<'a, T: fmt::Debug, const MAX_SIZE: usize> fmt::Debug for MaxArrayFormatter<'a, T, MAX_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dl = f.debug_list();
        for i in 0..self.0.len() {
            dl.entry(unsafe { self.0.array[i].assume_init_ref() });
        }
        dl.finish()
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
