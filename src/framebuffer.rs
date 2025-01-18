use core::slice;
use std::{alloc::{alloc, alloc_zeroed, dealloc, Layout}, mem, ops::{Index, IndexMut}, ptr::NonNull, slice::SliceIndex};

use crate::color::XRGB8888;

pub struct Framebuffer {
    framebuffer: NonNull<[XRGB8888]>,
    width: usize,
    height: usize,
}

impl Framebuffer {
    #[inline]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            framebuffer: unsafe {
                // use NonNull instead of Box so we can set the alignment
                NonNull::new(slice::from_raw_parts_mut(alloc_zeroed(Layout::from_size_align(
                    width.next_power_of_two() * height * size_of::<XRGB8888>(),
                    width.next_power_of_two() * size_of::<XRGB8888>()
                ).unwrap()) as *mut XRGB8888, width.next_power_of_two() * height)).unwrap()
            },
            width,
            height,
        }
    }
    #[inline]
    pub const fn width(&self) -> usize {
        self.width
    }
    #[inline]
    pub const fn height(&self) -> usize {
        self.height
    }
    #[inline]
    pub const fn pitch(&self) -> usize {
        self.width.next_power_of_two() * size_of::<XRGB8888>()
    }
    #[inline]
    pub const fn as_ptr(&self) -> *const XRGB8888 {
        self.framebuffer.as_ptr() as _
    }
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut XRGB8888 {
        self.framebuffer.as_ptr() as _
    }
}

impl Drop for Framebuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            dealloc(self.framebuffer.as_ptr() as *mut u8, Layout::from_size_align(
                self.width.next_power_of_two() * self.height * size_of::<XRGB8888>(),
                self.width.next_power_of_two() * size_of::<XRGB8888>()
            ).unwrap());
        }
    }
}

// 2D indexing
impl Index<usize> for Framebuffer {
    type Output = [XRGB8888];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let pitch = self.width.next_power_of_two();
        unsafe {
            &self.framebuffer.as_ref()[(index*pitch)..((index+1)*pitch)]
        }
    }
}

impl IndexMut<usize> for Framebuffer {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let pitch = self.width.next_power_of_two();
        unsafe {
            &mut self.framebuffer.as_mut()[(index*pitch)..((index+1)*pitch)]
        }
    }
}
