mod scanner;
mod token;

use std::error::Error;
use std::fmt;
use scanner::Scanner;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }

    pub fn run(&mut self, source: String) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(source);
        match scanner.scan_tokens() {
            Err(errors) => {
                Interpreter::report_errors(errors);
                Err(LoxError)
            },
            _ => Ok(())
        }
    }

    fn report_errors(errors: Vec<Box<Error>>) {
        for error in errors.iter() {
            println!("{}", error);
        }
    }
}

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

