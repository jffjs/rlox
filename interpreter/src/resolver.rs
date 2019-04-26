use crate::error::ResolverError;
use ast::{visitor::Visitor, Expr, FunStmt, ScopeId, Stmt};
use std::collections::HashMap;

type Scope = HashMap<String, bool>;
type ResolverResult = Result<(), ResolverError>;

pub struct Resolver {
    scopes: Vec<Scope>,
    pub locals: HashMap<ScopeId, usize>,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: vec![],
            locals: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, stmts: &Vec<Stmt>) -> Result<(), Vec<ResolverError>> {
        let mut errors = vec![];
        self.push_scope();
        for stmt in stmts {
            match self.resolve_stmt(stmt) {
                Ok(_) => (),
                Err(err) => errors.push(err),
            }
        }
        self.pop_scope();

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolverResult {
        self.visit_stmt(stmt)
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult {
        self.visit_expr(expr)
    }

    fn resolve_function(&mut self, function: &FunStmt) -> ResolverResult {
        self.push_scope();
        for param in &function.parameters {
            let name = &param.lexeme;
            self.declare(name.clone());
            self.define(name.clone());
        }
        for statement in &function.body {
            self.resolve_stmt(&statement)?;
        }
        self.pop_scope();
        Ok(())
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, false);
        }
    }

    fn define(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, true);
        }
    }

    fn resolve_local(&mut self, scope_id: ScopeId, name: &String) {
        let mut i = (self.scopes.len() - 1) as isize;
        while i >= 0 {
            let index = i as usize;
            if self.scopes[index].contains_key(name) {
                self.locals.insert(scope_id, self.scopes.len() - 1 - index);
                return;
            }
            i -= 1;
        }
    }
}

impl Visitor<ResolverResult> for Resolver {
    fn visit_stmt(&mut self, stmt: &Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block(block_stmt) => {
                self.push_scope();
                for statement in &block_stmt.statements {
                    self.resolve_stmt(&statement)?;
                }
                self.pop_scope();
            }
            Stmt::Expr(expr_stmt) => self.resolve_expr(&expr_stmt.expression)?,
            Stmt::Fun(fun_stmt) => {
                let name = &fun_stmt.name.lexeme;
                self.declare(name.clone());
                self.define(name.clone());
                self.resolve_function(fun_stmt)?;
            }
            Stmt::If(if_stmt) => {
                self.resolve_expr(&if_stmt.condition)?;
                self.resolve_stmt(&if_stmt.then_branch)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.resolve_stmt(else_branch)?;
                }
            }
            Stmt::Print(print_stmt) => self.resolve_expr(&print_stmt.expression)?,
            Stmt::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    self.resolve_expr(value)?;
                }
            }
            Stmt::While(while_stmt) => {
                self.resolve_expr(&while_stmt.condition)?;
                self.resolve_stmt(&while_stmt.body)?;
            }
            Stmt::Var(var_stmt) => {
                let name = &var_stmt.name.lexeme;
                self.declare(name.clone());
                if let Some(initializer) = &var_stmt.initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name.clone());
            }
        }
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> ResolverResult {
        match expr {
            Expr::Assign(assign_expr) => {
                self.resolve_expr(&assign_expr.value)?;
                self.resolve_local(assign_expr.scope_id, &assign_expr.name.lexeme);
            }
            Expr::Binary(binary_expr) => {
                self.resolve_expr(&binary_expr.left)?;
                self.resolve_expr(&binary_expr.right)?;
            }
            Expr::Call(call_expr) => {
                self.resolve_expr(&call_expr.callee)?;
                for arg in &call_expr.arguments {
                    self.resolve_expr(arg)?;
                }
            }
            Expr::Grouping(grouping_expr) => {
                self.resolve_expr(&grouping_expr.expression)?;
            }
            Expr::Literal(_) => (),
            Expr::Logical(logical_expr) => {
                self.resolve_expr(&logical_expr.left)?;
                self.resolve_expr(&logical_expr.right)?;
            }
            Expr::Unary(unary_expr) => {
                self.resolve_expr(&unary_expr.right)?;
            }
            Expr::Variable(var_expr) => {
                let name = &var_expr.name.lexeme;
                if let Some(scope) = self.scopes.last() {
                    if scope.get(name) == Some(&false) {
                        return Err(ResolverError::new(
                            var_expr.name.line,
                            "Cannot read local variable in its own intializer.".to_string(),
                        ));
                    }
                }
                self.resolve_local(var_expr.scope_id, name);
            }
        }
        Ok(())
    }
}
