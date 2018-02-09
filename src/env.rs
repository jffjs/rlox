use std::collections::HashMap;
use eval::RefValue;

type Scope = HashMap<String, RefValue>;

#[derive(Clone, Debug)]
pub struct Environment {
    scopes: Vec<Scope>,
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

    pub fn assign(&mut self, name: String, value: RefValue) -> Result<(), String> {
        let mut scope = self.current_scope;
        // let value = Rc::new(value_ref(val));

        while scope != 0 {
            if self.scopes[scope].contains_key(&name) {
                self.scopes[scope].insert(name, value);
                return Ok(());
            } else {
                scope -= 1;
            }
        }

        // scope is 0
        if self.scopes[scope].contains_key(&name) {
            self.scopes[scope].insert(name, value);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
    }

    pub fn define(&mut self, name: String, value: RefValue) {
        // let value = Rc::new(value_ref(val));
        self.scopes[self.current_scope].insert(name, value);
    }

    pub fn get(&self, name: &String) -> Option<RefValue> {
        let mut scope = self.current_scope;
        while scope != 0 {
            match self.get_in_scope(name, scope) {
                Some(val) => return Some(val),
                None => scope -= 1
            }
        }
        self.get_in_scope(name, scope)
    }

    fn get_in_scope(&self, name: &String, scope: usize) -> Option<RefValue> {
        match self.scopes[scope].get(name) {
            Some(v) => Some(v.clone()),
            None => None
        }
    }
}

#[cfg(test)]
mod env_tests {
    use env::Environment;
    use eval::{Value, value_ref};

    #[test]
    fn define_and_get() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), value_ref(Value::Number(4.0)));

        let expect = value_ref(Value::Number(4.0));
        let actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);
    }

    #[test]
    fn assign_success() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), value_ref(Value::Number(4.0)));
        let _result = env.assign(key.clone(), value_ref(Value::Boolean(true)));

        let expect = value_ref(Value::Boolean(true));
        let actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);
    }

    #[test]
    fn assign_fail() {
        let mut env = Environment::new();
        let key = String::from("foo");
        let err = env.assign(key.clone(), value_ref(Value::Boolean(true))).unwrap_err();

        assert_eq!("Undefined variable 'foo'.", err);
    }

    #[test]
    fn get_from_parent_scope() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), value_ref(Value::Number(4.0)));
        env.push_scope();

        env.define(key.clone(), value_ref(Value::Number(5.0)));
        env.push_scope();

        let expect = value_ref(Value::Number(5.0));
        let actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);
    }


    #[test]
    fn assign_to_parent() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), value_ref(Value::Number(4.0)));

        let mut expect = value_ref(Value::Number(4.0));
        let mut actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);

        env.push_scope();
        env.push_scope();
        let _result = env.assign(key.clone(), value_ref(Value::Number(5.0)));

        expect = value_ref(Value::Number(5.0));
        actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);
    }

    #[test]
    fn shadow_var() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), value_ref(Value::Number(4.0)));
        let mut expect = value_ref(Value::Number(4.0));
        let mut actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);

        env.push_scope();
        env.define(key.clone(), value_ref(Value::Number(5.0)));

        expect = value_ref(Value::Number(5.0));
        actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);

        env.pop_scope();
        expect = value_ref(Value::Number(4.0));
        actual = env.get(&key).unwrap();
        assert_eq!(expect, actual);
    }
}
