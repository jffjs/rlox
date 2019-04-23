
use crate::value::Value;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

type Scope = HashMap<String, Value>;

#[derive(Clone, Debug)]
pub struct Environment {
    scopes: RefCell<Vec<Scope>>,
    current_scope: Cell<usize>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            scopes: RefCell::new(vec![HashMap::new()]),
            current_scope: Cell::new(0),
        }
    }

    pub fn push_scope(&self) {
        let scope = self.current_scope.get();
        self.current_scope.set(scope + 1);
        self.scopes.borrow_mut().push(HashMap::new());
    }

    pub fn pop_scope(&self) {
        let scope = self.current_scope.get();
        match self.scopes.borrow_mut().pop() {
            Some(_) => self.current_scope.set(scope - 1),
            None => (),
        }
    }

    pub fn assign(&self, name: String, val: Value) -> Result<(), String> {
        let mut current_scope = self.current_scope.get();

        while current_scope != 0 {
            let scope = &mut self.scopes.borrow_mut()[current_scope];
            if scope.contains_key(&name) {
                scope.insert(name, val);
                return Ok(());
            } else {
                current_scope -= 1;
            }
        }

        // scope is 0
        let scope = &mut self.scopes.borrow_mut()[current_scope];
        if scope.contains_key(&name) {
            scope.insert(name, val);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
    }

    pub fn define(&self, name: String, val: Value) {
        self.scopes.borrow_mut()[self.current_scope.get()].insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<Value> {
        let mut scope = self.current_scope.get();
        while scope != 0 {
            match self.get_in_scope(name, scope) {
                Some(val) => return Some(val),
                None => scope -= 1,
            }
        }
        self.get_in_scope(name, scope)
    }

    fn get_in_scope(&self, name: &String, scope: usize) -> Option<Value> {
        if let Some(value) = self.scopes.borrow()[scope].get(name) {
            return Some(value.clone());
        }
        None
    }
}

#[cfg(test)]
mod env_tests {
    use super::*;

    #[test]
    fn define_and_get() {
        let env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));

        assert_eq!(Value::Number(4.0), env.get(&key).unwrap());
    }

    #[test]
    fn assign_success() {
        let env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        let _result = env.assign(key.clone(), Value::Boolean(true));

        assert_eq!(Value::Boolean(true), env.get(&key).unwrap());
    }

    #[test]
    fn assign_fail() {
        let env = Environment::new();
        let key = String::from("foo");
        let err = env.assign(key.clone(), Value::Boolean(true)).unwrap_err();

        assert_eq!("Undefined variable 'foo'.", err);
    }

    #[test]
    fn get_from_parent_scope() {
        let env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        env.push_scope();

        env.define(key.clone(), Value::Number(5.0));
        env.push_scope();

        assert_eq!(Value::Number(5.0), env.get(&key).unwrap());
    }

    #[test]
    fn assign_to_parent() {
        let env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        assert_eq!(Value::Number(4.0), env.get(&key).unwrap());

        env.push_scope();
        env.push_scope();
        let _result = env.assign(key.clone(), Value::Number(5.0));

        assert_eq!(Value::Number(5.0), env.get(&key).unwrap());
    }

    #[test]
    fn shadow_var() {
        let env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        assert_eq!(Value::Number(4.0), env.get(&key).unwrap());

        env.push_scope();
        env.define(key.clone(), Value::Number(5.0));

        assert_eq!(Value::Number(5.0), env.get(&key).unwrap());

        env.pop_scope();
        assert_eq!(Value::Number(4.0), env.get(&key).unwrap());
    }
}
