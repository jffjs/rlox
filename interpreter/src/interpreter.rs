use crate::{
    callable::call,
    environment::Environment,
    error::{runtime_error_result, RuntimeError},
    function::LoxFunction,
    native::define_native_functions,
    resolver::Resolver,
    value::Value,
};
use ast::{
    token::{Literal, Token, TokenType},
    visitor::Visitor,
    Expr, ScopeId, Stmt,
};
use std::{error::Error, rc::Rc};

pub type InterpreterResult = Result<Option<Value>, RuntimeError>;

pub struct Interpreter {
    pub environment: Option<Rc<Environment>>,
    resolver: Resolver,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = Environment::new(None);
        define_native_functions(&mut globals);
        Interpreter {
            environment: Some(Rc::new(globals)),
            resolver: Resolver::new(),
        }
    }

    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), Vec<Box<Error>>> {
        self.resolver.resolve(&stmts).map_err(|errs| {
            let mut boxed: Vec<Box<Error>> = vec![];
            for e in errs {
                boxed.push(Box::new(e));
            }
            boxed
        })?;

        let mut errors: Vec<Box<Error>> = vec![];
        for stmt in stmts.iter() {
            match self.visit_stmt(stmt) {
                Ok(_) => (),
                Err(err) => errors.push(Box::new(err)),
            }
        }

        if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn look_up_var(&mut self, name: &String, scope_id: &ScopeId) -> Option<Value> {
        if let Some(environment) = self.environment.take() {
            let value;
            if let Some(distance) = self.resolver.locals.get(scope_id) {
                // println!("{}", distance);
                value = environment.get_at(name, *distance)
            } else {
                value = environment.get(name)
            }
            self.environment = Some(environment);
            value
        } else {
            None
        }
    }

    fn assign_var(&mut self, name: String, value: Value, scope_id: &ScopeId) -> Result<(), String> {
        let environment = self.environment.take().unwrap();
        let result;
        if let Some(distance) = self.resolver.locals.get(scope_id) {
            result = environment.assign_at(name, value, *distance);
        } else {
            result = environment.assign(name, value);
        }
        self.environment = Some(environment);
        result
    }

    pub fn define_var(&mut self, name: String, value: Value) {
        let environment = self.environment.take().unwrap();
        environment.define(name, value);
        self.environment = Some(environment);
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<Environment>,
    ) -> InterpreterResult {
        let previous = self.environment.take();
        self.push_scope(environment);
        let mut result: InterpreterResult = Ok(None);
        for statement in statements {
            match self.visit_stmt(statement) {
                Ok(None) => (),
                Ok(Some(v)) => {
                    result = Ok(Some(v));
                    break;
                }
                Err(err) => {
                    result = Err(err);
                    break;
                }
            }
        }
        self.pop_scope(previous);
        result
    }

    fn push_scope(&mut self, environment: Rc<Environment>) {
        self.environment = Some(environment);
    }

    fn pop_scope(&mut self, environment: Option<Rc<Environment>>) {
        self.environment = environment;
    }
}

impl Visitor<InterpreterResult> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> InterpreterResult {
        match stmt {
            Stmt::Block(block_stmt) => {
                let environment = Environment::new(self.environment.clone());
                self.execute_block(&block_stmt.statements, Rc::new(environment))
            }
            Stmt::Expr(expr_stmt) => {
                self.visit_expr(&expr_stmt.expression)?;
                Ok(None)
            }
            Stmt::Fun(fun_stmt) => {
                let fun = LoxFunction::new(fun_stmt.clone(), self.environment.clone());
                self.define_var(fun_stmt.name.lexeme.clone(), Value::Function(fun));
                Ok(None)
            }
            Stmt::If(if_stmt) => {
                if let Some(condition) = self.visit_expr(&if_stmt.condition)? {
                    if is_truthy(&condition) {
                        Ok(self.visit_stmt(&if_stmt.then_branch)?)
                    } else {
                        if let Some(ref else_branch) = if_stmt.else_branch {
                            Ok(self.visit_stmt(else_branch)?)
                        } else {
                            Ok(None)
                        }
                    }
                } else {
                    panic!("Expression should always return a value");
                }
            }
            Stmt::Print(print_stmt) => {
                let expr_result = (self.visit_expr(&print_stmt.expression)?).unwrap();
                println!("{}", expr_result.print());
                Ok(None)
            }
            Stmt::Return(ret_stmt) => {
                let value = if let Some(ref expr) = ret_stmt.value {
                    self.visit_expr(expr)?
                } else {
                    Some(Value::Nil)
                };
                Ok(value)
            }
            Stmt::Var(var_stmt) => {
                let value = if let Some(ref initializer) = var_stmt.initializer {
                    (self.visit_expr(initializer)?).unwrap()
                } else {
                    Value::Nil
                };
                self.define_var(var_stmt.name.lexeme.clone(), value);
                Ok(None)
            }
            Stmt::While(while_stmt) => {
                let mut condition = (self.visit_expr(&while_stmt.condition)?).unwrap();
                while is_truthy(&condition) {
                    match self.visit_stmt(&while_stmt.body)? {
                        Some(v) => return Ok(Some(v)),
                        None => (),
                    }
                    condition = (self.visit_expr(&while_stmt.condition)?).unwrap();
                }
                Ok(None)
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> InterpreterResult {
        match expr {
            Expr::Assign(assign_expr) => {
                let name = &assign_expr.name;
                let value = self.visit_expr(&assign_expr.value)?;
                match self.assign_var(
                    name.lexeme.clone(),
                    value.clone().unwrap(),
                    &assign_expr.scope_id,
                ) {
                    Ok(_) => Ok(value),
                    Err(msg) => runtime_error_result(name, &msg),
                }
            }
            Expr::Binary(bin_expr) => {
                let left = self.visit_expr(&bin_expr.left)?;
                let right = self.visit_expr(&bin_expr.right)?;
                let operator = &bin_expr.operator;
                eval_binary_expr(operator, left.unwrap(), right.unwrap())
            }
            Expr::Call(call_expr) => {
                let callee = self.visit_expr(&call_expr.callee)?;
                let mut arguments = vec![];
                for arg in &call_expr.arguments {
                    arguments.push((self.visit_expr(arg)?).unwrap());
                }
                match callee.unwrap() {
                    Value::Function(fun) => call(&call_expr.paren, &fun, self, arguments),
                    Value::NativeFunction(fun) => call(&call_expr.paren, &fun, self, arguments),
                    _ => runtime_error_result(
                        &call_expr.paren,
                        "Can only call functions and classes.",
                    ),
                }
            }
            Expr::Grouping(group_expr) => self.visit_expr(&group_expr.expression),
            Expr::Literal(lit_expr) => match &lit_expr.value {
                Literal::Nil => Ok(Some(Value::Nil)),
                Literal::True => Ok(Some(Value::Boolean(true))),
                Literal::False => Ok(Some(Value::Boolean(false))),
                Literal::Number(n) => Ok(Some(Value::Number(*n))),
                Literal::String(s) => Ok(Some(Value::String(s.clone()))),
            },
            Expr::Logical(logical_expr) => {
                let left = (self.visit_expr(&logical_expr.left)?).unwrap();
                match &logical_expr.operator.token_type {
                    TokenType::Or => {
                        if is_truthy(&left) {
                            Ok(Some(left))
                        } else {
                            self.visit_expr(&logical_expr.right)
                        }
                    }
                    TokenType::And => {
                        if !is_truthy(&left) {
                            Ok(Some(left))
                        } else {
                            self.visit_expr(&logical_expr.right)
                        }
                    }
                    _ => panic!("Invalid logical expression. This is an uncaught parse error."),
                }
            }
            Expr::Unary(unary_expr) => {
                let right = (self.visit_expr(&unary_expr.right)?).unwrap();
                let operator = &unary_expr.operator;
                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Number(n) => Ok(Some(Value::Number(-n))),
                        _ => runtime_error_result(operator, "Operand must be a number."),
                    },
                    TokenType::Bang => Ok(Some(Value::Boolean(!is_truthy(&right)))),
                    _ => panic!("Invalid unary expression. This is an uncaught parse error."),
                }
            }
            Expr::Variable(var_expr) => {
                let name = &var_expr.name;
                if let Some(val) = self.look_up_var(&name.lexeme, &var_expr.scope_id) {
                    Ok(Some(val))
                } else {
                    runtime_error_result(name, &format!("Undefined variable '{}'", name.lexeme))
                }
            }
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        &Value::Nil => false,
        &Value::Boolean(b) => b,
        _ => true,
    }
}

fn is_equal(a: Value, b: Value) -> bool {
    match a {
        Value::Boolean(a_bool) => match b {
            Value::Boolean(b_bool) => a_bool == b_bool,
            _ => false,
        },
        Value::Function(a_fun) => match b {
            Value::Function(b_fun) => &a_fun == &b_fun,
            _ => false,
        },
        Value::Nil => match b {
            Value::Nil => true,
            _ => false,
        },
        Value::Number(a_num) => match b {
            Value::Number(b_num) => a_num == b_num,
            _ => false,
        },
        Value::String(a_str) => match b {
            Value::String(b_str) => a_str.eq(&b_str),
            _ => false,
        },
        Value::NativeFunction(a_fun) => match b {
            Value::NativeFunction(b_fun) => &a_fun == &b_fun,
            _ => false,
        },
    }
}

fn eval_binary_expr<'a>(operator: &Token, left: Value, right: Value) -> InterpreterResult {
    match operator.token_type {
        TokenType::EqualEqual => Ok(Some(Value::Boolean(is_equal(left, right)))),
        TokenType::BangEqual => Ok(Some(Value::Boolean(!is_equal(left, right)))),
        _ => match left {
            Value::Number(l_num) => match right {
                Value::Number(r_num) => match operator.token_type {
                    TokenType::Plus => Ok(Some(Value::Number(l_num + r_num))),
                    TokenType::Minus => Ok(Some(Value::Number(l_num - r_num))),
                    TokenType::Star => Ok(Some(Value::Number(l_num * r_num))),
                    TokenType::Slash => Ok(Some(Value::Number(l_num / r_num))),
                    TokenType::Greater => Ok(Some(Value::Boolean(l_num > r_num))),
                    TokenType::GreaterEqual => Ok(Some(Value::Boolean(l_num >= r_num))),
                    TokenType::Less => Ok(Some(Value::Boolean(l_num < r_num))),
                    TokenType::LessEqual => Ok(Some(Value::Boolean(l_num <= r_num))),
                    _ => panic!("Invalid binary expression. This is an uncaught parse error"),
                },
                _ => runtime_error_result(operator, "Right operand must be a Number."),
            },
            Value::String(l_str) => match right {
                Value::String(r_str) => match operator.token_type {
                    TokenType::Plus => Ok(Some(Value::String(format!("{}{}", l_str, r_str)))),
                    _ => panic!("Invalid binary expression. This is an uncaught parse error"),
                },
                _ => runtime_error_result(operator, "Right operand must be a String."),
            },
            _ => runtime_error_result(operator, "Left operand must be a Number or a String."),
        },
    }
}
