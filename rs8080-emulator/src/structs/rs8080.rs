use crate::structs::{ConditionalCodes, TwoU8, BC, DE, HL};
use crate::traits::{DataBus, OverflowMath};
use std::fmt::{self, Formatter};
extern crate rs8080_disassembler as disasm;
use crate::traits::{MemLimiter, WriteAction};
use crate::ClockCycles;
use disasm::{disassemble, Command};

/// Default mem access policy, allowing all writes and reads
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
    /// Creates new emulated CPU, mem access policy is allow all
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

    /// Sets new mem access policy
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
    }

    #[inline]
    /// Returns [Command](./../rs8080_disassembler/command/struct.Command.html) that
    /// implements `Display` trait
    pub fn disassemble_next(&self) -> Command {
        disassemble(&self.mem[self.pc as usize..])
    }

    /// Emulate next opcode pointed by program counter.
    /// Returns [ClockCycles](./struct.ClockCycles.html) spent on the opcode
    pub fn emulate_next(&mut self) -> ClockCycles {
        let mut cycles;
        let mem_from_pc = &self.mem[self.pc as usize..];
        self.pc.add_un(1);
        match *mem_from_pc {
            // NOP
            [0x0, ..] => cycles = 4.into(),
            // LXI B,D16
            [0x01, lo, hi, ..] => {
                cycles = 10.into();
                self.bc.set(TwoU8 { lo, hi });
                self.pc += 2;
            }
            // STAX B
            [0x02, ..] => {
                cycles = 7.into();
                self.write_mem(self.bc, self.a);
            }
            // INX B
            [0x03, ..] => {
                cycles = 5.into();
                self.bc.add_un(1);
            }
            // INR B
            [0x04, ..] => {
                cycles = 5.into();
                self.bc.b.add_un(1);
                self.cc.set_zspac(self.bc.b);
            }
            // DCR B
            [0x05, ..] => {
                cycles = 5.into();
                self.bc.b.sub_un(1);
                self.cc.set_zspac(self.bc.b);
            }
            // MVI B, D8
            [0x06, d8, ..] => {
                cycles = 7.into();
                self.bc.b = d8;
                self.pc += 1;
            }
            // RLC
            [0x07, ..] => {
                cycles = 4.into();
                self.cc.cy = self.a & 0b1000_0000 > 0;
                self.a = self.a.rotate_left(1);
            }
            // Nop (Undocumented)
            [0x08, ..] => cycles = 4.into(),
            // DAD B
            [0x09, ..] => {
                cycles = 10.into();
                self.dad(self.bc.into());
            }
            // LDAX B
            [0x0A, ..] => {
                cycles = 7.into();
                self.a = self.read_mem(self.bc);
            }
            // DCX B
            [0x0B, ..] => {
                cycles = 5.into();
                self.bc.sub_un(1);
            }
            // INR C
            [0x0C, ..] => {
                cycles = 5.into();
                self.bc.c.add_un(1);
                self.cc.set_zspac(self.bc.c);
            }
            // DCR C
            [0x0D, ..] => {
                cycles = 5.into();
                self.bc.c.sub_un(1);
                self.cc.set_zspac(self.bc.c);
            }
            // MVI C,D8
            [0x0E, d8, ..] => {
                cycles = 7.into();
                self.bc.c = d8;
                self.pc += 1;
            }
            // RRC
            [0x0F, ..] => {
                cycles = 4.into();
                self.cc.cy = self.a & 0x1 > 0;
                self.a = self.a.rotate_right(1);
            }

            // Nop (Undocumented)
            [0x10, ..] => cycles = 4.into(),
            // LXI D,D16
            [0x11, lo, hi, ..] => {
                cycles = 10.into();
                self.de.set(TwoU8{lo,hi});
                self.pc += 2;
            }
            // STAX D
            [0x12, ..] => {
                cycles = 7.into();
                self.write_mem(self.de, self.a);
            }
            // INX D
            [0x13, ..] => {
                cycles = 5.into();
                self.de.add_un(1);
            }
            // INR D
            [0x14, ..] => {
                cycles = 5.into();
                self.de.d.add_un(1);
                self.cc.set_zspac(self.de.d);
            }
            // DCR D
            [0x15, ..] => {
                cycles = 5.into();
                self.de.d.sub_un(1);
                self.cc.set_zspac(self.de.d);
            }
            // MVI D, D8
            [0x16, d8, ..] => {
                cycles = 7.into();
                self.de.d = d8;
                self.pc += 1;
            }
            // RAL
            [0x17, ..] => {
                cycles = 4.into();
                // let prev_cy = self.cc.cy;
                // self.cc.cy = self.a & 0b1000_0000 > 0;
                // self.a = self.a << 1;
                // self.a |= prev_cy as u8;
                let prev_cy = self.cc.cy;
                self.cc.cy = self.a & 0b1000_0000 > 0;
                self.a = self.a << 1;
                self.a |= prev_cy as u8;
            }
            // Nop (Undocumented)
            [0x18, ..] => cycles = 4.into(),
            // DAD D
            [0x19, ..] => {
                cycles = 10.into();
                self.dad(self.de.into());
            }
            // LDAX D
            [0x1A, ..] => {
                cycles = 7.into();
                self.a = self.read_mem(self.de);
            }
            // DCX D
            [0x1B, ..] => {
                cycles = 5.into();
                self.de.sub_un(1);
            }
            // INR E
            [0x1C, ..] => {
                cycles = 5.into();
                self.de.e.add_un(1);
                self.cc.set_zspac(self.de.e);
            }
            // DCR E
            [0x1D, ..] => {
                cycles = 5.into();
                self.de.e.sub_un(1);
                self.cc.set_zspac(self.de.e);
            }
            // MVI E,D8
            [0x1E, d8, ..] => {
                cycles = 7.into();
                self.de.e = d8;
                self.pc += 1;
            }
            // RAR
            [0x1F, ..] => {
                cycles = 4.into();
                // let prev_cy = self.cc.cy;
                // self.cc.cy = self.a & 0b0000_0001 > 0;
                // self.a = self.a >> 1;
                // self.a |= (prev_cy as u8) << 7 ;
                let prev_cy = self.cc.cy;
                self.cc.cy = self.a & 0b0000_0001 > 0;
                self.a = self.a >> 1;
                self.a |= (prev_cy as u8) << 7 ;
            }

            // Nop (Undocumented)
            [0x20, ..] => cycles = 4.into(),
            // LXI H,D16
            [0x21, lo, hi, ..] => {
                cycles = 10.into();
                self.hl.set(TwoU8 { lo, hi });
                self.pc += 2;
            }
            // SHLD adr
            [0x22, lo, hi, ..] => {
                cycles = 16.into();
                let adr: u16 = TwoU8 { lo, hi }.into();
                self.write_mem(adr, self.hl.l);
                self.write_mem(adr + 1, self.hl.h);
                self.pc += 2;
            }
            // INX H
            [0x23, ..] => {
                cycles = 5.into();
                self.hl.add_un(1);
            }
            // INR H
            [0x24, ..] => {
                cycles = 5.into();
                self.hl.h.add_un(1);
                self.cc.set_zspac(self.hl.h);
            }
            // DCR H
            [0x25, ..] => {
                cycles = 5.into();
                self.hl.h.sub_un(1);
                self.cc.set_zspac(self.hl.h);
            }
            // MVI H,D8
            [0x26, d8, ..] => {
                cycles = 7.into();
                self.hl.h = d8;
                self.pc += 1;
            }
            // DAA
            [0x27, ..] => {
                cycles = 4.into();
                if self.a & 0xf > 9 || self.cc.ac {
                    self.a.add_un(6);
                }
                if self.a & 0xf0 > 0x90 || self.cc.cy {
                    self.a.add_un(0x60);
                }
                self.cc.set_zspac(self.a);
                eprintln!("DAA");
            }
            // Nop (Undocumented)
            [0x28, ..] => cycles = 4.into(),
            // DAD H
            [0x29, ..] => {
                cycles = 10.into();
                self.dad(self.hl.into());
            }
            // LHLD adr
            [0x2A, lo, hi, ..] => {
                cycles = 16.into();
                let adr: u16 = TwoU8 { lo, hi }.into();
                self.hl.l = self.read_mem(adr);
                self.hl.h = self.read_mem(adr + 1);
                self.pc += 2;
            }
            // DCX H
            [0x2B, ..] => {
                cycles = 5.into();
                self.hl.sub_un(1);
            }
            // INR L
            [0x2C, ..] => {
                cycles = 5.into();
                self.hl.l.add_un(1);
                self.cc.set_zspac(self.hl.l);
            }
            // DCR L
            [0x2D, ..] => {
                cycles = 5.into();
                self.hl.l.sub_un(1);
                self.cc.set_zspac(self.hl.l);
            }
            // MVI L, D8
            [0x2E, d8, ..] => {
                cycles = 7.into();
                self.hl.l = d8;
                self.pc += 1;
            }
            // CMA
            [0x2F, ..] => {
                cycles = 4.into();
                self.a = !self.a;
            }

            // Nop (Undocumented)
            [0x30, ..] => cycles = 4.into(),
            // LXI SP, D16
            [0x31, lo, hi, ..] => {
                cycles = 10.into();
                self.sp = TwoU8 { lo, hi }.into();
                self.pc += 2;
            }
            // STA adr
            [0x32, lo, hi, ..] => {
                cycles = 13.into();
                self.write_mem(TwoU8 { lo, hi }, self.a);
                self.pc += 2;
            }
            // INX SP
            [0x33, ..] => {
                cycles = 5.into();
                self.sp.add_un(1);
            }
            // INR M
            [0x34, ..] => {
                cycles = 10.into();
                let mut x = self.read_mem(self.hl);
                x.add_un(1);
                self.write_mem(self.hl, x);
                self.cc.set_zspac(x);
            }
            // DCR M
            [0x35, ..] => {
                cycles = 10.into();
                let mut x = self.read_mem(self.hl);
                x.sub_un(1);
                self.write_mem(self.hl, x);
                self.cc.set_zspac(x);
            }
            // MVI M,D8
            [0x36, d8, ..] => {
                cycles = 10.into();
                self.write_mem(self.hl, d8);
                self.pc += 1;
            }
            // STC
            [0x37, ..] => {
                cycles = 4.into();
                self.cc.cy = true;
            }
            // Nop (Undocumented)
            [0x38, ..] => cycles = 4.into(),
            // DAD SP
            [0x39, ..] => {
                cycles = 10.into();
                self.dad(self.sp);
            }
            // LDA adr
            [0x3A, lo, hi, ..] => {
                cycles = 13.into();
                self.a = self.read_mem(TwoU8 { lo, hi });
                self.pc += 2;
            }
            // DCX SP
            [0x3B, ..] => {
                cycles = 5.into();
                self.sp.sub_un(1);
            }
            // INR A
            [0x3C, ..] => {
                cycles = 5.into();
                self.a.add_un(1);
                self.cc.set_zspac(self.a);
            }
            // DCR A
            [0x3D, ..] => {
                cycles = 5.into();
                self.a.sub_un(1);
                self.cc.set_zspac(self.a);
            }
            // MVI A,D8
            [0x3E, d8, ..] => {
                cycles = 7.into();
                self.a = d8;
                self.pc += 1;
            }
            // CMC
            [0x3F, ..] => {
                cycles = 4.into();
                self.cc.cy = !self.cc.cy;
            }

            // MOV B,B
            [0x40, ..] => {
                cycles = 5.into();
                self.bc.b = self.bc.b;
            }
            // MOV B,C
            [0x41, ..] => {
                cycles = 5.into();
                self.bc.b = self.bc.c;
            }
            // MOV B,D
            [0x42, ..] => {
                cycles = 5.into();
                self.bc.b = self.de.d;
            }
            // MOV B,E
            [0x43, ..] => {
                cycles = 5.into();
                self.bc.b = self.de.e;
            }
            // MOV B,H
            [0x44, ..] => {
                cycles = 5.into();
                self.bc.b = self.hl.h;
            }
            // MOV B,L
            [0x45, ..] => {
                cycles = 5.into();
                self.bc.b = self.hl.l;
            }
            // MOV B,M
            [0x46, ..] => {
                cycles = 7.into();
                self.bc.b = self.read_mem(self.hl);
            }
            // MOV B,A
            [0x47, ..] => {
                cycles = 5.into();
                self.bc.b = self.a;
            }
            // MOV C,B
            [0x48, ..] => {
                cycles = 5.into();
                self.bc.c = self.bc.b;
            }
            // MOV C,C
            [0x49, ..] => {
                cycles = 5.into();
                self.bc.c = self.bc.c;
            }
            // MOV C,D
            [0x4A, ..] => {
                cycles = 5.into();
                self.bc.c = self.de.d;
            }
            // MOV C,E
            [0x4B, ..] => {
                cycles = 5.into();
                self.bc.c = self.de.e;
            }
            // MOV C,H
            [0x4C, ..] => {
                cycles = 5.into();
                self.bc.c = self.hl.h;
            }
            // MOV C,L
            [0x4D, ..] => {
                cycles = 5.into();
                self.bc.c = self.hl.l;
            }
            // MOV C,M
            [0x4E, ..] => {
                cycles = 7.into();
                self.bc.c = self.read_mem(self.hl);
            }
            // MOV C,A
            [0x4F, ..] => {
                cycles = 5.into();
                self.bc.c = self.a;
            }

            // MOV D,B
            [0x50, ..] => {
                cycles = 5.into();
                self.de.d = self.bc.b;
            }
            // MOV D,C
            [0x51, ..] => {
                cycles = 5.into();
                self.de.d = self.bc.c;
            }
            // MOV D,D
            [0x52, ..] => {
                cycles = 5.into();
                self.de.d = self.de.d;
            }
            // MOV D,E
            [0x53, ..] => {
                cycles = 5.into();
                self.de.d = self.de.e;
            }
            // MOV D,H
            [0x54, ..] => {
                cycles = 5.into();
                self.de.d = self.hl.h;
            }
            // MOV D,L
            [0x55, ..] => {
                cycles = 5.into();
                self.de.d = self.hl.l;
            }
            // MOV D,M
            [0x56, ..] => {
                cycles = 7.into();
                self.de.d = self.read_mem(self.hl);
            }
            // MOV D,A
            [0x57, ..] => {
                cycles = 5.into();
                self.de.d = self.a;
            }
            // MOV E,B
            [0x58, ..] => {
                cycles = 5.into();
                self.de.e = self.bc.b;
            }
            // MOV E,C
            [0x59, ..] => {
                cycles = 5.into();
                self.de.e = self.bc.c;
            }
            // MOV E,D
            [0x5A, ..] => {
                cycles = 5.into();
                self.de.e = self.de.d;
            }
            // MOV E,E
            [0x5B, ..] => {
                cycles = 5.into();
                self.de.e = self.de.e;
            }
            // MOV E,H
            [0x5C, ..] => {
                cycles = 5.into();
                self.de.e = self.hl.h;
            }
            // MOV E,L
            [0x5D, ..] => {
                cycles = 5.into();
                self.de.e = self.hl.l;
            }
            // MOV E,M
            [0x5E, ..] => {
                cycles = 7.into();
                self.de.e = self.read_mem(self.hl);
            }
            // MOV E,A
            [0x5F, ..] => {
                cycles = 5.into();
                self.de.e = self.a;
            }

            // MOV H,B
            [0x60, ..] => {
                cycles = 5.into();
                self.hl.h = self.bc.b;
            }
            // MOV H,C
            [0x61, ..] => {
                cycles = 5.into();
                self.hl.h = self.bc.c;
            }
            // MOV H,D
            [0x62, ..] => {
                cycles = 5.into();
                self.hl.h = self.de.d;
            }
            // MOV H,E
            [0x63, ..] => {
                cycles = 5.into();
                self.hl.h = self.de.e;
            }
            // MOV H,H
            [0x64, ..] => {
                cycles = 5.into();
                self.hl.h = self.hl.h;
            }
            // MOV H,L
            [0x65, ..] => {
                cycles = 5.into();
                self.hl.h = self.hl.l;
            }
            // MOV H,M
            [0x66, ..] => {
                cycles = 7.into();
                self.hl.h = self.read_mem(self.hl);
            }
            // MOV H,A
            [0x67, ..] => {
                cycles = 5.into();
                self.hl.h = self.a;
            }
            // MOV L,B
            [0x68, ..] => {
                cycles = 5.into();
                self.hl.l = self.bc.b;
            }
            // MOV L,C
            [0x69, ..] => {
                cycles = 5.into();
                self.hl.l = self.bc.c;
            }
            // MOV L,D
            [0x6A, ..] => {
                cycles = 5.into();
                self.hl.l = self.de.d;
            }
            // MOV L,E
            [0x6B, ..] => {
                cycles = 5.into();
                self.hl.l = self.de.e;
            }
            // MOV L,H
            [0x6C, ..] => {
                cycles = 5.into();
                self.hl.l = self.hl.h;
            }
            // MOV L,L
            [0x6D, ..] => {
                cycles = 5.into();
                self.hl.l = self.hl.l;
            }
            // MOV L,M
            [0x6E, ..] => {
                cycles = 7.into();
                self.hl.l = self.read_mem(self.hl);
            }
            // MOV L,A
            [0x6F, ..] => {
                cycles = 5.into();
                self.hl.l = self.a;
            }

            // MOV M,B
            [0x70, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.bc.b);
            }
            // MOV M,C
            [0x71, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.bc.c);
            }
            // MOV M,D
            [0x72, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.de.d);
            }
            // MOV M,E
            [0x73, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.de.e);
            }
            // MOV M,H
            [0x74, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.hl.h);
            }
            // MOV M,L
            [0x75, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.hl.l);
            }
            // HLT
            [0x76, ..] => {
                // TODO: HLT
                todo!();
                cycles = 7.into();
            }
            // MOV M,A
            [0x77, ..] => {
                cycles = 7.into();
                self.write_mem(self.hl, self.a);
            }
            // MOV A,B
            [0x78, ..] => {
                cycles = 5.into();
                self.a = self.bc.b;
            }
            // MOV A,C
            [0x79, ..] => {
                cycles = 5.into();
                self.a = self.bc.c;
            }
            // MOV A,D
            [0x7A, ..] => {
                cycles = 5.into();
                self.a = self.de.d;
            }
            // MOV A,E
            [0x7B, ..] => {
                cycles = 5.into();
                self.a = self.de.e;
            }
            // MOV A,H
            [0x7C, ..] => {
                cycles = 5.into();
                self.a = self.hl.h;
            }
            // MOV A,L
            [0x7D, ..] => {
                cycles = 5.into();
                self.a = self.hl.l;
            }
            // MOV A,M
            [0x7E, ..] => {
                cycles = 7.into();
                self.a = self.read_mem(self.hl);
            }
            // MOV A,A
            [0x7F, ..] => {
                cycles = 5.into();
                self.a = self.a
            }

            // ADD B
            [0x80, ..] => {
                cycles = 4.into();
                self.add(self.bc.b);
            }
            // ADD C
            [0x81, ..] => {
                cycles = 4.into();
                self.add(self.bc.c);
            }
            // ADD D
            [0x82, ..] => {
                cycles = 4.into();
                self.add(self.de.d);
            }
            // ADD E
            [0x83, ..] => {
                cycles = 4.into();
                self.add(self.de.e);
            }
            // ADD H
            [0x84, ..] => {
                cycles = 4.into();
                self.add(self.hl.h);
            }
            // ADD L
            [0x85, ..] => {
                cycles = 4.into();
                self.add(self.hl.l);
            }
            // ADD M
            [0x86, ..] => {
                cycles = 7.into();
                self.add(self.read_mem(self.hl));
            }
            // ADD A
            [0x87, ..] => {
                cycles = 4.into();
                self.add(self.a);
            }
            // ADC B
            [0x88, ..] => {
                cycles = 4.into();
                self.adc(self.bc.b);
            }
            // ADC C
            [0x89, ..] => {
                cycles = 4.into();
                self.adc(self.bc.c);
            }
            // ADC D
            [0x8A, ..] => {
                cycles = 4.into();
                self.adc(self.de.d);
            }
            // ADC E
            [0x8B, ..] => {
                cycles = 4.into();
                self.adc(self.de.e);
            }
            // ADC H
            [0x8C, ..] => {
                cycles = 4.into();
                self.adc(self.hl.h);
            }
            // ADC L
            [0x8D, ..] => {
                cycles = 4.into();
                self.adc(self.hl.l);
            }
            // ADC M
            [0x8E, ..] => {
                cycles = 7.into();
                self.adc(self.read_mem(self.hl));
            }
            // ADC A
            [0x8F, ..] => {
                cycles = 4.into();
                self.adc(self.a);
            }

            // SUB B
            [0x90, ..] => {
                cycles = 4.into();
                self.sub(self.bc.b);
            }
            // SUB C
            [0x91, ..] => {
                cycles = 4.into();
                self.sub(self.bc.c);
            }
            // SUB D
            [0x92, ..] => {
                cycles = 4.into();
                self.sub(self.de.d);
            }
            // SUB E
            [0x93, ..] => {
                cycles = 4.into();
                self.sub(self.de.e);
            }
            // SUB H
            [0x94, ..] => {
                cycles = 4.into();
                self.sub(self.hl.h);
            }
            // SUB L
            [0x95, ..] => {
                cycles = 4.into();
                self.sub(self.hl.l);
            }
            // SUB M
            [0x96, ..] => {
                cycles = 7.into();
                self.sub(self.read_mem(self.hl));
            }
            // SUB A
            [0x97, ..] => {
                cycles = 4.into();
                self.sub(self.a);
            }
            // SBB B
            [0x98, ..] => {
                cycles = 4.into();
                self.sbb(self.bc.b);
            }
            // SBB C
            [0x99, ..] => {
                cycles = 4.into();
                self.sbb(self.bc.c);
            }
            // SBB D
            [0x9A, ..] => {
                cycles = 4.into();
                 self.sbb(self.de.d);
            }
            // SBB E
            [0x9B, ..] => {
                cycles = 4.into();
                self.sbb(self.de.e);
            }
            // SBB H
            [0x9C, ..] => {
                cycles = 4.into();
                self.sbb(self.hl.h);
            }
            // SBB L
            [0x9D, ..] => {
                cycles = 4.into();
                self.sbb(self.hl.l);
            }
            // SBB M
            [0x9E, ..] => {
                cycles = 7.into();
                self.sbb(self.read_mem(self.hl));
            }
            // SBB A
            [0x9F, ..] => {
                cycles = 4.into();
                self.sbb(self.a);
            }

            // ANA B
            [0xA0, ..] => {
                cycles = 4.into();
                self.ana(self.bc.b);
            }
            // ANA C
            [0xA1, ..] => {
                cycles = 4.into();
                self.ana(self.bc.c);
            }
            // ANA D
            [0xA2, ..] => {
                cycles = 4.into();
                self.ana(self.de.d);
            }
            // ANA E
            [0xA3, ..] => {
                cycles = 4.into();
                self.ana(self.de.e);
            }
            // ANA H
            [0xA4, ..] => {
                cycles = 4.into();
                self.ana(self.hl.h);
            }
            // ANA L
            [0xA5, ..] => {
                cycles = 4.into();
                self.ana(self.hl.l);
            }
            // ANA M
            [0xA6, ..] => {
                cycles = 7.into();
                self.ana(self.read_mem(self.hl));
            }
            // ANA A
            [0xA7, ..] => {
                cycles = 4.into();
                self.ana(self.a)
            }
            // XRA B
            [0xA8, ..] => {
                cycles = 4.into();
                self.xra(self.bc.b);
            }
            // XRA C
            [0xA9, ..] => {
                cycles = 4.into();
                self.xra(self.bc.c);
            }
            // XRA D
            [0xAA, ..] => {
                cycles = 4.into();
                self.xra(self.de.d);
            }
            // XRA E
            [0xAB, ..] => {
                cycles = 4.into();
                self.xra(self.de.e);
            }
            // XRA H
            [0xAC, ..] => {
                cycles = 4.into();
                self.xra(self.hl.h);
            }
            // XRA L
            [0xAD, ..] => {
                cycles = 4.into();
                self.xra(self.hl.l);
            }
            // XRA M
            [0xAE, ..] => {
                cycles = 7.into();
                self.xra(self.read_mem(self.hl));
            }
            // XRA A
            [0xAF, ..] => {
                cycles = 4.into();
                self.xra(self.a);
            }

            // ORA B
            [0xB0, ..] => {
                cycles = 4.into();
                self.ora(self.bc.b);
            }
            // ORA C
            [0xB1, ..] => {
                cycles = 4.into();
                self.ora(self.bc.c);
            }
            // ORA D
            [0xB2, ..] => {
                cycles = 4.into();
                self.ora(self.de.d);
            }
            // ORA E
            [0xB3, ..] => {
                cycles = 4.into();
                self.ora(self.de.e);
            }
            // ORA H
            [0xB4, ..] => {
                cycles = 4.into();
                self.ora(self.hl.h);
            }
            // ORA L
            [0xB5, ..] => {
                cycles = 4.into();
                self.ora(self.hl.l);
            }
            // ORA M
            [0xB6, ..] => {
                cycles = 7.into();
                self.ora(self.read_mem(self.hl));
            }
            // ORA A
            [0xB7, ..] => {
                cycles = 4.into();
                self.ora(self.a);
            }
            // CMP B
            [0xB8, ..] => {
                cycles = 4.into();
                self.cmp( self.bc.b);
            }
            // CMP C
            [0xB9, ..] => {
                cycles = 4.into();
                self.cmp( self.bc.c);
            }
            // CMP D
            [0xBA, ..] => {
                cycles = 4.into();
                self.cmp( self.de.d);
            }
            // CMP E
            [0xBB, ..] => {
                cycles = 4.into();
                self.cmp( self.de.e);
            }
            // CMP H
            [0xBC, ..] => {
                cycles = 4.into();
                self.cmp( self.hl.h);
            }
            // CMP L
            [0xBD, ..] => {
                cycles = 4.into();
                self.cmp( self.hl.l);
            }
            // CMP M
            [0xBE, ..] => {
                cycles = 7.into();
                self.cmp( self.read_mem(self.hl));
            }
            // CMP A
            [0xBF, ..] => {
                cycles = 4.into();
                self.cmp( self.a);
            }

            // RNZ
            [0xC0, ..] => {
                cycles = 5.into();
                if !self.cc.z{
                    cycles = 11.into();
                    self.ret();
                }
            }
            // POP B
            [0xC1, ..] => {
                cycles = 10.into();
                let data = self.pop();
                self.bc.set(data);
            }
            // JNZ adr
            [0xC2, lo, hi, ..] => {
                cycles = 10.into();
                if !self.cc.z {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // JMP adr
            [0xC3, lo, hi, ..] => {
                cycles = 10.into();
                self.pc = TwoU8 { lo, hi }.into();
            }
            // CNZ adr
            [0xC4, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if !self.cc.z {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH B
            [0xC5, ..] => {
                cycles = 11.into();
                self.push(self.bc.get_twou8());
            }
            // ADI D8
            [0xC6, d8, ..] => {
                cycles = 7.into();
                self.cc.cy = self.a.add_carry(d8);
                self.cc.set_zspac(self.a);
                self.pc += 1;
            }
            // RST 0
            [0xC7, ..] => {
                cycles = 11.into();
                self.call(0);
            }
            // RZ
            [0xC8, ..] => {
                cycles = 5.into();
                if self.cc.z{
                    cycles = 11.into();
                    self.ret();
                }
            }
            // RET
            [0xC9, ..] => {
                cycles = 10.into();
                self.ret();
            }
            // JZ adr
            [0xCA, lo, hi, ..] => {
                cycles = 10.into();
                if self.cc.z {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // Nop (Undocumented)
            [0xCB, ..] => cycles = 4.into(),
            // CZ adr
            [0xCC, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if self.cc.z {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // CALL adr
            [0xCD, lo, hi, ..] => {
                cycles = 17.into();
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
                cycles = 7.into();
                let carry1 = self.a.add_carry(d8);
                let carry2 = self.a.add_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 || carry2;
                self.pc += 1;
            }
            // RST 1
            [0xCF, ..] => {
                cycles = 11.into();
                self.call(0x8);
            }

            // RNC
            [0xD0, ..] => {
                cycles = 5.into();
                if !self.cc.cy {
                    cycles = 11.into();
                    self.ret();
                }
            }
            // POP D
            [0xD1, ..] => {
                cycles = 10.into();
                let x = self.pop();
                self.de.set(x);
            }
            // JNC adr
            [0xD2, lo, hi, ..] => {
                cycles = 10.into();
                if !self.cc.cy {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // OUT D8
            [0xD3, d8, ..] => {
                cycles = 10.into();
                self.io_device.port_out(self.a, d8);
                self.pc += 1;
            }
            // CNC adr
            [0xD4, lo, hi, ..] => {
                cycles = 11.into();
                self.pc.add_un(2);
                if !self.cc.cy {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH D
            [0xD5, ..] => {
                cycles = 11.into();
                self.push(self.de.get_twou8());
            }
            // SUI D8
            [0xD6, d8, ..] => {
                cycles = 7.into();
                self.cc.cy = self.a.sub_carry(d8);
                self.cc.set_zspac(self.a);
                self.pc += 1;
            }
            // RST 2
            [0xD7, ..] => {
                cycles = 11.into();
                self.call(0x10);
            }
            // RC
            [0xD8, ..] => {
                cycles = 5.into();
                if self.cc.cy {
                    cycles = 11.into();
                    self.ret();
                }
            }
            // Nop (Undocumented)
            [0xD9, ..] => cycles = 4.into(),
            // JC adr
            [0xDA, lo, hi, ..] => {
                cycles = 10.into();
                if self.cc.cy {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // IN D8
            [0xDB, d8, ..] => {
                cycles = 10.into();
                self.a = self.io_device.port_in(d8);
                self.pc += 1;
            }
            // CC adr
            [0xDC, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if self.cc.cy {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // Nop (Undocumented)
            [0xDD, ..] => cycles = 4.into(),
            // SBI D8
            [0xDE, d8, ..] => {
                cycles = 7.into();
                let carry1 = self.a.sub_carry(d8);
                let carry2 = self.a.sub_carry(self.cc.cy as u8);
                self.cc.set_zspac(self.a);
                self.cc.cy = carry1 || carry2;
                self.pc += 1;
            }
            // RST 3
            [0xDF, ..] => {
                cycles = 11.into();
                self.call(0x18);
            }

            // RPO
            [0xE0, ..] => {
                cycles = 5.into();
                if !self.cc.p {
                    cycles = 11.into();
                    self.ret();
                }
            }
            // POP H
            [0xE1, ..] => {
                cycles = 10.into();
                let x = self.pop();
                self.hl.set(x);
            }
            // JPO adr
            [0xE2, lo, hi, ..] => {
                cycles = 10.into();
                if !self.cc.p {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // XTHL
            [0xE3, ..] => {
                cycles = 18.into();
                let a = self.pop();
                self.push(self.hl.get_twou8());
                self.hl.set(a);
            }
            // CPO adr
            [0xE4, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if !self.cc.p {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH H
            [0xE5, ..] => {
                cycles = 11.into();
                self.push(self.hl.get_twou8());
            }
            // ANI D8
            [0xE6, d8, ..] => {
                cycles = 7.into();
                self.a &= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc += 1;
            }
            // RST 4
            [0xE7, ..] => {
                cycles = 11.into();
                self.call(0x20);
            }
            // RPE
            [0xE8, ..] => {
                cycles = 5.into();
                if self.cc.p {
                    cycles = 11.into();
                    self.ret();
                }
            }
            // PCHL
            [0xE9, ..] => {
                cycles = 5.into();
                self.pc = self.hl.into();
            }
            // JPE adr
            [0xEA, lo, hi, ..] => {
                cycles = 10.into();
                if self.cc.p {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // XCHG
            [0xEB, ..] => {
                cycles = 4.into();
                let x = self.hl;
                self.hl.set(self.de);
                self.de.set(x);
            }
            // CPE adr
            [0xEC, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if self.cc.p {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // Nop (Undocumented)
            [0xED, ..] => cycles = 4.into(),
            // XRI D8
            [0xEE, d8, ..] => {
                cycles = 7.into();
                self.a ^= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc += 1;
            }
            // RST 5
            [0xEF, ..] => {
                cycles = 11.into();
                self.call(0x28);
            }

            // RP
            [0xF0, ..] => {
                cycles = 5.into();
                if !self.cc.s {
                    cycles = 11.into();
                    self.ret();
                }
            }
            // POP PSW
            [0xF1, ..] => {
                cycles = 10.into();
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
            // JP adr
            [0xF2, lo, hi, ..] => {
                cycles = 10.into();
                if !self.cc.s {
                    self.pc = TwoU8 { lo, hi }.into();
                } else {
                    self.pc += 2;
                }
            }
            // DI - disable interrupt
            [0xF3, ..] => {
                cycles = 4.into();
                self.int_enable = false;
            }
            // CP adr
            [0xF4, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if !self.cc.s {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // PUSH PSW
            [0xF5, ..] => {
                cycles = 11.into();
                // [a : u8][ 7, 6, 5,  4, 3, 2, 1,  0 ]
                // [a : u8][ S, Z,  , AC,  , P,  , CY ]
                let mut data = 0;
                data |= (self.cc.s as u8) << 7;
                data |= (self.cc.z as u8) << 6;
                data |= (self.cc.ac as u8) << 4;
                data |= (self.cc.p as u8) << 2;
                data |= (self.cc.cy as u8) << 0;
                self.push(TwoU8::new(data, self.a));
            }
            // ORI D8
            [0xF6, d8, ..] => {
                cycles = 7.into();
                self.a |= d8;
                self.cc.set_zspac(self.a);
                self.cc.cy = false;
                self.pc.add_un(1);
            }
            // RST 6
            [0xF7, ..] => {
                cycles = 11.into();
                self.call(0x30);
            }
            // RM
            [0xF8, ..] => {
                cycles = 5.into();
                if self.cc.s {
                    cycles = 11.into();
                    self.ret();
                }
            }
            // SPHL
            [0xF9, ..] => {
                cycles = 5.into();
                self.sp = self.hl.into();
            }
            // JM adr
            [0xFA, lo, hi, ..] => {
                cycles = 10.into();
                self.pc += 2;
                if self.cc.s {
                    self.pc = TwoU8 { lo, hi }.into();
                }
            }
            // EI - Enable interrupt
            [0xFB, ..] => {
                cycles = 4.into();
                self.int_enable = true;
            }
            // CM adr
            [0xFC, lo, hi, ..] => {
                cycles = 11.into();
                self.pc += 2;
                if self.cc.s {
                    cycles = 17.into();
                    self.call(TwoU8 { lo, hi }.into());
                }
            }
            // Nop (Undocumented)
            [0xFD, ..] => cycles = 4.into(),
            // CPI D8
            [0xFE, d8, ..] => {
                cycles = 7.into();
                self.cc.set_cmp(self.a, d8);
                self.pc.add_un(1);
            }
            // RST 7
            [0xFF, ..] => {
                cycles = 11.into();
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
    /// Returns program counter
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
    /// `Call adr`, where `adr = interrupt_num * 0x8`,
    /// and sets `int_enable` to `false`
    pub fn generate_interrupt(&mut self, interrupt_num: u16) {
        self.int_enable = false;
        self.call(8 * interrupt_num);
    }

    pub fn call_interrupt(&mut self, call_adr : u16){
        //self.int_enable = false;
        self.call(call_adr);
    }
    
    //fn inx(&mut self, rp : &mut)

    fn dad(&mut self, rp : u16){
        self.cc.cy = self.hl.add_carry(rp);
    }

    fn cmp(&mut self, regm : u8){
        self.cc.set_cmp(self.a, regm);
    }

    fn ora(&mut self, regm : u8){
        self.a |= regm;
        self.cc.set_zspac(self.a);
        self.cc.cy = false;
    }

    fn xra(&mut self, regm : u8){
        self.a ^= regm;
        self.cc.set_zspac(self.a);
        self.cc.cy = false;
    }

    fn ana(&mut self, regm : u8){
        self.a &= regm;
        self.cc.set_zspac(self.a);
        self.cc.cy = false;
    }

    fn sbb(&mut self, regm : u8){
        let carry1 = self.a.sub_carry(regm);
        let carry2 = self.a.sub_carry(self.cc.cy as u8);
        self.cc.set_zspac(self.a);
        self.cc.cy = carry1 || carry2;
    }

    fn sub(&mut self, regm : u8){
        self.cc.cy = self.a.sub_carry(regm);
        self.cc.set_zspac(self.a);
    }

    fn add(&mut self, regm : u8){
        self.cc.cy = self.a.add_carry(regm);
        self.cc.set_zspac(self.a);
    }

    fn adc(&mut self, regm : u8){
        let carry1 = self.a.add_carry(self.cc.cy as u8);
        let carry2 = self.a.add_carry(regm);
        self.cc.set_zspac(self.a);
        self.cc.cy = carry1 || carry2;
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
    /// Returns whether interrupts are enabled or not
    pub fn int_enabled(&self) -> bool {
        self.int_enable
    }
}
