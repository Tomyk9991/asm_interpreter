use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::program_error::ParseError;
use crate::register::Register;

#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Register(Register),
    StackPointer(usize),
}

impl Display for Destination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Destination::Register(register) => format!("{register}"),
            Destination::StackPointer(stack_pointer) => format!("Stack[{stack_pointer}]")
        })
    }
}

impl FromStr for Destination {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let ["sp", "[", index, "]"] = &s.replace('[', " [ ").replace(']', " ] ").split_whitespace().collect::<Vec<_>>()[..] {
            return Ok(Destination::StackPointer(index.parse::<usize>()?));
        }
        match s {
            "rax" => Ok(Destination::Register(Register::Rax)),
            "rbx" => Ok(Destination::Register(Register::Rbx)),
            "rcx" => Ok(Destination::Register(Register::Rcx)),
            a => Err(ParseError::new(&format!("Destination unknown: {a}")))
        }
    }
}