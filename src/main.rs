use core::slice;
use std::{cell::Cell, collections::VecDeque, env, ffi::{c_char, c_int, c_void, CString, OsString}, fs, mem::MaybeUninit, process::{self, ExitCode}, ptr::{null, null_mut, NonNull}, thread, time::{Duration, Instant}};

use nes::Nes;
use sdl3::{event::Event, keyboard::Keycode, render::TextureAccess, sys::{audio::{SDL_AudioSpec, SDL_AudioStream, SDL_GetAudioStreamAvailable, SDL_OpenAudioDeviceStream, SDL_PutAudioStreamData, SDL_ResumeAudioStreamDevice, SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, SDL_AUDIO_S16}, events::{SDL_Event, SDL_EventType, SDL_EVENT_KEY_DOWN, SDL_EVENT_KEY_UP, SDL_EVENT_QUIT}, init::{SDL_AppResult, SDL_Init, SDL_APP_CONTINUE, SDL_APP_FAILURE, SDL_APP_SUCCESS, SDL_INIT_AUDIO, SDL_INIT_VIDEO}, keycode::{SDL_Keycode, SDLK_DOWN, SDLK_LEFT, SDLK_RETURN, SDLK_RIGHT, SDLK_RSHIFT, SDLK_UP, SDLK_X, SDLK_Z}, main::{SDL_EnterAppMainCallbacks, SDL_RunApp, SDL_main_func}, pixels::{SDL_PIXELFORMAT_ABGR8888, SDL_PIXELFORMAT_ARGB32, SDL_PIXELFORMAT_ARGB8888, SDL_PIXELFORMAT_BGRA32, SDL_PIXELFORMAT_RGB24, SDL_PIXELFORMAT_RGBA32, SDL_PIXELFORMAT_RGBA8888}, render::{SDL_CreateTexture, SDL_CreateWindowAndRenderer, SDL_DestroyTexture, SDL_GetCurrentRenderOutputSize, SDL_GetRenderOutputSize, SDL_GetRenderScale, SDL_LockTexture, SDL_LockTextureToSurface, SDL_RenderPresent, SDL_RenderTexture, SDL_Renderer, SDL_Texture, SDL_UnlockTexture, SDL_TEXTUREACCESS_STREAMING}, video::{SDL_GetWindowPixelFormat, SDL_Window, SDL_WINDOW_HIGH_PIXEL_DENSITY}}};

struct State {
    // video
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,

    // scaling
    texture_width: usize,
    texture_height: usize,
    nes_width: usize,
    nes_height: usize,
    nes_pitch: usize,

    // audio
    stream: *mut SDL_AudioStream,

    // nes
    nes: Nes<Controller>,
    controller_state: Cell<ControllerState>,

    // debug
    now: VecDeque<Instant>,
}

enum AppResult<T> {
    Continue(T),
    Success,
    Failure,
}

fn init() -> AppResult<Box<State>> {
    // check if we have provided an argument
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return AppResult::Failure;
    }
    
    // controller init
    let controller_state = Cell::new(ControllerState::new());

    // nes init
    let game = fs::read(&args[1]).unwrap();
    let nes = Nes::load_from_memory(&game[..])
        .unwrap();
    
    if !unsafe { SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) } {
        return AppResult::Failure;
    }

    let mut state = Box::new(State {
        window: null_mut(),
        renderer: null_mut(),
        texture: null_mut(),

        texture_width: 0,
        texture_height: 0,
        nes_width: 256,
        nes_height: 240,
        nes_pitch: 256*4,
        
        stream: null_mut(),

        nes,
        controller_state,

        now: VecDeque::with_capacity(2048),
    });

    let controller = Controller::new(NonNull::new(&raw mut state.controller_state).unwrap());
    state.nes.connect(0, controller);

    if !unsafe { SDL_CreateWindowAndRenderer(c"nes".as_ptr(), 240*4, 240*3, SDL_WINDOW_HIGH_PIXEL_DENSITY, &mut state.window, &mut state.renderer) } {
        return AppResult::Failure;
    }

    let mut w = MaybeUninit::uninit();
    let mut h = MaybeUninit::uninit();
    if !unsafe { SDL_GetRenderOutputSize(state.renderer, w.as_mut_ptr(), h.as_mut_ptr()) } {
        return AppResult::Failure;
    }
    let w = unsafe { w.assume_init() };
    let h = unsafe { h.assume_init() };

    state.texture = unsafe { SDL_CreateTexture(state.renderer, SDL_PIXELFORMAT_BGRA32, SDL_TEXTUREACCESS_STREAMING, w, h) };
    if state.texture == null_mut() {
        return AppResult::Failure;
    }
    state.texture_width = w as usize;
    state.texture_height = h as usize;

    // audio init
    let audio_spec = SDL_AudioSpec {
        format: SDL_AUDIO_S16,
        channels: 2,
        freq: 48000,
    };
    state.stream = unsafe { SDL_OpenAudioDeviceStream(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, &audio_spec, None, null_mut()) };
    unsafe { SDL_ResumeAudioStreamDevice(state.stream) };

    AppResult::Continue(state)
}

fn iterate(state: &mut State) -> AppResult<()> {
    // ensure the latency doesn't get any higher than 64 ms
    if unsafe { SDL_GetAudioStreamAvailable(state.stream) } < (48000*64*4)/1000 - 798*4 {
        // debug: get framerate
        let now = Instant::now();
        if state.now.len() == 2048 {
            state.now.pop_front();
        }
        state.now.push_back(now);
        // get the average
        if state.now.len() >= 2 {
            let mut sum = Duration::ZERO;
            for i in 1..state.now.len() {
                sum += state.now[i] - state.now[i-1];
            }
            let avg = 1.0 / (sum / state.now.len() as u32).as_secs_f64();
            eprintln!("framerate: {}", avg);
        }

        state.nes.run();

        // queue up new audio
        let mut buf = [0; 798*2];
        state.nes.play_audio(&mut buf);
        unsafe { SDL_PutAudioStreamData(state.stream, &buf as *const [i16] as _, (buf.len()*size_of::<i16>()) as _) };

        let src = state.nes.framebuffer();
        let mut pixels = MaybeUninit::uninit();
        let mut pitch = MaybeUninit::uninit();
        if unsafe { SDL_LockTexture(state.texture, null(), pixels.as_mut_ptr(), pitch.as_mut_ptr()) } {
            let pixels = unsafe { pixels.assume_init() } as *mut u8;
            let pitch = unsafe { pitch.assume_init() } as usize;

            // sdl doesn't have nearest neighbor filtering...
            // do it ourselves
            let dst = unsafe { slice::from_raw_parts_mut(
                pixels, state.texture_height * pitch
            ) };
            for y in 0..state.texture_height {
                let dst_y = y*pitch;
                let src_y = (y*state.nes_height/state.texture_height)*state.nes_pitch;
                for x in 0..state.texture_width {
                    // could be more optimizied?
                    let dst_x = x<<2;
                    let src_x = (x*state.nes_width/state.texture_width)<<2;
                    dst[dst_y+dst_x] = src[src_y+src_x];
                    dst[dst_y+dst_x+1] = src[src_y+src_x+1];
                    dst[dst_y+dst_x+2] = src[src_y+src_x+2];
                    dst[dst_y+dst_x+3] = src[src_y+src_x+3];
                }
            }
            unsafe { SDL_UnlockTexture(state.texture) };
        }
    }

    unsafe { SDL_RenderTexture(state.renderer, state.texture, null(), null()) };

    unsafe { SDL_RenderPresent(state.renderer) };

    AppResult::Continue(())
}

fn event(state: &mut State, event: &mut Event) -> AppResult<()> {
    // looks like event is called on the main thread (on macos)
    let mut controller_state = state.controller_state.get();

    match event {
        Event::Quit {..} => return AppResult::Success,

        Event::KeyDown { keycode: Some(Keycode::X), .. } => {
            controller_state.set_a(true);
        },
        Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
            controller_state.set_b(true);
        },
        Event::KeyDown { keycode: Some(Keycode::RShift), .. } => {
            controller_state.set_select(true);
        },
        Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
            controller_state.set_start(true);
        },
        Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
            controller_state.set_up(true);
        },
        Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
            controller_state.set_down(true);
        },
        Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
            controller_state.set_left(true);
        },
        Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
            controller_state.set_right(true);
        },

        Event::KeyUp { keycode: Some(Keycode::X), .. } => {
            controller_state.set_a(false);
            state.controller_state.set(controller_state);
        },
        Event::KeyUp { keycode: Some(Keycode::Z), .. } => {
            controller_state.set_b(false);
        },
        Event::KeyUp { keycode: Some(Keycode::RShift), .. } => {
            controller_state.set_select(false);
        },
        Event::KeyUp { keycode: Some(Keycode::Return), .. } => {
            controller_state.set_start(false);
        },
        Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
            controller_state.set_up(false);
        },
        Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
            controller_state.set_down(false);
        },
        Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
            controller_state.set_left(false);
        },
        Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
            controller_state.set_right(false);
        },

        _ => {},
    };

    state.controller_state.set(controller_state);
    AppResult::Continue(())
}

fn quit(state: Option<Box<State>>, _result: AppResult<()>) {
    if let Some(state) = state {
        unsafe { SDL_DestroyTexture(state.texture) };
    }
}

// controller init
#[derive(Copy, Clone)]
struct ControllerState {
    state: u8,
}

impl ControllerState {
    pub const fn new() -> Self {
        Self {
            state: 0x00,
        }
    }

    pub const fn clear(&mut self) {
        self.set_a(false);
        self.set_b(false);
        self.set_select(false);
        self.set_start(false);
        self.set_up(false);
        self.set_down(false);
        self.set_left(false);
        self.set_right(false);
    }

    pub const fn a(&self) -> bool {
        (self.state & (1 << 0)) != 0
    }

    pub const fn b(&self) -> bool {
        (self.state & (1 << 1)) != 0
    }

    pub const fn select(&self) -> bool {
        (self.state & (1 << 2)) != 0
    }

    pub const fn start(&self) -> bool {
        (self.state & (1 << 3)) != 0
    }

    pub const fn up(&self) -> bool {
        (self.state & (1 << 4)) != 0
    }

    pub const fn down(&self) -> bool {
        (self.state & (1 << 5)) != 0
    }

    pub const fn left(&self) -> bool {
        (self.state & (1 << 6)) != 0
    }

    pub const fn right(&self) -> bool {
        (self.state & (1 << 7)) != 0
    }

    pub const fn set_a(&mut self, value: bool) {
        self.state = (self.state & !(1 << 0)) | ((value as u8) << 0);
    }

    pub const fn set_b(&mut self, value: bool) {
        self.state = (self.state & !(1 << 1)) | ((value as u8) << 1);
    }

    pub const fn set_select(&mut self, value: bool) {
        self.state = (self.state & !(1 << 2)) | ((value as u8) << 2);
    }

    pub const fn set_start(&mut self, value: bool) {
        self.state = (self.state & !(1 << 3)) | ((value as u8) << 3);
    }

    pub const fn set_up(&mut self, value: bool) {
        self.state = (self.state & !(1 << 4)) | ((value as u8) << 4);
    }

    pub const fn set_down(&mut self, value: bool) {
        self.state = (self.state & !(1 << 5)) | ((value as u8) << 5);
    }

    pub const fn set_left(&mut self, value: bool) {
        self.state = (self.state & !(1 << 6)) | ((value as u8) << 6);
    }

    pub const fn set_right(&mut self, value: bool) {
        self.state = (self.state & !(1 << 7)) | ((value as u8) << 7);
    }
}

struct Controller {
    state: NonNull<Cell<ControllerState>>,
    a: bool,
    b: bool,
    select: bool,
    start: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Controller {
    pub const fn new(state: NonNull<Cell<ControllerState>>) -> Self {
        Self {
            state,
            a: false,
            b: false,
            select: false,
            start: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

impl nes::Controller for Controller {
    fn poll(&mut self) {
        // this is done in the sdl event loop
        let state = unsafe { self.state.as_ref().get() };
        self.a = state.a();
        self.b = state.b();
        self.select = state.select();
        self.start = state.start();
        self.up = state.up();
        self.down = state.down();
        self.left = state.left();
        self.right = state.right();
    }

    fn a(&self) -> bool {
        self.a
    }

    fn b(&self) -> bool {
        self.b
    }

    fn select(&self) -> bool {
        self.select
    }

    fn start(&self) -> bool {
        self.start
    }

    fn up(&self) -> bool {
        self.up
    }
    
    fn down(&self) -> bool {
        self.down
    }

    fn left(&self) -> bool {
        self.left
    }

    fn right(&self) -> bool {
        self.right
    }
}

// sdl entrypoints

#[allow(non_snake_case)]
extern "C" fn SDL_AppInit(appstate: *mut *mut c_void, _argc: c_int, _argv: *mut *mut c_char) -> SDL_AppResult {
    // we want to ensure that SDL_AppIterate and SDL_AppEvent
    // are called with a valid state, while also not requiring
    // the user to return a state if the program ends immediately
    // on SDL_AppInit
    match init() {
        AppResult::Continue(state) => {
            unsafe { (*(appstate as *mut MaybeUninit<Option<Box<State>>>)).write(Some(state)) };
            SDL_APP_CONTINUE
        },
        AppResult::Success => {
            unsafe { (*(appstate as *mut MaybeUninit<Option<Box<State>>>)).write(None) };
            SDL_APP_SUCCESS
        },
        AppResult::Failure => {
            unsafe { (*(appstate as *mut MaybeUninit<Option<Box<State>>>)).write(None) };
            SDL_APP_FAILURE
        }
    }
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppIterate(appstate: *mut c_void) -> SDL_AppResult {
    let state = unsafe { (*(&raw const appstate as *mut Box<State>)).as_mut() };

    match iterate(state) {
        AppResult::Continue(..) => SDL_APP_CONTINUE,
        AppResult::Success => SDL_APP_SUCCESS,
        AppResult::Failure => SDL_APP_FAILURE,
    }
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppEvent(appstate: *mut c_void, e: *mut SDL_Event) -> SDL_AppResult {
    let state = unsafe { (*(&raw const appstate as *mut Box<State>)).as_mut() };
    let mut e = Event::from_ll(unsafe { *e });

    match event(state, &mut e) {
        AppResult::Continue(..) => SDL_APP_CONTINUE,
        AppResult::Success => SDL_APP_SUCCESS,
        AppResult::Failure => SDL_APP_FAILURE,
    }
}

#[allow(non_snake_case)]
extern "C" fn SDL_AppQuit(appstate: *mut c_void, result: SDL_AppResult) {
    let state = unsafe { std::mem::take(&mut *(&raw const appstate as *mut Option<Box<State>>)) };
    let result = match result {
        SDL_APP_CONTINUE => AppResult::Continue(()),
        SDL_APP_SUCCESS => AppResult::Success,
        SDL_APP_FAILURE => AppResult::Failure,
        _ => unreachable!(),
    };

    quit(state, result)
}

#[allow(non_snake_case)]
unsafe extern "C" fn SDL_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    SDL_EnterAppMainCallbacks(argc, argv, Some(SDL_AppInit), Some(SDL_AppIterate), Some(SDL_AppEvent), Some(SDL_AppQuit))
}

fn main() -> ExitCode {
    // we have access to args through std::env::args(),
    // so no need to pass them along here
    ExitCode::from(unsafe { SDL_RunApp(0, null_mut(), Some(SDL_main), null_mut()) } as u8)
}
