use std::{cell::Cell, ptr::NonNull};

use gfx::{Color, Framebuffer, Texture};

// from mesen
const PALETTE: [Color; 0x40] = [
    Color::new(0x66, 0x66, 0x66),
    Color::new(0x00, 0x2a, 0x88),
    Color::new(0x14, 0x12, 0xa7),
    Color::new(0x3b, 0x00, 0xa4),
    Color::new(0x5c, 0x00, 0x7e),
    Color::new(0x6e, 0x00, 0x40),
    Color::new(0x6c, 0x06, 0x00),
    Color::new(0x56, 0x1d, 0x00),
    Color::new(0x33, 0x35, 0x00),
    Color::new(0x0b, 0x48, 0x00),
    Color::new(0x00, 0x52, 0x00),
    Color::new(0x00, 0x4f, 0x08),
    Color::new(0x00, 0x40, 0x4d),
    Color::new(0x00, 0x00, 0x00),
    Color::new(0x00, 0x00, 0x00),
    Color::new(0x00, 0x00, 0x00),

    Color::new(0xad, 0xad, 0xad),
    Color::new(0x15, 0x5f, 0xd9),
    Color::new(0x42, 0x40, 0xff),
    Color::new(0x75, 0x27, 0xfe),
    Color::new(0xa0, 0x1a, 0xcc),
    Color::new(0xb7, 0x1e, 0x7b),
    Color::new(0xb5, 0x31, 0x20),
    Color::new(0x99, 0x4e, 0x00),
    Color::new(0x6b, 0x6d, 0x00),
    Color::new(0x38, 0x87, 0x00),
    Color::new(0x0c, 0x93, 0x00),
    Color::new(0x00, 0x8f, 0x32),
    Color::new(0x00, 0x7c, 0x8d),
    Color::new(0x00, 0x00, 0x00),
    Color::new(0x00, 0x00, 0x00),
    Color::new(0x00, 0x00, 0x00),

    Color::new(0xff, 0xfe, 0xff),
    Color::new(0x64, 0xb0, 0xff),
    Color::new(0x92, 0x90, 0xff),
    Color::new(0xc6, 0x76, 0xff),
    Color::new(0xf3, 0x6a, 0xff),
    Color::new(0xfe, 0x6e, 0xcc),
    Color::new(0xfe, 0x81, 0x70),
    Color::new(0xea, 0x9e, 0x22),
    Color::new(0xbc, 0xbe, 0x00),
    Color::new(0x88, 0xd8, 0x00),
    Color::new(0x5c, 0xe4, 0x30),
    Color::new(0x45, 0xe0, 0x82),
    Color::new(0x48, 0xcd, 0xde),
    Color::new(0x4f, 0x4f, 0x4f),
    Color::new(0x00, 0x00, 0x00),
    Color::new(0x00, 0x00, 0x00),

    Color::new(0xff, 0xfe, 0xff),
    Color::new(0xc0, 0xdf, 0xff),
    Color::new(0xd3, 0xd2, 0xff),
    Color::new(0xe8, 0xc8, 0xff),
    Color::new(0xfb, 0xc2, 0xff),
    Color::new(0xfe, 0xc4, 0xea),
    Color::new(0xfe, 0xcc, 0xc5),
    Color::new(0xf7, 0xd8, 0xa5),
    Color::new(0xe4, 0xe5, 0x94),
    Color::new(0xcf, 0xef, 0x96),
    Color::new(0xbd, 0xf4, 0xab),
    Color::new(0xb3, 0xf3, 0xcc),
    Color::new(0xb5, 0xeb, 0xf2),
    Color::new(0xb8, 0xb8, 0xb8),
    Color::new(0x00, 0x00, 0x00),
    Color::new(0x00, 0x00, 0x00),
];

// PPU
pub struct Ppu {
    framebuffer: Framebuffer,
    chr: Box<[u8]>,
    chr0: Texture<u8>,
    chr1: Texture<u8>,
    bg: Color,
    palette: Texture<Color>,
    mem: [u8; 0x800],
    pal: [u8; 0x20],

    oam: [u8; 0x100],

    // register related values
    latch: bool,
    pub ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    oamaddr: u8,
    ppuscroll_x: u8,
    ppuscroll_y: u8,
    ppuaddr: u16,
    ppudata_buf: u8,

    // cycles
    cycles: NonNull<Cell<usize>>,
}

impl Ppu {
    pub fn new(chr: &[u8], cycles: NonNull<Cell<usize>>) -> Self {
        Self {
            framebuffer: Framebuffer::new(256, 240),
            chr: unsafe {
                let mut c = Box::new_uninit_slice(chr.len()).assume_init();
                c.copy_from_slice(chr);
                c
            },
            chr0: Texture::from_2bpp(&chr[0..0x1000], 0x80, 0x80),
            chr1: Texture::from_2bpp(&chr[0x1000..0x2000], 0x80, 0x80),
            bg: PALETTE[0],
            palette: Texture::new(&[
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                Color::TRANSPARENT, PALETTE[0], PALETTE[0], PALETTE[0],
                ], 4, 8),
            mem: [0; 0x800],
            pal: [0; 0x20],
            oam: [0; 0x100],

            latch: false,
            ppuctrl: 0,
            ppumask: 0,
            ppustatus: 0,
            oamaddr: 0,
            ppuscroll_x: 0,
            ppuscroll_y: 0,
            ppuaddr: 0,
            ppudata_buf: 0,

            cycles,
        }
    }
}

impl Ppu {
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    pub fn clear(&mut self) {
        self.framebuffer.clear(self.bg);
    }

    fn draw_bg(&mut self, bg: usize) {
        for i in 0x0..0x3c0 {
            let tile = self.mem[(bg<<10)|i];
            let pal = (self.mem[(bg<<10)|0x3c0|((i>>4)&0x38)|((i>>2)&0x7)]>>(((i>>4)&0x4)|(i&0x2)))&0x3;
            let u = ((tile&0xf)<<3) as usize;
            let v = ((tile>>4)<<3) as usize;
            let x = (((i&0x1f)<<3) as isize) - (self.ppuscroll_x as isize) + ((((self.ppuctrl & 0x01) ^ ((bg & 0x01) as u8)) as isize) << 8);
            let y = (((i>>5)<<3) as isize) - (self.ppuscroll_y as isize);
            self.framebuffer.draw_paletted(&self.chr1, x, y, u, v, 8, 8, &self.palette, pal as usize, false, false);
        }
    }

    fn draw_sprite(&mut self, spr: usize) {
        let y = self.oam[(spr<<2)|0] as isize + 1;
        let tile = self.oam[(spr<<2)|1];
        let pal = self.oam[(spr<<2)|2] & 0x03;
        let flip_x = (self.oam[(spr<<2)|2] & 0x40) != 0;
        let flip_y = (self.oam[(spr<<2)|2] & 0x80) != 0;
        let x = self.oam[(spr<<2)|3] as isize;

        let u = ((tile as usize)&0xf)<<3;
        let v = ((tile as usize)>>4)<<3;

        self.framebuffer.draw_paletted(&self.chr0, x, y, u, v, 8, 8, &self.palette, 4+pal as usize, flip_x, flip_y);
    }

    pub fn draw(&mut self) {
        // sprites behind the background
        if (self.ppumask & 0x10) != 0 {
            for spr in (0x0..0x40).rev() {
                if (self.oam[(spr<<2)|2] & 0x20) != 0 {
                    self.draw_sprite(spr);
                }
            }
        }
        
        if (self.ppumask & 0x08) != 0 {
            for bg in 0..2 {
                self.draw_bg(bg);
            }
        }

        // sprites in front of the background
        if (self.ppumask & 0x10) != 0 {
            for spr in (0x0..0x40).rev() {
                if (self.oam[(spr<<2)|2] & 0x20) == 0 {
                    self.draw_sprite(spr);
                }
            }
        }
    }

    pub fn write_ppuctrl(&mut self, value: u8) {
        self.ppuctrl = value;
    }

    pub fn write_ppumask(&mut self, value: u8) {
        self.ppumask = value;
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        // check for sprite 0 hit
        // todo: account for switching nametables
        // todo: account for screen scroll
        // todo: account for sprite flipping vertically/horizontally
        let cycles = unsafe { self.cycles.as_ref().get() };
        let y = (cycles * 3) / 341;
        let x = (cycles * 3) % 341;
        let spr0_y = self.oam[0] as usize + 1;
        let spr0_tile = self.oam[1] as usize;
        let spr0_x = self.oam[3] as usize;
        let mut spr0hit = false;
        for i in spr0_y..(spr0_y+8).min(y).min(0xf0) {
            for j in spr0_x..(spr0_x+8).min(0x100) {
                let tile = self.mem[((i>>3)<<5)|(j>>3)] as usize;
                if self.chr1[((tile>>4)<<3)+(i&0x7)][((tile&0xf)<<3)+(j&0xf)] != 0
                    && self.chr0[((spr0_tile>>4)<<3)+(i-spr0_y)][((spr0_tile&0xf)<<3)+(j-spr0_x)] != 0 {
                    spr0hit = true;
                    break;
                }
            }
        }
        if y >= spr0_y && y < (spr0_y+8).min(0xf0) {
            for j in spr0_x..(spr0_x+8).min(x+1).min(0x100) {
                let tile = self.mem[((y>>3)<<5)|(j>>3)] as usize;
                if self.chr1[((tile>>4)<<3)+(y&0x7)][((tile&0xf)<<3)+(j&0xf)] != 0
                    && self.chr0[((spr0_tile>>4)<<3)+(y-spr0_y)][((spr0_tile&0xf)<<3)+(j-spr0_x)] != 0 {
                    spr0hit = true;
                    break;
                }
            }
        }

        self.ppustatus = (((y>240) as u8)<<7)|((spr0hit as u8)<<6);

        self.latch = false;

        self.ppustatus
    }

    pub fn write_oamaddr(&mut self, value: u8) {
        self.oamaddr = value;
    }

    pub fn read_oamdata(&self) -> u8 {
        self.oam[self.oamaddr as usize]
    }

    pub fn write_oamdata(&mut self, value: u8) {
        self.oam[self.oamaddr as usize] = value;
        self.oamaddr = self.oamaddr.wrapping_add(1);
    }

    pub fn write_ppuscroll(&mut self, value: u8) {
        if !self.latch {
            self.ppuscroll_x = value;
        } else {
            self.ppuscroll_y = value;
        }
        self.latch ^= true;
    }

    pub fn write_ppuaddr(&mut self, value: u8) {
        if !self.latch {
            self.ppuaddr = ((value as u16) << 8) | (self.ppuaddr & 0x00ff);
        } else {
            self.ppuaddr = (self.ppuaddr & 0xff00) | (value as u16);
        }
        self.latch ^= true;
    }

    pub fn read_ppudata(&mut self) -> u8 {
        let ppuaddr = self.ppuaddr;
        self.ppuaddr = self.ppuaddr.wrapping_add(if (self.ppuctrl & 0x04) == 0 { 0x01 } else { 0x20 });
        let ppudata = self.ppudata_buf;
        self.ppudata_buf = match ppuaddr & 0x3fff {
            0x0000..0x1000 => {
                // pattern table 1
                self.chr[ppuaddr as usize]
                // let chr = &self.chr0;
                // let y = (((ppuaddr>>5)&0x78)|(ppuaddr&7)) as usize;
                // let x = ((ppuaddr>>1)&0x78) as usize;
                // let shift = ((ppuaddr>>3)&1) as usize;
                // (((chr[y][x] >> shift) & 1) << 7)
                //     | (((chr[y][x|1] >> shift) & 1) << 6)
                //     | (((chr[y][x|2] >> shift) & 1) << 5)
                //     | (((chr[y][x|3] >> shift) & 1) << 4)
                //     | (((chr[y][x|4] >> shift) & 1) << 3)
                //     | (((chr[y][x|5] >> shift) & 1) << 2)
                //     | (((chr[y][x|6] >> shift) & 1) << 1)
                //     | ((chr[y][x|7] >> shift) & 1)
            },
            0x1000..0x2000 => {
                // pattern table 2
                self.chr[ppuaddr as usize]
                // let chr = &self.chr1;
                // let y = (((ppuaddr>>5)&0x78)|(ppuaddr&7)) as usize;
                // let x = ((ppuaddr>>1)&0x78) as usize;
                // let shift = ((ppuaddr>>3)&1) as usize;
                // (((chr[y][x] >> shift) & 1) << 7)
                //     | (((chr[y][x|1] >> shift) & 1) << 6)
                //     | (((chr[y][x|2] >> shift) & 1) << 5)
                //     | (((chr[y][x|3] >> shift) & 1) << 4)
                //     | (((chr[y][x|4] >> shift) & 1) << 3)
                //     | (((chr[y][x|5] >> shift) & 1) << 2)
                //     | (((chr[y][x|6] >> shift) & 1) << 1)
                //     | ((chr[y][x|7] >> shift) & 1)
            },
            0x2000..0x2400 => {
                // nametable 1
                self.mem[(ppuaddr & 0x7ff) as usize]
            },
            0x2400..0x2800 => {
                // nametable 2
                self.mem[(ppuaddr & 0x7ff) as usize]
            },
            0x2800..0x2c00 => {
                // nametable 3
                self.mem[(ppuaddr & 0x7ff) as usize]
            },
            0x2c00..0x3000 => {
                // nametable 4
                self.mem[(ppuaddr & 0x7ff) as usize]
            },
            0x3f00..0x4000 => {
                // palette ram
                self.pal[(ppuaddr & 0x1f) as usize]
            },
            _ => panic!("memory read out of range: 0x{:x}", ppuaddr),
        };

        ppudata
    }

    pub fn write_ppudata(&mut self, value: u8) {
        let ppuaddr = self.ppuaddr;
        self.ppuaddr = self.ppuaddr.wrapping_add(if (self.ppuctrl & 0x04) == 0 { 0x01 } else { 0x20 });
        match ppuaddr & 0x3fff {
            0x2000..0x2400 => {
                // nametable 1
                self.mem[(ppuaddr & 0x7ff) as usize] = value;
            },
            0x2400..0x2800 => {
                // nametable 2
                self.mem[(ppuaddr & 0x7ff) as usize] = value;
            },
            0x2800..0x2c00 => {
                // nametable 3
                self.mem[(ppuaddr & 0x7ff) as usize] = value;
            },
            0x2c00..0x3000 => {
                // nametable 4
                self.mem[(ppuaddr & 0x7ff) as usize] = value;
            },
            0x3f00..0x4000 => {
                // palette ram
                let mut addr = ppuaddr & 0x1f;
                if (addr & 0x13) == 0x10 { addr &= 0xf; }
                self.pal[addr as usize] = value;
                if addr == 0x0 {
                    self.bg = PALETTE[value as usize];
                } else if (addr & 0x3) != 0 {
                    self.palette[(addr as usize) >> 2][(addr as usize) & 0x3] = PALETTE[value as usize];
                }
            },
            _ => panic!("memory write out of range: 0x{:x}", ppuaddr&0x3fff),
        }
    }
}
