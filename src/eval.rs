use std::error::Error;
use std::fmt;
use ast;
use token;

#[derive(Debug)]
pub enum ExprResult {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String)
}

impl fmt::Display for ExprResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ExprResult::Nil => write!(f, "nil"),
            &ExprResult::Boolean(b) => write!(f, "{}", b),
            &ExprResult::Number(n) => write!(f, "{}", n),
            &ExprResult::String(ref s) => write!(f, "\"{}\"", s)
        }
    }
}

impl<'a> ast::Expr<'a> {
    pub fn evaluate(self) -> Result<ExprResult, RuntimeError> {
        match self {
            ast::Expr::Literal(lit_expr) => match lit_expr.value {
                token::Literal::Nil => Ok(ExprResult::Nil),
                token::Literal::True => Ok(ExprResult::Boolean(true)),
                token::Literal::False => Ok(ExprResult::Boolean(false)),
                token::Literal::Number(n) => Ok(ExprResult::Number(n)),
                token::Literal::String(s) => Ok(ExprResult::String(s.clone()))
            },
            ast::Expr::Grouping(group_expr) => group_expr.expression.evaluate(),
            ast::Expr::Unary(unary_expr) => {
                let right = unary_expr.right.evaluate()?;
                let operator = unary_expr.operator.token_type;
                match right {
                    ExprResult::Number(n) => {
                        match operator {
                            token::TokenType::Minus => Ok(ExprResult::Number(-n)),
                            _ => panic!("Invalid unary expression found.")
                        }
                    },
                    _ => {
                        match operator {
                            token::TokenType::Bang => Ok(ExprResult::Boolean(!is_truthy(right))),
                            _ => runtime_error(&unary_expr.operator, "Operand must be a number.")
                        }
                    }
                }
            },
            ast::Expr::Binary(bin_expr) => {
                let left = bin_expr.left.evaluate();
                let right = bin_expr.right.evaluate();
                let operator = bin_expr.operator.token_type;
                eval_binary_expr(operator, left?, right?)
            }
        }
    }
}

fn runtime_error(token: &token::Token, msg: &str) -> Result<ExprResult, RuntimeError> {
    Result::Err(RuntimeError::new(token.line, String::from(msg)))
}

fn is_truthy(val: ExprResult) -> bool {
    match val {
        ExprResult::Nil => false,
        ExprResult::Boolean(b) => b,
        _ => true
    }
}

fn is_equal(a: ExprResult, b: ExprResult) -> bool {
    match a {
        ExprResult::Number(a_num) => match b {
            ExprResult::Number(b_num) => a_num == b_num,
            _ => false
        },
        ExprResult::String(a_str) => match b {
            ExprResult::String(b_str) => a_str.eq(&b_str),
            _ => false
        },
        ExprResult::Boolean(a_bool) => match b {
            ExprResult::Boolean(b_bool) => a_bool == b_bool,
            _ => false
        },
        ExprResult::Nil => match b {
            ExprResult::Nil => true,
            _ => false
        }
    }
}

fn eval_binary_expr<'a>(operator: token::TokenType,
                        left: ExprResult,
                        right: ExprResult) -> Result<ExprResult, RuntimeError> {
    match operator {
        token::TokenType::EqualEqual => Ok(ExprResult::Boolean(is_equal(left, right))),
        token::TokenType::BangEqual => Ok(ExprResult::Boolean(!is_equal(left, right))),
        _ => match left {
            ExprResult::Number(l_num) => match right {
                ExprResult::Number(r_num) => match operator {
                    token::TokenType::Plus => Ok(ExprResult::Number(l_num + r_num)),
                    token::TokenType::Minus => Ok(ExprResult::Number(l_num - r_num)),
                    token::TokenType::Star => Ok(ExprResult::Number(l_num * r_num)),
                    token::TokenType::Slash => Ok(ExprResult::Number(l_num / r_num)),
                    token::TokenType::Greater => Ok(ExprResult::Boolean(l_num > r_num)),
                    token::TokenType::GreaterEqual => Ok(ExprResult::Boolean(l_num >= r_num)),
                    token::TokenType::Less => Ok(ExprResult::Boolean(l_num < r_num)),
                    token::TokenType::LessEqual => Ok(ExprResult::Boolean(l_num <= r_num)),
                    _ => panic!("Invalid operator.")
                },
                _ => panic!("Invalid right operand.")
            },
            ExprResult::String(l_str) => match right {
                ExprResult::String(r_str) => match operator {
                    token::TokenType::Plus => Ok(ExprResult::String(format!("{}{}", l_str, r_str))),
                    _ => panic!("Invalid operator.")
                },
                _ => panic!("Invalid right operand.")
            },
            _ => panic!("Invalid left operand.")
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
