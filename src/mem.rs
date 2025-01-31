use std::ptr::NonNull;

use crate::{apu::Apu, ppu::Ppu, Controller};

pub trait Mem {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    #[inline]
    fn read16(&mut self, addr: u16) -> u16 {
        // default
        (self.read(addr) as u16) |
            ((self.read(addr.wrapping_add(1)) as u16) << 8)
    }
    #[inline]
    fn write16(&mut self, addr: u16, value: u16) {
        // default
        self.write(addr, value as u8);
        self.write(addr.wrapping_add(1), (value >> 8) as u8)
    }
}

pub struct Memory<'a> {
    mem: Box<[u8]>,
    text: Box<[u8]>,
    
    apu: NonNull<Apu>,
    ppu: NonNull<Ppu>,

    // controllers
    c_strobe: bool,
    c1: Option<&'a mut dyn Controller>,
    c1_index: u8,
    c2: Option<&'a mut dyn Controller>,
    c2_index: u8,
}

impl Memory<'_> {
    pub fn new(text: &[u8], apu: NonNull<Apu>, ppu: NonNull<Ppu>) -> Self {
        Self {
            mem: unsafe { Box::new_uninit_slice(0x800).assume_init() },
            text: unsafe {
                let mut t = Box::new_uninit_slice(text.len()).assume_init();
                t.copy_from_slice(text);
                t
            },
            apu,
            ppu,
            c_strobe: false,
            c1: None,
            c1_index: 0,
            c2: None,
            c2_index: 0,
        }
    }
}

impl<'a> Memory<'a> {
    pub fn connect_controller(&mut self, port: usize, controller: &'a mut dyn Controller) {
        match port {
            0 => self.c1 = Some(controller),
            1 => self.c2 = Some(controller),
            _ => {},
        }
    }
}

impl Mem for Memory<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        let apu = unsafe { self.apu.as_mut() };
        let ppu = unsafe { self.ppu.as_mut() };
        match addr {
            0x0..=0x1fff => self.mem[(addr & 0x7ff) as usize],
            0x2000..=0x3fff => {
                // ppu registers mirror every 8 bytes
                match addr & 0x7 {
                    0x0 => panic!("memory read out of range: ppuctrl"),
                    0x1 => panic!("memory read out of range: ppumask"),
                    0x2 => ppu.read_ppustatus(),
                    0x3 => panic!("memory read out of range: oamaddr"),
                    0x4 => { eprintln!("read from oamdata!"); 0 },
                    0x5 => panic!("memory read out of range: ppuscroll"),
                    0x6 => panic!("memory read out of range: ppuaddr"),
                    0x7 => ppu.read_ppudata(),
                    _ => unreachable!(),
                }
            },
            0x4000 => panic!("memory read out of range: sq1_vol"),
            0x4001 => panic!("memory read out of range: sq1_sweep"),
            0x4002 => panic!("memory read out of range: sq1_lo"),
            0x4003 => panic!("memory read out of range: sq1_hi"),
            0x4004 => panic!("memory read out of range: sq2_vol"),
            0x4005 => panic!("memory read out of range: sq2_sweep"),
            0x4006 => panic!("memory read out of range: sq2_lo"),
            0x4007 => panic!("memory read out of range: sq2_hi"),
            0x4008 => panic!("memory read out of range: tri_linear"),
            0x400a => panic!("memory read out of range: tri_lo"),
            0x400b => panic!("memory read out of range: tri_hi"),
            0x400c => panic!("memory read out of range: noise_vol"),
            0x400e => panic!("memory read out of range: noise_lo"),
            0x400f => panic!("memory read out of range: noise_hi"),
            0x4010 => panic!("memory read out of range: dmc_freq"),
            0x4011 => panic!("memory read out of range: dmc_raw"),
            0x4012 => panic!("memory read out of range: dmc_start"),
            0x4013 => panic!("memory read out of range: dmc_len"),
            0x4014 => panic!("memory read out of range: oamdma"),
            0x4015 => apu.read_snd_chn(),
            0x4016 => {
                if self.c_strobe {
                    // just return a
                    if let Some(c1) = &mut self.c1 {
                        c1.poll();
                        0x40 | (c1.a() as u8)
                    } else {
                        0x40
                    }
                } else {
                    // return next input
                    let state;
                    if let Some(c1) = &self.c1 {
                        state = match self.c1_index {
                            0 => c1.a(),
                            1 => c1.b(),
                            2 => c1.select(),
                            3 => c1.start(),
                            4 => c1.up() && !c1.down(),
                            5 => c1.down() && !c1.up(),
                            6 => c1.left() && !c1.right(),
                            7 => c1.right() && !c1.left(),
                            _ => true,
                        };
                    } else {
                        state = if self.c1_index < 8 { false } else { true };
                    }
                    if self.c1_index < 8 { self.c1_index += 1; }
                    0x40 | (state as u8)
                }
            },
            0x4017 => {
                if self.c_strobe {
                    // just return a
                    if let Some(c2) = &mut self.c2 {
                        c2.poll();
                        0x40 | (c2.a() as u8)
                    } else {
                        0x40
                    }
                } else {
                    // return next input
                    let state;
                    if let Some(c2) = &self.c2 {
                        state = match self.c2_index {
                            0 => c2.a(),
                            1 => c2.b(),
                            2 => c2.select(),
                            3 => c2.start(),
                            4 => c2.up() && !c2.down(),
                            5 => c2.down() && !c2.up(),
                            6 => c2.left() && !c2.right(),
                            7 => c2.right() && !c2.left(),
                            _ => true,
                        };
                    } else {
                        state = if self.c2_index < 8 { false } else { true };
                    }
                    if self.c2_index < 8 { self.c2_index += 1; }
                    0x40 | (state as u8)
                }
            },
            0x8000..=0xffff => self.text[(addr & 0x7fff) as usize],
            _ => panic!("memory read out of range: ${:x}", addr),
        }
    }
    fn write(&mut self, addr: u16, value: u8) {
        let apu = unsafe { self.apu.as_mut() };
        let ppu = unsafe { self.ppu.as_mut() };
        match addr {
            0x0..=0x1fff => self.mem[(addr & 0x7ff) as usize] = value,
            0x2000..=0x3fff =>
                // ppu registers mirror every 8 bytes
                match addr & 0x7 {
                    0x0 => ppu.write_ppuctrl(value),
                    0x1 => ppu.write_ppumask(value),
                    0x2 => panic!("memory write out of range: ppustatus"),
                    0x3 => eprintln!("write to oamaddr: 0x{:x}!", value),
                    0x4 => eprintln!("write to oamdata: 0x{:x}!", value),
                    0x5 => ppu.write_ppuscroll(value),
                    0x6 => ppu.write_ppuaddr(value),
                    0x7 => ppu.write_ppudata(value),
                    _ => unreachable!(),
                },
            0x4000 => apu.write_sq1_vol(value),
            0x4001 => apu.write_sq1_sweep(value),
            0x4002 => apu.write_sq1_lo(value),
            0x4003 => apu.write_sq1_hi(value),
            0x4004 => apu.write_sq2_vol(value),
            0x4005 => apu.write_sq2_sweep(value),
            0x4006 => apu.write_sq2_lo(value),
            0x4007 => apu.write_sq2_hi(value),
            0x4008 => apu.write_tri_linear(value),
            0x400a => apu.write_tri_lo(value),
            0x400b => apu.write_tri_hi(value),
            0x400c => apu.write_noise_vol(value),
            0x400e => apu.write_noise_lo(value),
            0x400f => apu.write_noise_hi(value),
            0x4010 => apu.write_dmc_freq(value),
            0x4011 => apu.write_dmc_raw(value),
            0x4012 => apu.write_dmc_start(value),
            0x4013 => apu.write_dmc_len(value),
            0x4014 => {
                // oamdma
                for addr in ((value as u16) << 8)..=((value as u16) << 8)|0xff {
                    let value = self.read(addr);
                    ppu.write_oamdata(value);
                }
            }
            0x4015 => apu.write_snd_chn(value),
            0x4016 => {
                self.c_strobe = (value & 1) != 0;
                // poll controllers and reset shift registers
                if !self.c_strobe {
                    if let Some(c1) = &mut self.c1 {
                        c1.poll();
                    }
                    if let Some(c2) = &mut self.c2 {
                        c2.poll();
                    }
                    self.c1_index = 0;
                    self.c2_index = 0;
                }
            },
            0x4017 => apu.write_joy2(value),
            _ => panic!("memory write out of range: ${:x}", addr),
        }
    }
}
