use std::fmt;
use token;

pub enum Expr<'a> {
    Binary(Binary<'a>),
    Grouping(Grouping<'a>),
    Literal(Literal),
    Unary(Unary<'a>)
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

pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a token::Token,
    pub right: Box<Expr<'a>>
}

impl<'a> Binary<'a> {
    pub fn new(left: Box<Expr<'a>>, operator: &'a token::Token, right: Box<Expr<'a>>) -> Binary<'a> {
        Binary { left, operator, right}
    }
}

pub struct Grouping<'a> {
    pub expression: Box<Expr<'a>>
}

impl<'a> Grouping<'a> {
    pub fn new(expression: Box<Expr<'a>>) -> Grouping<'a> {
        Grouping { expression }
    }
}

pub struct Literal {
    pub value: Box<token::Literal>
}

impl Literal {
    pub fn new(value: Box<token::Literal>) -> Literal {
        Literal { value }
    }
}

pub struct Unary<'a> {
    pub operator: &'a token::Token,
    pub right: Box<Expr<'a>>
}

impl<'a> Unary<'a> {
    pub fn new(operator: &'a token::Token, right: Box<Expr<'a>>) -> Unary<'a> {
        Unary { operator, right }
    }
}

