use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::command::Command;
use crate::interpreter::{Interpreter, SemanticError};
use crate::program_error::{ParseError, ProgramError};

#[derive(Debug, Clone)]
pub enum JumpDestination {
    /// Name of the label, index in the commands vector
    Label(String)
}

impl JumpDestination {
    /// Checks if the label has the provided command in all code_paths
    pub fn ends_with(&self, interpreter: &Interpreter, last_command: fn(&Command) -> bool, error: fn(&String) -> SemanticError) -> Result<(), ProgramError> {
        let JumpDestination::Label(target_label) = self;

        let possible_index = interpreter.source_code.iter().position(|a|
            matches!(a, Command::Label(source_label) if *source_label == *target_label)
        );

        if let Some(mut index) = possible_index {
            while let Some(inner_labels_command) = interpreter.source_code.get(index) {
                match inner_labels_command {
                    Command::Label(label) if *label != *target_label  => return Err(error(target_label).into()),
                    potential_last_command if last_command(potential_last_command) => return Ok(()),
                    _ => index += 1
                }
            }
        } else {
            return Err(ProgramError::LabelNotFound(target_label.to_string()));
        }

        Ok(())
    }
}

impl Display for JumpDestination {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            JumpDestination::Label(l) => l
        })
    }
}

impl FromStr for JumpDestination {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(JumpDestination::Label(s.to_string()))
    }
}