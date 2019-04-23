use crate::{
    callable::Callable,
    environment::Environment,
    interpreter::{Interpreter, InterpreterResult},
    value::Value,
};
use ast::visitor::Visitor;
use std::{fmt, rc::Rc};

#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub declaration: ast::FunStmt,
    pub closure: Rc<Environment>,
}

impl LoxFunction {
    pub fn new(declaration: ast::FunStmt, closure: Rc<Environment>) -> LoxFunction {
        LoxFunction {
            declaration,
            closure,
        }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }

    fn call(&self, int: &mut Interpreter, args: Vec<Value>) -> InterpreterResult {
        int.environment.push_scope();
        for (i, param) in self.declaration.parameters.iter().enumerate() {
            int.environment
                .define(param.lexeme.clone(), args[i].clone());
        }

        let result = int.visit_stmt(&self.declaration.body);
        int.environment.pop_scope();
        result
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &LoxFunction) -> bool {
        self.declaration == other.declaration
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fun {}>", self.declaration.name.lexeme)
    }
}
