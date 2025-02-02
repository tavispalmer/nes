use std::{ffi::{c_char, c_uint, c_void}, ptr, slice};

use crate::{retro, Controller, Nes};

// core
static mut NES: Option<Nes<RetroPad>> = None;
static mut BUF: [u8; 256*240*4] = [0; 256*240*4];

static mut ENVIRON_CB: retro::environment_t = {
    unsafe extern "system" fn environ_cb(_cmd: c_uint, _data: *mut c_void) -> bool {
        panic!(concat!("Unable to load ", stringify!(ENVIRON_CB)))
    }
    environ_cb
};

static mut VIDEO_CB: retro::video_refresh_t = {
    unsafe extern "system" fn video_cb(_data: *const c_void, _width: c_uint, _height: c_uint, _pitch: usize) {
        panic!(concat!("Unable to load ", stringify!(VIDEO_CB)))
    }
    video_cb
};

static mut AUDIO_CB: retro::audio_sample_t = {
    unsafe extern "system" fn audio_cb(_left: i16, _right: i16) {
        panic!(concat!("Unable to load ", stringify!(AUDIO_CB)))
    }
    audio_cb
};

static mut AUDIO_BATCH_CB: retro::audio_sample_batch_t = {
    unsafe extern "system" fn audio_batch_cb(_data: *const i16, _frames: usize) -> usize {
        panic!(concat!("Unable to load ", stringify!(AUDIO_BATCH_CB)))
    }
    audio_batch_cb
};

static mut INPUT_POLL_CB: retro::input_poll_t = {
    unsafe extern "system" fn input_poll_cb() {
        panic!(concat!("Unable to load ", stringify!(INPUT_POLL_CB)))
    }
    input_poll_cb
};

static mut INPUT_STATE_CB: retro::input_state_t = {
    unsafe extern "system" fn input_state_cb(_port: c_uint, _device: c_uint, _index: c_uint, _id: c_uint) -> i16 {
        panic!(concat!("Unable to load ", stringify!(INPUT_STATE_CB)))
    }
    input_state_cb
};

const SAMPLE_RATE: usize = 48000;
const SAMPLE_COUNT: usize = (11*4*341*262*SAMPLE_RATE)/(236250000);

static mut AUDIO_BUF: [i16; SAMPLE_COUNT<<1] = [0; SAMPLE_COUNT<<1];

struct RetroPad {
}

impl Controller for RetroPad {
    fn poll(&mut self) {
        unsafe {
            INPUT_POLL_CB()
        }
    }

    fn a(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_B) != 0
        }
    }
    fn b(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_Y) != 0
        }
    }
    fn select(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_SELECT) != 0
        }
    }
    fn start(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_START) != 0
        }
    }
    fn up(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_UP) != 0
        }
    }
    fn down(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_DOWN) != 0
        }
    }
    fn left(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_LEFT) != 0
        }
    }
    fn right(&self) -> bool {
        unsafe {
            INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_RIGHT) != 0
        }
    }
}

#[no_mangle]
pub extern "system" fn retro_init() {}

#[no_mangle]
pub extern "system" fn retro_deinit() {}

#[no_mangle]
pub extern "system" fn retro_api_version() -> c_uint { 
    retro::API_VERSION
}

#[no_mangle]
pub extern "system" fn retro_set_controller_port_device(_port: c_uint, _device: c_uint) {}

#[no_mangle]
pub extern "system" fn retro_get_system_info(info: *mut retro::system_info) {
    unsafe {
        *info = retro::system_info::default()
            .library_name(c"TestCore")
            .library_version(c"v1");
    }
}

#[no_mangle]
pub extern "system" fn retro_get_system_av_info(info: *mut retro::system_av_info) {
    unsafe {
        *info = retro::system_av_info::default()
            .timing(retro::system_timing::default()
                .fps(60.0)
                .sample_rate(SAMPLE_RATE as f64))
            .geometry(retro::game_geometry::default()
                .base_width(320)
                .base_height(240)
                .max_width(320)
                .max_height(240)
                .aspect_ratio(4.0 / 3.0));
    }
}

#[no_mangle]
pub extern "system" fn retro_set_environment(cb: retro::environment_t) {
    unsafe {
        ENVIRON_CB = cb;
    }
}

#[no_mangle]
pub extern "system" fn retro_set_video_refresh(cb: retro::video_refresh_t) {
    unsafe {
        VIDEO_CB = cb;
    }
}

#[no_mangle]
pub extern "system" fn retro_set_audio_sample(cb: retro::audio_sample_t) {
    unsafe {
        AUDIO_CB = cb;
    }
}

#[no_mangle]
pub extern "system" fn retro_set_audio_sample_batch(cb: retro::audio_sample_batch_t) {
    unsafe {
        AUDIO_BATCH_CB = cb;
    }
}

#[no_mangle]
pub extern "system" fn retro_set_input_poll(cb: retro::input_poll_t) {
    unsafe {
        INPUT_POLL_CB = cb;
    }
}

#[no_mangle]
pub extern "system" fn retro_set_input_state(cb: retro::input_state_t) {
    unsafe {
        INPUT_STATE_CB = cb;
    }
}

#[no_mangle]
pub extern "system" fn retro_reset() {}

#[no_mangle]
pub extern "system" fn retro_run() {
    unsafe {
        NES.as_mut().unwrap().run();
        BUF.copy_from_slice(NES.as_mut().unwrap().framebuffer());
        VIDEO_CB(BUF.as_ptr() as _, 256, 240 , 256*4);
        NES.as_mut().unwrap().play_audio(&mut AUDIO_BUF);
        AUDIO_BATCH_CB(AUDIO_BUF.as_ptr(), AUDIO_BUF.len()>>1);
    }
}

#[no_mangle]
pub extern "system" fn retro_load_game(info: *const retro::game_info) -> bool {
    unsafe {
        let fmt = retro::pixel_format::XRGB8888;
        if !ENVIRON_CB(retro::ENVIRONMENT_SET_PIXEL_FORMAT, &raw const fmt as _) {
            eprintln!("XRGB8888 is not supported.");
            return false;
        }

        let info = &*info;
        let game = slice::from_raw_parts(
            info.data as *const u8,
            info.size,
        );
        NES = Nes::load_from_memory(game);
        if let Some(nes) = NES.as_mut() {
            nes.connect(0, RetroPad {});
            true
        } else {
            false
        }
    }
}

#[no_mangle]
pub extern "system" fn retro_unload_game() {
    unsafe {
        NES = None;
    }
}

#[no_mangle]
pub extern "system" fn retro_get_region() -> c_uint {
    retro::REGION_NTSC
}

#[no_mangle]
pub extern "system" fn retro_load_game_special(_type: c_uint, _info: *const retro::game_info, _num: usize) -> bool {
    false
}

#[no_mangle]
pub extern "system" fn retro_serialize_size() -> usize {
    0
}

#[no_mangle]
pub extern "system" fn retro_serialize(_data: *mut c_void, _size: usize) -> bool {
    false
}

#[no_mangle]
pub extern "system" fn retro_unserialize(_data: *const c_void, _size: usize) -> bool {
    false
}

#[no_mangle]
pub extern "system" fn retro_get_memory_data(_id: c_uint) -> *mut c_void {
    ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn retro_get_memory_size(_id: c_uint) -> usize {
    0
}

#[no_mangle]
pub extern "system" fn retro_cheat_reset() {}

#[no_mangle]
pub extern "system" fn retro_cheat_set(_index: c_uint, _enabled: bool, _code: *const c_char) {}
