use std::fmt::Formatter;
use Argument::*;

extern crate strum;
#[macro_use]
extern crate strum_macros;

pub mod command;
pub use command::*;

pub mod argument;
pub use argument::*;

#[derive(IntoStaticStr)]
pub enum Cmd {
    NOP,
    NOPU,
    LXI,
    STAX,
    INX,
    INR,
    DCR,
    MVI,
    RLC,
    DAD,
    LDAX,
    DCX,
    RRC,
    RAL,
    RAR,
    RIM,
    SHLD,
    DAA,
    LHLD,
    CMA,
    SIM,
    STA,
    STC,
    LDA,
    CMC,
    MOV,
    HLT,
    ADD,
    ADC,
    SUB,
    SBB,
    ANA,
    XRA,
    ORA,
    CMP,
    RNZ,
    POP,
    JNZ,
    JMP,
    CNZ,
    PUSH,
    ADI,
    RST,
    RZ,
    RET,
    JZ,
    CZ,
    CALL,
    ACI,
    RNC,
    JNC,
    OUT,
    CNC,
    SUI,
    RC,
    JC,
    IN,
    CC,
    SBI,
    RPO,
    JPO,
    XTHL,
    CPO,
    ANI,
    RPE,
    PCHL,
    JPE,
    XCHG,
    CPE,
    XRI,
    RP,
    JP,
    DI,
    CP,
    ORI,
    RM,
    SPHL,
    JM,
    EI,
    CM,
    CPI,
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl AsRef<str> for Cmd {
    fn as_ref(&self) -> &'static str {
        self.into()
    }
}

pub fn disassemble(bytes: &[u8]) -> Command {
    // use Cmd::*;
    let mut command: Command = match *bytes {
        [0x0, ..] => (Cmd::NOP, 1).into(),
        [0x01, d16_lo, d16_hi, ..] => (Cmd::LXI, B, Addr(d16_lo, d16_hi), 3).into(),
        [0x02, ..] => (Cmd::STAX, B, 1).into(),
        [0x03, ..] => (Cmd::INX, B, 1).into(),
        [0x04, ..] => (Cmd::INR, B, 1).into(),
        [0x05, ..] => (Cmd::DCR, B, 1).into(),
        [0x06, d8, ..] => (Cmd::MVI, vec![B, D8(d8)], 2).into(),
        [0x07, ..] => (Cmd::RLC, 1).into(),
        [0x08, ..] => (Cmd::NOPU, 1).into(),
        [0x09, ..] => (Cmd::DAD, B, 1).into(),
        [0x0A, ..] => (Cmd::LDAX, B, 1).into(),
        [0x0B, ..] => (Cmd::DCX, (B), 1).into(),
        [0x0C, ..] => (Cmd::INR, C, 1).into(),
        [0x0D, ..] => (Cmd::DCR, C, 1).into(),
        [0x0E, d8, ..] => (Cmd::MVI, (C), D8(d8), 2).into(),
        [0x0F, ..] => (Cmd::RRC, 1).into(),

        [0x10, ..] => (Cmd::NOPU, 1).into(),
        [0x11, d16_lo, d16_hi, ..] => (Cmd::LXI, (D), D16(d16_lo, d16_hi), 3).into(),
        [0x12, ..] => (Cmd::STAX, D, 1).into(),
        [0x13, ..] => (Cmd::INX, D, 1).into(),
        [0x14, ..] => (Cmd::INR, D, 1).into(),
        [0x15, ..] => (Cmd::DCR, D, 1).into(),
        [0x16, d8, ..] => (Cmd::MVI, (D), D8(d8), 2).into(),
        [0x17, ..] => (Cmd::RAL, 1).into(),
        [0x18, ..] => (Cmd::NOPU, 1).into(),
        [0x19, ..] => (Cmd::DAD, D, 1).into(),
        [0x1A, ..] => (Cmd::LDAX, D, 1).into(),
        [0x1B, ..] => (Cmd::DCX, D, 1).into(),
        [0x1C, ..] => (Cmd::INR, E, 1).into(),
        [0x1D, ..] => (Cmd::DCR, E, 1).into(),
        [0x1E, d8, ..] => (Cmd::MVI, (E), D8(d8), 2).into(),
        [0x1F, ..] => (Cmd::RAR, 1).into(),

        [0x20, ..] => (Cmd::RIM, 1).into(),
        [0x21, d16_lo, d16_hi, ..] => (Cmd::LXI, (H), D16(d16_lo, d16_hi), 3).into(),
        [0x22, adr_lo, adr_hi, ..] => (Cmd::SHLD, Addr(adr_lo, adr_hi), 3).into(),
        [0x23, ..] => (Cmd::INX, H, 1).into(),
        [0x24, ..] => (Cmd::INR, H, 1).into(),
        [0x25, ..] => (Cmd::DCR, H, 1).into(),
        [0x26, d8, ..] => (Cmd::MVI, (H), D8(d8), 2).into(),
        [0x27, ..] => (Cmd::DAA, 1).into(),
        [0x28, ..] => (Cmd::NOPU, 1).into(),
        [0x29, ..] => (Cmd::DAD, H, 1).into(),
        [0x2A, adr_lo, adr_hi, ..] => (Cmd::LHLD, Addr(adr_lo, adr_hi), 3).into(),
        [0x2B, ..] => (Cmd::DCX, H, 1).into(),
        [0x2C, ..] => (Cmd::INR, L, 1).into(),
        [0x2D, ..] => (Cmd::DCR, L, 1).into(),
        [0x2E, d8, ..] => (Cmd::MVI, L, D8(d8), 2).into(),
        [0x2F, ..] => (Cmd::CMA, 1).into(),

        [0x30, ..] => (Cmd::SIM, 1).into(),
        [0x31, lo, hi, ..] => (Cmd::LXI, SP, D16(lo, hi), 3).into(),
        [0x32, lo, hi, ..] => (Cmd::STA, Addr(lo, hi), 3).into(),
        [0x33, ..] => (Cmd::INX, SP, 1).into(),
        [0x34, ..] => (Cmd::INR, M, 1).into(),
        [0x35, ..] => (Cmd::DCR, M, 1).into(),
        [0x36, d8, ..] => (Cmd::MVI, M, D8(d8), 2).into(),
        [0x37, ..] => (Cmd::STC, 1).into(),
        [0x38, ..] => (Cmd::NOPU, 1).into(),
        [0x39, ..] => (Cmd::DAD, 1).into(),
        [0x3A, lo, hi, ..] => (Cmd::LDA, Addr(lo, hi), 3).into(),
        [0x3B, ..] => (Cmd::DCX, SP, 1).into(),
        [0x3C, ..] => (Cmd::INR, A, 1).into(),
        [0x3D, ..] => (Cmd::DCR, A, 1).into(),
        [0x3E, d8, ..] => (Cmd::MVI, A, D8(d8), 2).into(),
        [0x3F, ..] => (Cmd::CMC, 1).into(),

        [0x40, ..] => (Cmd::MOV, B, B, 1).into(),
        [0x41, ..] => (Cmd::MOV, B, C, 1).into(),
        [0x42, ..] => (Cmd::MOV, B, D, 1).into(),
        [0x43, ..] => (Cmd::MOV, B, E, 1).into(),
        [0x44, ..] => (Cmd::MOV, B, H, 1).into(),
        [0x45, ..] => (Cmd::MOV, B, L, 1).into(),
        [0x46, ..] => (Cmd::MOV, B, M, 1).into(),
        [0x47, ..] => (Cmd::MOV, B, A, 1).into(),
        [0x48, ..] => (Cmd::MOV, C, B, 1).into(),
        [0x49, ..] => (Cmd::MOV, C, C, 1).into(),
        [0x4A, ..] => (Cmd::MOV, C, D, 1).into(),
        [0x4B, ..] => (Cmd::MOV, C, E, 1).into(),
        [0x4C, ..] => (Cmd::MOV, C, H, 1).into(),
        [0x4D, ..] => (Cmd::MOV, C, L, 1).into(),
        [0x4E, ..] => (Cmd::MOV, C, M, 1).into(),
        [0x4F, ..] => (Cmd::MOV, C, A, 1).into(),

        [0x50, ..] => (Cmd::MOV, D, B, 1).into(),
        [0x51, ..] => (Cmd::MOV, D, C, 1).into(),
        [0x52, ..] => (Cmd::MOV, D, D, 1).into(),
        [0x53, ..] => (Cmd::MOV, D, E, 1).into(),
        [0x54, ..] => (Cmd::MOV, D, H, 1).into(),
        [0x55, ..] => (Cmd::MOV, D, L, 1).into(),
        [0x56, ..] => (Cmd::MOV, D, M, 1).into(),
        [0x57, ..] => (Cmd::MOV, D, A, 1).into(),
        [0x58, ..] => (Cmd::MOV, E, B, 1).into(),
        [0x59, ..] => (Cmd::MOV, E, C, 1).into(),
        [0x5A, ..] => (Cmd::MOV, E, D, 1).into(),
        [0x5B, ..] => (Cmd::MOV, E, E, 1).into(),
        [0x5C, ..] => (Cmd::MOV, E, H, 1).into(),
        [0x5D, ..] => (Cmd::MOV, E, L, 1).into(),
        [0x5E, ..] => (Cmd::MOV, E, M, 1).into(),
        [0x5F, ..] => (Cmd::MOV, E, A, 1).into(),

        [0x60, ..] => (Cmd::MOV, H, B, 1).into(),
        [0x61, ..] => (Cmd::MOV, H, C, 1).into(),
        [0x62, ..] => (Cmd::MOV, H, D, 1).into(),
        [0x63, ..] => (Cmd::MOV, H, E, 1).into(),
        [0x64, ..] => (Cmd::MOV, H, H, 1).into(),
        [0x65, ..] => (Cmd::MOV, H, L, 1).into(),
        [0x66, ..] => (Cmd::MOV, H, M, 1).into(),
        [0x67, ..] => (Cmd::MOV, H, A, 1).into(),
        [0x68, ..] => (Cmd::MOV, L, B, 1).into(),
        [0x69, ..] => (Cmd::MOV, L, C, 1).into(),
        [0x6A, ..] => (Cmd::MOV, L, D, 1).into(),
        [0x6B, ..] => (Cmd::MOV, L, E, 1).into(),
        [0x6C, ..] => (Cmd::MOV, L, H, 1).into(),
        [0x6D, ..] => (Cmd::MOV, L, L, 1).into(),
        [0x6E, ..] => (Cmd::MOV, L, M, 1).into(),
        [0x6F, ..] => (Cmd::MOV, L, A, 1).into(),

        [0x70, ..] => (Cmd::MOV, M, B, 1).into(),
        [0x71, ..] => (Cmd::MOV, M, C, 1).into(),
        [0x72, ..] => (Cmd::MOV, M, D, 1).into(),
        [0x73, ..] => (Cmd::MOV, M, E, 1).into(),
        [0x74, ..] => (Cmd::MOV, M, H, 1).into(),
        [0x75, ..] => (Cmd::MOV, M, L, 1).into(),
        [0x76, ..] => (Cmd::HLT, 1).into(),
        [0x77, ..] => (Cmd::MOV, M, A, 1).into(),
        [0x78, ..] => (Cmd::MOV, A, B, 1).into(),
        [0x79, ..] => (Cmd::MOV, A, C, 1).into(),
        [0x7A, ..] => (Cmd::MOV, A, D, 1).into(),
        [0x7B, ..] => (Cmd::MOV, A, E, 1).into(),
        [0x7C, ..] => (Cmd::MOV, A, H, 1).into(),
        [0x7D, ..] => (Cmd::MOV, A, L, 1).into(),
        [0x7E, ..] => (Cmd::MOV, A, M, 1).into(),
        [0x7F, ..] => (Cmd::MOV, A, A, 1).into(),

        [0x80, ..] => (Cmd::ADD, B, 1).into(),
        [0x81, ..] => (Cmd::ADD, C, 1).into(),
        [0x82, ..] => (Cmd::ADD, D, 1).into(),
        [0x83, ..] => (Cmd::ADD, E, 1).into(),
        [0x84, ..] => (Cmd::ADD, H, 1).into(),
        [0x85, ..] => (Cmd::ADD, L, 1).into(),
        [0x86, ..] => (Cmd::ADD, M, 1).into(),
        [0x87, ..] => (Cmd::ADD, A, 1).into(),
        [0x88, ..] => (Cmd::ADC, B, 1).into(),
        [0x89, ..] => (Cmd::ADC, C, 1).into(),
        [0x8A, ..] => (Cmd::ADC, D, 1).into(),
        [0x8B, ..] => (Cmd::ADC, E, 1).into(),
        [0x8C, ..] => (Cmd::ADC, H, 1).into(),
        [0x8D, ..] => (Cmd::ADC, L, 1).into(),
        [0x8E, ..] => (Cmd::ADC, M, 1).into(),
        [0x8F, ..] => (Cmd::ADC, A, 1).into(),

        [0x90, ..] => (Cmd::SUB, B, 1).into(),
        [0x91, ..] => (Cmd::SUB, C, 1).into(),
        [0x92, ..] => (Cmd::SUB, D, 1).into(),
        [0x93, ..] => (Cmd::SUB, E, 1).into(),
        [0x94, ..] => (Cmd::SUB, H, 1).into(),
        [0x95, ..] => (Cmd::SUB, L, 1).into(),
        [0x96, ..] => (Cmd::SUB, M, 1).into(),
        [0x97, ..] => (Cmd::SUB, A, 1).into(),
        [0x98, ..] => (Cmd::SBB, B, 1).into(),
        [0x99, ..] => (Cmd::SBB, C, 1).into(),
        [0x9A, ..] => (Cmd::SBB, D, 1).into(),
        [0x9B, ..] => (Cmd::SBB, E, 1).into(),
        [0x9C, ..] => (Cmd::SBB, H, 1).into(),
        [0x9D, ..] => (Cmd::SBB, L, 1).into(),
        [0x9E, ..] => (Cmd::SBB, M, 1).into(),
        [0x9F, ..] => (Cmd::SBB, A, 1).into(),

        [0xA0, ..] => (Cmd::ANA, B, 1).into(),
        [0xA1, ..] => (Cmd::ANA, C, 1).into(),
        [0xA2, ..] => (Cmd::ANA, D, 1).into(),
        [0xA3, ..] => (Cmd::ANA, E, 1).into(),
        [0xA4, ..] => (Cmd::ANA, H, 1).into(),
        [0xA5, ..] => (Cmd::ANA, L, 1).into(),
        [0xA6, ..] => (Cmd::ANA, M, 1).into(),
        [0xA7, ..] => (Cmd::ANA, A, 1).into(),
        [0xA8, ..] => (Cmd::XRA, B, 1).into(),
        [0xA9, ..] => (Cmd::XRA, C, 1).into(),
        [0xAA, ..] => (Cmd::XRA, D, 1).into(),
        [0xAB, ..] => (Cmd::XRA, E, 1).into(),
        [0xAC, ..] => (Cmd::XRA, H, 1).into(),
        [0xAD, ..] => (Cmd::XRA, L, 1).into(),
        [0xAE, ..] => (Cmd::XRA, M, 1).into(),
        [0xAF, ..] => (Cmd::XRA, A, 1).into(),

        [0xB0, ..] => (Cmd::ORA, B, 1).into(),
        [0xB1, ..] => (Cmd::ORA, C, 1).into(),
        [0xB2, ..] => (Cmd::ORA, D, 1).into(),
        [0xB3, ..] => (Cmd::ORA, E, 1).into(),
        [0xB4, ..] => (Cmd::ORA, H, 1).into(),
        [0xB5, ..] => (Cmd::ORA, L, 1).into(),
        [0xB6, ..] => (Cmd::ORA, M, 1).into(),
        [0xB7, ..] => (Cmd::ORA, A, 1).into(),
        [0xB8, ..] => (Cmd::CMP, B, 1).into(),
        [0xB9, ..] => (Cmd::CMP, C, 1).into(),
        [0xBA, ..] => (Cmd::CMP, D, 1).into(),
        [0xBB, ..] => (Cmd::CMP, E, 1).into(),
        [0xBC, ..] => (Cmd::CMP, H, 1).into(),
        [0xBD, ..] => (Cmd::CMP, L, 1).into(),
        [0xBE, ..] => (Cmd::CMP, M, 1).into(),
        [0xBF, ..] => (Cmd::CMP, A, 1).into(),

        [0xC0, ..] => (Cmd::RNZ, 1).into(),
        [0xC1, ..] => (Cmd::POP, B, 1).into(),
        [0xC2, lo, hi, ..] => (Cmd::JNZ, Addr(lo, hi), 3).into(),
        [0xC3, lo, hi, ..] => (Cmd::JMP, Addr(lo, hi), 3).into(),
        [0xC4, lo, hi, ..] => (Cmd::CNZ, Addr(lo, hi), 3).into(),
        [0xC5, ..] => (Cmd::PUSH, B, 1).into(),
        [0xC6, d8, ..] => (Cmd::ADI, D8(d8), 2).into(),
        [0xC7, ..] => (Cmd::RST, D8(0), 1).into(),
        [0xC8, ..] => (Cmd::RZ, 1).into(),
        [0xC9, ..] => (Cmd::RET, 1).into(),
        [0xCA, lo, hi, ..] => (Cmd::JZ, Addr(lo, hi), 3).into(),
        [0xCB, ..] => (Cmd::NOPU, 1).into(),
        [0xCC, lo, hi, ..] => (Cmd::CZ, Addr(lo, hi), 3).into(),
        [0xCD, lo, hi, ..] => (Cmd::CALL, Addr(lo, hi), 3).into(),
        [0xCE, d8, ..] => (Cmd::ACI, D8(d8), 2).into(),
        [0xCF, ..] => (Cmd::RST, D8(1), 1).into(),

        [0xD0, ..] => (Cmd::RNC, 1).into(),
        [0xD1, ..] => (Cmd::POP, D, 1).into(),
        [0xD2, lo, hi, ..] => (Cmd::JNC, Addr(lo, hi), 3).into(),
        [0xD3, d8, ..] => (Cmd::OUT, D8(d8), 2).into(),
        [0xD4, lo, hi, ..] => (Cmd::CNC, Addr(lo, hi), 3).into(),
        [0xD5, ..] => (Cmd::PUSH, D, 1).into(),
        [0xD6, d8, ..] => (Cmd::SUI, D8(d8), 2).into(),
        [0xD7, ..] => (Cmd::RST, D8(2), 1).into(),
        [0xD8, ..] => (Cmd::RC, 1).into(),
        [0xD9, ..] => (Cmd::NOPU, 1).into(),
        [0xDA, lo, hi, ..] => (Cmd::JC, Addr(lo, hi), 3).into(),
        [0xDB, d8, ..] => (Cmd::IN, D8(d8), 2).into(),
        [0xDC, lo, hi, ..] => (Cmd::CC, Addr(lo, hi), 3).into(),
        [0xDD, ..] => (Cmd::NOPU, 1).into(),
        [0xDE, d8, ..] => (Cmd::SBI, D8(d8), 2).into(),
        [0xDF, ..] => (Cmd::RST, D8(3), 1).into(),

        [0xE0, ..] => (Cmd::RPO, 1).into(),
        [0xE1, ..] => (Cmd::POP, H, 1).into(),
        [0xE2, lo, hi, ..] => (Cmd::JPO, Addr(lo, hi), 3).into(),
        [0xE3, ..] => (Cmd::XTHL, 1).into(),
        [0xE4, lo, hi, ..] => (Cmd::CPO, Addr(lo, hi), 3).into(),
        [0xE5, ..] => (Cmd::PUSH, H, 1).into(),
        [0xE6, d8, ..] => (Cmd::ANI, D8(d8), 2).into(),
        [0xE7, ..] => (Cmd::RST, D8(4), 1).into(),
        [0xE8, ..] => (Cmd::RPE, 1).into(),
        [0xE9, ..] => (Cmd::PCHL, 1).into(),
        [0xEA, lo, hi, ..] => (Cmd::JPE, Addr(lo, hi), 3).into(),
        [0xEB, ..] => (Cmd::XCHG, 1).into(),
        [0xEC, lo, hi, ..] => (Cmd::CPE, Addr(lo, hi), 3).into(),
        [0xED, ..] => (Cmd::NOPU, 1).into(),
        [0xEE, d8, ..] => (Cmd::XRI, D8(d8), 2).into(),
        [0xEF, ..] => (Cmd::RST, D8(5), 1).into(),

        [0xF0, ..] => (Cmd::RP, 1).into(),
        [0xF1, ..] => (Cmd::POP, PSW, 1).into(),
        [0xF2, lo, hi, ..] => (Cmd::JP, Addr(lo, hi), 3).into(),
        [0xF3, ..] => (Cmd::DI, 1).into(),
        [0xF4, lo, hi, ..] => (Cmd::CP, Addr(lo, hi), 3).into(),
        [0xF5, ..] => (Cmd::PUSH, PSW, 1).into(),
        [0xF6, d8, ..] => (Cmd::ORI, D8(d8), 2).into(),
        [0xF7, ..] => (Cmd::RST, D8(6), 1).into(),
        [0xF8, ..] => (Cmd::RM, 1).into(),
        [0xF9, ..] => (Cmd::SPHL, 1).into(),
        [0xFA, lo, hi, ..] => (Cmd::JM, Addr(lo, hi), 3).into(),
        [0xFB, ..] => (Cmd::EI, 1).into(),
        [0xFC, lo, hi, ..] => (Cmd::CM, Addr(lo, hi), 3).into(),
        [0xFD, ..] => (Cmd::NOPU, 1).into(),
        [0xFE, d8, ..] => (Cmd::CPI, D8(d8), 2).into(),
        [0xFF, ..] => (Cmd::RST, D8(7), 1).into(),

        _ => {
            let what = bytes.iter().take(3).collect::<Vec<_>>();
            eprintln!("Next three bytes: {:X?}", what);
            todo!()
        } //[0x,..] => (vec!["".into(), ], 1),
    };
    command.bytes = Some(bytes[..command.size as usize].to_vec());
    command
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn it_works() {
//         let rom :Vec<u8> = vec![00, 00, 00, 0xc3, 0xd4, 0x18, 00, 00];
//         //let rom :Vec<u8> = vec![ 0x1, 0xFF, 0x12 ];
//         //let rom: Vec<u8> = vec![0xc3, 0xFF, 0x12];
//         let mut right = rom.as_slice();
//         while !right.is_empty() {
//             let size = disassemble(right);
//             right = right.split_at(size as usize).1;
//         }
//         assert_eq!(disassemble(&rom), 3);
//     }
// }
