extern crate ast;
use std::error::Error;

mod callable;
mod environment;
mod function;
mod interpreter;
mod runtime_error;
mod value;

// Public interface
pub struct Interpreter {
    internal: interpreter::Interpreter,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            internal: interpreter::Interpreter::new(),
        }
    }

    pub fn run(&mut self, program: Vec<ast::Stmt>) -> Result<(), Vec<Box<Error>>> {
        self.internal.run(program)
    }
}
