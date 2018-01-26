use std::fmt;
use token::{Literal, Token};

#[derive(Debug)]
pub enum Expr<'a> {
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr),
    Unary(UnaryExpr<'a>),
    Variable(VariableExpr<'a>)
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
            &Expr::Variable(ref var_expr) => {
                write!(f, "{}", &var_expr.name.lexeme)
            }
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


impl<'a> Expr<'a> {
    pub fn binary(left: Expr<'a>, operator: &'a Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Binary(BinaryExpr::new(left, operator, right))
    }

    pub fn unary(operator: &'a Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Unary(UnaryExpr::new(operator, right))
    }

    pub fn literal(lit: Literal) -> Expr<'a> {
        Expr::Literal(LiteralExpr::new(lit))
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(GroupingExpr::new(expr))
    }

    pub fn variable(name: &'a Token) -> Expr<'a> {
        Expr::Variable(VariableExpr::new(name))
    }
}

#[derive(Debug)]
pub struct BinaryExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a Token,
    pub right: Box<Expr<'a>>
}

impl<'a> BinaryExpr<'a> {
    fn new(left: Expr<'a>, operator: &'a Token, right: Expr<'a>) -> BinaryExpr<'a> {
        BinaryExpr { left: Box::new(left), operator, right: Box::new(right) }
    }
}

#[derive(Debug)]
pub struct GroupingExpr<'a> {
    pub expression: Box<Expr<'a>>
}

impl<'a> GroupingExpr<'a> {
    pub fn new(expression: Expr<'a>) -> GroupingExpr<'a> {
        GroupingExpr { expression: Box::new(expression) }
    }
}

#[derive(Debug)]
pub struct LiteralExpr {
    pub value: Literal
}

impl LiteralExpr {
    fn new(value: Literal) -> LiteralExpr {
        LiteralExpr { value }
    }
}

#[derive(Debug)]
pub struct UnaryExpr<'a> {
    pub operator: &'a Token,
    pub right: Box<Expr<'a>>
}

impl<'a> UnaryExpr<'a> {
    fn new(operator: &'a Token, right: Expr<'a>) -> UnaryExpr<'a> {
        UnaryExpr { operator, right: Box::new(right) }
    }
}

#[derive(Debug)]
pub struct VariableExpr<'a> {
    pub name: &'a Token
}

impl<'a> VariableExpr<'a> {
    fn new(name: &'a Token) -> VariableExpr<'a> {
        VariableExpr { name }
    }
}

#[derive(Debug)]
pub enum Stmt<'a> {
    Expr(ExprStmt<'a>),
    Print(PrintStmt<'a>),
    Var(VarStmt<'a>)
}

impl<'a> Stmt<'a> {
    pub fn expr(expression: Expr<'a>) -> Stmt<'a> {
        Stmt::Expr(ExprStmt::new(expression))
    }

    pub fn print(expression: Expr<'a>) -> Stmt<'a> {
        Stmt::Print(PrintStmt::new(expression))
    }

    pub fn var(name: &'a Token) -> Stmt<'a> {
        Stmt::Var(VarStmt::new(name, None))
    }

    pub fn var_init(name: &'a Token, initializer: Expr<'a>) -> Stmt<'a> {
        Stmt::Var(VarStmt::new(name, Some(Box::new(initializer))))
    }
}

#[derive(Debug)]
pub struct ExprStmt<'a> {
    pub expression: Box<Expr<'a>>
}

impl<'a> ExprStmt<'a> {
    fn new(expression: Expr<'a>) -> ExprStmt<'a> {
        ExprStmt { expression: Box::new(expression) }
    }
}

#[derive(Debug)]
pub struct PrintStmt<'a> {
    pub expression: Box<Expr<'a>>
}

impl<'a> PrintStmt<'a> {
    fn new(expression: Expr<'a>) -> PrintStmt<'a> {
        PrintStmt { expression: Box::new(expression) }
    }
}

#[derive(Debug)]
pub struct VarStmt<'a> {
    name: &'a Token,
    initializer: Option<Box<Expr<'a>>>
}

impl<'a> VarStmt<'a> {
    fn new(name: &'a Token, initializer: Option<Box<Expr<'a>>>) -> VarStmt<'a> {
        VarStmt { name, initializer }
    }
}
