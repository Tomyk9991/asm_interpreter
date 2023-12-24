use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use crate::assignment::{Assignment, OperationError, Type};
use crate::destination::Destination;
use crate::interpreter::{RegisterMemory, StackFrame};
use crate::register::Register;

#[derive(Debug)]
pub struct Memory {
    pub rax: Type,
    pub rbx: Type,
    pub rcx: Type,
    pub stack_frame: VecDeque<StackFrame>,
    pub stack: Vec<Type>
}

#[derive(Error, Debug)]
pub enum MemoryError {
    Write(Destination),
    Read(Assignment),
    OperationError(#[from] OperationError)
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            MemoryError::Write(d) => format!("Cannot write at: {d}"),
            MemoryError::Read(a) => format!("Cannot not read at: {a}"),
            MemoryError::OperationError(o) => format!("Cannot operate: {o}")
        })
    }
}

impl Memory {
    pub fn set(&mut self, destination: &Destination, value: Type) -> Result<(), MemoryError> {
        match destination {
            Destination::Register(register) => {
                match register {
                    Register::Rax => self.rax = value,
                    Register::Rbx => self.rbx = value,
                    Register::Rcx => self.rcx = value
                }
            }
            Destination::StackPointer(index) => {
                if *index >= 64 {
                    return Err(MemoryError::Write(destination.clone()));
                }

                self.stack[*index] = value;
            }
        }

        Ok(())
    }

    pub fn register_state(&self) -> RegisterMemory {
        (self.rax.clone(), self.rbx.clone(), self.rcx.clone())
    }
}