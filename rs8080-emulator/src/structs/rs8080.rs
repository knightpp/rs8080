use crate::structs::{ConditionalCodes, TwoU8, BC, DE, HL};
use crate::traits::{DataBus, OverflowMath};
use std::fmt::{self, Formatter};
extern crate rs8080_disassembler as disasm;
use crate::traits::{MemLimiter, WriteAction};
use disasm::{disassemble, Command};

pub(crate) struct AllowAll {}
impl MemLimiter for AllowAll {
    fn check_write(&self, _: u16, _: u8) -> WriteAction {
        WriteAction::Allow
    }
    fn check_read(&self, _: u16, read_byte: u8) -> u8 {
        read_byte
    }
}

/// Intel 8080
pub struct RS8080 {
    //registers
    a: u8,
    bc: BC,
    de: DE,
    hl: HL,
    /// stack pointer
    sp: u16,
    /// program counter
    pc: u16,
    mem: [u8; 0xFFFF],
    /// Conditional codes
    cc: ConditionalCodes,
    /// Interrupts enabled
    int_enable: bool,
    io_device: Box<dyn DataBus + Send + Sync>,
    mem_limiter: Box<dyn MemLimiter + Send + Sync>,
}

impl fmt::Display for RS8080 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a={:02x}|bc={:02x}{:02x}|de={:02x}{:02x}|\
            hl={:02x}{:02x}|pc={:04x}|sp={:04x}   {}",
            self.a,
            self.bc.b,
            self.bc.c,
            self.de.d,
            self.de.e,
            self.hl.h,
            self.hl.l,
            self.pc,
            self.sp,
            self.cc,
        )
    }
}

impl RS8080 {
    #[inline]
    pub fn get_mem_slice(&self, r: std::ops::Range<usize>) -> &[u8] {
        &self.mem[r]
    }

    pub fn new(io_device: Box<dyn DataBus + Send + Sync>) -> RS8080 {
        RS8080 {
            a: 0,
            bc: BC { b: 0, c: 0 },
            de: DE { d: 0, e: 0 },
            hl: HL { h: 0, l: 0 },
            sp: 0,
            pc: 0,
            mem: [0; 0xFFFF],
            cc: ConditionalCodes {
                z: false,
                s: false,
                p: false,
                cy: false,
                ac: false,
            },
            int_enable: false,
            io_device: io_device,
            mem_limiter: Box::new(AllowAll {}),
        }
    }

    pub fn set_mem_limiter(&mut self, new_mem_limiter: Box<dyn MemLimiter + Send + Sync>) {
        self.mem_limiter = new_mem_limiter;
    }

    pub fn get_io_mut(&mut self) -> &mut Box<dyn DataBus + Send + Sync> {
        &mut self.io_device
    }

    #[inline(always)]
    pub fn get_mut_mem(&mut self) -> &mut [u8] {
        &mut self.mem
    }
    #[inline(always)]
    pub fn get_mem(&self) -> &[u8] {
        &self.mem
    }
    /// # Panics
    /// length of slice > mem
    #[inline]
    pub fn load_to_mem(&mut self, slice: &[u8], offset: u16) {
        if slice.len() > self.mem.len() {
            panic!("input was too large for emulated memory (max 0xFFFF)");
        }
        self.mem[offset as usize..(slice.len() + offset as usize)].copy_from_slice(slice);
        // for (i, x) in slice.iter().enumerate().map(|(i, x)|(i+ offset as usize, x)) {
        //     self.mem[i] = *x;
        // }
    }

    #[inline]
    pub fn disassemble_next(&self) -> Command {
        disassemble(&self.mem[self.pc as usize..])
    }

    pub fn emulate_next(&mut self) {
        //let cmd = disassemble(&self.mem[self.pc as usize..]);
        //self.pc as usize + 3
        let mem_from_pc = &self.mem[self.pc as usize..];
        self.pc.add_un(1);
        match *mem_from_pc {
            [0x0, ..] => {}
            [0x01, lo, hi, ..] => {
                self.bc.set(TwoU8 { lo, hi });
                self.pc += 2;
            }
            [0x02, ..] => {
                self.write_mem(self.bc, self.a);
            }
            [0x03, ..] => {
                self.bc += 1;
            }
            [0x04, ..] => {
                self.bc.b.add_un(1);
                self.cc.set_zspac(self.bc.b);
            }
            [0x05, ..] => {
                self.bc.b.sub_un(1);
                self.cc.set_zspac(self.bc.b);
            }
            [0x06, d8, ..] => {
                self.bc.b = d8;
                self.pc += 1;
            }
            [0x07, ..] => {
                self.cc.cy = self.a & 0b1000_0000 > 0;
                self.a = self.a.rotate_left(1);
            }
            [0x08, ..] => {}
            [0x09, ..] => {
                let carry = self.hl.add_carry(self.bc.into());
                self.cc.cy = carry;
            }
            [0x0A, ..] => {
                self.a = self.read_mem(self.bc);
            }
            [0x0B, ..] => {
                self.bc -= 1;
            }
            [0x0C, ..] => {
                let rez = self.bc.c as u16 + 1;
                self.cc.set_zspac(rez);
                self.bc.c = rez as u8;
            }
            [0x0D, ..] => {
                self.bc.c.sub_un(1);
                self.cc.set_zspac(self.bc.c);
            }
            [0x0E, d8, ..] => {
                self.bc.c = d8;
                self.pc += 1;
            }
            [0x0F, ..] => {
                self.cc.cy = self.a & 0x1 > 0;
                self.a = self.a.rotate_right(1);
            }

            [0x10, ..] => {}
            [0x11, d16_lo, d16_hi, ..] => {
                self.de.d = d16_hi;
                self.de.e = d16_lo;
                self.pc += 2;
            }
            [0x12, ..] => {
                self.write_mem(self.de, self.a);
            }
            [0x13, ..] => {
                self.de += 1;
            }
            [0x14, ..] => {
                let rez = self.de.d as u16 + 1;
                self.cc.set_zspac(rez);
                self.de.d = rez as u8;
            }
            [0x15, ..] => {
                let rez = self.de.d as u16 - 1;
                self.cc.set_zspac(rez);
                self.de.d = rez as u8;
            }
            [0x16, d8, ..] => {
                self.de.d = d8;
                self.pc += 1;
            }
            [0x17, ..] => {
                let prev_cy = self.cc.cy;
                self.cc.cy = self.a & 0b1000_0000 > 0;
                self.a = self.a << 1;
                self.a |= prev_cy as u8;
            }
            [0x18, ..] => {}
            [0x19, ..] => {
                let carry = self.hl.add_carry(self.de.into());
                self.cc.cy = carry;
            }
            [0x1A, ..] => {
                self.a = self.read_mem(self.de);
            }
            [0x1B, ..] => {
                self.de -= 1;
            }
            [0x1C, ..] => {
                let rez = (self.de.e as u16).wrapping_add(1);
                self.de.e = rez as u8;
                self.cc.set_zspac(rez);
            }
            [0x1D, ..] => {
                let rez = (self.de.e as u16).wrapping_sub(1);
                self.de.e = rez as u8;
                self.cc.set_zspac(rez);
            }
            [0x1E, d8, ..] => {
                self.de.e = d8;
                self.pc += 1;
            }
            [0x1F, ..] => {
                let prev_cy = self.cc.cy;
                self.cc.cy = self.a & 0b0000_0001 > 0;
                self.a = self.a >> 1;
                self.a |= prev_cy as u8;
            }

            [0x20, ..] => {}
            [0x21, lo, hi, ..] => {
                self.hl.set(TwoU8 { lo, hi });
                self.pc += 2;
            }
            [0x22, lo, hi, ..] => {
                let adr: u16 = TwoU8 { lo, hi }.into();
                self.write_mem(adr, self.hl.l);
                self.write_mem(adr + 1, self.hl.h);
                self.pc += 2;
            }
            [0x23, ..] => {
                self.hl += 1;
            }
            [0x24, ..] => {
                let x = self.hl.h as u16 + 1;
                self.hl.h = x as u8;
                self.cc.set_zspac(x);
            }
            [0x25, ..] => {
                let x = self.hl.h as u16 - 1;
                self.hl.h = x as u8;
                self.cc.set_zspac(x);
            }
            [0x26, d8, ..] => {
                self.hl.h = d8;
                self.pc += 1;
            }
            [0x27, ..] => {
                // TODO: DAA
                todo!()
            }
            [0x28, ..] => {}
            [0x29, ..] => {
                let carry = self.hl.add_carry(self.hl.into());
                self.cc.cy = carry;
            }
            [0x2A, lo, hi, ..] => {
                let adr: u16 = TwoU8 { lo, hi }.into();
                self.hl.l = self.read_mem(adr);
                self.hl.h = self.read_mem(adr + 1);
                self.pc += 2;
            }
            [0x2B, ..] => {
                self.hl -= 1;
            }
            [0x2C, ..] => {
                self.hl.l.add_un(1);
                self.cc.set_zspac(self.hl.l);
            }
            [0x2D, ..] => {
                let x = self.hl.l as u16 - 1;
                self.hl.l = x as u8;
                self.cc.set_zspac(x);
            }
            [0x2E, d8, ..] => {
                self.hl.l = d8;
                self.pc += 1;
            }
            [0x2F, ..] => {
                self.a = !self.a;
            }

            [0x30, ..] => {}
            [0x31, lo, hi, ..] => {
                self.sp = TwoU8 { lo, hi }.into();
                self.pc += 2;
            }
            [0x32, lo, hi, ..] => {
                self.write_mem(TwoU8 { lo, hi }, self.a);
                self.pc += 2;
            }
            [0x33, ..] => {
                self.sp = self.sp.wrapping_add(1);
            }
            [0x34, ..] => {
                let mut x = self.read_mem(self.hl);
                x.add_un(1);
                self.write_mem(self.hl, x);
                self.cc.set_zspac(x);
            }
            [0x35, ..] => {
                let mut x = self.read_mem(self.hl);
                x.sub_un(1);
                self.write_mem(self.hl, x);
                self.cc.set_zspac(x);
            }
            [0x36, d8, ..] => {
                self.write_mem(self.hl, d8);
                self.pc += 1;
            }
            [0x37, ..] => {
                self.cc.cy = true;
            }
            [0x38, ..] => {}
            [0x39, ..] => {
                let carry = self.hl.add_carry(self.sp);
                self.cc.cy = carry;
            }
            [0x3A, lo, hi, ..] => {
                self.a = self.read_mem(TwoU8 { lo, hi });
                self.pc += 2;
            }
            [0x3B, ..] => {
                self.sp.sub_un(1);
            }
            [0x3C, ..] => {
                self.a.add_un(1);
                self.cc.set_zspac(self.a);
            }
            [0x3D, ..] => {
                self.a.sub_un(1);
                self.cc.set_zspac(self.a);
            }
            [0x3E, d8, ..] => {
                self.a = d8;
                self.pc += 1;
            }
            [0x3F, ..] => {
                self.cc.cy = !self.cc.cy;
            }

            [0x40, ..] => {
                //self.bc.b = self.bc.b;
            }
            [0x41, ..] => {
                self.bc.b = self.bc.c;
            }
            [0x42, ..] => {
                self.bc.b = self.de.d;
            }
            [0x43, ..] => {
                self.bc.b = self.de.e;
            }
            [0x44, ..] => {
                self.bc.b = self.hl.h;
            }
            [0x45, ..] => {
                self.bc.b = self.hl.l;
            }
            [0x46, ..] => {
                self.bc.b = self.read_mem(self.hl);
            }
            [0x47, ..] => {
                self.bc.b = self.a;
            }
            [0x48, ..] => {
                self.bc.c = self.bc.b;
            }
            [0x49, ..] => {
                self.bc.c = self.bc.c;
            }
            [0x4A, ..] => {
                self.bc.c = self.de.d;
            }
            [0x4B, ..] => {
                self.bc.c = self.de.e;
            }
            [0x4C, ..] => {
                self.bc.c = self.hl.h;
            }
            [0x4D, ..] => {
                self.bc.c = self.hl.l;
            }
            [0x4E, ..] => {
                self.bc.c = self.read_mem(self.hl);
            }
            [0x4F, ..] => {
                self.bc.c = self.a;
            }

            [0x50, ..] => {
                self.de.d = self.bc.b;
            }
            [0x51, ..] => {
                self.de.d = self.bc.c;
            }
            [0x52, ..] => {
                self.de.d = self.de.d;
            }
            [0x53, ..] => {
                self.de.d = self.de.e;
            }
            [0x54, ..] => {
                self.de.d = self.hl.h;
            }
            [0x55, ..] => {
                self.de.d = self.hl.l;
            }
            [0x56, ..] => {
                self.de.d = self.read_mem(self.hl);
            }
            [0x57, ..] => {
                self.de.d = self.a;
            }
            [0x58, ..] => {
                self.de.e = self.bc.b;
            }
            [0x59, ..] => {
                self.de.e = self.bc.c;
            }
            [0x5A, ..] => {
                self.de.e = self.de.d;
            }
            [0x5B, ..] => {
                self.de.e = self.de.e;
            }
            [0x5C, ..] => {
                self.de.e = self.hl.h;
            }
            [0x5D, ..] => {
                self.de.e = self.hl.l;
            }
            [0x5E, ..] => {
                self.de.e = self.read_mem(self.hl);
            }
            [0x5F, ..] => {
                self.de.e = self.a;
            }

            [0x60, ..] => {
                self.hl.h = self.bc.b;
            }
            [0x61, ..] => {
                self.hl.h = self.bc.c;
            }
            [0x62, ..] => {
                self.hl.h = self.de.d;
            }
            [0x63, ..] => {
                self.hl.h = self.de.e;
            }
            [0x64, ..] => {
                self.hl.h = self.hl.h;
            }
            [0x65, ..] => {
                self.hl.h = self.hl.l;
            }
            [0x66, ..] => {
                self.hl.h = self.read_mem(self.hl);
            }
            [0x67, ..] => {
                self.hl.h = self.a;
            }
            [0x68, ..] => {
                self.hl.l = self.bc.b;
            }
            [0x69, ..] => {
                self.hl.l = self.bc.c;
            }
            [0x6A, ..] => {
                self.hl.l = self.de.d;
            }
            [0x6B, ..] => {
                self.hl.l = self.de.e;
            }
            [0x6C, ..] => {
                self.hl.l = self.hl.h;
            }
            [0x6D, ..] => {
                self.hl.l = self.hl.l;
            }
            [0x6E, ..] => {
                self.hl.l = self.read_mem(self.hl);
            }
            [0x6F, ..] => {
                self.hl.l = self.a;
            }

            [0x70, ..] => {
                self.write_mem(self.hl, self.bc.b);
            }
            [0x71, ..] => {
                self.write_mem(self.hl, self.bc.c);
            }
            [0x72, ..] => {
                self.write_mem(self.hl, self.de.d);
            }
            [0x73, ..] => {
                self.write_mem(self.hl, self.de.e);
            }
            [0x74, ..] => {
                self.write_mem(self.hl, self.hl.h);
            }
            [0x75, ..] => {
                self.write_mem(self.hl, self.hl.l);
            }
            [0x76, ..] => {
                // TODO: HLT
                todo!()
            }
            [0x77, ..] => {
                self.write_mem(self.hl, self.a);
            }
            [0x78, ..] => {
                self.a = self.bc.b;
            }
            [0x79, ..] => {
                self.a = self.bc.c;
            }
            [0x7A, ..] => {
                self.a = self.de.d;
            }
            [0x7B, ..] => {
                self.a = self.de.e;
            }
            [0x7C, ..] => {
                self.a = self.hl.h;
            }
            [0x7D, ..] => {
                self.a = self.hl.l;
            }
            [0x7E, ..] => self.a = self.read_mem(self.hl),
            [0x7F, ..] => {
                //self.a = self.a
            }

            [0x80, ..] => {
                let carry = self.a.add_carry(self.bc.b);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x81, ..] => {
                let carry = self.a.add_carry(self.bc.c);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x82, ..] => {
                let carry = self.a.add_carry(self.de.d);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x83, ..] => {
                let carry = self.a.add_carry(self.de.e);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x84, ..] => {
                let carry = self.a.add_carry(self.hl.h);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x85, ..] => {
                let carry = self.a.add_carry(self.hl.l);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x86, ..] => {
                let carry = self.a.add_carry(self.read_mem(self.hl));
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x87, ..] => {
                let carry = self.a.add_carry(self.a);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x88, ..] => {
                let mut x = self.bc.b;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x89, ..] => {
                let mut x = self.bc.c;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x8A, ..] => {
                let mut x = self.de.d;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x8B, ..] => {
                let mut x = self.de.e;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x8C, ..] => {
                let mut x = self.hl.h;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x8D, ..] => {
                let mut x = self.hl.l;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x8E, ..] => {
                let mut x = self.read_mem(self.hl);
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x8F, ..] => {
                let mut x = self.a;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }

            [0x90, ..] => {
                let carry = self.a.sub_carry(self.bc.b);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x91, ..] => {
                let carry = self.a.sub_carry(self.bc.c);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x92, ..] => {
                let carry = self.a.sub_carry(self.de.d);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x93, ..] => {
                let carry = self.a.sub_carry(self.de.e);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x94, ..] => {
                let carry = self.a.sub_carry(self.hl.h);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x95, ..] => {
                let carry = self.a.sub_carry(self.hl.l);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x96, ..] => {
                let carry = self.a.sub_carry(self.read_mem(self.hl));
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x97, ..] => {
                let carry = self.a.sub_carry(self.a);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            [0x98, ..] => {
                let carry1 = self.a.sub_carry(self.bc.b);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x99, ..] => {
                let carry1 = self.a.sub_carry(self.bc.c);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x9A, ..] => {
                let carry1 = self.a.sub_carry(self.de.d);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x9B, ..] => {
                let carry1 = self.a.sub_carry(self.de.e);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x9C, ..] => {
                let carry1 = self.a.sub_carry(self.hl.h);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x9D, ..] => {
                let carry1 = self.a.sub_carry(self.hl.l);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x9E, ..] => {
                let carry1 = self.a.sub_carry(self.read_mem(self.hl));
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            [0x9F, ..] => {
                let carry1 = self.a.sub_carry(self.a);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }

            [0xA0, ..] => {
                self.a &= self.bc.b;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA1, ..] => {
                self.a &= self.bc.c;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA2, ..] => {
                self.a &= self.de.d;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA3, ..] => {
                self.a &= self.de.e;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA4, ..] => {
                self.a &= self.hl.h;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA5, ..] => {
                self.a &= self.hl.l;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA6, ..] => {
                self.a &= self.read_mem(self.hl);
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA7, ..] => {
                self.a &= self.a;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA8, ..] => {
                self.a ^= self.bc.b;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xA9, ..] => {
                self.a ^= self.bc.c;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xAA, ..] => {
                self.a ^= self.de.d;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xAB, ..] => {
                self.a ^= self.de.e;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xAC, ..] => {
                self.a ^= self.hl.h;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xAD, ..] => {
                self.a ^= self.hl.l;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xAE, ..] => {
                self.a ^= self.read_mem(self.hl);
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xAF, ..] => {
                self.a ^= self.a;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }

            [0xB0, ..] => {
                self.a |= self.bc.b;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB1, ..] => {
                self.a |= self.bc.c;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB2, ..] => {
                self.a |= self.de.d;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB3, ..] => {
                self.a |= self.de.e;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB4, ..] => {
                self.a |= self.hl.h;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB5, ..] => {
                self.a |= self.hl.l;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB6, ..] => {
                self.a |= self.read_mem(self.hl);
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB7, ..] => {
                self.a |= self.a;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            [0xB8, ..] => {
                self.cc.set_cmp(self.a, self.bc.b);
            }
            [0xB9, ..] => {
                self.cc.set_cmp(self.a, self.bc.c);
            }
            [0xBA, ..] => {
                self.cc.set_cmp(self.a, self.de.d);
            }
            [0xBB, ..] => {
                self.cc.set_cmp(self.a, self.de.e);
            }
            [0xBC, ..] => {
                self.cc.set_cmp(self.a, self.hl.h);
            }
            [0xBD, ..] => {
                self.cc.set_cmp(self.a, self.hl.l);
            }
            [0xBE, ..] => {
                self.cc.set_cmp(self.a, self.read_mem(self.hl));
            }
            [0xBF, ..] => {
                self.cc.set_cmp(self.a, self.a);
            }

            [0xC0, ..] => {
                if self.cc.z == false {
                    self.ret();
                }
            }
            [0xC1, ..] => {
                let data = self.pop();
                self.bc.set(data);
            }
            // JNZ
            [0xC2, lo, hi, ..] => {
                if !self.cc.z {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xC3, lo, hi, ..] => {
                self.pc = TwoU8 { lo, hi }.into();
            }
            [0xC4, lo, hi, ..] => {
                self.pc += 2;
                if !self.cc.z {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xC5, ..] => {
                self.push(self.bc.get_twou8());
            }
            [0xC6, d8, ..] => {
                self.cc.cy = self.a.add_carry(d8);
                self.cc.set_zspac(self.a);
                self.pc += 1;
            }
            [0xC7, ..] => {
                self.call(0);
            }
            [0xC8, ..] => {
                if self.cc.z == true {
                    self.ret();
                }
            }
            [0xC9, ..] => {
                self.ret();
            }
            // JZ
            [0xCA, lo, hi, ..] => {
                if self.cc.z {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xCB, ..] => {}
            [0xCC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.z {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xCD, lo, hi, ..] => {
                self.pc.add_un(2);
                self.call(TwoU8 { lo, hi }.into());
                //code to show messages from cpudiag.bin program
                // let d16 : u16 = TwoU8{lo, hi}.into();
                // if d16 == 5{
                //     if self.bc.c == 9{
                //         let offset : u16 = self.de.into();
                //         let str = self.mem.iter().skip((offset + 3) as usize)
                //             .take_while(|x|**x != b'$').copied().collect::<Vec<u8>>();
                //         let str = String::from_utf8(str).unwrap();
                //         let number = self.mem.iter().skip( offset as usize + 3 + str.len()).take(2).copied().collect::<Vec<u8>>();

                //         if number.len() == 2{
                //             println!("{}0x{:02X}{:02X}", str, number[1], number[0]);
                //         }else{
                //             println!("{}", str);
                //         }
                //         std::process::exit(-1);

                //     }else if self.bc.c == 2{
                //         println!("print char routine called");
                //     }
                // }else if d16 == 0{
                //     println!("perhaps good? exitting");
                //     //exit(0);
                // }else{
                //     self.call(TwoU8{lo, hi}.into());
                // }
            }
            [0xCE, d8, ..] => {
                let carry1 = self.a.add_carry(d8);
                let carry2 = self.a.add_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 || carry2;
                self.pc += 1;
            }
            [0xCF, ..] => {
                self.call(0x8);
            }

            [0xD0, ..] => {
                if !self.cc.cy {
                    self.ret();
                }
            }
            [0xD1, ..] => {
                let x = self.pop();
                self.de.set(x);
            }
            // JNC
            [0xD2, lo, hi, ..] => {
                if !self.cc.cy {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xD3, d8, ..] => {
                // special, OUT
                self.io_device.port_out(self.a, d8);
                self.pc += 1;
            }
            [0xD4, lo, hi, ..] => {
                self.pc.add_un(2);
                if !self.cc.cy {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xD5, ..] => {
                self.push(self.de.get_twou8());
            }
            [0xD6, d8, ..] => {
                let carry = self.a.sub_carry(d8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
                self.pc += 1;
            }
            [0xD7, ..] => {
                self.call(0x10);
            }
            [0xD8, ..] => {
                if self.cc.cy {
                    self.ret();
                }
            }
            [0xD9, ..] => {}
            // JC
            [0xDA, lo, hi, ..] => {
                if self.cc.cy {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xDB, d8, ..] => {
                // special, IN
                self.a = self.io_device.port_in(d8);
                self.pc += 1;
            }
            [0xDC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.cy {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xDD, ..] => {}
            [0xDE, d8, ..] => {
                let carry1 = self.a.sub_carry(d8);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 || carry2;
                self.pc += 1;
            }
            [0xDF, ..] => {
                self.call(0x18);
            }

            [0xE0, ..] => {
                if !self.cc.p {
                    self.ret();
                }
            }
            [0xE1, ..] => {
                let x = self.pop();
                self.hl.set(x);
            }
            // JPO
            [0xE2, lo, hi, ..] => {
                if !self.cc.p {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xE3, ..] => {
                let a = self.pop();
                self.push(self.hl.get_twou8());
                self.hl.set(a);
            }
            [0xE4, lo, hi, ..] => {
                self.pc += 2;
                if !self.cc.p {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xE5, ..] => {
                self.push(self.hl.get_twou8());
            }
            [0xE6, d8, ..] => {
                self.a &= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc += 1;
            }
            [0xE7, ..] => {
                self.call(0x20);
            }
            [0xE8, ..] => {
                if self.cc.p {
                    self.ret();
                }
            }
            [0xE9, ..] => {
                self.pc = self.hl.into();
            }
            // JPE
            [0xEA, lo, hi, ..] => {
                if self.cc.p {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xEB, ..] => {
                let x = self.hl;
                self.hl.set(self.de);
                self.de.set(x);
            }
            [0xEC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.p {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xED, ..] => {}
            [0xEE, d8, ..] => {
                self.a ^= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc += 1;
            }
            [0xEF, ..] => {
                self.call(0x28);
            }

            [0xF0, ..] => {
                if !self.cc.s {
                    self.ret();
                }
            }
            [0xF1, ..] => {
                // 15                               0
                // [a : u8][ 7, 6, 5,  4, 3, 2, 1,  0 ]
                // [a : u8][ S, Z,  , AC,  , P,  , CY ]
                let popped = self.pop();
                let x = popped.lo;
                self.a = popped.hi;
                self.cc.s = x & 0b1000_0000 > 0;
                self.cc.z = x & 0b0100_0000 > 0;
                self.cc.ac = x & 0b0001_0000 > 0;
                self.cc.p = x & 0b0000_0100 > 0;
                self.cc.cy = x & 0b0000_0001 > 0;
            }
            // JP
            [0xF2, lo, hi, ..] => {
                if !self.cc.s {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            [0xF3, ..] => {
                // special
                // DI - disable interrupt
                self.int_enable = false;
            }
            [0xF4, lo, hi, ..] => {
                self.pc += 2;
                if !self.cc.s {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xF5, ..] => {
                // [a : u8][ 7, 6, 5,  4, 3, 2, 1,  0 ]
                // [a : u8][ S, Z,  , AC,  , P,  , CY ]
                let mut data = 0;
                data |= (self.cc.s as u8) << 7;
                data |= (self.cc.z as u8) << 6;
                //data |= (self.cc.s as u16) << 7;
                data |= (self.cc.ac as u8) << 4;
                //data |= (self.cc.s as u16) << 7;
                data |= (self.cc.p as u8) << 2;
                //data |= (self.cc.s as u16) << 7;
                data |= (self.cc.cy as u8) << 0;
                self.push(TwoU8::new(data, self.a));
            }
            [0xF6, d8, ..] => {
                self.a |= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc.add_un(1);
            }
            [0xF7, ..] => {
                self.call(0x30);
            }
            [0xF8, ..] => {
                if self.cc.s {
                    self.ret();
                }
            }
            [0xF9, ..] => {
                self.sp = self.hl.into();
            }
            // JM
            [0xFA, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.s {
                    self.pc = TwoU8 { lo, hi }.into();
                }
            }
            [0xFB, ..] => {
                // EI - Enable interrupt
                self.int_enable = true;
            }
            [0xFC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.s {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            [0xFD, ..] => {}
            [0xFE, d8, ..] => {
                self.cc.set_cmp(self.a, d8);
                self.pc.add_un(1);
            }
            [0xFF, ..] => {
                self.call(0x38);
            }

            _ => {
                eprintln!("unreachable reached, panic!");
                unreachable!();
            }
        };
    }

    #[inline(always)]
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    fn read_mem(&self, adr: impl Into<usize> + Copy) -> u8 {
        let adr: usize = adr.into();
        self.mem_limiter.check_read(adr as u16, self.mem[adr])
    }

    fn write_mem(&mut self, adr: impl Into<usize> + Copy, value: u8) {
        let adr: usize = adr.into();
        let action = self.mem_limiter.check_write(adr as u16, value);
        match action {
            WriteAction::Allow => self.mem[adr] = value,
            WriteAction::NewByte(b) => self.mem[adr] = b,
            WriteAction::Ignore => {}
        }
    }

    #[inline]
    pub fn generate_interrupt(&mut self, interrupt_num: u16) {
        self.int_enable = false;
        self.call(8 * interrupt_num);
    }

    #[inline]
    pub fn generate_int(&mut self, adr: u16) {
        self.int_enable = false;
        self.call(adr);
    }

    fn pop(&mut self) -> TwoU8 {
        let lo = self.read_mem(self.sp);
        let hi = self.read_mem(self.sp + 1);
        self.sp.add_un(2);
        TwoU8::new(lo, hi)
    }

    fn call(&mut self, adr: u16) {
        self.push(self.pc.into());
        self.pc = adr;
        // println!("CALL: {:04X}\nSP: {:04X}\nAdr: {:04X}", adr, self.sp, adr);
    }

    fn push(&mut self, data: TwoU8) {
        //let t : u16 = data.into();
        // println!("PUSH: data={:04X}", t);

        self.write_mem(self.sp - 1, data.hi);
        self.write_mem(self.sp - 2, data.lo);
        self.sp.sub_un(2);
    }

    fn ret(&mut self) {
        let data = self.pop();
        self.pc = data.into();
    }

    #[inline]
    pub fn int_enabled(&self) -> bool {
        self.int_enable
    }
}
