use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use thiserror::Error;
use crate::assignment::Type;

use crate::command::Command;
use crate::address::Address;
use crate::jump::JumpDestination;
use crate::memory::Memory;
use crate::program_error::{ParseError, ProgramError};

#[derive(Debug)]
pub struct Interpreter {
    pub program_pointer: usize,
    pub memory: Memory,
    pub source_code: Vec<Command>,
}

fn pretty_print_stack(min: usize, stack: &[Type]) -> Vec<String> {
    let mut printing_stack = vec![];

    pretty_print_stack_helper(min, stack, &mut printing_stack);
    return printing_stack;
}

fn pretty_print_stack_helper(min: usize, stack: &[Type], printing_stack: &mut Vec<String>) {
    if let Some(typed_position) = stack.iter().enumerate().position(|(index, a)| index >= min && *a != Type::Untyped) {
        if typed_position != min {
            printing_stack.push(format!("{min}..{}: {}", typed_position - 1,Type::Untyped));
        }

        printing_stack.push(format!("{typed_position}: {}", stack[typed_position]));
        pretty_print_stack_helper(typed_position + 1, stack, printing_stack);
    } else {
        printing_stack.push(format!("{min}..{end}: {}", Type::Untyped, end = stack.len()));
    }
}

impl Display for Interpreter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interpreter")
            .field("rax", &self.memory.rax)
            .field("rbx", &self.memory.rbx)
            .field("rcx", &self.memory.rcx)
            .field("stack", &pretty_print_stack(0, &self.memory.stack))
            .finish()
    }
}

pub type RegisterMemory = (Type, Type, Type);

#[derive(Debug)]
pub struct StackFrame {
    pub return_address: usize,
    pub entered_with_jmp: bool,
    pub destination: Option<Address>,
    pub register_state: RegisterMemory,
}


impl FromStr for Interpreter {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut commands = vec![];

        for line in s.lines() {
            if line.is_empty() { continue; }
            if line.trim().starts_with(';') { continue; }

            commands.push(Command::from_str(line)?);
        }

        Ok(Self {
            memory: Memory {
                rax: Type::Untyped,
                rbx: Type::Untyped,
                rcx: Type::Untyped,
                stack_frame: Vec::new(),
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
    LeaveMissing { label: String },
}

impl Display for SemanticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SemanticError::ReturnMissing { label } => format!("The label '{label}' is used with an expected return value, but no `ret ASSIGNMENT` is provided for all code paths"),
            SemanticError::LeaveMissing { label } => format!("The label '{label}' is used with a leave command, but no leave command is provided in all code paths"),
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
                },
                _ => {}
            }
        }

        Ok(())
    }

    fn search_label_jump(&mut self, target_label: &str) -> Result<(), ProgramError> {
        let potential_index = self.source_code.iter().position(|a|
            matches!(a, Command::Label(source_label) if *source_label == *target_label)
        );

        if let Some(index) = potential_index {
            self.program_pointer = index;
            return Ok(())
        } else {
            return Err(ProgramError::LabelNotFound(target_label.to_string()));
        }
    }

    /// Resulting in new return_value, if holding
    pub fn mutate(&mut self, command: &Command) -> Result<Option<Type>, ProgramError> {
        match command {
            Command::CallVoid(JumpDestination::Label(target_label)) | Command::CallRet(_, JumpDestination::Label(target_label)) | Command::Jmp(JumpDestination::Label(target_label)) => {
                self.search_label_jump(target_label)?;
            },
            Command::JumpLess(assignment, JumpDestination::Label(target_label)) => {
                if let Type::Integer(value) = self.memory.get(assignment)? {
                    if value == -1 {
                        self.search_label_jump(target_label)?
                    } else {
                        self.memory.stack_frame.pop();
                    }

                }
            },
            Command::JumpGreater(assignment, JumpDestination::Label(target_label)) => {
                if let Type::Integer(value) = self.memory.get(assignment)? {
                    if value == 1 {
                        self.search_label_jump(target_label)?
                    } else {
                        self.memory.stack_frame.pop();
                    }

                }
            }
            Command::JumpNotEqual(assignment, JumpDestination::Label(target_label)) => {
                if let Type::Integer(value) = self.memory.get(assignment)? {
                    if value != 0 {
                        self.search_label_jump(target_label)?
                    } else {
                        self.memory.stack_frame.pop();
                    }
                }
            }
            Command::JumpEqual(assignment, JumpDestination::Label(target_label)) => {
                if let Type::Integer(value) = self.memory.get(assignment)? {
                    if value == 0 {
                        self.search_label_jump(target_label)?
                    } else {
                        self.memory.stack_frame.pop();
                    }
                }
            }

            Command::Return(assignment) => {
                let value = self.memory.get(assignment)?;
                if self.memory.stack_frame.is_empty() {
                    return Ok(Some(value));
                } else if let Some(stack_frame) = self.memory.stack_frame.pop() {
                    if !stack_frame.entered_with_jmp {
                        (self.memory.rax, self.memory.rbx, self.memory.rcx) = stack_frame.register_state;
                    }

                    if let Some(destination) = stack_frame.destination {
                        self.memory.set(&destination, value)?;
                    }

                    self.program_pointer = stack_frame.return_address;
                }
            }
            Command::Leave => {
                if self.memory.stack_frame.is_empty() {
                    return Ok(Some(Type::Integer(0)))
                } else if let Some(stack_frame) = self.memory.stack_frame.pop() {
                    assert_eq!(stack_frame.destination, None);

                    if !stack_frame.entered_with_jmp {
                        (self.memory.rax, self.memory.rbx, self.memory.rcx) = stack_frame.register_state;
                    }

                    self.program_pointer = stack_frame.return_address;
                }
            },
            Command::Compare(_, _, _) |
            Command::LoadEffectiveAddress(_, _) | Command::Mov(_, _) |
            Command::Add(_, _, _)               | Command::Sub(_, _, _) |
            Command::Label(_)                   | Command::Syscall(_) => {}
        }

        Ok(None)
    }
}