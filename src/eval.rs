use std::error::Error;
use std::fmt;
use ast;
use env::Environment;
use token;

impl ast::Stmt {
    pub fn execute(&self, env: &mut Environment) -> Result<Option<Value>, RuntimeError> {
        match self {
            &ast::Stmt::Block(ref block_stmt) => {
                env.push_scope();
                for statement in &block_stmt.statements {
                    match statement.execute(env) {
                        Ok(None) => (),
                        Ok(Some(v)) => {
                            env.pop_scope();
                            return Ok(Some(v));
                        }
                        Err(err) => {
                            env.pop_scope();
                            return Err(err);
                        }
                    }
                }
                env.pop_scope();
                Ok(None)
            },
            &ast::Stmt::Expr(ref expr_stmt) => {
                expr_stmt.expression.evaluate(env)?;
                Ok(None)
            },
            &ast::Stmt::Fun(ref fun_stmt) => {
                let fun = LoxFunction::new(fun_stmt.clone());
                env.define(fun.declaration.name.lexeme.clone(), Value::Function(fun));
                Ok(None)
            }
            &ast::Stmt::If(ref if_stmt) => {
                let condition = if_stmt.condition.evaluate(env)?;
                if is_truthy(&condition) {
                    Ok(if_stmt.then_branch.execute(env)?)
                } else {
                    match if_stmt.else_branch {
                        Some(ref else_branch) => Ok(else_branch.execute(env)?),
                        None => Ok(None)
                    }
                }
            },
            &ast::Stmt::Print(ref print_stmt) => {
                let expr_result = print_stmt.expression.evaluate(env)?;
                println!("{}", expr_result.print());
                Ok(None)
            },
            &ast::Stmt::Return(ref ret_stmt) => {
                let value = match &ret_stmt.value {
                    &Some(ref expr) => expr.evaluate(env)?,
                    &None => Value::Nil
                };
                Ok(Some(value))
            },
            &ast::Stmt::Var(ref var_stmt) => {
                let value;
                match var_stmt.initializer {
                    Some(ref initializer) => value = initializer.evaluate(env)?,
                    None => value = Value::Nil
                }
                env.define(var_stmt.name.lexeme.clone(), value);
                Ok(None)
            },
            &ast::Stmt::While(ref while_stmt) => {
                let mut condition = while_stmt.condition.evaluate(env)?;
                while is_truthy(&condition) {
                    match while_stmt.body.execute(env) {
                        Ok(None) => (),
                        Ok(Some(v)) => return Ok(Some(v)),
                        Err(err) => return Err(err)
                    }
                    condition = while_stmt.condition.evaluate(env)?;
                }
                Ok(None)
            }
        }
    }
}

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
            &Value::String(ref s) => write!(f, "\"{}\"", s)
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
            &Value::String(ref s) => format!("{}", s)
        }
    }
}

trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, env: &mut Environment, args: Vec<Value>) -> Result<Value, RuntimeError>;
}

fn call<T: Callable>(paren: &token::Token, callee: T, env: &mut Environment, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if callee.arity() != args.len() {
        return runtime_error(paren, &format!("Expected {} arguments but got {}.", callee.arity(), args.len()))
    }
    callee.call(env, args)
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoxFunction {
    pub declaration: ast::FunStmt
}

impl LoxFunction {
    pub fn new(declaration: ast::FunStmt) -> LoxFunction {
        LoxFunction { declaration }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }

    fn call(&self, env: &mut Environment, args: Vec<Value>) -> Result<Value, RuntimeError> {
        env.push_scope();
        for (i, param) in self.declaration.parameters.iter().enumerate() {
            env.define(param.lexeme.clone(), args[i].clone());
        }

        let result = match self.declaration.body.execute(env) {
            Ok(Some(v)) => Ok(v),
            Ok(None) => Ok(Value::Nil),
            Err(err) => Err(err)
        };
        env.pop_scope();
        result
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fun {}>", self.declaration.name.lexeme)
    }
}

// #[derive(Debug, PartialEq)]
// pub struct Class {
// }

// impl Callable for Class {
//     fn call(&self, env: &mut Environment, args: Vec<Value>) -> Value {
//         Value::Nil
//     }
// }

impl ast::Expr {
    pub fn evaluate(&self, env: &mut Environment) -> Result<Value, RuntimeError> {
        match self {
            &ast::Expr::Assign(ref assign_expr) => {
                let name = &assign_expr.name;
                let value = assign_expr.value.evaluate(env)?;
                match env.assign(name.lexeme.clone(), value.clone()) {
                    Ok(_) => Ok(value),
                    Err(msg) => runtime_error(name, &msg)
                }
            },
            &ast::Expr::Binary(ref bin_expr) => {
                let left = bin_expr.left.evaluate(env);
                let right = bin_expr.right.evaluate(env);
                let operator = &bin_expr.operator;
                eval_binary_expr(operator, left?, right?)
            },
            &ast::Expr::Call(ref call_expr) => {
                let callee = call_expr.callee.evaluate(env)?;
                let mut arguments = vec![];
                for arg in &call_expr.arguments {
                    arguments.push(arg.evaluate(env)?);
                }

                match callee {
                    Value::Function(fun) => call(&call_expr.paren, fun, env, arguments),
                    // Value::Class(class) => call(class, env, arguments),
                    _ => runtime_error(&call_expr.paren, "Can only call functions and classes.")
                }
            },
            &ast::Expr::Grouping(ref group_expr) => group_expr.expression.evaluate(env),
            &ast::Expr::Literal(ref lit_expr) => match &lit_expr.value {
                &token::Literal::Nil => Ok(Value::Nil),
                &token::Literal::True => Ok(Value::Boolean(true)),
                &token::Literal::False => Ok(Value::Boolean(false)),
                &token::Literal::Number(n) => Ok(Value::Number(n)),
                &token::Literal::String(ref s) => Ok(Value::String(s.clone()))
            },
            &ast::Expr::Logical(ref logical_expr) => {
                let left = logical_expr.left.evaluate(env)?;
                match logical_expr.operator.token_type {
                    token::TokenType::Or => {
                        if is_truthy(&left) {
                            Ok(left)
                        } else {
                            logical_expr.right.evaluate(env)
                        }
                    },
                    token::TokenType::And => {
                        if !is_truthy(&left) {
                            Ok(left)
                        } else {
                            logical_expr.right.evaluate(env)
                        }
                    },
                    _ => panic!("Invalid logical epxression. This is an uncaught parse error.")
                }
            }
            &ast::Expr::Unary(ref unary_expr) => {
                let right = unary_expr.right.evaluate(env)?;
                let operator = &unary_expr.operator;
                match operator.token_type {
                    token::TokenType::Minus => match right {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => runtime_error(&operator, "Operand must be a number.")
                    },
                    token::TokenType::Bang => Ok(Value::Boolean(!is_truthy(&right))),
                    _ => panic!("Invalid unary expression. This is an uncaught parse error.")
                }
            },
            &ast::Expr::Variable(ref var_expr) => {
                let name = &var_expr.name;
                match env.get(&name.lexeme) {
                    Some(val) => Ok(val.clone()),
                    None => runtime_error(name, &format!("Undefined variable '{}'", name.lexeme))
                }
            },
            // _ => panic!("I don't know how to evaluate this yet.")
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        &Value::Nil => false,
        &Value::Boolean(b) => b,
        _ => true
    }
}

fn is_equal(a: Value, b: Value) -> bool {
    match a {
        Value::Boolean(a_bool) => match b {
            Value::Boolean(b_bool) => a_bool == b_bool,
            _ => false
        },
        Value::Function(a_fun) => match b {
            Value::Function(b_fun) => &a_fun == &b_fun,
            _ => false
        },
        Value::Nil => match b {
            Value::Nil => true,
            _ => false
        }
        Value::Number(a_num) => match b {
            Value::Number(b_num) => a_num == b_num,
            _ => false
        },
        Value::String(a_str) => match b {
            Value::String(b_str) => a_str.eq(&b_str),
            _ => false
        }
    }
}

fn eval_binary_expr<'a>(operator: &token::Token,
                        left: Value,
                        right: Value) -> Result<Value, RuntimeError> {
    match operator.token_type {
        token::TokenType::EqualEqual => Ok(Value::Boolean(is_equal(left, right))),
        token::TokenType::BangEqual => Ok(Value::Boolean(!is_equal(left, right))),
        _ => match left {
            Value::Number(l_num) => match right {
                Value::Number(r_num) => match operator.token_type {
                    token::TokenType::Plus => Ok(Value::Number(l_num + r_num)),
                    token::TokenType::Minus => Ok(Value::Number(l_num - r_num)),
                    token::TokenType::Star => Ok(Value::Number(l_num * r_num)),
                    token::TokenType::Slash => Ok(Value::Number(l_num / r_num)),
                    token::TokenType::Greater => Ok(Value::Boolean(l_num > r_num)),
                    token::TokenType::GreaterEqual => Ok(Value::Boolean(l_num >= r_num)),
                    token::TokenType::Less => Ok(Value::Boolean(l_num < r_num)),
                    token::TokenType::LessEqual => Ok(Value::Boolean(l_num <= r_num)),
                    _ => panic!("Invalid binary expression. This is an uncaught parse error")
                },
                _ => runtime_error(operator, "Right operand must be a Number.")
            },
            Value::String(l_str) => match right {
                Value::String(r_str) => match operator.token_type {
                    token::TokenType::Plus => Ok(Value::String(format!("{}{}", l_str, r_str))),
                    _ => panic!("Invalid binary expression. This is an uncaught parse error")
                },
                _ => runtime_error(operator, "Right operand must be a String.")
            },
            _ => runtime_error(operator, "Left operand must be a Number or a String.")
        }

    }
}

#[derive(Debug)]
pub struct RuntimeError {
    msg: String,
    line: u32

}

impl RuntimeError {
    fn new(line: u32, msg: String) -> RuntimeError {
        RuntimeError { msg, line }
    }
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.description())
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

fn runtime_error(token: &token::Token, msg: &str) -> Result<Value, RuntimeError> {
    Result::Err(RuntimeError::new(token.line, String::from(msg)))
}
