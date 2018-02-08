use std::collections::HashMap;
use eval::Value;

#[derive(Clone, Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
    current_scope: usize
}

impl Environment {
    pub fn new() -> Environment {
        Environment { scopes: vec![HashMap::new()], current_scope: 0 }
    }

    pub fn push_scope(&mut self) {
        self.current_scope += 1;
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        match self.scopes.pop() {
            Some(_) => self.current_scope -= 1,
            None => ()
        }
    }

    pub fn assign(&mut self, name: String, val: Value) -> Result<(), String> {
        let mut scope = self.current_scope;

        while scope != 0 {
            if self.scopes[scope].contains_key(&name) {
                self.scopes[scope].insert(name, val);
                return Ok(());
            } else {
                scope -= 1;
            }
        }

        // scope is 0
        if self.scopes[scope].contains_key(&name) {
            self.scopes[scope].insert(name, val);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
    }

    pub fn define(&mut self, name: String, val: Value) {
        self.scopes[self.current_scope].insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<&Value> {
        let mut scope = self.current_scope;
        while scope != 0 {
            match self.get_in_scope(name, scope) {
                Some(val) => return Some(val),
                None => scope -= 1
            }
        }
        self.get_in_scope(name, scope)
    }

    fn get_in_scope(&self, name: &String, scope: usize) -> Option<&Value> {
        self.scopes[scope].get(name)
    }
}

#[cfg(test)]
mod env_tests {
    use env::Environment;
    use eval::Value;

    #[test]
    fn define_and_get() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));

        assert_eq!(Value::Number(4.0), *env.get(&key).unwrap());
    }

    #[test]
    fn assign_success() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        let _result = env.assign(key.clone(), Value::Boolean(true));

        assert_eq!(Value::Boolean(true), *env.get(&key).unwrap());
    }

    #[test]
    fn assign_fail() {
        let mut env = Environment::new();
        let key = String::from("foo");
        let err = env.assign(key.clone(), Value::Boolean(true)).unwrap_err();

        assert_eq!("Undefined variable 'foo'.", err);
    }

    #[test]
    fn get_from_parent_scope() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        env.push_scope();

        env.define(key.clone(), Value::Number(5.0));
        env.push_scope();

        assert_eq!(Value::Number(5.0), *env.get(&key).unwrap());
    }


    #[test]
    fn assign_to_parent() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        assert_eq!(Value::Number(4.0), *env.get(&key).unwrap());

        env.push_scope();
        env.push_scope();
        let _result = env.assign(key.clone(), Value::Number(5.0));

        assert_eq!(Value::Number(5.0), *env.get(&key).unwrap());
    }

    #[test]
    fn shadow_var() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        assert_eq!(Value::Number(4.0), *env.get(&key).unwrap());

        env.push_scope();
        env.define(key.clone(), Value::Number(5.0));

        assert_eq!(Value::Number(5.0), *env.get(&key).unwrap());

        env.pop_scope();
        assert_eq!(Value::Number(4.0), *env.get(&key).unwrap());
    }
}
