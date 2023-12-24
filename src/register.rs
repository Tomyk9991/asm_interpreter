use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Register::Rax => "rax",
            Register::Rbx => "rbx",
            Register::Rcx => "rcx"
        })
    }
}