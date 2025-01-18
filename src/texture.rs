use std::{alloc::{alloc, dealloc, Layout}, ops::{Index, IndexMut}, ptr::NonNull, slice};

use crate::{color::Color, framebuffer::Framebuffer};

pub struct Texture {
    texture: NonNull<Color>,
    width: usize,
    height: usize,
}

impl Texture {
    #[inline]
    pub fn new(texture: &[Color], width: usize, height: usize) -> Self {
        Self {
            texture: unsafe { 
                let tex = NonNull::new(alloc(Layout::array::<Color>(width * height).unwrap()) as *mut Color).unwrap();
                slice::from_raw_parts_mut(tex.as_ptr(), width * height).copy_from_slice(texture);
                tex
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
}

impl Index<usize> for Texture {
    type Output = [Color];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &slice::from_raw_parts(
                self.texture.as_ptr(),
                self.width * self.height
            )[(index*self.width)..((index+1)*self.width)]
        }
    }
}

impl IndexMut<usize> for Texture {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            &mut slice::from_raw_parts_mut(
                self.texture.as_ptr(),
                self.width * self.height
            )[(index*self.width)..((index+1)*self.width)]
        }
    }
}

impl Drop for Texture {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            dealloc(self.texture.as_ptr() as *mut u8, Layout::array::<Color>(self.width * self.height).unwrap());
        }
    }
}
