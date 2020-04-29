use crate::{disassemble, Argument, Cmd, MyToString};
use std::fmt::{self, Formatter};

pub struct Command {
    pub cmd: Cmd,
    pub args: Vec<Argument>,
    pub size: u8,
    pub bytes: Option<Vec<u8>>,
}

pub struct Iter<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for Iter<'a> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }
        let cmd = disassemble(self.bytes);
        self.bytes = self.bytes.split_at(cmd.size as usize).1;
        Some(cmd)
    }
}

impl Command {
    pub fn new(cmd: Cmd, args: Vec<Argument>, size: u8) -> Self {
        Command {
            cmd,
            args,
            size,
            bytes: None,
        }
    }

    pub fn iterator(bytes: &[u8]) -> Iter {
        Iter { bytes }
    }

    pub fn get_bytes(&self) -> &[u8] {
        self.bytes.as_ref().unwrap()
    }
}

impl From<(Cmd, Vec<Argument>, u8)> for Command {
    fn from(tuple: (Cmd, Vec<Argument>, u8)) -> Self {
        Command::new(tuple.0, tuple.1, tuple.2)
    }
}

impl From<(Cmd, Argument, u8)> for Command {
    fn from(tuple: (Cmd, Argument, u8)) -> Self {
        Command::new(tuple.0, vec![tuple.1], tuple.2)
    }
}

impl From<(Cmd, Argument, Argument, u8)> for Command {
    fn from(tuple: (Cmd, Argument, Argument, u8)) -> Self {
        Command::new(tuple.0, vec![tuple.1, tuple.2], tuple.3)
    }
}

impl From<(Cmd, u8)> for Command {
    fn from(tuple: (Cmd, u8)) -> Self {
        Command::new(tuple.0, vec![], tuple.1)
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes_str = format!("{:02X?}", self.bytes.as_ref().unwrap());
        if !self.args.is_empty() {
            write!(
                f,
                "{:>12} {:4} {:>}",
                bytes_str,
                self.cmd.as_ref(),
                self.args.to_string()
            )
        } else {
            write!(f, "{:>12} {:4}", bytes_str, self.cmd.as_ref())
        }
    }
}
