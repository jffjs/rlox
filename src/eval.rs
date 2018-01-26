use std::error::Error;
use std::fmt;
use ast;
use token;

impl<'a> ast::Stmt<'a> {
    pub fn execute(self) -> Result<(), RuntimeError> {
        match self {
            ast::Stmt::Expr(expr_stmt) => {
                expr_stmt.expression.evaluate()?;
                Ok(())
            },
            ast::Stmt::Print(print_stmt) => {
                let expr_result = print_stmt.expression.evaluate()?;
                println!("{}", expr_result);
                Ok(())
            },
            ast::Stmt::Var(var_stmt) => {
                Err(runtime_error(var_stmt.name, "Not supported yet.").unwrap_err())
            }
        }
    }
}

#[derive(Debug)]
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

impl<'a> ast::Expr<'a> {
    pub fn evaluate(self) -> Result<EvalResult, RuntimeError> {
        match self {
            ast::Expr::Literal(lit_expr) => match lit_expr.value {
                token::Literal::Nil => Ok(EvalResult::Nil),
                token::Literal::True => Ok(EvalResult::Boolean(true)),
                token::Literal::False => Ok(EvalResult::Boolean(false)),
                token::Literal::Number(n) => Ok(EvalResult::Number(n)),
                token::Literal::String(s) => Ok(EvalResult::String(s.clone()))
            },
            ast::Expr::Grouping(group_expr) => group_expr.expression.evaluate(),
            ast::Expr::Unary(unary_expr) => {
                let right = unary_expr.right.evaluate()?;
                let operator = unary_expr.operator;
                match operator.token_type {
                    token::TokenType::Minus => match right {
                        EvalResult::Number(n) => Ok(EvalResult::Number(-n)),
                        _ => runtime_error(&operator, "Operand must be a number.")
                    },
                    token::TokenType::Bang => Ok(EvalResult::Boolean(!is_truthy(right))),
                    _ => panic!("Invalid unary expression. Check parser.")
                }
            },
            ast::Expr::Binary(bin_expr) => {
                let left = bin_expr.left.evaluate();
                let right = bin_expr.right.evaluate();
                let operator = bin_expr.operator;
                eval_binary_expr(&operator, left?, right?)
            },
            _ => panic!("I don't know how to evaluate this yet.")
        }
    }
}

fn is_truthy(val: EvalResult) -> bool {
    match val {
        EvalResult::Nil => false,
        EvalResult::Boolean(b) => b,
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
