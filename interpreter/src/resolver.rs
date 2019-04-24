use crate::error::ResolverError;
use ast::{visitor::Visitor, Expr, ScopeId, Stmt};
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
        for stmt in stmts {
            match self.resolve_stmt(stmt) {
                Ok(_) => (),
                Err(err) => errors.push(err),
            }
        }

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
        for i in (1..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(name) {
                self.locals.insert(scope_id, self.scopes.len() - 1 - i);
                return;
            }
        }
    }
}

impl Visitor<ResolverResult> for Resolver {
    fn visit_stmt(&mut self, stmt: &Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block(block_stmt) => {
                self.push_scope();
                self.resolve(&block_stmt.statements);
                self.pop_scope();
            }
            Stmt::Var(var_stmt) => {
                let name = &var_stmt.name.lexeme;
                self.declare(name.clone());
                if let Some(initializer) = &var_stmt.initializer {
                    self.resolve_expr(initializer);
                }
                self.define(name.clone());
            }
            _ => (),
        }
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> ResolverResult {
        match expr {
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
            _ => (),
        }
        Ok(())
    }
}
