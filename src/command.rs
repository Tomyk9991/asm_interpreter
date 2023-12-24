use std::str::FromStr;
use crate::assignment::{Assignment, OperationError, Type};
use crate::destination::Destination;
use crate::interpreter::{StackFrame};
use crate::jump::JumpDestination;
use crate::memory::{Memory, MemoryError};
use crate::program_error::ParseError;
use crate::register::Register;

#[derive(Debug, Clone)]
pub enum Command {
    Mov(Destination, Assignment),
    Add(Destination, Assignment, Assignment),
    Sub(Destination, Assignment, Assignment),
    /// call will build a stack frame
    CallRet(Destination, JumpDestination),
    CallVoid(JumpDestination),
    /// jmp will just jump without storing and restoring rax, rbx, rcx
    Jmp(JumpDestination),
    Label(String),
    Return(Assignment),
    Syscall(JumpDestination),
    Leave
}

impl Command {
    /// Returns an optional, if some, containing a return value
    pub fn execute(&self, memory: &mut Memory, program_pointer: usize) -> Result<(), MemoryError> {
        match self {
            Command::Mov(destination, assigment) => {
                memory.set(destination, self.get_value(memory, assigment)?)?;
            }
            Command::Add(destination, operand1, operand2) => {
                let result = self.get_value(memory, operand1)? + self.get_value(memory, operand2)?;
                memory.set(destination, result)?;
            },
            Command::Sub(destination, operand1, operand2) => {
                let result = self.get_value(memory, operand1)?.sub(&self.get_value(memory, operand2)?)?;
                memory.set(destination, result)?;
            }
            Command::CallRet(destination, JumpDestination::Label(_)) => {
                let stack_frame = StackFrame {
                    return_address: program_pointer,
                    entered_with_jmp: false,
                    destination: Some(destination.clone()),
                    register_state: memory.register_state(),
                };

                memory.stack_frame.push_back(stack_frame);
            }
            Command::CallVoid(JumpDestination::Label(_)) => {
                let stack_frame = StackFrame {
                    return_address: program_pointer,
                    entered_with_jmp: false,
                    destination: None,
                    register_state: memory.register_state(),
                };

                memory.stack_frame.push_back(stack_frame);
            },
            Command::Jmp(JumpDestination::Label(_)) => {
                let stack_frame = StackFrame {
                    return_address: program_pointer,
                    entered_with_jmp: true,
                    destination: None,
                    register_state: memory.register_state(),
                };

                memory.stack_frame.push_back(stack_frame);
            }
            Command::Syscall(JumpDestination::Label(label)) => {
                if *label == "printf" {
                    match &memory.rax {
                        Type::String(format) => {
                            let final_str = if format.contains("{}") {
                                format.replace("{}", &memory.rbx.to_string_raw())
                            } else {
                                format.to_string()
                            };

                            println!("{final_str}");
                        }
                        rest => return Err(OperationError::WrongType { expected: "String".to_string(), actual: format!("{rest}") }.into())
                    }
                }
            }
            Command::Label(_) | Command::Return(_) | Command::Leave => {}
        }

        Ok(())
    }

    pub fn get_value(&self, memory: &Memory, assignment: &Assignment) -> Result<Type, MemoryError> {
        match assignment {
            Assignment::Value(value) => Ok(value.clone()),
            Assignment::Register(Destination::Register(register)) => {
                match register {
                    Register::Rax => Ok(memory.rax.clone()),
                    Register::Rbx => Ok(memory.rbx.clone()),
                    Register::Rcx => Ok(memory.rcx.clone())
                }
            }
            Assignment::Register(Destination::StackPointer(index)) => {
                if *index >= 64 {
                    return Err(MemoryError::Read(Assignment::Register(Destination::StackPointer(*index))));
                }

                Ok(memory.stack[*index].clone())
            }
        }
    }
}

impl FromStr for Command {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = merge_quotes(s);

        return if let [instruction] = &split[..] {
            match *instruction {
                "leave" => Ok(Command::Leave),
                label_name if label_name.ends_with(':') => Ok(Command::Label(label_name[0..label_name.chars().count() - 1].to_string())),
                a => Err(ParseError::new(&format!("Unknown instruction: {a}")))
            }
        } else if let [instruction, operand] = &split[..] {
            match *instruction {
                "syscall" => Ok(Command::Syscall(JumpDestination::from_str(operand)?)),
                "jmp" => Ok(Command::Jmp(JumpDestination::from_str(operand)?)),
                "ret" => Ok(Command::Return(Assignment::from_str(operand)?)),
                "call" => Ok(Command::CallVoid(JumpDestination::from_str(operand)?)),
                a => Err(ParseError::new(&format!("Unknown instruction: {a}")))
            }
        }
        else if let [instruction, destination, assignment] = &split[..] {
            match *instruction {
                "mov" => Ok(Command::Mov(Destination::from_str(destination)?, Assignment::from_str(assignment)?)),
                "call" => Ok(Command::CallRet(Destination::from_str(destination)?, JumpDestination::from_str(assignment)?)),
                a => Err(ParseError::new(&format!("Unknown instruction: {a}")))
            }
        } else if let [instruction, destination, operand1, operand2] = &split[..] {
            match *instruction {
                "add" => Ok(Command::Add(Destination::from_str(destination)?, Assignment::from_str(operand1)?, Assignment::from_str(operand2)?)),
                "sub" => Ok(Command::Sub(Destination::from_str(destination)?, Assignment::from_str(operand1)?, Assignment::from_str(operand2)?)),
                a => Err(ParseError::new(&format!("Unknown instruction: {a}")))
            }
        } else {
            Err(ParseError::new(&format!("Unknown size of instructions '{}'", s)))
        }
    }
}

fn merge_quotes(target: &str) -> Vec<&str> {
    let mut result = vec![];
    let mut word_range = 0..0;
    let mut open_bracket = false;

    for char in target.chars() {
        match char {
            ' ' if !open_bracket => {
                let word = &target[word_range.clone()];
                if !word.is_empty() {
                    result.push(word);
                }
                word_range.start = word_range.end + 1;
                word_range.end += 1;
            },
            '"' if !open_bracket => {
                open_bracket = true;
                word_range.end += 1;
            },
            '"' if open_bracket => {
                open_bracket = false;
                word_range.end += 1;
            }
            _ => { word_range.end += 1; }
        }
    }

    result.push(&target[word_range.clone()]);
    return result;
}