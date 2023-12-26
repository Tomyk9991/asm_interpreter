use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;
use crate::program_error::ParseError;
use crate::register::Register;

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Register(Register),
    StackPointer(usize),
    Reference(Destination)
}


impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Address::Register(register) => format!("{register}"),
            Address::StackPointer(stack_pointer) => format!("0x{stack_pointer}"),
            Address::Reference(destination) => format!("{}", *destination),
        })
    }
}

pub trait TryAdd<T> {
    type Output;
    type Error;
    fn try_add(&self, other: &T) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Error, Clone)]
pub enum TryOperateTypes {
    IncompatibleTypes(String, String),
}

impl Display for TryOperateTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TryOperateTypes::IncompatibleTypes(a, b) => format!("({a}, {b})")
        })
    }
}

impl TryAdd<Address> for Address {
    type Output = Address;
    type Error = TryOperateTypes;

    fn try_add(&self, rhs: &Address) -> Result<Self::Output, Self::Error>  {
        match (&self, &rhs) {
            (Address::StackPointer(i), Address::StackPointer(j)) => Ok(Address::StackPointer(*i + *j)),
            (a1, a2) => Err(TryOperateTypes::IncompatibleTypes((*a1).clone().to_string(), (*a2).clone().to_string()))
        }
    }
}

impl TryAdd<isize> for Address {
    type Output = Address;
    type Error = TryOperateTypes;

    fn try_add(&self, rhs: &isize) -> Result<Self::Output, Self::Error>  {
        match (&self, rhs) {
            (Address::StackPointer(i), j) => Ok(Address::StackPointer((((*i) as isize) + *j) as usize)),
            (a1, a2) => Err(TryOperateTypes::IncompatibleTypes((*a1).to_string(), (*a2).to_string()))
        }
    }
}

impl FromStr for Address {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let ["sp"] = &s.split_whitespace().collect::<Vec<_>>()[..] {
            return Ok(Address::StackPointer(0));
        }

        if let ["[", address_str @ .., "]"] = &s.replace('[', " [ ").replace(']', " ] ").split_whitespace().collect::<Vec<_>>()[..] {
            let address = Self::from_str(&address_str.join(""))?;

            return match address {
                Address::Reference(reference) => Ok(Address::Reference(reference)),
                Address::Register(register) => Ok(Address::Reference(Destination::Register(register))),
                Address::StackPointer(s) => Ok(Address::Reference(Destination::StackPointer(s)))
            }
        }

        if let ["sp", "[", index, "]"] = &s.replace('[', " [ ").replace(']', " ] ").split_whitespace().collect::<Vec<_>>()[..] {
            return Ok(Address::StackPointer(index.parse::<usize>()?));
        }

        match s {
            "rax" => Ok(Address::Register(Register::Rax)),
            "rbx" => Ok(Address::Register(Register::Rbx)),
            "rcx" => Ok(Address::Register(Register::Rcx)),
            a => Err(ParseError::new(&format!("Address unknown: {a}")))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Destination {
    Register(Register),
    StackPointer(usize)
}

impl Display for Destination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Destination::Register(r) => format!("{r}"),
            Destination::StackPointer(s) => format!("0x{s}")
        })
    }
}
