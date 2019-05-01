use crate::value::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type Scope = HashMap<String, Value>;

#[derive(Debug)]
pub struct Environment {
    pub enclosing: Option<Rc<Environment>>,
    values: RefCell<Scope>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<Environment>>) -> Environment {
        Environment {
            enclosing,
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn define(&self, name: String, value: Value) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn assign(&self, name: String, value: Value) -> Result<(), String> {
        if self.values.borrow().contains_key(&name) {
            self.values.borrow_mut().insert(name, value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
    }

    pub fn assign_at(&self, name: String, value: Value, distance: usize) -> Result<(), String> {
        if distance == 0 {
            self.assign(name, value)
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.assign_at(name, value, distance - 1)
            } else {
                Err(format!("Undefined variable '{}'", name))
            }
        }
    }

    pub fn get(&self, name: &String) -> Option<Value> {
        if let Some(value) = self.values.borrow().get(name) {
            Some(value.clone())
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.get(name)
            } else {
                None
            }
        }
    }

    pub fn get_at(&self, name: &String, distance: usize) -> Option<Value> {
        if distance == 0 {
            self.get(name)
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.get_at(name, distance - 1)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn define_and_get() {
        let env = Environment::new(None);
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));

        assert_eq!(Value::Number(4.0), env.get(&key).unwrap());
    }

    #[test]
    fn assign_success() {
        let env = Environment::new(None);
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        let _result = env.assign(key.clone(), Value::Boolean(true));

        assert_eq!(Value::Boolean(true), env.get(&key).unwrap());
    }

    #[test]
    fn assign_fail() {
        let env = Environment::new(None);
        let key = String::from("foo");
        let err = env.assign(key.clone(), Value::Boolean(true)).unwrap_err();

        assert_eq!("Undefined variable 'foo'.", err);
    }

    #[test]
    fn get_from_parent_scope() {
        let key = String::from("foo");
        let parent = Environment::new(None);
        parent.define(key.clone(), Value::Number(4.0));

        let env = Environment::new(Some(Rc::new(parent)));

        assert_eq!(env.get(&key), Some(Value::Number(4.0)));
        env.define(key.clone(), Value::Number(5.0));
        assert_eq!(env.get(&key), Some(Value::Number(5.0)));
    }

    #[test]
    fn assign_to_parent() {
        let key = String::from("foo");
        let parent = Environment::new(None);
        parent.define(key.clone(), Value::Number(4.0));

        let env = Environment::new(Some(Rc::new(parent)));

        assert_eq!(env.get(&key), Some(Value::Number(4.0)));
        let _result = env.assign(key.clone(), Value::Number(5.0));
        assert_eq!(env.get(&key), Some(Value::Number(5.0)));
    }

    #[test]
    fn shadow_var() {
        let key = String::from("foo");
        let parent = Environment::new(None);
        parent.define(key.clone(), Value::Number(4.0));

        let mut env = Environment::new(Some(Rc::new(parent)));

        assert_eq!(env.get(&key), Some(Value::Number(4.0)));
        env.define(key.clone(), Value::Number(5.0));
        assert_eq!(env.get(&key), Some(Value::Number(5.0)));

        let env = env.enclosing.take().unwrap();
        assert_eq!(env.get(&key), Some(Value::Number(4.0)));
    }
}
