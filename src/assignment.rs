use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::str::FromStr;
use thiserror::Error;
use crate::destination::Destination;
use crate::program_error::ParseError;

#[derive(Debug, Clone)]
pub enum Assignment {
    Value(Type),
    Register(Destination),
}

#[derive(Debug, Error, Clone)]
pub enum OperationError {
    Subtraction(Type, Type),
    WrongType { expected: String, actual: String }
}

impl Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            OperationError::Subtraction(t1, t2) => {
                format!("Attempted subtracting two incompatible types: [{t1}] - [{t2}]")
            }
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
    Untyped
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::String(a) => format!("{a:?}"),
            Type::Integer(a) => format!("{a}"),
            Type::Untyped => "Untyped".to_string()
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::String(a) => format!("String '{a}'"),
            Type::Integer(a) => format!("Integer '{a}'"),
            Type::Untyped => "Untyped".to_string()
        })
    }
}

impl Type {
    pub fn sub(&self, other: &Type) -> Result<Type, OperationError> {
        if let (Type::Integer(a), Type::Integer(b)) = (self, other) {
            return Ok(Type::Integer(a - b));
        }

        Err(OperationError::Subtraction(self.clone(), other.clone()))
    }

    pub fn to_string_raw(&self) -> String {
        match self {
            Type::String(a) => format!("{a}"),
            Type::Integer(a) => format!("{a}"),
            Type::Untyped => "".to_string()
        }
    }
}

impl Add for Type {
    type Output = Type;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Type::Integer(o1), Type::Integer(o2)) => Type::Integer(o1 + o2),
            (a, b) => Type::String(format!("{a}{b}", a = a.to_string_raw(), b = b.to_string_raw())),
        }
    }
}

impl FromStr for Assignment {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(destination) = Destination::from_str(s) {
            return Ok(Assignment::Register(destination));
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
            Assignment::Register(destination) => format!("{destination}"),
        })
    }
}