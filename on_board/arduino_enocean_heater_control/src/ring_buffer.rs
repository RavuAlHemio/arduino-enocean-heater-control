use core::cell::UnsafeCell;

use buildingblocks::ring_buffer::RingBuffer;
use cortex_m::interrupt;


/// A ring buffer which is protected by a critical section (disabled interrupts).
pub struct CriticalRingBuffer<T: Copy + Default, const SIZE: usize> {
    buffer: UnsafeCell<RingBuffer<T, SIZE>>,
}
impl<T: Copy + Default, const SIZE: usize> CriticalRingBuffer<T, SIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new(RingBuffer::new()),
        }
    }

    pub fn push(&self, value: T) -> bool {
        interrupt::free(|_cs| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.push(value)
        })
    }

    pub fn force_push(&self, value: T) {
        interrupt::free(|_cs| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.force_push(value)
        })
    }

    pub fn pop(&self) -> Option<T> {
        interrupt::free(|_| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.pop()
        })
    }

    pub fn pop_fill(&self, buf: &mut [T]) -> bool {
        interrupt::free(|_| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.pop_fill(buf)
        })
    }

    pub fn peek_at(&self, index: usize) -> Option<T> {
        interrupt::free(|_| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.peek_at(index)
        })
    }

    pub fn peek_fill(&self, buf: &mut [T]) -> bool {
        interrupt::free(|_| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.peek_fill(buf)
        })
    }

    pub fn len(&self) -> usize {
        interrupt::free(|_| {
            let mut_ref = unsafe { &mut *self.buffer.get() };
            mut_ref.len()
        })
    }

    #[inline] pub const fn max_size(&self) -> usize { SIZE }
}
unsafe impl<T: Copy + Default + Sync, const SIZE: usize> Sync for CriticalRingBuffer<T, SIZE> {}
impl<T: Copy + Default, const SIZE: usize> Clone for CriticalRingBuffer<T, SIZE> {
    fn clone(&self) -> Self {
        let inner_buffer = unsafe { (&*self.buffer.get()).clone() };
        Self {
            buffer: UnsafeCell::new(inner_buffer),
        }
    }
}
