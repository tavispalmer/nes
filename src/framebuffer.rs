use core::slice;
use std::{alloc::{alloc, alloc_zeroed, dealloc, Layout}, mem, ops::{Index, IndexMut}, ptr::NonNull, slice::SliceIndex};

use crate::{color::XRGB8888, texture::Texture};

pub struct Framebuffer {
    framebuffer: NonNull<XRGB8888>,
    width: usize,
    height: usize,
}

impl Framebuffer {
    #[inline]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            framebuffer: unsafe {
                // use NonNull instead of Box so we can set the alignment
                NonNull::new(alloc_zeroed(Layout::from_size_align(
                    width.next_power_of_two() * height * size_of::<XRGB8888>(),
                    width.next_power_of_two() * size_of::<XRGB8888>()
                ).unwrap()) as *mut XRGB8888).unwrap()
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

    pub fn draw(&mut self, texture: &Texture, x: usize, y: usize) {
        for y_off in 0..(texture.height()).min(self.height() - y) {
            for x_off in 0..(texture.width()).min(self.width() - x) {
                // simple blending
                let src = &texture[y_off][x_off];
                let dst = &mut self[y + y_off][x + x_off];
                let src_mul = src.a() as u16;
                let dst_mul = 0xff - src_mul;
            
                *dst = XRGB8888::new(
                    ((dst.r() as u16 * dst_mul + src.r() as u16 * src_mul) / 0xff) as u8,
                    ((dst.g() as u16 * dst_mul + src.g() as u16 * src_mul) / 0xff) as u8,
                    ((dst.b() as u16 * dst_mul + src.b() as u16 * src_mul) / 0xff) as u8,
                );
            }
        }
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
            &slice::from_raw_parts(
                self.framebuffer.as_ptr(),
                self.width.next_power_of_two() * self.height
            )[(index*pitch)..((index+1)*pitch)]
        }
    }
}

impl IndexMut<usize> for Framebuffer {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let pitch = self.width.next_power_of_two();
        unsafe {
            &mut slice::from_raw_parts_mut(
                self.framebuffer.as_ptr(),
                self.width.next_power_of_two() * self.height
            )[(index*pitch)..((index+1)*pitch)]
        }
    }
}
