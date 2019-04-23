#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub declaration: ast::FunStmt,
    pub closure: Rc<Environment>
}

impl LoxFunction {
    pub fn new(declaration: ast::FunStmt, closure: Rc<Environment>) -> LoxFunction {
        LoxFunction { declaration, closure }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }

    fn call(&self, env: Rc<Environment>, args: Vec<Value>) -> Result<Value, RuntimeError> {
        env.push_scope();
        for (i, param) in self.declaration.parameters.iter().enumerate() {
            env.define(param.lexeme.clone(), args[i].clone());
        }

        let result = match self.declaration.body.execute(env.clone()) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(Value::Nil),
            Err(err) => Err(err),
        };
        env.pop_scope();
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
