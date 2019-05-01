use crate::{Expr, Stmt};

pub trait Visitor<T> {
  fn visit_stmt(&mut self, stmt: &Stmt) -> T;
  fn visit_expr(&mut self, expr: &Expr) -> T;
}
