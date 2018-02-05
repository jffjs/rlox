use std::fmt;
use token::{Literal, Token};

#[derive(Debug)]
pub enum Stmt<'a> {
    Block(BlockStmt<'a>),
    Expr(ExprStmt<'a>),
    If(IfStmt<'a>),
    Print(PrintStmt<'a>),
    Var(VarStmt<'a>),
    While(WhileStmt<'a>)
}

impl<'a> Stmt<'a> {
    pub fn block(statements: Vec<Stmt<'a>>) -> Stmt<'a> {
        Stmt::Block(BlockStmt::new(statements))
    }

    pub fn expr(expression: Expr<'a>) -> Stmt<'a> {
        Stmt::Expr(ExprStmt::new(expression))
    }

    pub fn if_then(condition: Expr<'a>, then_branch: Stmt<'a>) -> Stmt<'a> {
        Stmt::If(IfStmt::new(condition, then_branch, None))
    }

    pub fn if_then_else(condition: Expr<'a>, then_branch: Stmt<'a>, else_branch: Stmt<'a>) -> Stmt<'a> {
        Stmt::If(IfStmt::new(condition, then_branch, Some(Box::new(else_branch))))
    }

    pub fn print(expression: Expr<'a>) -> Stmt<'a> {
        Stmt::Print(PrintStmt::new(expression))
    }

    pub fn var(name: &'a Token) -> Stmt<'a> {
        Stmt::Var(VarStmt::new(name, None))
    }

    pub fn var_init(name: &'a Token, initializer: Expr<'a>) -> Stmt<'a> {
        Stmt::Var(VarStmt::new(name, Some(initializer)))
    }

    pub fn while_loop(condition: Expr<'a>, body: Stmt<'a>) -> Stmt<'a> {
        Stmt::While(WhileStmt::new(condition, body))
    }
}

#[derive(Debug)]
pub struct BlockStmt<'a> {
    pub statements: Vec<Stmt<'a>>
}

impl<'a> BlockStmt<'a> {
    fn new(statements: Vec<Stmt<'a>>) -> BlockStmt<'a> {
        BlockStmt { statements }
    }
}

#[derive(Debug)]
pub struct ExprStmt<'a> {
    pub expression: Expr<'a>
}

impl<'a> ExprStmt<'a> {
    fn new(expression: Expr<'a>) -> ExprStmt<'a> {
        ExprStmt { expression }
    }
}

#[derive(Debug)]
pub struct IfStmt<'a> {
    pub condition: Expr<'a>,
    pub then_branch: Box<Stmt<'a>>,
    pub else_branch: Option<Box<Stmt<'a>>>
}

impl<'a> IfStmt<'a> {
    fn new(condition: Expr<'a>, then_branch: Stmt<'a>, else_branch: Option<Box<Stmt<'a>>>) -> IfStmt<'a> {
        IfStmt { condition, then_branch: Box::new(then_branch), else_branch }
    }
}

#[derive(Debug)]
pub struct PrintStmt<'a> {
    pub expression: Expr<'a>
}

impl<'a> PrintStmt<'a> {
    fn new(expression: Expr<'a>) -> PrintStmt<'a> {
        PrintStmt { expression }
    }
}

#[derive(Debug)]
pub struct VarStmt<'a> {
    pub name: &'a Token,
    pub initializer: Option<Expr<'a>>
}

impl<'a> VarStmt<'a> {
    fn new(name: &'a Token, initializer: Option<Expr<'a>>) -> VarStmt<'a> {
        VarStmt { name, initializer }
    }
}

#[derive(Debug)]
pub struct WhileStmt<'a> {
    pub condition: Expr<'a>,
    pub body: Box<Stmt<'a>>
}

impl<'a> WhileStmt<'a> {
    fn new(condition: Expr<'a>, body: Stmt<'a>) -> WhileStmt<'a> {
        WhileStmt { condition, body: Box::new(body) }
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    Assign(AssignExpr<'a>),
    Binary(BinaryExpr<'a>),
    Grouping(GroupingExpr<'a>),
    Literal(LiteralExpr),
    Logical(LogicalExpr<'a>),
    Unary(UnaryExpr<'a>),
    Variable(VariableExpr<'a>)
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expr::Assign(ref assign_expr) => {
                write!(f, "(= {} {})", assign_expr.name.lexeme, assign_expr.value)
            },
            &Expr::Binary(ref bin_expr) => {
                write!(f, "{}", parenthesize(&bin_expr.operator.lexeme, vec![&bin_expr.left, &bin_expr.right]))
            },
            &Expr::Grouping(ref group_expr) => {
                write!(f, "{}", parenthesize("group", vec![&group_expr.expression]))
            },
            &Expr::Literal(ref lit_expr) => {
                write!(f, "{}", &lit_expr.value.to_string())
            },
            &Expr::Logical(ref log_expr) => {
                write!(f, "{}", parenthesize(&log_expr.operator.lexeme, vec![&log_expr.left, &log_expr.right]))
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
    pub fn assign(name: &'a Token, value: Expr<'a>) -> Expr<'a> {
        Expr::Assign(AssignExpr::new(name, value))
    }

    pub fn binary(left: Expr<'a>, operator: &'a Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Binary(BinaryExpr::new(left, operator, right))
    }

    pub fn unary(operator: &'a Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Unary(UnaryExpr::new(operator, right))
    }

    pub fn literal(lit: Literal) -> Expr<'a> {
        Expr::Literal(LiteralExpr::new(lit))
    }

    pub fn logical(left: Expr<'a>, operator: &'a Token, right: Expr<'a>) -> Expr<'a> {
        Expr::Logical(LogicalExpr::new(left, operator, right))
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(GroupingExpr::new(expr))
    }

    pub fn variable(name: &'a Token) -> Expr<'a> {
        Expr::Variable(VariableExpr::new(name))
    }
}

#[derive(Debug)]
pub struct AssignExpr<'a> {
    pub name: &'a Token,
    pub value: Box<Expr<'a>>
}

impl<'a> AssignExpr<'a> {
    pub fn new(name: &'a Token, value: Expr<'a>) -> AssignExpr<'a> {
        AssignExpr { name, value: Box::new(value) }
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
pub struct LogicalExpr<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: &'a Token,
    pub right: Box<Expr<'a>>
}

impl<'a> LogicalExpr<'a> {
    fn new(left: Expr<'a>, operator: &'a Token, right: Expr<'a>) -> LogicalExpr<'a> {
        LogicalExpr { left: Box::new(left), operator, right: Box::new(right) }
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
