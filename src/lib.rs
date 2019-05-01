use interpreter::Interpreter;
use parser::parse;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct LoxError;

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for LoxError {
    fn description(&self) -> &str {
        "Error."
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub struct Repl {
    interpreter: Interpreter,
}

impl Repl {
    pub fn new() -> Repl {
        Repl {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, source: String) -> Result<(), LoxError> {
        match parse(source) {
            Ok(program) => match self.interpreter.run(program) {
                Ok(_) => Ok(()),
                Err(errors) => {
                    report_errors(errors);
                    Err(LoxError)
                }
            },
            Err(errors) => {
                report_errors(errors);
                Err(LoxError)
            }
        }
    }
}

fn report_errors(errors: Vec<Box<Error>>) {
    for error in errors {
        println!("{}", error);
    }
}
