use std::{cell::Cell, ptr::NonNull};

use crate::{mem::Mem, Memory};

pub struct Cpu<'a> {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sr: u8,
    pub sp: u8,

    pub mem: NonNull<Memory<'a>>,

    pub cycles: NonNull<Cell<usize>>,
}

impl<'a> Cpu<'a> {
    pub fn new(mem: NonNull<Memory<'a>>, cycles: NonNull<Cell<usize>>) -> Self {
        let mut cpu = Self {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sr: 0,
            sp: 0,
            mem,
            cycles,
        };

        cpu.pc = cpu.read16(0xfffc);
        unsafe { cpu.cycles.as_ref().set(0) };
        cpu
    }
}

impl Cpu<'_> {
    #[inline]
    const fn n(&self) -> bool {
        ((self.sr >> 7) & 1) != 0
    }
    #[inline]
    const fn v(&self) -> bool {
        ((self.sr >> 6) & 1) != 0
    }
    #[inline]
    const fn d(&self) -> bool {
        ((self.sr >> 3) & 1) != 0
    }
    #[inline]
    const fn i(&self) -> bool {
        ((self.sr >> 2) & 1) != 0
    }
    #[inline]
    const fn z(&self) -> bool {
        ((self.sr >> 1) & 1) != 0
    }
    #[inline]
    const fn c(&self) -> bool {
        ((self.sr >> 0) & 1) != 0
    }

    #[inline]
    const fn set_n(&mut self, n: bool) {
        self.sr = (self.sr & !(1 << 7)) | ((n as u8) << 7)
    }
    #[inline]
    const fn set_v(&mut self, v: bool) {
        self.sr = (self.sr & !(1 << 6)) | ((v as u8) << 6)
    }
    #[inline]
    const fn set_d(&mut self, d: bool) {
        self.sr = (self.sr & !(1 << 3)) | ((d as u8) << 3)
    }
    #[inline]
    const fn set_i(&mut self, i: bool) {
        self.sr = (self.sr & !(1 << 2)) | ((i as u8) << 2)
    }
    #[inline]
    const fn set_z(&mut self, z: bool) {
        self.sr = (self.sr & !(1 << 1)) | ((z as u8) << 1)
    }
    #[inline]
    const fn set_c(&mut self, c: bool) {
        self.sr = (self.sr & !(1 << 0)) | ((c as u8) << 0)
    }



    // a: 0, c: 0
    pub fn brk(&mut self) {
        self.add_cycles(6);
        panic!("unimplemented: brk")
    }
    pub fn php(&mut self) {
        self.write(0x100 | self.sp as u16, (self.sr & 0xcf) | 0x30);
        self.sp = self.sp.wrapping_sub(1);
        self.add_cycles(1);
    }
    pub fn bpl(&mut self, value: u8) {
        if !self.n() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn clc(&mut self) {
        self.set_c(false);
        self.add_cycles(1);
    }

    // a: 1, c: 0
    pub fn jsr(&mut self, value: u16) {
        self.write16(0x100 | self.sp.wrapping_sub(1) as u16, self.pc.wrapping_sub(1));
        self.sp = self.sp.wrapping_sub(2);
        self.pc = value;
        self.add_cycles(1);
    }
    pub fn bit(&mut self, value: u8) {
        self.set_n((value as i8) < 0);
        self.set_v(((value >> 6) & 1) != 0);
        self.set_z((self.a & value) == 0);
    }
    pub fn plp(&mut self) {
        self.sp = self.sp.wrapping_add(1);
        self.sr = (self.sr & !0xcf) | (self.read(0x100 | self.sp as u16) & 0xcf);
        self.add_cycles(2);
    }
    pub fn bmi(&mut self, value: u8) {
        if self.n() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn sec(&mut self) {
        self.set_c(true);
        self.add_cycles(1);
    }

    // a: 2, c: 0
    pub fn rti(&mut self) {
        // return from interrupt
        self.sp = self.sp.wrapping_add(1);
        self.sr = (self.sr & !0xcf) | (self.read(0x100 | self.sp as u16) & 0xcf);
        self.sp = self.sp.wrapping_add(2);
        self.pc = self.read16(0x100 | self.sp.wrapping_sub(1) as u16);
        self.add_cycles(2);
    }
    pub fn pha(&mut self) {
        self.write(0x100 | self.sp as u16, self.a);
        self.sp = self.sp.wrapping_sub(1);
        self.add_cycles(1);
    }
    pub fn jmp(&mut self, value: u16) {
        self.pc = value;
    }
    pub fn bvc(&mut self, value: u8) {
        if !self.v() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn cli(&mut self) {
        self.set_i(false);
        self.add_cycles(1);
    }

    // a: 3, c: 0
    pub fn rts(&mut self) {
        // pull from stack
        self.sp = self.sp.wrapping_add(2);
        self.pc = self.read16(0x100 | self.sp.wrapping_sub(1) as u16).wrapping_add(1);
        self.add_cycles(3);
    }
    pub fn pla(&mut self) {
        self.sp = self.sp.wrapping_add(1);
        let result = self.read(0x100 | (self.sp as u16));
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result;
        self.add_cycles(2);
    }
    pub fn bvs(&mut self, value: u8) {
        if self.v() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn sei(&mut self) {
        self.set_i(true);
        self.add_cycles(1);
    }

    // a: 4, c: 0
    pub fn sty(&mut self) -> u8 {
        self.y
    }
    pub fn dey(&mut self) {
        let result = self.y.wrapping_sub(1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.y = result;
        self.add_cycles(1);
    }
    pub fn bcc(&mut self, value: u8) {
        if !self.c() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn tya(&mut self) {
        let result = self.y;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result;
        self.add_cycles(1);
    }

    // a: 5, c: 0
    pub fn ldy(&mut self, value: u8) {
        let result = value;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.y = result;
    }
    pub fn tay(&mut self) {
        let result = self.a;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.y = result;
        self.add_cycles(1);
    }
    pub fn bcs(&mut self, value: u8) {
        if self.c() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn clv(&mut self) {
        self.set_v(false);
        self.add_cycles(1);
    }

    // a: 6, c: 0
    pub fn cpy(&mut self, value: u8) {
        let (result, c) = self.y.overflowing_sub(value);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c(!c);
    }
    pub fn iny(&mut self) {
        let result = self.y.wrapping_add(1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.y = result;
        self.add_cycles(1);
    }
    pub fn bne(&mut self, value: u8) {
        if !self.z() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn cld(&mut self) {
        self.set_d(false);
        self.add_cycles(1);
    }

    // a: 7, c: 0
    pub fn cpx(&mut self, value: u8) {
        let (result, c) = self.x.overflowing_sub(value);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c(!c);
    }
    pub fn inx(&mut self) {
        let result = self.x.wrapping_add(1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.x = result;
        self.add_cycles(1);
    }
    pub fn beq(&mut self, value: u8) {
        if self.z() {
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(value as i8 as i16 as u16);
            let new_page = self.pc >> 8;
            self.add_cycles(1 + (old_page != new_page) as usize);
        }
    }
    pub fn sed(&mut self) {
        self.set_d(true);
        self.add_cycles(1);
    }



    // a: 0, c: 1
    pub fn ora(&mut self, value: u8) {
        let result = self.a | value;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result
    }

    // a: 1, c: 1
    pub fn and(&mut self, value: u8) {
        let result = self.a & value;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result
    }

    // a: 2, c: 1
    pub fn eor(&mut self, value: u8) {
        let result = self.a ^ value;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result;
    }

    // a: 3, c: 1
    pub fn adc(&mut self, value: u8) {
        // overflow flag is checked less often
        // so use result from unsigned add for total result
        // (so second add can be optimized out if inlined)
        let (result, c) = self.a.overflowing_add(value);
        let (result, c2) = result.overflowing_add(self.c() as u8);
        let c = c || c2;

        // eprintln!("adc: 0x{:x} + 0x{:x} + {} = 0x{:x}", self.a, value, self.c(), result);

        let (v_result, v) = (self.a as i8).overflowing_add(value as i8);
        let (_, v2) = v_result.overflowing_add(self.c() as i8);
        let v = v || v2;

        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_v(v);
        self.set_c(c);

        self.a = result;
    }

    // a: 4, c: 1
    pub fn sta(&mut self) -> u8 {
        self.a
    }

    // a: 5, c: 1
    pub fn lda(&mut self, value: u8) {
        let result = value;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result;
    }

    // a: 6, c: 1
    pub fn cmp(&mut self, value: u8) {
        let (result, c) = self.a.overflowing_sub(value);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c(!c);
    }

    // a: 7, c: 1
    pub fn sbc(&mut self, value: u8) {
        // overflow flag is checked less often
        // so use result from unsigned sub for total result
        // (so second sub can be optimized out if inlined)
        //
        // todo: check if carry flag math is right
        let (result, c) = self.a.overflowing_sub(value);
        let (result, c2) = result.overflowing_sub((!self.c()) as u8);
        let c = c || c2;

        let (v_result, v) = (self.a as i8).overflowing_sub(value as i8);
        let (_, v2) = v_result.overflowing_sub((!self.c()) as i8);
        let v = v || v2;

        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_v(v);
        self.set_c(!c);

        self.a = result;
    }



    // a: 0, c: 2
    pub fn asl(&mut self, value: u8) -> u8 {
        let result = value << 1;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c(((value >> 7) & 1) != 0);
        self.add_cycles(1);
        result
    }

    // a: 1, c: 2
    pub fn rol(&mut self, value: u8) -> u8 {
        let result = (value << 1) | (self.c() as u8);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c(((value >> 7) & 1) != 0);
        self.add_cycles(1);
        result
    }
    
    // a: 2, c: 2
    pub fn lsr(&mut self, value: u8) -> u8 {
        let result = value >> 1;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c((value & 1) != 0);
        self.add_cycles(1);
        result
    }

    // a: 3, c: 2
    pub fn ror(&mut self, value: u8) -> u8 {
        let result = ((self.c() as u8) << 7) | (value >> 1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.set_c((value & 1) != 0);
        self.add_cycles(1);
        result
    }

    // a: 4, c: 2
    pub fn stx(&mut self) -> u8 {
        self.x
    }
    pub fn txa(&mut self) {
        let result = self.x;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.a = result;
        self.add_cycles(1);
    }
    pub fn txs(&mut self) {
        self.sp = self.x;
        self.add_cycles(1);
    }

    // a: 5, c: 2
    pub fn ldx(&mut self, value: u8) {
        let result = value;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.x = result;
    }
    pub fn tax(&mut self) {
        let result = self.a;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.x = result;
        self.add_cycles(1);
    }
    pub fn tsx(&mut self) {
        let result = self.sp;
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.x = result;
        self.add_cycles(1);
    }

    // a: 6, c: 2
    pub fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.add_cycles(1);
        result
    }
    pub fn dex(&mut self) {
        let result = self.x.wrapping_sub(1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.x = result;
        self.add_cycles(1);
    }

    // a: 7, c: 2
    pub fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.set_n((result as i8) < 0);
        self.set_z(result == 0);
        self.add_cycles(1);
        result
    }
    pub fn nop(&self) {
        self.add_cycles(1);
    }



    // INTERRUPTS
    pub fn nmi(&mut self) {
        // trigger an nmi interrupt
        // todo: does this take cpu cycles to execute?

        // push pc
        self.write16(0x100 | self.sp.wrapping_sub(1) as u16, self.pc);
        self.sp = self.sp.wrapping_sub(2);
        // push sr
        self.write(0x100 | self.sp as u16, self.sr);
        self.sp = self.sp.wrapping_sub(1);
        // goto interrupt routine
        self.pc = self.read16(0xfffa);
    }

    // ADDRESSING MODES
    fn immediate(&mut self) -> u16 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    fn zeropage(&mut self) -> u16 {
        let addr = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        addr
    }
    fn zeropage_x(&mut self) -> u16 {
        let addr = self.read(self.pc).wrapping_add(self.x) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.add_cycles(1);
        addr
    }
    fn zeropage_y(&mut self) -> u16 {
        let addr = self.read(self.pc).wrapping_add(self.y) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.add_cycles(1);
        addr
    }
    fn absolute(&mut self) -> u16 {
        let addr = self.read16(self.pc);
        self.pc = self.pc.wrapping_add(2);
        addr
    }
    fn absolute_x(&mut self, store: bool) -> u16 {
        let addr = self.read16(self.pc);
        let addr2 = addr.wrapping_add(self.x as u16);
        self.pc = self.pc.wrapping_add(2);
        self.add_cycles((((addr >> 8) != (addr2 >> 8)) || store) as usize);
        addr2
    }
    fn absolute_y(&mut self, store: bool) -> u16 {
        let addr = self.read16(self.pc);
        let addr2 = addr.wrapping_add(self.y as u16);
        self.pc = self.pc.wrapping_add(2);
        self.add_cycles((((addr >> 8) != (addr2 >> 8)) || store) as usize);
        addr2
    }
    fn indirect(&mut self) -> u16 {
        let addr = self.read16(self.pc);
        self.pc = self.pc.wrapping_add(2);
        self.read16(addr)
    }
    fn indirect_x(&mut self) -> u16 {
        let addr = self.read(self.pc).wrapping_add(self.x) as u16;
        self.pc = self.pc.wrapping_add(1);
        let addr = self.read16(addr);
        self.add_cycles(1);
        addr
    }
    fn indirect_y(&mut self, store: bool) -> u16 {
        let addr = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let addr = self.read16(addr);
        let addr2 = addr.wrapping_add(self.y as u16);
        self.add_cycles((((addr >> 8) != (addr2 >> 8)) || store) as usize);
        addr2
    }

    fn read(&mut self, addr: u16) -> u8 {
        let mem = unsafe { self.mem.as_mut() };
        let result = mem.read(addr);
        self.add_cycles(1);
        result
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        let mem = unsafe { self.mem.as_mut() };
        mem.write(addr, value);
        self.add_cycles(1);
    }

    fn read16(&mut self, addr: u16) -> u16 {
        let mem = unsafe { self.mem.as_mut() };
        let result = mem.read16(addr);
        self.add_cycles(2);
        result
    }

    fn write16(&mut self, addr: u16, value: u16) {
        let mem = unsafe { self.mem.as_mut() };
        mem.write16(addr, value);
        self.add_cycles(2);
    }

    fn add_cycles(&self, value: usize) {
        let cycles = unsafe { self.cycles.as_ref() };
        cycles.set(cycles.get() + value);
    }

    pub fn execute(&mut self) {
        // self.print_next();
        // eprintln!();
        // get instruction
        let opcode = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        // determine instruction:
        match opcode {
            // a: 0, c: 0
            0x00 => self.brk(),
            0x08 => self.php(),
            0x10 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bpl(value);
            },
            0x18 => self.clc(),
            // a: 1, c: 0
            0x20 => {
                let addr = self.absolute();
                self.jsr(addr);
            },
            0x24 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.bit(value);
            },
            0x28 => self.plp(),
            0x2c => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.bit(value);
            },
            0x30 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bmi(value);
            },
            0x38 => self.sec(),
            // a: 2, c: 0
            0x40 => self.rti(),
            0x48 => self.pha(),
            0x4c => {
                let addr = self.absolute();
                self.jmp(addr);
            },
            0x50 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bvc(value);
            },
            0x58 => self.cli(),
            // a: 3, c: 0
            0x60 => self.rts(),
            0x68 => self.pla(),
            0x6c => {
                let addr = self.indirect();
                self.jmp(addr);
            },
            0x70 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bvs(value);
            },
            0x78 => self.sei(),
            // a: 4, c: 0
            0x84 => {
                let addr = self.zeropage();
                let result = self.sty();
                self.write(addr, result);
            },
            0x88 => self.dey(),
            0x8c => {
                let addr = self.absolute();
                let result = self.sty();
                self.write(addr, result);
            },
            0x90 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bcc(value);
            },
            0x94 => {
                let addr = self.zeropage_x();
                let result = self.sty();
                self.write(addr, result);
            },
            0x98 => self.tya(),
            // a: 5, c: 0
            0xa0 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.ldy(value);
            },
            0xa4 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.ldy(value);
            },
            0xa8 => self.tay(),
            0xac => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.ldy(value);
            },
            0xb0 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bcs(value);
            },
            0xb4 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.ldy(value);
            },
            0xb8 => self.clv(),
            0xbc => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.ldy(value);
            },
            // a: 6, c: 0
            0xc0 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.cpy(value);
            },
            0xc4 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.cpy(value);
            },
            0xc8 => self.iny(),
            0xcc => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.cpy(value);
            },
            0xd0 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.bne(value);
            },
            0xd8 => self.cld(),
            // a: 7, c: 0
            0xe0 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.cpx(value);
            },
            0xe4 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.cpx(value);
            },
            0xe8 => self.inx(),
            0xec => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.cpx(value);
            },
            0xf0 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.beq(value);
            },
            0xf8 => self.sed(),

            // a: 0, c: 1
            0x01 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.ora(value);
            },
            0x05 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.ora(value);
            },
            0x09 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.ora(value);
            },
            0x0d => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.ora(value);
            },
            0x11 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.ora(value);
            },
            0x15 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.ora(value);
            },
            0x19 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.ora(value);
            },
            0x1d => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.ora(value);
            },
            // a: 1, c: 1
            0x21 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.and(value);
            },
            0x25 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.and(value);
            },
            0x29 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.and(value);
            },
            0x2d => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.and(value);
            },
            0x31 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.and(value);
            },
            0x35 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.and(value);
            },
            0x39 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.and(value);
            },
            0x3d => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.and(value);
            },
            // a: 2, c: 1
            0x41 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.eor(value);
            },
            0x45 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.eor(value);
            },
            0x49 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.eor(value);
            },
            0x4d => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.eor(value);
            },
            0x51 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.eor(value);
            },
            0x55 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.eor(value);
            },
            0x59 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.eor(value);
            },
            0x5d => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.eor(value);
            },
            // a: 3, c: 1
            0x61 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.adc(value);
            },
            0x65 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.adc(value);
            },
            0x69 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.adc(value);
            },
            0x6d => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.adc(value);
            },
            0x71 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.adc(value);
            },
            0x75 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.adc(value);
            },
            0x79 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.adc(value);
            },
            0x7d => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.adc(value);
            },
            // a: 4, c: 1
            0x81 => {
                let addr = self.indirect_x();
                let result = self.sta();
                self.write(addr, result);
            },
            0x85 => {
                let addr = self.zeropage();
                let result = self.sta();
                self.write(addr, result);
            },
            0x8d => {
                let addr = self.absolute();
                let result = self.sta();
                self.write(addr, result);
            },
            0x91 => {
                let addr = self.indirect_y(true);
                let result = self.sta();
                self.write(addr, result);
            },
            0x95 => {
                let addr = self.zeropage_x();
                let result = self.sta();
                self.write(addr, result);
            },
            0x99 => {
                let addr = self.absolute_y(true);
                let result = self.sta();
                self.write(addr, result);
            },
            0x9d => {
                let addr = self.absolute_x(true);
                let result = self.sta();
                self.write(addr, result);
            },
            // a: 5, c: 1
            0xa1 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.lda(value);
            },
            0xa5 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.lda(value);
            },
            0xa9 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.lda(value);
            },
            0xad => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.lda(value);
            },
            0xb1 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.lda(value);
            },
            0xb5 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.lda(value);
            },
            0xb9 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.lda(value);
            },
            0xbd => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.lda(value);
            },
            // a: 6, c: 1
            0xc1 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.cmp(value);
            },
            0xc5 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.cmp(value);
            },
            0xc9 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.cmp(value);
            },
            0xcd => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.cmp(value);
            },
            0xd1 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.cmp(value);
            },
            0xd5 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.cmp(value);
            },
            0xd9 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.cmp(value);
            },
            0xdd => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.cmp(value);
            },
            // a: 7, c: 1
            0xe1 => {
                let addr = self.indirect_x();
                let value = self.read(addr);
                self.sbc(value);
            },
            0xe5 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.sbc(value);
            },
            0xe9 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.sbc(value);
            },
            0xed => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.sbc(value);
            },
            0xf1 => {
                let addr = self.indirect_y(false);
                let value = self.read(addr);
                self.sbc(value);
            },
            0xf5 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                self.sbc(value);
            },
            0xf9 => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.sbc(value);
            },
            0xfd => {
                let addr = self.absolute_x(false);
                let value = self.read(addr);
                self.sbc(value);
            },

            // a: 0, c: 2
            0x06 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                let result = self.asl(value);
                self.write(addr, result);
            },
            0x0a => self.a = self.asl(self.a),
            0x0e => {
                let addr = self.absolute();
                let value = self.read(addr);
                let result = self.asl(value);
                self.write(addr, result);
            },
            0x16 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                let result = self.asl(value);
                self.write(addr, result);
            },
            0x1e => {
                let addr = self.absolute_x(true);
                let value = self.read(addr);
                let result = self.asl(value);
                self.write(addr, result);
            },
            // a: 1, c: 2
            0x26 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                let result = self.rol(value);
                self.write(addr, result);
            },
            0x2a => self.a = self.rol(self.a),
            0x2e => {
                let addr = self.absolute();
                let value = self.read(addr);
                let result = self.rol(value);
                self.write(addr, result);
            },
            0x36 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                let result = self.rol(value);
                self.write(addr, result);
            },
            0x3e => {
                let addr = self.absolute_x(true);
                let value = self.read(addr);
                let result = self.rol(value);
                self.write(addr, result);
            },
            // a: 2, c: 2
            0x46 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                let result = self.lsr(value);
                self.write(addr, result);
            },
            0x4a => self.a = self.lsr(self.a),
            0x4e => {
                let addr = self.absolute();
                let value = self.read(addr);
                let result = self.lsr(value);
                self.write(addr, result);
            },
            0x56 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                let result = self.lsr(value);
                self.write(addr, result);
            },
            0x5e => {
                let addr = self.absolute_x(true);
                let value = self.read(addr);
                let result = self.lsr(value);
                self.write(addr, result);
            },
            // a: 3, c: 2
            0x66 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                let result = self.ror(value);
                self.write(addr, result);
            },
            0x6a => self.a = self.ror(self.a),
            0x6e => {
                let addr = self.absolute();
                let value = self.read(addr);
                let result = self.ror(value);
                self.write(addr, result);
            },
            0x76 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                let result = self.ror(value);
                self.write(addr, result);
            },
            0x7e => {
                let addr = self.absolute_x(true);
                let value = self.read(addr);
                let result = self.ror(value);
                self.write(addr, result);
            },
            // a: 4, c: 2
            0x86 => {
                let addr = self.zeropage();
                let result = self.stx();
                self.write(addr, result);
            },
            0x8a => self.txa(),
            0x8e => {
                let addr = self.absolute();
                let result = self.stx();
                self.write(addr, result);
            },
            0x96 => {
                let addr = self.zeropage_y();
                let result = self.stx();
                self.write(addr, result);
            },
            0x9a => self.txs(),
            // a: 5, c: 2
            0xa2 => {
                let addr = self.immediate();
                let value = self.read(addr);
                self.ldx(value);
            }
            0xa6 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                self.ldx(value);
            },
            0xaa => self.tax(),
            0xae => {
                let addr = self.absolute();
                let value = self.read(addr);
                self.ldx(value);
            },
            0xb6 => {
                let addr = self.zeropage_y();
                let value = self.read(addr);
                self.ldx(value);
            },
            0xba => self.tsx(),
            0xbe => {
                let addr = self.absolute_y(false);
                let value = self.read(addr);
                self.ldx(value);
            },
            // a: 6, c: 2
            0xc6 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                let result = self.dec(value);
                self.write(addr, result);
            },
            0xca => self.dex(),
            0xce => {
                let addr = self.absolute();
                let value = self.read(addr);
                let result = self.dec(value);
                self.write(addr, result);
            },
            0xd6 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                let result = self.dec(value);
                self.write(addr, result);
            },
            0xde => {
                let addr = self.absolute_x(true);
                let value = self.read(addr);
                let result = self.dec(value);
                self.write(addr, result);
            },
            // a: 7, c: 2
            0xe6 => {
                let addr = self.zeropage();
                let value = self.read(addr);
                let result = self.inc(value);
                self.write(addr, result);
            },
            0xea => self.nop(),
            0xee => {
                let addr = self.absolute();
                let value = self.read(addr);
                let result = self.inc(value);
                self.write(addr, result);
            },
            0xf6 => {
                let addr = self.zeropage_x();
                let value = self.read(addr);
                let result = self.inc(value);
                self.write(addr, result);
            },
            0xfe => {
                let addr = self.absolute_x(true);
                let value = self.read(addr);
                let result = self.inc(value);
                self.write(addr, result);
            },

            // fallback
            _ => panic!("unimplemented: opcode {opcode:x}")
        }
    }

    // for debugging
    #[allow(unused)]
    fn print_next(&mut self) {
        let mem = unsafe { self.mem.as_mut() };
        
        // get instruction
        let opcode = mem.read(self.pc);

        eprintln!("next instruction:");
        eprint!("  ");

        match opcode {
            0x00 => eprintln!("brk"),
            0x08 => eprintln!("php"),
            0x10 => eprintln!("bpl ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0x18 => eprintln!("clc"),

            0x20 => eprintln!("jsr ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x24 => eprintln!("bit ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x28 => eprintln!("plp"),
            0x2c => eprintln!("bit ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x30 => eprintln!("bmi ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0x38 => eprintln!("sec"),

            0x40 => eprintln!("rti"),
            0x48 => eprintln!("pha"),
            0x4c => eprintln!("jmp ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x50 => eprintln!("bvc ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0x58 => eprintln!("cli"),

            0x60 => eprintln!("rts"),
            0x68 => eprintln!("pla"),
            0x6c => eprintln!("jmp (${:x})", mem.read16(self.pc.wrapping_add(1))),
            0x70 => eprintln!("bvs ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0x78 => eprintln!("sei"),

            0x84 => eprintln!("sty ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x88 => eprintln!("dey"),
            0x8c => eprintln!("sty ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x90 => eprintln!("bcc ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0x94 => eprintln!("sty ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x98 => eprintln!("tya"),

            0xa0 => eprintln!("ldy #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xa4 => eprintln!("ldy ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xa8 => eprintln!("tay"),
            0xac => eprintln!("ldy ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xb0 => eprintln!("bcs ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0xb4 => eprintln!("ldy ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0xb8 => eprintln!("clv"),
            0xbc => eprintln!("ldy ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0xc0 => eprintln!("cpy #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xc4 => eprintln!("cpy ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xc8 => eprintln!("iny"),
            0xcc => eprintln!("cpy ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xd0 => eprintln!("bne ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0xd8 => eprintln!("cld"),
            
            0xe0 => eprintln!("cpx #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xe4 => eprintln!("cpx ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xe8 => eprintln!("inx"),
            0xec => eprintln!("cpx ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xf0 => eprintln!("beq ${:x}", (self.pc.wrapping_add(2) as i16).wrapping_add(mem.read(self.pc.wrapping_add(1)) as i8 as i16) as u16),
            0xf8 => eprintln!("sed"),



            0x01 => eprintln!("ora (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0x05 => eprintln!("ora ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x09 => eprintln!("ora #${:x}", mem.read(self.pc.wrapping_add(1))),
            0x0d => eprintln!("ora ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x11 => eprintln!("ora (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0x15 => eprintln!("ora ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x19 => eprintln!("ora ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0x1d => eprintln!("ora ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x21 => eprintln!("and (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0x25 => eprintln!("and ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x29 => eprintln!("and #${:x}", mem.read(self.pc.wrapping_add(1))),
            0x2d => eprintln!("and ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x31 => eprintln!("and (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0x35 => eprintln!("and ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x39 => eprintln!("and ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0x3d => eprintln!("and ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x41 => eprintln!("eor (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0x45 => eprintln!("eor ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x49 => eprintln!("eor #${:x}", mem.read(self.pc.wrapping_add(1))),
            0x4d => eprintln!("eor ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x51 => eprintln!("eor (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0x55 => eprintln!("eor ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x59 => eprintln!("eor ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0x5d => eprintln!("eor ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x61 => eprintln!("adc (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0x65 => eprintln!("adc ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x69 => eprintln!("adc #${:x}", mem.read(self.pc.wrapping_add(1))),
            0x6d => eprintln!("adc ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x71 => eprintln!("adc (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0x75 => eprintln!("adc ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x79 => eprintln!("adc ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0x7d => eprintln!("adc ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x81 => eprintln!("sta (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0x85 => eprintln!("sta ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x8d => eprintln!("sta ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x91 => eprintln!("sta (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0x95 => eprintln!("sta ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x99 => eprintln!("sta ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0x9d => eprintln!("sta ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0xa1 => eprintln!("lda (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0xa5 => eprintln!("lda ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xa9 => eprintln!("lda #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xad => eprintln!("lda ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xb1 => eprintln!("lda (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0xb5 => eprintln!("lda ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0xb9 => eprintln!("lda ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0xbd => eprintln!("lda ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0xc1 => eprintln!("cmp (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0xc5 => eprintln!("cmp ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xc9 => eprintln!("cmp #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xcd => eprintln!("cmp ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xd1 => eprintln!("cmp (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0xd5 => eprintln!("cmp ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0xd9 => eprintln!("cmp ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0xdd => eprintln!("cmp ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0xe1 => eprintln!("sbc (${:x},x)", mem.read(self.pc.wrapping_add(1))),
            0xe5 => eprintln!("sbc ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xe9 => eprintln!("sbc #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xed => eprintln!("sbc ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xf1 => eprintln!("sbc (${:x}),y", mem.read(self.pc.wrapping_add(1))),
            0xf5 => eprintln!("sbc ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0xf9 => eprintln!("sbc ${:x},y", mem.read16(self.pc.wrapping_add(1))),
            0xfd => eprintln!("sbc ${:x},x", mem.read16(self.pc.wrapping_add(1))),



            0x06 => eprintln!("asl ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x0a => eprintln!("asl a"),
            0x0e => eprintln!("asl ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x16 => eprintln!("asl ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x1e => eprintln!("asl ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x26 => eprintln!("rol ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x2a => eprintln!("rol a"),
            0x2e => eprintln!("rol ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x36 => eprintln!("rol ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x3e => eprintln!("rol ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x46 => eprintln!("lsr ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x4a => eprintln!("lsr a"),
            0x4e => eprintln!("lsr ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x56 => eprintln!("lsr ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x5e => eprintln!("lsr ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x66 => eprintln!("ror ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x6a => eprintln!("ror a"),
            0x6e => eprintln!("ror ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x76 => eprintln!("ror ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0x7e => eprintln!("ror ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0x86 => eprintln!("stx ${:x}", mem.read(self.pc.wrapping_add(1))),
            0x8a => eprintln!("txa"),
            0x8e => eprintln!("stx ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0x96 => eprintln!("stx ${:x},y", mem.read(self.pc.wrapping_add(1))),
            0x9a => eprintln!("txs"),

            0xa2 => eprintln!("ldx #${:x}", mem.read(self.pc.wrapping_add(1))),
            0xa6 => eprintln!("ldx ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xaa => eprintln!("tax"),
            0xae => eprintln!("ldx ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xb6 => eprintln!("ldx ${:x},y", mem.read(self.pc.wrapping_add(1))),
            0xba => eprintln!("tsx"),
            0xbe => eprintln!("ldx ${:x},y", mem.read16(self.pc.wrapping_add(1))),

            0xc6 => eprintln!("dec ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xca => eprintln!("dex"),
            0xce => eprintln!("dec ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xd6 => eprintln!("dec ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0xde => eprintln!("dec ${:x},x", mem.read16(self.pc.wrapping_add(1))),

            0xe6 => eprintln!("inc ${:x}", mem.read(self.pc.wrapping_add(1))),
            0xea => eprintln!("nop"),
            0xee => eprintln!("inc ${:x}", mem.read16(self.pc.wrapping_add(1))),
            0xf6 => eprintln!("inc ${:x},x", mem.read(self.pc.wrapping_add(1))),
            0xfe => eprintln!("inc ${:x},x", mem.read16(self.pc.wrapping_add(1))),



            _ => eprintln!("unknown instruction: {opcode:x}"),
        }

        eprintln!("registers:");
        eprintln!("  pc=0x{:x}, a=0x{:x}, x=0x{:x}, y=0x{:x}, sp=0x1{:02x}", self.pc, self.a, self.x, self.y, self.sp);
        eprintln!("  n={},v={},d={},i={},z={},c={}", self.n(), self.v(), self.d(), self.i(), self.z(), self.c());
        eprintln!("cycle: {}", unsafe { self.cycles.as_ref().get() });
    }
}
