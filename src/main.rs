mod command;
mod assignment;
mod address;
mod register;
mod jump;
mod interpreter;
mod memory;
mod program_error;

use std::str::FromStr;
use crate::assignment::Type;
use crate::interpreter::Interpreter;
use crate::program_error::ProgramError;


fn run() -> Result<isize, ProgramError> {
    let mut interpreter = Interpreter::from_str(include_str!("./array_init.asm"))?;
    interpreter.semantic_check()?;

    let mut exit_code: isize = 0;


    while let Some(command) = interpreter.source_code.get(interpreter.program_pointer) {
        let command = command.clone();
        command.execute(&mut interpreter.memory, interpreter.program_pointer)?;

        if let Some(holding_value) = interpreter.mutate(&command)? {
            exit_code = match holding_value {
                Type::String(_) => 1,
                Type::Integer(a) => a,
                Type::Address(_) => 1,
                Type::Untyped => 1,
            };
            break;
        }

        interpreter.program_pointer += 1;
    }

    println!("{}", interpreter);
    Ok(exit_code)
}


fn main() {
    match run() {
        Ok(exit_code) => println!("Process finished with: {exit_code}"),
        Err(err) => eprintln!("{err}")
    }
}
