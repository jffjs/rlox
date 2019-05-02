use crate::{
    class::LoxClass,
    function::{LoxFunction, NativeFunction},
};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Class(LoxClass),
    Function(LoxFunction),
    NativeFunction(NativeFunction),
    Number(f64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Class(ref class) => write!(f, "{}", class),
            Value::Function(ref fun) => write!(f, "{}", fun),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(ref s) => write!(f, "\"{}\"", s),
            Value::NativeFunction(ref fun) => write!(f, "{}", fun),
        }
    }
}

impl Value {
    pub fn print(&self) -> String {
        match self {
            Value::Nil => format!("nil"),
            Value::Boolean(b) => format!("{}", b),
            Value::Class(class) => format!("{}", class),
            Value::Function(fun) => format!("{}", fun),
            Value::Number(n) => format!("{}", n),
            Value::String(s) => format!("{}", s),
            Value::NativeFunction(fun) => format!("{}", fun),
        }
    }
}
