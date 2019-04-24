use crate::{
    interpreter::{Interpreter, InterpreterResult},
    runtime_error::runtime_error_result,
    value::Value,
};
use ast::token::Token;

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, int: &mut Interpreter, args: Vec<Value>) -> InterpreterResult;
}

pub fn call<T: Callable>(
    paren: &Token,
    callee: &T,
    int: &mut Interpreter,
    args: Vec<Value>,
) -> InterpreterResult {
    if callee.arity() != args.len() {
        return runtime_error_result(
            paren,
            &format!(
                "Expected {} arguments but got {}.",
                callee.arity(),
                args.len()
            ),
        );
    }
    callee.call(int, args)
}
