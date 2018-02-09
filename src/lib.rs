mod scanner;
mod parser;
mod token;
mod ast;
mod eval;
mod env;

use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use env::Environment;
use parser::parse;
use scanner::Scanner;

pub struct Interpreter {
    env: Rc<RefCell<Environment>>
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { env: Rc::new(RefCell::new(Environment::new())) }
    }

    pub fn run(&mut self, source: String) -> Result<(), LoxError> {
        let scanner = Scanner::new(source);

        match scanner.scan_tokens() {
            Err(errors) => {
                Interpreter::report_errors(errors);
                Err(LoxError)
            },
            Ok(tokens) => {
                let mut errors: Vec<Box<Error>> = vec![];
                match parse(&tokens) {
                    Ok(statements) => {
                        for statement in statements {
                            match statement.execute(self.env.clone()) {
                                Ok(_) => (),
                                Err(error) => errors.push(Box::new(error))
                            }
                        }
                        if errors.len() > 0 {
                            Interpreter::report_errors(errors);
                            Err(LoxError)
                        } else {
                            Ok(())
                        }
                    },
                    Err(errors) => {
                        Interpreter::report_errors(errors);
                        Err(LoxError)
                    }
                }
            }
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

