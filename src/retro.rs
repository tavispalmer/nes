#![allow(non_camel_case_types)]

use std::{ffi::{c_char, c_int, c_uint, c_void, CStr}, marker::PhantomData, mem, ptr::null};

pub const API_VERSION: c_uint = 1;

pub const DEVICE_JOYPAD: c_uint = 1;

pub const DEVICE_ID_JOYPAD_UP: c_uint = 4;

pub const ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;

pub const ENVIRONMENT_GET_VARIABLE_UPDATE: c_uint = 17;

pub const ENVIRONMENT_SET_SUPPORT_NO_GAME: c_uint = 18;

pub const ENVIRONMENT_GET_LOG_INTERFACE: c_uint = 27;

pub const REGION_NTSC: c_uint = 0;

#[repr(C)]
pub enum log_level {
    DEBUG = 0,
    INFO,
    WARN,
    ERROR,
    DUMMY = c_int::MAX as _,
}

pub type log_printf_t = unsafe extern "C" fn(level: log_level, fmt: *const c_char, ...);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct log_callback {
    pub log: log_printf_t,
}

impl Default for log_callback {
    #[inline]
    fn default() -> Self {
        unsafe {
            Self {
                log: mem::transmute::<*const c_void, _>(null()),
            }
        }
    }
}

#[repr(C)]
pub enum pixel_format {
    TYPE_0RGB1555 = 0,
    XRGB8888 = 1,
    RGB565 = 2,
    UNKNOWN = c_int::MAX as _,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct system_info<'a> {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
    pub _marker: PhantomData<&'a ()>,
}

impl Default for system_info<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            library_name: null(),
            library_version: null(),
            valid_extensions: null(),
            need_fullpath: false,
            block_extract: false,
            _marker: PhantomData,
        }
    }
}

impl<'a> system_info<'a> {
    #[inline]
    pub fn library_name(mut self, library_name: &'a CStr) -> Self {
        self.library_name = library_name.as_ptr();
        self
    }
    #[inline]
    pub fn library_version(mut self, library_version: &'a CStr) -> Self {
        self.library_version = library_version.as_ptr();
        self
    }
    #[inline]
    pub fn valid_extensions(mut self, valid_extensions: &'a CStr) -> Self {
        self.valid_extensions = valid_extensions.as_ptr();
        self
    }
    #[inline]
    pub fn need_fullpath(mut self, need_fullpath: bool) -> Self {
        self.need_fullpath = need_fullpath;
        self
    }
    #[inline]
    pub fn block_extract(mut self, block_extract: bool) -> Self {
        self.block_extract = block_extract;
        self
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct game_geometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: f32,
}

impl Default for game_geometry {
    #[inline]
    fn default() -> Self {
        Self {
            base_width: 0,
            base_height: 0,
            max_width: 0,
            max_height: 0,
            aspect_ratio: 0.0,
        }
    }
}

impl game_geometry {
    #[inline]
    pub fn base_width(mut self, base_width: c_uint) -> Self {
        self.base_width = base_width;
        self
    }
    #[inline]
    pub fn base_height(mut self, base_height: c_uint) -> Self {
        self.base_height = base_height;
        self
    }
    #[inline]
    pub fn max_width(mut self, max_width: c_uint) -> Self {
        self.max_width = max_width;
        self
    }
    #[inline]
    pub fn max_height(mut self, max_height: c_uint) -> Self {
        self.max_height = max_height;
        self
    }
    #[inline]
    pub fn aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct system_timing {
    pub fps: f64,
    pub sample_rate: f64,
}

impl Default for system_timing {
    #[inline]
    fn default() -> Self {
        Self {
            fps: 0.0,
            sample_rate: 0.0,
        }
    }
}

impl system_timing {
    #[inline]
    pub fn fps(mut self, fps: f64) -> Self {
        self.fps = fps;
        self
    }
    #[inline]
    pub fn sample_rate(mut self, sample_rate: f64) -> Self {
        self.sample_rate = sample_rate;
        self
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct system_av_info {
    pub geometry: game_geometry,
    pub timing: system_timing,
}

impl Default for system_av_info {
    #[inline]
    fn default() -> Self {
        Self {
            geometry: game_geometry::default(),
            timing: system_timing::default(),
        }
    }
}

impl system_av_info {
    #[inline]
    pub fn geometry(mut self, geometry: game_geometry) -> Self {
        self.geometry = geometry;
        self
    }
    #[inline]
    pub fn timing(mut self, timing: system_timing) -> Self {
        self.timing = timing;
        self
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct game_info<'a> {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
    pub _marker: PhantomData<&'a ()>,
}

pub type environment_t = unsafe extern "system" fn(cmd: c_uint, data: *mut c_void) -> bool;

pub type video_refresh_t = unsafe extern "system" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: usize);

pub type audio_sample_t = unsafe extern "system" fn(left: i16, right: i16);

pub type audio_sample_batch_t = unsafe extern "system" fn(data: *const i16, frames: usize) -> usize;

pub type input_poll_t = unsafe extern "system" fn();

pub type input_state_t = unsafe extern "system" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16;
