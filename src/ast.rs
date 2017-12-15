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

impl<'a> Expr<'a> {
    pub fn binary(left: Expr<'a>, operator: &'a token::Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Binary(
            Binary::new(
                Box::new(left),
                operator,
                Box::new(right)
            )
        )
    }

    pub fn unary(operator: &'a token::Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Unary(Unary::new(operator, Box::new(right)))
    }

    pub fn literal(lit: token::Literal) -> Expr<'a> {
        Expr::Literal(Literal::new(lit))
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(Grouping::new(Box::new(expr)))
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
    fn new(left: Box<Expr<'a>>, operator: &'a token::Token, right: Box<Expr<'a>>) -> Binary<'a> {
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
    pub value: token::Literal
}

impl Literal {
    fn new(value: token::Literal) -> Literal {
        Literal { value }
    }
}

pub struct Unary<'a> {
    pub operator: &'a token::Token,
    pub right: Box<Expr<'a>>
}

impl<'a> Unary<'a> {
    fn new(operator: &'a token::Token, right: Box<Expr<'a>>) -> Unary<'a> {
        Unary { operator, right }
    }
}

