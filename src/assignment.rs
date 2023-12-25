use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;
use crate::address::{Address, Destination, TryAdd, TryAddError};
use crate::program_error::ParseError;

#[derive(Debug, Clone)]
pub enum Assignment {
    Value(Type),
    Address(Address),
}


impl From<Destination> for Assignment {
    fn from(destination: Destination) -> Self {
        match destination {
            Destination::Register(register) => Assignment::Address(Address::Register(register.clone())),
            Destination::StackPointer(s) => Assignment::Address(Address::StackPointer(s)),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum OperationError {
    Subtraction(Type, Type),
    TryAdd(#[from] TryAddError),
    WrongType { expected: String, actual: String }
}

impl Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            OperationError::Subtraction(t1, t2) => {
                format!("Attempted subtracting two incompatible types: [{t1}] - [{t2}]")
            }
            OperationError::TryAdd(a) => format!("Attempting adding two incompatible types: {a}"),
            OperationError::WrongType { expected, actual } => {
                format!("Type {expected} is expected but the actual value was {actual}")
            }
        })
    }
}

#[derive(Clone, PartialEq)]
pub enum Type {
    String(String),
    Integer(isize),
    Address(Address),
    Untyped
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::String(a) => format!("{a:?}"),
            Type::Integer(a) => format!("{a}"),
            Type::Address(a) => format!("[{a}]"),
            Type::Untyped => "Untyped".to_string(),
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::String(a) => format!("String '{a}'"),
            Type::Integer(a) => format!("Integer '{a}'"),
            Type::Address(a) => format!("Address '[{a}]'"),
            Type::Untyped => "Untyped".to_string(),
        })
    }
}

impl Type {
    pub fn sub(&self, other: &Type) -> Result<Type, OperationError> {
        if let (Type::Integer(a), Type::Integer(b)) = (self, other) {
            return Ok(Type::Integer(a - b));
        }

        if let (Type::Address(Address::StackPointer(i)), Type::Address(Address::StackPointer(j))) = (self, other) {
            return Ok(Type::Address(Address::StackPointer(i - j)));
        }

        Err(OperationError::Subtraction(self.clone(), other.clone()))
    }

    pub fn add(&self, other: &Type) -> Result<Type, OperationError> {
        match (self, other) {
            (Type::Integer(o1), Type::Integer(o2)) => Ok(Type::Integer(o1 + o2)),

            (Type::Address(addr), Type::Integer(i)) => Ok(Type::Address(addr.try_add(i)?)),
            (Type::Address(addr1), Type::Address(addr2)) => Ok(Type::Address(addr1.try_add(addr2)?)),

            (Type::String(a), Type::Integer(b)) => Ok(Type::String(format!("{a}{b}"))),
            (Type::Integer(a), Type::String(b)) => Ok(Type::String(format!("{a}{b}"))),
            (Type::String(a), Type::String(b)) => Ok(Type::String(format!("{a}{b}"))),

            (a, b) => Ok(Type::String(format!("{a}{b}", a = a.to_string_raw(), b = b.to_string_raw()))),
        }
    }

    pub fn to_string_raw(&self) -> String {
        match self {
            Type::String(a) => format!("{a}"),
            Type::Integer(a) => format!("{a}"),
            Type::Address(a) => format!("{a}"),
            Type::Untyped => "".to_string(),
        }
    }
}

impl FromStr for Assignment {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(destination) = Address::from_str(s) {
            return Ok(Assignment::Address(destination));
        }

        if let Ok(value) = s.parse::<isize>() {
            return Ok(Assignment::Value(Type::Integer(value)));
        }

        let binding = s.replace('"', " \" ");
        let line = binding
            .split(' ')
            .collect::<Vec<_>>();

        if let ["\"", strings @ .., "\""] = &line[..] {
            return Ok(Assignment::Value(Type::String(strings.join(" "))));
        } else if let [_, "\"", strings @ .., "\"", _] = &line[..] {
            return Ok(Assignment::Value(Type::String(strings.join(" "))));
        }

        Err(ParseError::new(&format!("{s} cannot be parsed as an assignment")))
    }
}

impl Display for Assignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Assignment::Value(value) => format!("{value}"),
            Assignment::Address(destination) => format!("{destination}"),
        })
    }
}