use crate::{
    callable::Callable,
    environment::v2::Environment,
    interpreter::{Interpreter, InterpreterResult},
    value::Value,
};
use ast::visitor::Visitor;
use snowflake::ProcessUniqueId;
use std::{
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub declaration: ast::FunStmt,
    pub id: ProcessUniqueId,
    pub closure: Option<Rc<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: ast::FunStmt, closure: Option<Rc<Environment>>) -> LoxFunction {
        let fun = LoxFunction {
            declaration,
            id: ProcessUniqueId::new(),
            closure,
        };
        fun
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }

    fn call(&self, int: &mut Interpreter, args: Vec<Value>) -> InterpreterResult {
        let environment = Environment::new(self.closure.clone());
        for (i, param) in self.declaration.parameters.iter().enumerate() {
            environment.define(param.lexeme.clone(), args[i].clone());
        }

        let result = int.execute_block(&self.declaration.body, Rc::new(environment));
        match result {
            Ok(None) => Ok(Some(Value::Nil)),
            _ => result,
        }
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &LoxFunction) -> bool {
        self.id == other.id
    }
}

impl Eq for LoxFunction {}

impl Hash for LoxFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fun {}>", self.declaration.name.lexeme)
    }
}

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub fun: Rc<dyn Fn(Vec<Value>) -> Value>,
}

impl NativeFunction {
    pub fn new(name: String, arity: usize, fun: Rc<dyn Fn(Vec<Value>) -> Value>) -> NativeFunction {
        NativeFunction { name, arity, fun }
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, _int: &mut Interpreter, args: Vec<Value>) -> InterpreterResult {
        let value = (self.fun)(args);
        Ok(Some(value))
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NativeFunction {{ name: {}, arity: {} }}",
            self.name, self.arity
        )
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fun {}>", self.name)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &NativeFunction) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}
