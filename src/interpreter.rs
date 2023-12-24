use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use thiserror::Error;
use crate::assignment::Type;

use crate::command::Command;
use crate::destination::Destination;
use crate::jump::JumpDestination;
use crate::memory::Memory;
use crate::program_error::{ParseError, ProgramError};

#[derive(Debug)]
pub struct Interpreter {
    pub program_pointer: usize,
    pub register_states: Memory,
    pub source_code: Vec<Command>,
}

impl Display for Interpreter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("rax", &self.register_states.rax)
            .field("rbx", &self.register_states.rbx)
            .field("rcx", &self.register_states.rcx)
            .field("stack", &self.register_states.stack.iter().filter(|a| **a != Type::Untyped).collect::<Vec<_>>())
            .finish()
    }
}

pub type RegisterMemory = (Type, Type, Type);

#[derive(Debug)]
pub struct StackFrame {
    pub return_address: usize,
    pub entered_with_jmp: bool,
    pub destination: Option<Destination>,
    pub register_state: RegisterMemory,
}


impl FromStr for Interpreter {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut commands = vec![];
        for line in s.lines() {
            if line.is_empty() { continue; }
            commands.push(Command::from_str(line)?);
        }

        Ok(Self {
            register_states: Memory {
                rax: Type::Untyped,
                rbx: Type::Untyped,
                rcx: Type::Untyped,
                stack_frame: VecDeque::new(),
                stack: vec![Type::Untyped; 64],
            },
            program_pointer: 0,
            source_code: commands,
        })
    }
}

#[derive(Debug, Error)]
pub enum SemanticError {
    ReturnMissing { label: String },
    LeaveMissing { label: String }
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SemanticError::ReturnMissing { label } => format!("The label '{label}' is used with an expected return value, but no `ret ASSIGNMENT` is provided for all code paths"),
            SemanticError::LeaveMissing { label } => format!("The label '{label}' is used with a leave command, but no leave command is provided in all code paths")
        })
    }
}

impl Interpreter {
    pub fn semantic_check(&self) -> Result<(), ProgramError> {
        for command in &self.source_code {
            // if call is ran with a label, this label must have a ret command in all code paths
            match command {
                Command::CallRet(_, jump_destination) => {
                    jump_destination.ends_with(&self, |command| matches!(command, Command::Return(_)), |target_label| SemanticError::ReturnMissing { label: target_label.to_string() })?;
                },
                Command::CallVoid(jump_destination) => {
                    jump_destination.ends_with(&self, |command| matches!(command, Command::Return(_) | Command::Leave), |target_label| SemanticError::LeaveMissing { label: target_label.to_string() })?;
                },
                Command::Jmp(jump_destination) => {
                    jump_destination.ends_with(&self, |command| matches!(command, Command::Return(_) | Command::Leave), |target_label| SemanticError::LeaveMissing { label: target_label.to_string() })?;
                }
                _ => {}
            }
        }


        Ok(())
    }

    /// Resulting in new return_value, if holding
    pub fn mutate(&mut self, command: &Command) -> Result<Option<Type>, ProgramError> {
        match command {
            Command::CallVoid(JumpDestination::Label(target_label)) | Command::CallRet(_, JumpDestination::Label(target_label)) | Command::Jmp(JumpDestination::Label(target_label)) => {
                let potential_index = self.source_code.iter().position(|a|
                    matches!(a, Command::Label(source_label) if *source_label == *target_label)
                );

                if let Some(index) = potential_index {
                    self.program_pointer = index;
                } else {
                    return Err(ProgramError::LabelNotFound(target_label.to_string()));
                }
            }
            Command::Return(assignment) => {
                let value = command.get_value(&self.register_states, assignment)?;
                if self.register_states.stack_frame.is_empty() {
                    return Ok(Some(value));
                } else if let Some(stack_frame) = self.register_states.stack_frame.pop_front() {
                    if let Some(destination) = stack_frame.destination {
                        self.register_states.set(&destination, value)?;
                    }

                    if !stack_frame.entered_with_jmp {
                        (self.register_states.rax, self.register_states.rbx, self.register_states.rcx) = stack_frame.register_state;
                    }

                    self.program_pointer = stack_frame.return_address;
                }
            }
            Command::Leave => {
                if self.register_states.stack_frame.is_empty() {
                    return Ok(None)
                } else if let Some(stack_frame) = self.register_states.stack_frame.pop_front() {
                    assert_eq!(stack_frame.destination, None);

                    if !stack_frame.entered_with_jmp {
                        (self.register_states.rax, self.register_states.rbx, self.register_states.rcx) = stack_frame.register_state;
                    }

                    self.program_pointer = stack_frame.return_address;
                }
            }
            Command::Mov(_, _) | Command::Add(_, _, _) | Command::Sub(_, _, _) | Command::Label(_) | Command::Syscall(_) => {}
        }

        Ok(None)
    }
}