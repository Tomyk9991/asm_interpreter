use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use crate::assignment::{Assignment, OperationError, Type};
use crate::address::{Address};
use crate::interpreter::{RegisterMemory, StackFrame};
use crate::register::Register;

#[derive(Debug)]
pub struct Memory {
    pub rax: Type,
    pub rbx: Type,
    pub rcx: Type,
    pub stack_frame: Vec<StackFrame>,
    pub stack: Vec<Type>
}

#[derive(Error, Debug)]
pub enum MemoryError {
    Write(Address),
    Read(Assignment),
    SegmentationFault(String),
    OperationError(#[from] OperationError)
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MemoryError::Write(d) => format!("Cannot write at: {d}"),
            MemoryError::Read(a) => format!("Cannot not read at: {a}"),
            MemoryError::OperationError(o) => format!("Cannot operate: {o}"),
            MemoryError::SegmentationFault(fault_message) => format!("Segentation fault: {fault_message}")
        })
    }
}

impl Memory {
    pub fn get(&self, assignment: &Assignment) -> Result<Type, MemoryError> {
        match assignment {
            Assignment::Value(value) => Ok(value.clone()),
            Assignment::Address(Address::Register(register)) => {
                match register {
                    Register::Rax => Ok(self.rax.clone()),
                    Register::Rbx => Ok(self.rbx.clone()),
                    Register::Rcx => Ok(self.rcx.clone())
                }
            }
            Assignment::Address(Address::StackPointer(index)) => {
                if *index >= self.stack.len() {
                    return Err(MemoryError::Read(Assignment::Address(Address::StackPointer(*index))));
                }

                Ok(self.stack[*index].clone())
            },
            Assignment::Address(Address::Reference(reference)) => {
                let a = Assignment::from(reference.clone());

                Ok(self.get(&a)?)
            }
        }
    }

    pub fn set(&mut self, destination: &Address, value: Type) -> Result<(), MemoryError> {
        fn usize_from(memory: &Memory, ty: &Type) -> Result<usize, MemoryError> {
            match ty {
                Type::Integer(integer_value) => {
                    if *integer_value <= 0 || *integer_value as usize >= memory.stack.len() {
                        return Err(MemoryError::Read(Assignment::Value(Type::Integer(*integer_value))));
                    }

                    Ok(*integer_value as usize)
                }
                Type::Address(a) => {
                    return match a {
                        Address::StackPointer(i) => Ok(*i),
                        Address::Register(_) => Err(MemoryError::SegmentationFault("Cannot read a registers position".to_string())),
                        Address::Reference(_) => Err(MemoryError::SegmentationFault("Only single pointers supported".to_string()))
                    };
                }
                Type::String(t) => return Err(MemoryError::Read(Assignment::Value(Type::String(t.to_string())))),
                Type::Untyped => return Err(MemoryError::Read(Assignment::Value(Type::Untyped)))
            }
        }

        match destination {
            Address::Register(register) => {
                match register {
                    Register::Rax => self.rax = value,
                    Register::Rbx => self.rbx = value,
                    Register::Rcx => self.rcx = value
                }
            }
            Address::StackPointer(index) => {
                if *index >= self.stack.len() {
                    return Err(MemoryError::Write(destination.clone()));
                }

                self.stack[*index] = value;
            },
            Address::Reference(destination) => {
                let a = Assignment::from(destination.clone());
                let ty = &self.get(&a)?;
                let address = usize_from(self, ty)?;

                self.stack[address] = value;
            }
        }

        Ok(())
    }

    pub fn register_state(&self) -> RegisterMemory {
        (self.rax.clone(), self.rbx.clone(), self.rcx.clone())
    }
}