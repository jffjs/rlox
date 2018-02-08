use std::fmt;
use token::{Literal, Token};

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Block(BlockStmt),
    Expr(ExprStmt),
    Fun(FunStmt),
    If(IfStmt),
    Print(PrintStmt),
    Var(VarStmt),
    While(WhileStmt)
}

impl Stmt {
    pub fn block(statements: Vec<Stmt>) -> Stmt {
        Stmt::Block(BlockStmt::new(statements))
    }

    pub fn expr(expression: Expr) -> Stmt {
        Stmt::Expr(ExprStmt::new(expression))
    }

    pub fn function(name: &Token, params: Vec<Token>, body: Stmt) -> Stmt {
        Stmt::Fun(FunStmt::new(name.clone(), params, body))
    }

    pub fn if_then(condition: Expr, then_branch: Stmt) -> Stmt {
        Stmt::If(IfStmt::new(condition, then_branch, None))
    }

    pub fn if_then_else(condition: Expr, then_branch: Stmt, else_branch: Stmt) -> Stmt {
        Stmt::If(IfStmt::new(condition, then_branch, Some(Box::new(else_branch))))
    }

    pub fn print(expression: Expr) -> Stmt {
        Stmt::Print(PrintStmt::new(expression))
    }

    pub fn var(name: &Token) -> Stmt {
        Stmt::Var(VarStmt::new(name.clone(), None))
    }

    pub fn var_init(name: &Token, initializer: Expr) -> Stmt {
        Stmt::Var(VarStmt::new(name.clone(), Some(initializer)))
    }

    pub fn while_loop(condition: Expr, body: Stmt) -> Stmt {
        Stmt::While(WhileStmt::new(condition, body))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>
}

impl BlockStmt {
    fn new(statements: Vec<Stmt>) -> BlockStmt {
        BlockStmt { statements }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprStmt {
    pub expression: Expr
}

impl ExprStmt {
    fn new(expression: Expr) -> ExprStmt {
        ExprStmt { expression }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunStmt {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Box<Stmt>
}

impl FunStmt {
    fn new(name: Token, parameters: Vec<Token>, body: Stmt) -> FunStmt {
        FunStmt { name, parameters, body: Box::new(body) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>
}

impl IfStmt {
    fn new(condition: Expr, then_branch: Stmt, else_branch: Option<Box<Stmt>>) -> IfStmt {
        IfStmt { condition, then_branch: Box::new(then_branch), else_branch }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintStmt {
    pub expression: Expr
}

impl PrintStmt {
    fn new(expression: Expr) -> PrintStmt {
        PrintStmt { expression }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Expr>
}

impl VarStmt {
    fn new(name: Token, initializer: Option<Expr>) -> VarStmt {
        VarStmt { name, initializer }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>
}

impl WhileStmt {
    fn new(condition: Expr, body: Stmt) -> WhileStmt {
        WhileStmt { condition, body: Box::new(body) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Assign(AssignExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Logical(LogicalExpr),
    Unary(UnaryExpr),
    Variable(VariableExpr)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expr::Assign(ref assign_expr) => {
                write!(f, "(= {} {})", assign_expr.name.lexeme, assign_expr.value)
            },
            &Expr::Binary(ref bin_expr) => {
                write!(f, "{}", parenthesize(&bin_expr.operator.lexeme, vec![&bin_expr.left, &bin_expr.right]))
            },
            &Expr::Call(ref call_expr) => {
                write!(f, "{}", parenthesize_call(&call_expr.callee.to_string(), &call_expr.arguments))
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

fn parenthesize(name: &str, exprs: Vec<&Expr>) -> String {
    let mut result = String::from("(");
    result.push_str(name);
    for expr in exprs {
        result.push_str(" ");
        result.push_str(&expr.to_string());
    }
    result.push_str(")");
    result
}

fn parenthesize_call(callee: &str, args: &Vec<Expr>) -> String {
    let mut result = String::from("(");
    result.push_str(callee);
    for expr in args {
        result.push_str(" ");
        result.push_str(&expr.to_string());
    }
    result.push_str(")");
    result
}


impl Expr {
    pub fn assign(name: &Token, value: Expr) -> Expr {
        Expr::Assign(AssignExpr::new(name.clone(), value))
    }

    pub fn binary(left: Expr, operator: &Token, right: Expr) -> Expr {
        Expr::Binary(BinaryExpr::new(left, operator.clone(), right))
    }

    pub fn call(callee: Expr, paren: &Token, args: Vec<Expr>) -> Expr {
        Expr::Call(CallExpr::new(callee, paren.clone(), args))
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(GroupingExpr::new(expr))
    }

    pub fn literal(lit: Literal) -> Expr {
        Expr::Literal(LiteralExpr::new(lit))
    }

    pub fn logical(left: Expr, operator: &Token, right: Expr) -> Expr {
        Expr::Logical(LogicalExpr::new(left, operator.clone(), right))
    }

    pub fn unary(operator: &Token, right: Expr) -> Expr {
        Expr::Unary(UnaryExpr::new(operator.clone(), right))
    }

    pub fn variable(name: &Token) -> Expr {
        Expr::Variable(VariableExpr::new(name.clone()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub name: Token,
    pub value: Box<Expr>
}

impl AssignExpr {
    pub fn new(name: Token, value: Expr) -> AssignExpr {
        AssignExpr { name, value: Box::new(value) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>
}

impl BinaryExpr {
    fn new(left: Expr, operator: Token, right: Expr) -> BinaryExpr {
        BinaryExpr { left: Box::new(left), operator, right: Box::new(right) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>
}

impl CallExpr {
    fn new(callee: Expr, paren: Token, arguments: Vec<Expr>) -> CallExpr {
        CallExpr { callee: Box::new(callee), paren, arguments }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupingExpr {
    pub expression: Box<Expr>
}

impl GroupingExpr {
    pub fn new(expression: Expr) -> GroupingExpr {
        GroupingExpr { expression: Box::new(expression) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiteralExpr {
    pub value: Literal
}

impl LiteralExpr {
    fn new(value: Literal) -> LiteralExpr {
        LiteralExpr { value }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicalExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>
}

impl LogicalExpr {
    fn new(left: Expr, operator: Token, right: Expr) -> LogicalExpr {
        LogicalExpr { left: Box::new(left), operator, right: Box::new(right) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>
}

impl UnaryExpr {
    fn new(operator: Token, right: Expr) -> UnaryExpr {
        UnaryExpr { operator, right: Box::new(right) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableExpr {
    pub name: Token
}

impl VariableExpr {
    fn new(name: Token) -> VariableExpr {
        VariableExpr { name }
    }
}
