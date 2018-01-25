use std::fmt;
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

#[derive(Debug)]
pub enum Expr<'a> {
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr),
    Unary(UnaryExpr<'a>)
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expr::Binary(ref bin_expr) => {
                write!(f, "{}", parenthesize(&bin_expr.operator.lexeme, vec![&bin_expr.left, &bin_expr.right]))
            },
            &Expr::Grouping(ref group_expr) => {
                write!(f, "{}", parenthesize("group", vec![&group_expr.expression]))
            },
            &Expr::Literal(ref lit_expr) => {
                write!(f, "{}", &lit_expr.value.to_string())
            },
            &Expr::Unary(ref unary_expr) => {
                write!(f, "{}", parenthesize(&unary_expr.operator.lexeme, vec![&unary_expr.right]))
            },
        }
    }
}

impl<'a> Expr<'a> {
    pub fn binary(left: Expr<'a>, operator: &'a token::Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Binary(
            BinaryExpr::new(
                Box::new(left),
                operator,
                Box::new(right)
            )
        )
    }

    pub fn unary(operator: &'a token::Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Unary(UnaryExpr::new(operator, Box::new(right)))
    }

    pub fn literal(lit: token::Literal) -> Expr<'a> {
        Expr::Literal(LiteralExpr::new(lit))
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(GroupingExpr::new(Box::new(expr)))
    }

    pub fn evaluate(self) -> ExprResult {
        match self {
            Expr::Literal(lit_expr) => match lit_expr.value {
                token::Literal::Nil => ExprResult::Nil,
                token::Literal::True => ExprResult::Boolean(true),
                token::Literal::False => ExprResult::Boolean(false),
                token::Literal::Number(n) => ExprResult::Number(n),
                token::Literal::String(s) => ExprResult::String(s.clone())
            },
            Expr::Grouping(group_expr) => group_expr.expression.evaluate(),
            Expr::Unary(unary_expr) => {
                let right = unary_expr.right.evaluate();
                let operator = unary_expr.operator.token_type;
                match right {
                    ExprResult::Number(n) => {
                        match operator {
                            token::TokenType::Minus => ExprResult::Number(-n),
                            _ => panic!("Invalid unary expression found.")
                        }
                    },
                    _ => {
                        match operator {
                            token::TokenType::Bang => ExprResult::Boolean(!is_truthy(right)),
                            _ => panic!("Invalid unary expression found.")
                        }
                    }
                }
            },
            Expr::Binary(bin_expr) => {
                let left = bin_expr.left.evaluate();
                let right = bin_expr.right.evaluate();
                let operator = bin_expr.operator.token_type;
                eval_binary_expr(operator, left, right)
            }
        }
    }
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

fn eval_binary_expr(operator: token::TokenType, left: ExprResult, right: ExprResult) -> ExprResult {
    match operator {
        token::TokenType::EqualEqual => ExprResult::Boolean(is_equal(left, right)),
        token::TokenType::BangEqual => ExprResult::Boolean(!is_equal(left, right)),
        _ => match left {
            ExprResult::Number(l_num) => match right {
                ExprResult::Number(r_num) => match operator {
                    token::TokenType::Plus => ExprResult::Number(l_num + r_num),
                    token::TokenType::Minus => ExprResult::Number(l_num - r_num),
                    token::TokenType::Star => ExprResult::Number(l_num * r_num),
                    token::TokenType::Slash => ExprResult::Number(l_num / r_num),
                    token::TokenType::Greater => ExprResult::Boolean(l_num > r_num),
                    token::TokenType::GreaterEqual => ExprResult::Boolean(l_num >= r_num),
                    token::TokenType::Less => ExprResult::Boolean(l_num < r_num),
                    token::TokenType::LessEqual => ExprResult::Boolean(l_num <= r_num),
                    _ => panic!("Invalid operator.")
                },
                _ => panic!("Invalid right operand.")
            },
            ExprResult::String(l_str) => match right {
                ExprResult::String(r_str) => match operator {
                    token::TokenType::Plus => ExprResult::String(format!("{}{}", l_str, r_str)),
                    _ => panic!("Invalid operator.")
                },
                _ => panic!("Invalid right operand.")
            },
            _ => panic!("Invalid left operand.")
        }

    }
}


fn parenthesize(name: &str, exprs: Vec<&Box<Expr>>) -> String {
    let mut result = String::from("(");
    result.push_str(name);
    for expr in &exprs {
        result.push_str(" ");
        result.push_str(&expr.to_string());
    }
    result.push_str(")");
    result
}

#[derive(Debug)]
pub struct BinaryExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a token::Token,
    pub right: Box<Expr<'a>>
}

impl<'a> BinaryExpr<'a> {
    fn new(left: Box<Expr<'a>>, operator: &'a token::Token, right: Box<Expr<'a>>) -> BinaryExpr<'a> {
        BinaryExpr { left, operator, right}
    }
}

#[derive(Debug)]
pub struct GroupingExpr<'a> {
    pub expression: Box<Expr<'a>>
}

impl<'a> GroupingExpr<'a> {
    pub fn new(expression: Box<Expr<'a>>) -> GroupingExpr<'a> {
        GroupingExpr { expression }
    }
}

#[derive(Debug)]
pub struct LiteralExpr {
    pub value: token::Literal
}

impl LiteralExpr {
    fn new(value: token::Literal) -> LiteralExpr {
        LiteralExpr { value }
    }
}

#[derive(Debug)]
pub struct UnaryExpr<'a> {
    pub operator: &'a token::Token,
    pub right: Box<Expr<'a>>
}

impl<'a> UnaryExpr<'a> {
    fn new(operator: &'a token::Token, right: Box<Expr<'a>>) -> UnaryExpr<'a> {
        UnaryExpr { operator, right }
    }
}

