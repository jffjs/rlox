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
                    None => value = EvalResult::Nil
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

#[derive(Debug, PartialEq)]
pub enum EvalResult {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String)
}

impl fmt::Display for EvalResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EvalResult::Nil => write!(f, "nil"),
            &EvalResult::Boolean(b) => write!(f, "{}", b),
            &EvalResult::Number(n) => write!(f, "{}", n),
            &EvalResult::String(ref s) => write!(f, "\"{}\"", s)
        }
    }
}

impl EvalResult {
    fn clone(&self) -> EvalResult {
        match self {
            &EvalResult::Nil => EvalResult::Nil,
            &EvalResult::Boolean(b) => EvalResult::Boolean(b),
            &EvalResult::Number(n) => EvalResult::Number(n),
            &EvalResult::String(ref s) => EvalResult::String(s.clone())
        }
    }

    pub fn print(&self) -> String {
        match self {
            &EvalResult::Nil => format!("nil"),
            &EvalResult::Boolean(b) => format!("{}", b),
            &EvalResult::Number(n) => format!("{}", n),
            &EvalResult::String(ref s) => format!("{}", s)
        }
    }
}

impl<'a> ast::Expr<'a> {
    pub fn evaluate(&self, env: &mut Environment) -> Result<EvalResult, RuntimeError> {
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
            &ast::Expr::Grouping(ref group_expr) => group_expr.expression.evaluate(env),
            &ast::Expr::Literal(ref lit_expr) => match &lit_expr.value {
                &token::Literal::Nil => Ok(EvalResult::Nil),
                &token::Literal::True => Ok(EvalResult::Boolean(true)),
                &token::Literal::False => Ok(EvalResult::Boolean(false)),
                &token::Literal::Number(n) => Ok(EvalResult::Number(n)),
                &token::Literal::String(ref s) => Ok(EvalResult::String(s.clone()))
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
                    _ => panic!("Invalid logical epxression. Check parser.")
                }
            }
            &ast::Expr::Unary(ref unary_expr) => {
                let right = unary_expr.right.evaluate(env)?;
                let operator = unary_expr.operator;
                match operator.token_type {
                    token::TokenType::Minus => match right {
                        EvalResult::Number(n) => Ok(EvalResult::Number(-n)),
                        _ => runtime_error(&operator, "Operand must be a number.")
                    },
                    token::TokenType::Bang => Ok(EvalResult::Boolean(!is_truthy(&right))),
                    _ => panic!("Invalid unary expression. Check parser.")
                }
            },
            &ast::Expr::Variable(ref var_expr) => {
                let name = &var_expr.name;
                match env.get(&name.lexeme) {
                    Some(val) => Ok(val.clone()),
                    None => runtime_error(name, &format!("Undefined variable '{}'", name.lexeme))
                }
            },
            _ => panic!("I don't know how to evaluate this yet.")
        }
    }
}

fn is_truthy(val: &EvalResult) -> bool {
    match val {
        &EvalResult::Nil => false,
        &EvalResult::Boolean(b) => b,
        _ => true
    }
}

fn is_equal(a: EvalResult, b: EvalResult) -> bool {
    match a {
        EvalResult::Number(a_num) => match b {
            EvalResult::Number(b_num) => a_num == b_num,
            _ => false
        },
        EvalResult::String(a_str) => match b {
            EvalResult::String(b_str) => a_str.eq(&b_str),
            _ => false
        },
        EvalResult::Boolean(a_bool) => match b {
            EvalResult::Boolean(b_bool) => a_bool == b_bool,
            _ => false
        },
        EvalResult::Nil => match b {
            EvalResult::Nil => true,
            _ => false
        }
    }
}

fn eval_binary_expr<'a>(operator: &token::Token,
                        left: EvalResult,
                        right: EvalResult) -> Result<EvalResult, RuntimeError> {
    match operator.token_type {
        token::TokenType::EqualEqual => Ok(EvalResult::Boolean(is_equal(left, right))),
        token::TokenType::BangEqual => Ok(EvalResult::Boolean(!is_equal(left, right))),
        _ => match left {
            EvalResult::Number(l_num) => match right {
                EvalResult::Number(r_num) => match operator.token_type {
                    token::TokenType::Plus => Ok(EvalResult::Number(l_num + r_num)),
                    token::TokenType::Minus => Ok(EvalResult::Number(l_num - r_num)),
                    token::TokenType::Star => Ok(EvalResult::Number(l_num * r_num)),
                    token::TokenType::Slash => Ok(EvalResult::Number(l_num / r_num)),
                    token::TokenType::Greater => Ok(EvalResult::Boolean(l_num > r_num)),
                    token::TokenType::GreaterEqual => Ok(EvalResult::Boolean(l_num >= r_num)),
                    token::TokenType::Less => Ok(EvalResult::Boolean(l_num < r_num)),
                    token::TokenType::LessEqual => Ok(EvalResult::Boolean(l_num <= r_num)),
                    _ => panic!("Invalid binary expression. Check parser")
                },
                _ => runtime_error(operator, "Right operand must be a Number.")
            },
            EvalResult::String(l_str) => match right {
                EvalResult::String(r_str) => match operator.token_type {
                    token::TokenType::Plus => Ok(EvalResult::String(format!("{}{}", l_str, r_str))),
                    _ => panic!("Invalid binary expression. Check parser")
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

fn runtime_error(token: &token::Token, msg: &str) -> Result<EvalResult, RuntimeError> {
    Result::Err(RuntimeError::new(token.line, String::from(msg)))
}
