use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use thiserror::Error;
use crate::interpreter::SemanticError;
use crate::memory::MemoryError;

#[derive(Debug, Error)]
pub struct ParseError {
    message: String,
}


impl From<ParseIntError> for ParseError {
    fn from(value: ParseIntError) -> Self {
        ParseError::new(&format!("Cannot parse value: {}", value))
    }
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed parsing the string: {}", self.message)
    }
}


#[derive(Debug, Error)]
pub enum ProgramError {
    Parse(#[from] ParseError),
    Memory(#[from] MemoryError),
    Semantic(#[from] SemanticError),
    LabelNotFound(String),
}

impl Display for ProgramError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProgramError::Parse(p) => format!("{p}"),
            ProgramError::Memory(m) => format!("{m}"),
            ProgramError::LabelNotFound(jump_destination) => format!("Cannot find jmp destination {jump_destination}"),
            ProgramError::Semantic(s) => format!("{s}")
        })
    }
}