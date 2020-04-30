use crate::structs::{ConditionalCodes, TwoU8, BC, DE, HL};
use crate::traits::{DataBus, OverflowMath};
use std::fmt::{self, Formatter};
extern crate rs8080_disassembler as disasm;
use crate::traits::{MemLimiter, WriteAction};
use crate::ClockCycles;
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

    pub fn emulate_next(&mut self) -> ClockCycles {
        let mut cycles = ClockCycles(0);
        let mem_from_pc = &self.mem[self.pc as usize..];
        self.pc.add_un(1);
        match *mem_from_pc {
            // NOP
            [0x0, ..] => cycles = 4.into(),
            // LXI B,D16
            [0x01, lo, hi, ..] => {
                self.bc.set(TwoU8 { lo, hi });
                self.pc += 2;
            }
            // STAX B
            [0x02, ..] => {
                self.write_mem(self.bc, self.a);
            }
            // INX B
            [0x03, ..] => {
                self.bc += 1;
            }
            // INR B
            [0x04, ..] => {
                self.bc.b.add_un(1);
                self.cc.set_zspac(self.bc.b);
            }
            // DCR B
            [0x05, ..] => {
                self.bc.b.sub_un(1);
                self.cc.set_zspac(self.bc.b);
            }
            // MVI B, D8
            [0x06, d8, ..] => {
                self.bc.b = d8;
                self.pc += 1;
            }
            // RLC
            [0x07, ..] => {
                self.cc.cy = self.a & 0b1000_0000 > 0;
                self.a = self.a.rotate_left(1);
            }
            // Nop (Undocumented)
            [0x08, ..] => {}
            // DAD B
            [0x09, ..] => {
                let carry = self.hl.add_carry(self.bc.into());
                self.cc.cy = carry;
            }
            // LDAX B
            [0x0A, ..] => {
                self.a = self.read_mem(self.bc);
            }
            // DCX B
            [0x0B, ..] => {
                self.bc -= 1;
            }
            // INR C
            [0x0C, ..] => {
                let rez = self.bc.c as u16 + 1;
                self.cc.set_zspac(rez);
                self.bc.c = rez as u8;
            }
            // DCR C
            [0x0D, ..] => {
                self.bc.c.sub_un(1);
                self.cc.set_zspac(self.bc.c);
            }
            // MVI C,D8
            [0x0E, d8, ..] => {
                self.bc.c = d8;
                self.pc += 1;
            }
            // RRC
            [0x0F, ..] => {
                self.cc.cy = self.a & 0x1 > 0;
                self.a = self.a.rotate_right(1);
            }

            // Nop (Undocumented)
            [0x10, ..] => {}
            // LXI D,D16
            [0x11, d16_lo, d16_hi, ..] => {
                self.de.d = d16_hi;
                self.de.e = d16_lo;
                self.pc += 2;
            }
            // STAX D
            [0x12, ..] => {
                self.write_mem(self.de, self.a);
            }
            // INX D
            [0x13, ..] => {
                self.de += 1;
            }
            // INR D
            [0x14, ..] => {
                let rez = self.de.d as u16 + 1;
                self.cc.set_zspac(rez);
                self.de.d = rez as u8;
            }
            // DCR D
            [0x15, ..] => {
                let rez = self.de.d as u16 - 1;
                self.cc.set_zspac(rez);
                self.de.d = rez as u8;
            }
            // MVI D, D8
            [0x16, d8, ..] => {
                self.de.d = d8;
                self.pc += 1;
            }
            // RAL
            [0x17, ..] => {
                let prev_cy = self.cc.cy;
                self.cc.cy = self.a & 0b1000_0000 > 0;
                self.a = self.a << 1;
                self.a |= prev_cy as u8;
            }
            // Nop (Undocumented)
            [0x18, ..] => {}
            // DAD D
            [0x19, ..] => {
                let carry = self.hl.add_carry(self.de.into());
                self.cc.cy = carry;
            }
            // LDAX D
            [0x1A, ..] => {
                self.a = self.read_mem(self.de);
            }
            // DCX D
            [0x1B, ..] => {
                self.de -= 1;
            }
            // INR E
            [0x1C, ..] => {
                let rez = (self.de.e as u16).wrapping_add(1);
                self.de.e = rez as u8;
                self.cc.set_zspac(rez);
            }
            // DCR E
            [0x1D, ..] => {
                let rez = (self.de.e as u16).wrapping_sub(1);
                self.de.e = rez as u8;
                self.cc.set_zspac(rez);
            }
            // MVI E,D8
            [0x1E, d8, ..] => {
                self.de.e = d8;
                self.pc += 1;
            }
            // RAR
            [0x1F, ..] => {
                let prev_cy = self.cc.cy;
                self.cc.cy = self.a & 0b0000_0001 > 0;
                self.a = self.a >> 1;
                self.a |= prev_cy as u8;
            }

            // Nop (Undocumented)
            [0x20, ..] => {}
            // LXI H,D16
            [0x21, lo, hi, ..] => {
                self.hl.set(TwoU8 { lo, hi });
                self.pc += 2;
            }
            // SHLD adr
            [0x22, lo, hi, ..] => {
                let adr: u16 = TwoU8 { lo, hi }.into();
                self.write_mem(adr, self.hl.l);
                self.write_mem(adr + 1, self.hl.h);
                self.pc += 2;
            }
            // INX H
            [0x23, ..] => {
                self.hl += 1;
            }
            // INR H
            [0x24, ..] => {
                let x = self.hl.h as u16 + 1;
                self.hl.h = x as u8;
                self.cc.set_zspac(x);
            }
            // DCR H
            [0x25, ..] => {
                let x = self.hl.h as u16 - 1;
                self.hl.h = x as u8;
                self.cc.set_zspac(x);
            }
            // MVI H,D8
            [0x26, d8, ..] => {
                self.hl.h = d8;
                self.pc += 1;
            }
            // DAA
            [0x27, ..] => {
                // TODO: DAA
                todo!();
                if self.a & 0xf > 9 {
                    self.a += 6;
                }
                if self.a & 0xf0 > 0x90 {
                    self.a.add_un(0x60);
                    self.cc.set_zspac(self.a);
                }
            }
            // Nop (Undocumented)
            [0x28, ..] => {}
            // DAD H
            [0x29, ..] => {
                let carry = self.hl.add_carry(self.hl.into());
                self.cc.cy = carry;
            }
            // LHLD adr
            [0x2A, lo, hi, ..] => {
                let adr: u16 = TwoU8 { lo, hi }.into();
                self.hl.l = self.read_mem(adr);
                self.hl.h = self.read_mem(adr + 1);
                self.pc += 2;
            }
            // DCX H
            [0x2B, ..] => {
                self.hl -= 1;
            }
            // INR L
            [0x2C, ..] => {
                self.hl.l.add_un(1);
                self.cc.set_zspac(self.hl.l);
            }
            // DCR L
            [0x2D, ..] => {
                let x = self.hl.l as u16 - 1;
                self.hl.l = x as u8;
                self.cc.set_zspac(x);
            }
            // MVI L, D8
            [0x2E, d8, ..] => {
                self.hl.l = d8;
                self.pc += 1;
            }
            // CMA
            [0x2F, ..] => {
                self.a = !self.a;
            }

            // Nop (Undocumented)
            [0x30, ..] => {}
            // LXI SP, D16
            [0x31, lo, hi, ..] => {
                self.sp = TwoU8 { lo, hi }.into();
                self.pc += 2;
            }
            // STA adr
            [0x32, lo, hi, ..] => {
                self.write_mem(TwoU8 { lo, hi }, self.a);
                self.pc += 2;
            }
            // INX SP
            [0x33, ..] => {
                self.sp = self.sp.wrapping_add(1);
            }
            // INR M
            [0x34, ..] => {
                let mut x = self.read_mem(self.hl);
                x.add_un(1);
                self.write_mem(self.hl, x);
                self.cc.set_zspac(x);
            }
            // DCR M
            [0x35, ..] => {
                let mut x = self.read_mem(self.hl);
                x.sub_un(1);
                self.write_mem(self.hl, x);
                self.cc.set_zspac(x);
            }
            // MVI M,D8
            [0x36, d8, ..] => {
                self.write_mem(self.hl, d8);
                self.pc += 1;
            }
            // STC
            [0x37, ..] => {
                self.cc.cy = true;
            }
            // Nop (Undocumented)
            [0x38, ..] => {}
            // DAD SP
            [0x39, ..] => {
                let carry = self.hl.add_carry(self.sp);
                self.cc.cy = carry;
            }
            // LDA adr
            [0x3A, lo, hi, ..] => {
                self.a = self.read_mem(TwoU8 { lo, hi });
                self.pc += 2;
            }
            // DCX SP
            [0x3B, ..] => {
                self.sp.sub_un(1);
            }
            // INR A
            [0x3C, ..] => {
                self.a.add_un(1);
                self.cc.set_zspac(self.a);
            }
            // DCR A
            [0x3D, ..] => {
                self.a.sub_un(1);
                self.cc.set_zspac(self.a);
            }
            // MVI A,D8
            [0x3E, d8, ..] => {
                self.a = d8;
                self.pc += 1;
            }
            // CMC
            [0x3F, ..] => {
                self.cc.cy = !self.cc.cy;
            }

            // MOV B,B
            [0x40, ..] => {
                //self.bc.b = self.bc.b;
            }
            // MOV B,C
            [0x41, ..] => {
                self.bc.b = self.bc.c;
            }
            // MOV B,D
            [0x42, ..] => {
                self.bc.b = self.de.d;
            }
            // MOV B,E
            [0x43, ..] => {
                self.bc.b = self.de.e;
            }
            // MOV B,H
            [0x44, ..] => {
                self.bc.b = self.hl.h;
            }
            // MOV B,L
            [0x45, ..] => {
                self.bc.b = self.hl.l;
            }
            // MOV B,M
            [0x46, ..] => {
                self.bc.b = self.read_mem(self.hl);
            }
            // MOV B,A
            [0x47, ..] => {
                self.bc.b = self.a;
            }
            // MOV C,B
            [0x48, ..] => {
                self.bc.c = self.bc.b;
            }
            // MOV C,C
            [0x49, ..] => {
                self.bc.c = self.bc.c;
            }
            // MOV C,D
            [0x4A, ..] => {
                self.bc.c = self.de.d;
            }
            // MOV C,E
            [0x4B, ..] => {
                self.bc.c = self.de.e;
            }
            // MOV C,H
            [0x4C, ..] => {
                self.bc.c = self.hl.h;
            }
            // MOV C,L
            [0x4D, ..] => {
                self.bc.c = self.hl.l;
            }
            // MOV C,M
            [0x4E, ..] => {
                self.bc.c = self.read_mem(self.hl);
            }
            // MOV C,A
            [0x4F, ..] => {
                self.bc.c = self.a;
            }

            // MOV D,B
            [0x50, ..] => {
                self.de.d = self.bc.b;
            }
            // MOV D,C
            [0x51, ..] => {
                self.de.d = self.bc.c;
            }
            // MOV D,D
            [0x52, ..] => {
                self.de.d = self.de.d;
            }
            // MOV D,E
            [0x53, ..] => {
                self.de.d = self.de.e;
            }
            // MOV D,H
            [0x54, ..] => {
                self.de.d = self.hl.h;
            }
            // MOV D,L
            [0x55, ..] => {
                self.de.d = self.hl.l;
            }
            // MOV D,M
            [0x56, ..] => {
                self.de.d = self.read_mem(self.hl);
            }
            // MOV D,A
            [0x57, ..] => {
                self.de.d = self.a;
            }
            // MOV E,B
            [0x58, ..] => {
                self.de.e = self.bc.b;
            }
            // MOV E,C
            [0x59, ..] => {
                self.de.e = self.bc.c;
            }
            // MOV E,D
            [0x5A, ..] => {
                self.de.e = self.de.d;
            }
            // MOV E,E
            [0x5B, ..] => {
                self.de.e = self.de.e;
            }
            // MOV E,H
            [0x5C, ..] => {
                self.de.e = self.hl.h;
            }
            // MOV E,L
            [0x5D, ..] => {
                self.de.e = self.hl.l;
            }
            // MOV E,M
            [0x5E, ..] => {
                self.de.e = self.read_mem(self.hl);
            }
            // MOV E,A
            [0x5F, ..] => {
                self.de.e = self.a;
            }

            // MOV H,B
            [0x60, ..] => {
                self.hl.h = self.bc.b;
            }
            // MOV H,C
            [0x61, ..] => {
                self.hl.h = self.bc.c;
            }
            // MOV H,D
            [0x62, ..] => {
                self.hl.h = self.de.d;
            }
            // MOV H,E
            [0x63, ..] => {
                self.hl.h = self.de.e;
            }
            // MOV H,H
            [0x64, ..] => {
                self.hl.h = self.hl.h;
            }
            // MOV H,L
            [0x65, ..] => {
                self.hl.h = self.hl.l;
            }
            // MOV H,M
            [0x66, ..] => {
                self.hl.h = self.read_mem(self.hl);
            }
            // MOV H,A
            [0x67, ..] => {
                self.hl.h = self.a;
            }
            // MOV L,B
            [0x68, ..] => {
                self.hl.l = self.bc.b;
            }
            // MOV L,C
            [0x69, ..] => {
                self.hl.l = self.bc.c;
            }
            // MOV L,D
            [0x6A, ..] => {
                self.hl.l = self.de.d;
            }
            // MOV L,E
            [0x6B, ..] => {
                self.hl.l = self.de.e;
            }
            // MOV L,H
            [0x6C, ..] => {
                self.hl.l = self.hl.h;
            }
            // MOV L,L
            [0x6D, ..] => {
                self.hl.l = self.hl.l;
            }
            // MOV L,M
            [0x6E, ..] => {
                self.hl.l = self.read_mem(self.hl);
            }
            // MOV L,A
            [0x6F, ..] => {
                self.hl.l = self.a;
            }

            // MOV M,B
            [0x70, ..] => {
                self.write_mem(self.hl, self.bc.b);
            }
            // MOV M,C
            [0x71, ..] => {
                self.write_mem(self.hl, self.bc.c);
            }
            // MOV M,D
            [0x72, ..] => {
                self.write_mem(self.hl, self.de.d);
            }
            // MOV M,E
            [0x73, ..] => {
                self.write_mem(self.hl, self.de.e);
            }
            // MOV M,H
            [0x74, ..] => {
                self.write_mem(self.hl, self.hl.h);
            }
            // MOV M,L
            [0x75, ..] => {
                self.write_mem(self.hl, self.hl.l);
            }
            // HLT
            [0x76, ..] => {
                // TODO: HLT
                todo!()
            }
            // MOV M,A
            [0x77, ..] => {
                self.write_mem(self.hl, self.a);
            }
            // MOV A,B
            [0x78, ..] => {
                self.a = self.bc.b;
            }
            // MOV A,C
            [0x79, ..] => {
                self.a = self.bc.c;
            }
            // MOV A,D
            [0x7A, ..] => {
                self.a = self.de.d;
            }
            // MOV A,E
            [0x7B, ..] => {
                self.a = self.de.e;
            }
            // MOV A,H
            [0x7C, ..] => {
                self.a = self.hl.h;
            }
            // MOV A,L
            [0x7D, ..] => {
                self.a = self.hl.l;
            }
            // MOV A,M
            [0x7E, ..] => self.a = self.read_mem(self.hl),
            // MOV A,A
            [0x7F, ..] => {
                //self.a = self.a
            }

            // ADD B
            [0x80, ..] => {
                let carry = self.a.add_carry(self.bc.b);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD C
            [0x81, ..] => {
                let carry = self.a.add_carry(self.bc.c);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD D
            [0x82, ..] => {
                let carry = self.a.add_carry(self.de.d);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD E
            [0x83, ..] => {
                let carry = self.a.add_carry(self.de.e);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD H
            [0x84, ..] => {
                let carry = self.a.add_carry(self.hl.h);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD L
            [0x85, ..] => {
                let carry = self.a.add_carry(self.hl.l);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD M
            [0x86, ..] => {
                let carry = self.a.add_carry(self.read_mem(self.hl));
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADD A
            [0x87, ..] => {
                let carry = self.a.add_carry(self.a);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // ADC B
            [0x88, ..] => {
                let mut x = self.bc.b;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC C
            [0x89, ..] => {
                let mut x = self.bc.c;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC D
            [0x8A, ..] => {
                let mut x = self.de.d;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC E
            [0x8B, ..] => {
                let mut x = self.de.e;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC H
            [0x8C, ..] => {
                let mut x = self.hl.h;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC L
            [0x8D, ..] => {
                let mut x = self.hl.l;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC M
            [0x8E, ..] => {
                let mut x = self.read_mem(self.hl);
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // ADC A
            [0x8F, ..] => {
                let mut x = self.a;
                let carry1 = x.add_carry(self.cc.cy as u8);
                let carry2 = self.a.add_carry(x);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }

            // SUB B
            [0x90, ..] => {
                let carry = self.a.sub_carry(self.bc.b);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB C
            [0x91, ..] => {
                let carry = self.a.sub_carry(self.bc.c);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB D
            [0x92, ..] => {
                let carry = self.a.sub_carry(self.de.d);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB E
            [0x93, ..] => {
                let carry = self.a.sub_carry(self.de.e);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB H
            [0x94, ..] => {
                let carry = self.a.sub_carry(self.hl.h);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB L
            [0x95, ..] => {
                let carry = self.a.sub_carry(self.hl.l);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB M
            [0x96, ..] => {
                let carry = self.a.sub_carry(self.read_mem(self.hl));
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SUB A
            [0x97, ..] => {
                let carry = self.a.sub_carry(self.a);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
            }
            // SBB B
            [0x98, ..] => {
                let carry1 = self.a.sub_carry(self.bc.b);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB C
            [0x99, ..] => {
                let carry1 = self.a.sub_carry(self.bc.c);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB D
            [0x9A, ..] => {
                let carry1 = self.a.sub_carry(self.de.d);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB E
            [0x9B, ..] => {
                let carry1 = self.a.sub_carry(self.de.e);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB H
            [0x9C, ..] => {
                let carry1 = self.a.sub_carry(self.hl.h);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB L
            [0x9D, ..] => {
                let carry1 = self.a.sub_carry(self.hl.l);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB M
            [0x9E, ..] => {
                let carry1 = self.a.sub_carry(self.read_mem(self.hl));
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }
            // SBB A
            [0x9F, ..] => {
                let carry1 = self.a.sub_carry(self.a);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 | carry2;
            }

            // ANA B
            [0xA0, ..] => {
                self.a &= self.bc.b;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA C
            [0xA1, ..] => {
                self.a &= self.bc.c;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA D
            [0xA2, ..] => {
                self.a &= self.de.d;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA E
            [0xA3, ..] => {
                self.a &= self.de.e;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA H
            [0xA4, ..] => {
                self.a &= self.hl.h;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA L
            [0xA5, ..] => {
                self.a &= self.hl.l;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA M
            [0xA6, ..] => {
                self.a &= self.read_mem(self.hl);
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ANA A
            [0xA7, ..] => {
                self.a &= self.a;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA B
            [0xA8, ..] => {
                self.a ^= self.bc.b;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA C
            [0xA9, ..] => {
                self.a ^= self.bc.c;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA D
            [0xAA, ..] => {
                self.a ^= self.de.d;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA E
            [0xAB, ..] => {
                self.a ^= self.de.e;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA H
            [0xAC, ..] => {
                self.a ^= self.hl.h;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA L
            [0xAD, ..] => {
                self.a ^= self.hl.l;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA M
            [0xAE, ..] => {
                self.a ^= self.read_mem(self.hl);
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // XRA A
            [0xAF, ..] => {
                self.a ^= self.a;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }

            // ORA B
            [0xB0, ..] => {
                self.a |= self.bc.b;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA C
            [0xB1, ..] => {
                self.a |= self.bc.c;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA D
            [0xB2, ..] => {
                self.a |= self.de.d;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA E
            [0xB3, ..] => {
                self.a |= self.de.e;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA H
            [0xB4, ..] => {
                self.a |= self.hl.h;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA L
            [0xB5, ..] => {
                self.a |= self.hl.l;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA M
            [0xB6, ..] => {
                self.a |= self.read_mem(self.hl);
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // ORA A
            [0xB7, ..] => {
                self.a |= self.a;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
            }
            // CMP B
            [0xB8, ..] => {
                self.cc.set_cmp(self.a, self.bc.b);
            }
            // CMP C
            [0xB9, ..] => {
                self.cc.set_cmp(self.a, self.bc.c);
            }
            // CMP D
            [0xBA, ..] => {
                self.cc.set_cmp(self.a, self.de.d);
            }
            // CMP E
            [0xBB, ..] => {
                self.cc.set_cmp(self.a, self.de.e);
            }
            // CMP H
            [0xBC, ..] => {
                self.cc.set_cmp(self.a, self.hl.h);
            }
            // CMP L
            [0xBD, ..] => {
                self.cc.set_cmp(self.a, self.hl.l);
            }
            // CMP M
            [0xBE, ..] => {
                self.cc.set_cmp(self.a, self.read_mem(self.hl));
            }
            // CMP A
            [0xBF, ..] => {
                self.cc.set_cmp(self.a, self.a);
            }

            // RNZ
            [0xC0, ..] => {
                if self.cc.z == false {
                    self.ret();
                }
            }
            // POP B
            [0xC1, ..] => {
                let data = self.pop();
                self.bc.set(data);
            }
            // JNZ
            // JNZ adr
            [0xC2, lo, hi, ..] => {
                if !self.cc.z {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // JMP adr
            [0xC3, lo, hi, ..] => {
                self.pc = TwoU8 { lo, hi }.into();
            }
            // CNZ adr
            [0xC4, lo, hi, ..] => {
                self.pc += 2;
                if !self.cc.z {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH B
            [0xC5, ..] => {
                self.push(self.bc.get_twou8());
            }
            // ADI D8
            [0xC6, d8, ..] => {
                self.cc.cy = self.a.add_carry(d8);
                self.cc.set_zspac(self.a);
                self.pc += 1;
            }
            // RST 0
            [0xC7, ..] => {
                self.call(0);
            }
            // RZ
            [0xC8, ..] => {
                if self.cc.z == true {
                    self.ret();
                }
            }
            // RET
            [0xC9, ..] => {
                self.ret();
            }
            // JZ
            // JZ adr
            [0xCA, lo, hi, ..] => {
                if self.cc.z {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // Nop (Undocumented)
            [0xCB, ..] => {}
            // CZ adr
            [0xCC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.z {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // CALL adr
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
            // ACI D8
            [0xCE, d8, ..] => {
                let carry1 = self.a.add_carry(d8);
                let carry2 = self.a.add_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 || carry2;
                self.pc += 1;
            }
            // RST 1
            [0xCF, ..] => {
                self.call(0x8);
            }

            // RNC
            [0xD0, ..] => {
                if !self.cc.cy {
                    self.ret();
                }
            }
            // POP D
            [0xD1, ..] => {
                let x = self.pop();
                self.de.set(x);
            }
            // JNC
            // JNC adr
            [0xD2, lo, hi, ..] => {
                if !self.cc.cy {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // OUT D8
            [0xD3, d8, ..] => {
                // special, OUT
                self.io_device.port_out(self.a, d8);
                self.pc += 1;
            }
            // CNC adr
            [0xD4, lo, hi, ..] => {
                self.pc.add_un(2);
                if !self.cc.cy {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH D
            [0xD5, ..] => {
                self.push(self.de.get_twou8());
            }
            // SUI D8
            [0xD6, d8, ..] => {
                let carry = self.a.sub_carry(d8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry;
                self.pc += 1;
            }
            // RST 2
            [0xD7, ..] => {
                self.call(0x10);
            }
            // RC
            [0xD8, ..] => {
                if self.cc.cy {
                    self.ret();
                }
            }
            // Nop (Undocumented)
            [0xD9, ..] => {}
            // JC
            // JC adr
            [0xDA, lo, hi, ..] => {
                if self.cc.cy {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // IN D8
            [0xDB, d8, ..] => {
                // special, IN
                self.a = self.io_device.port_in(d8);
                self.pc += 1;
            }
            // CC adr
            [0xDC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.cy {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // Nop (Undocumented)
            [0xDD, ..] => {}
            // SBI D8
            [0xDE, d8, ..] => {
                let carry1 = self.a.sub_carry(d8);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 || carry2;
                self.pc += 1;
            }
            // RST 3
            [0xDF, ..] => {
                self.call(0x18);
            }

            // RPO
            [0xE0, ..] => {
                if !self.cc.p {
                    self.ret();
                }
            }
            // POP H
            [0xE1, ..] => {
                let x = self.pop();
                self.hl.set(x);
            }
            // JPO
            // JPO adr
            [0xE2, lo, hi, ..] => {
                if !self.cc.p {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // XTHL
            [0xE3, ..] => {
                let a = self.pop();
                self.push(self.hl.get_twou8());
                self.hl.set(a);
            }
            // CPO adr
            [0xE4, lo, hi, ..] => {
                self.pc += 2;
                if !self.cc.p {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH H
            [0xE5, ..] => {
                self.push(self.hl.get_twou8());
            }
            // ANI D8
            [0xE6, d8, ..] => {
                self.a &= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc += 1;
            }
            // RST 4
            [0xE7, ..] => {
                self.call(0x20);
            }
            // RPE
            [0xE8, ..] => {
                if self.cc.p {
                    self.ret();
                }
            }
            // PCHL
            [0xE9, ..] => {
                self.pc = self.hl.into();
            }
            // JPE
            // JPE adr
            [0xEA, lo, hi, ..] => {
                if self.cc.p {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // XCHG
            [0xEB, ..] => {
                let x = self.hl;
                self.hl.set(self.de);
                self.de.set(x);
            }
            // CPE adr
            [0xEC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.p {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // Nop (Undocumented)
            [0xED, ..] => {}
            // XRI D8
            [0xEE, d8, ..] => {
                self.a ^= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc += 1;
            }
            // RST 5
            [0xEF, ..] => {
                self.call(0x28);
            }

            // RP
            [0xF0, ..] => {
                if !self.cc.s {
                    self.ret();
                }
            }
            // POP PSW
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
            // JP adr
            [0xF2, lo, hi, ..] => {
                if !self.cc.s {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // DI
            [0xF3, ..] => {
                // special
                // DI - disable interrupt
                self.int_enable = false;
            }
            // CP adr
            [0xF4, lo, hi, ..] => {
                self.pc += 2;
                if !self.cc.s {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH PSW
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
            // ORI D8
            [0xF6, d8, ..] => {
                self.a |= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc.add_un(1);
            }
            // RST 6
            [0xF7, ..] => {
                self.call(0x30);
            }
            // RM
            [0xF8, ..] => {
                if self.cc.s {
                    self.ret();
                }
            }
            // SPHL
            [0xF9, ..] => {
                self.sp = self.hl.into();
            }
            // JM
            // JM adr
            [0xFA, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.s {
                    self.pc = TwoU8 { lo, hi }.into();
                }
            }
            // EI
            [0xFB, ..] => {
                // EI - Enable interrupt
                self.int_enable = true;
            }
            // CM adr
            [0xFC, lo, hi, ..] => {
                self.pc += 2;
                if self.cc.s {
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // Nop (Undocumented)
            [0xFD, ..] => {}
            // CPI D8
            [0xFE, d8, ..] => {
                self.cc.set_cmp(self.a, d8);
                self.pc.add_un(1);
            }
            // RST 7
            [0xFF, ..] => {
                self.call(0x38);
            }

            _ => {
                eprintln!("unreachable reached, panic!");
                unreachable!();
            }
        };
        cycles
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
