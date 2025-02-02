use std::{cell::Cell, ptr::NonNull, slice};

use apu::Apu;
use cpu::Cpu;
use mem::Memory;
use ppu::Ppu;

mod retro;
mod nsf;
mod ffi;

pub mod apu;
pub mod cpu;
pub mod mem;
pub mod ppu;

pub trait Controller {
    fn poll(&mut self);

    fn a(&self) -> bool;
    fn b(&self) -> bool;
    fn select(&self) -> bool;
    fn start(&self) -> bool;
    fn up(&self) -> bool;
    fn down(&self) -> bool;
    fn left(&self) -> bool;
    fn right(&self) -> bool;
}

pub struct Nes<C: Controller> {
    mem: Box<Memory<C>>,
    apu: Box<Apu>,
    cpu: Box<Cpu<C>>,
    ppu: Box<Ppu>,

    cycles: Box<Cell<usize>>,
}

impl<C: Controller> Nes<C> {
    pub fn load_from_memory(game: &[u8]) -> Option<Self> {
        // todo: check ines header
        let hdr = &game[..0x10];
        if hdr[0] != b'N' || hdr[1] != b'E' || hdr[2] != b'S' || hdr[3] != b'\x1a' {
            return None;
        }

        // size of prg rom in 16 kb units
        let prg_len = hdr[4];
        if prg_len != 0x02 {
            return None;
        }

        // size of chr rom in 8 kb units
        let chr_len = hdr[5];
        if chr_len != 0x01 {
            return None;
        }

        if hdr[6] != 0x01 {
            return None;
        }

        if hdr[7] != 0x00 {
            return None;
        }

        if hdr[8] != 0x00 {
            return None;
        }

        if hdr[9] != 0x00 {
            return None;
        }

        if hdr[10] != 0x00 {
            return None;
        }

        let text = &game[0x10..0x10+0x8000];
        let chr = &game[0x10+0x8000..0x10+0x8000+0x2000];

        let mut cycles = Box::new(Cell::new(0));
        let mut apu = Box::new(Apu::new());
        let mut ppu = Box::new(Ppu::new(chr, NonNull::new(cycles.as_mut()).unwrap()));
        let mut mem = Box::new(Memory::new(text, NonNull::new(apu.as_mut()).unwrap(), NonNull::new(ppu.as_mut()).unwrap()));
        let cpu = Box::new(Cpu::new(NonNull::new(mem.as_mut()).unwrap(), NonNull::new(cycles.as_mut()).unwrap()));
        Some(Self {
            mem,
            apu,
            cpu,
            ppu,
            cycles,
        })
    }

    pub fn run(&mut self) {
        // runs for one frame
        let mut nmi_sent = false;
        let mut screen_drawn = false;
        while self.cycles.get() < 29781 {
            if self.cycles.get() > 27252 && screen_drawn == false {
                // the entire screen has been drawn
                screen_drawn = true;
                self.ppu.clear();
                self.ppu.draw();
            }
            if self.cycles.get() >= 27280 && nmi_sent == false {
                nmi_sent = true;
                if (self.ppu.ppuctrl & 0x80) != 0 {
                    eprintln!("nmi!");
                    self.cpu.nmi();
                }
            }
            self.cpu.execute();
        }
        self.cycles.set(0);
    }

    pub fn framebuffer(&mut self) -> &[u8] {
        // todo: fix this
        unsafe {
            slice::from_raw_parts(
                self.ppu.framebuffer().as_ptr() as _,
                self.ppu.framebuffer().width() * self.ppu.framebuffer().height() * 4,
            )
        }
    }

    pub fn play_audio(&mut self, buf: &mut [i16]) {
        self.apu.tick(buf)
    }

    pub fn connect(&mut self, port: usize, controller: C) {
        self.mem.connect_controller(port, controller);
    }
}
