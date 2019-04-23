#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    // Class(Class),
    Function(LoxFunction),
    // NativeFun(NativeFun),
    Number(f64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Nil => write!(f, "nil"),
            &Value::Boolean(b) => write!(f, "{}", b),
            &Value::Function(ref fun) => write!(f, "<fun {}>", fun),
            &Value::Number(n) => write!(f, "{}", n),
            &Value::String(ref s) => write!(f, "\"{}\"", s),
        }
    }
}

impl Value {
    pub fn print(&self) -> String {
        match self {
            &Value::Nil => format!("nil"),
            &Value::Boolean(b) => format!("{}", b),
            &Value::Function(ref fun) => format!("{}", fun),
            &Value::Number(n) => format!("{}", n),
            &Value::String(ref s) => format!("{}", s),
        }
    }
}
