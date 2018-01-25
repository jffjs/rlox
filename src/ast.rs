use std::fmt;
use token;

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

