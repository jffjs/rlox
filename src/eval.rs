use std::error::Error;
use std::fmt;
use ast;
use env::Environment;
use token;

impl<'a> ast::Stmt<'a> {
    pub fn execute(&self, env: &mut Environment) -> Result<(), RuntimeError> {
        match self {
            &ast::Stmt::Block(ref block_stmt) => {
                env.push_scope();
                for statement in &block_stmt.statements {
                    match statement.execute(env) {
                        Ok(_) => (),
                        Err(err) => return Err(err)
                    }
                }
                env.pop_scope();
                Ok(())
            },
            &ast::Stmt::Expr(ref expr_stmt) => {
                expr_stmt.expression.evaluate(env)?;
                Ok(())
            },
            &ast::Stmt::If(ref if_stmt) => {
                let condition = if_stmt.condition.evaluate(env)?;
                if is_truthy(&condition) {
                    if_stmt.then_branch.execute(env)?;
                } else {
                    match if_stmt.else_branch {
                        Some(ref else_branch) => else_branch.execute(env)?,
                        None => ()
                    }
                }
                Ok(())
            },
            &ast::Stmt::Print(ref print_stmt) => {
                let expr_result = print_stmt.expression.evaluate(env)?;
                println!("{}", expr_result.print());
                Ok(())
            },
            &ast::Stmt::Var(ref var_stmt) => {
                let value;
                match var_stmt.initializer {
                    Some(ref initializer) => value = initializer.evaluate(env)?,
                    None => value = Value::Nil
                }
                env.define(var_stmt.name.lexeme.clone(), value);
                Ok(())
            },
            &ast::Stmt::While(ref while_stmt) => {
                let mut condition = while_stmt.condition.evaluate(env)?;
                while is_truthy(&condition) {
                    while_stmt.body.execute(env)?;
                    condition = while_stmt.condition.evaluate(env)?;
                }
                Ok(())
            }
        }
    }
}

trait Callable {
    fn call(&self, env: &mut Environment, args: Vec<Value>) -> Result<Value, RuntimeError>;
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Function(Function),
    Number(f64),
    String(String),
}

impl Callable for Value {
    fn call(&self, env: &mut Environment, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match self {
            &Value::Function(ref fun) => Ok(fun.call(env, args)),
            _ => panic!("Value is not callable. This is an uncaught parse error.")
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Nil => write!(f, "nil"),
            &Value::Boolean(b) => write!(f, "{}", b),
            &Value::Function(ref fun) => write!(f, ""), // TODO: implement
            &Value::Number(n) => write!(f, "{}", n),
            &Value::String(ref s) => write!(f, "\"{}\"", s)
        }
    }
}

impl Value {
    fn clone(&self) -> Value {
        match self {
            &Value::Nil => Value::Nil,
            &Value::Boolean(b) => Value::Boolean(b),
            &Value::Function(ref fun) => Value::Function(Function {}), // TODO: implement
            &Value::Number(n) => Value::Number(n),
            &Value::String(ref s) => Value::String(s.clone())
        }
    }

    pub fn print(&self) -> String {
        match self {
            &Value::Nil => format!("nil"),
            &Value::Boolean(b) => format!("{}", b),
            &Value::Function(ref fun) => format!("fun"), // TODO: implement
            &Value::Number(n) => format!("{}", n),
            &Value::String(ref s) => format!("{}", s)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
}

impl Function {
    fn call(&self, env: &mut Environment, args: Vec<Value>) -> Value{
        Value::Nil
    }
}

impl<'a> ast::Expr<'a> {
    pub fn evaluate(&self, env: &mut Environment) -> Result<Value, RuntimeError> {
        match self {
            &ast::Expr::Assign(ref assign_expr) => {
                let name = assign_expr.name;
                let value = assign_expr.value.evaluate(env)?;
                match env.assign(name.lexeme.clone(), value.clone()) {
                    Ok(_) => Ok(value),
                    Err(msg) => runtime_error(name, &msg)
                }
            },
            &ast::Expr::Binary(ref bin_expr) => {
                let left = bin_expr.left.evaluate(env);
                let right = bin_expr.right.evaluate(env);
                let operator = bin_expr.operator;
                eval_binary_expr(&operator, left?, right?)
            },
            &ast::Expr::Call(ref call_expr) => {
                let callee = call_expr.callee.evaluate(env)?;
                let mut arguments = vec![];
                for arg in &call_expr.arguments {
                    arguments.push(arg.evaluate(env)?);
                }

                match callee {
                    Value::Function(_) => callee.call(env, arguments),
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
                let operator = unary_expr.operator;
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
        },
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
