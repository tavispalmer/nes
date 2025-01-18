use std::{ffi::{c_char, c_uint, c_void}, mem::MaybeUninit, ptr, slice};

use color::{Color, XRGB8888};
use framebuffer::Framebuffer;
use texture::Texture;

mod color;
mod framebuffer;
mod texture;

mod retro;

static mut FRAME_BUF: Option<Framebuffer> = None;
static mut TEXTURE: Option<Texture> = None;

static mut LOG_CB: Option<retro::log_printf_t> = None;

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

static mut X_COORD: usize = 0;
static mut Y_COORD: usize = 0;
static mut MOUSE_REL_X: isize = 0;
static mut MOUSE_REL_Y: isize = 0;

#[no_mangle]
pub extern "system" fn retro_init() {
    unsafe {
        FRAME_BUF = Some(Framebuffer::new(320, 240));
        TEXTURE = Some(Texture::new(&[
            Color::BLUE; 256
        ], 16, 16));
    }
}

#[no_mangle]
pub extern "system" fn retro_deinit() {
    unsafe {
        FRAME_BUF = None;
        TEXTURE = None;
    }
}

#[no_mangle]
pub extern "system" fn retro_api_version() -> c_uint { 
    retro::API_VERSION
}

#[no_mangle]
pub extern "system" fn retro_set_controller_port_device(port: c_uint, device: c_uint) {
    unsafe {
        if let Some(log_cb) = LOG_CB {
            log_cb(retro::log_level::INFO, c"Plugging device %u into port %u.\n".as_ptr(), device, port);
        } else {
            eprintln!("Plugging device {device} into port {port}.");
        }
    }
}

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
                .sample_rate(0.0))
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

        let no_content = true;
        cb(retro::ENVIRONMENT_SET_SUPPORT_NO_GAME, &raw const no_content as _);

        let mut logging: MaybeUninit<retro::log_callback> = MaybeUninit::uninit();
        if cb(retro::ENVIRONMENT_GET_LOG_INTERFACE, &raw mut logging as _) {
            LOG_CB = Some(logging.assume_init().log);
        }
        else {
            LOG_CB = None;
        }
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
pub extern "system" fn retro_reset() {
    unsafe {
        X_COORD = 0;
        Y_COORD = 0;
    }
}

fn update_input() {
    unsafe {
        INPUT_POLL_CB();
        if INPUT_STATE_CB(0, retro::DEVICE_JOYPAD, 0, retro::DEVICE_ID_JOYPAD_UP) != 0 {
            /* stub */
        }
    }
}

fn render_checkered() {
    unsafe {
        let buf = FRAME_BUF.as_mut().unwrap();

        for y in 0..buf.height() {
            let index_y = ((y - Y_COORD) >> 4) & 1;
            for x in 0..buf.width() {
                let index_x = ((x - X_COORD) >> 4) & 1;
                buf[y][x] = if (index_y ^ index_x) != 0 { XRGB8888::RED } else { XRGB8888::LIME };
            }
        }

        for y in (MOUSE_REL_Y - 5) as usize..=(MOUSE_REL_Y + 5) as usize {
            for x in (MOUSE_REL_X - 5) as usize..=(MOUSE_REL_X + 5) as usize {
                buf[y][x] = XRGB8888::BLUE;
            }
        }

        buf.draw(TEXTURE.as_ref().unwrap(), 2, 0);

        VIDEO_CB(buf.as_ptr() as _, buf.width() as _, buf.height() as _, buf.pitch());
    }
}

fn check_variables() {}

fn audio_callback() {
    unsafe {
        AUDIO_CB(0, 0);
    }
}

#[no_mangle]
pub extern "system" fn retro_run() {
    update_input();
    render_checkered();
    audio_callback();

    unsafe {
        let mut updated = false;
        if ENVIRON_CB(retro::ENVIRONMENT_GET_VARIABLE_UPDATE, &raw mut updated as _) && updated {
            check_variables();
        }
    }
}

#[no_mangle]
pub extern "system" fn retro_load_game(_info: *const retro::game_info) -> bool {
    unsafe {
        let fmt = retro::pixel_format::XRGB8888;
        if !ENVIRON_CB(retro::ENVIRONMENT_SET_PIXEL_FORMAT, &raw const fmt as _) {
            if let Some(log_cb) = LOG_CB {
                log_cb(retro::log_level::INFO, c"XRGB8888 is not supported.\n".as_ptr());
            }
            else {
                eprintln!("XRGB8888 is not supported.");
            }
            return false;
        }
    }

    check_variables();

    true
}

#[no_mangle]
pub extern "system" fn retro_unload_game() {}

#[no_mangle]
pub extern "system" fn retro_get_region() -> c_uint {
    retro::REGION_NTSC
}

#[no_mangle]
pub extern "system" fn retro_load_game_special(type_: c_uint, _info: *const retro::game_info, num: usize) -> bool {
    if type_ != 0x200 {
        return false;
    }
    if num != 2 {
        return false;
    }
    return retro_load_game(ptr::null());
}

#[no_mangle]
pub extern "system" fn retro_serialize_size() -> usize {
    2
}

#[no_mangle]
pub extern "system" fn retro_serialize(data_: *mut c_void, size: usize) -> bool {
    if size < 2 {
        return false;
    }

    unsafe {
        let data = slice::from_raw_parts_mut(data_ as *mut u8, 2);
        data[0] = X_COORD as u8;
        data[1] = Y_COORD as u8;
    }
    true
}

#[no_mangle]
pub extern "system" fn retro_unserialize(data_: *const c_void, size: usize) -> bool {
    if size < 2 {
        return false;
    }

    unsafe {
        let data = slice::from_raw_parts(data_ as *mut u8, 2);
        X_COORD = data[0] as usize;
        Y_COORD = data[1] as usize;
    }
    true
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
