use std::collections::HashMap;
use eval::EvalResult;

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<HashMap<String, EvalResult>>,
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

    pub fn assign(&mut self, name: String, val: EvalResult) -> Result<(), String> {
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

    pub fn define(&mut self, name: String, val: EvalResult) {
        self.scopes[self.current_scope].insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<&EvalResult> {
        let mut scope = self.current_scope;
        while scope != 0 {
            match self.get_in_scope(name, scope) {
                Some(val) => return Some(val),
                None => scope -= 1
            }
        }
        self.get_in_scope(name, scope)
    }

    fn get_in_scope(&self, name: &String, scope: usize) -> Option<&EvalResult> {
        self.scopes[scope].get(name)
    }
}

#[cfg(test)]
mod env_tests {
    use env::Environment;
    use eval::EvalResult;

    #[test]
    fn define_and_get() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));

        assert_eq!(EvalResult::Number(4.0), *env.get(&key).unwrap());
    }

    #[test]
    fn assign_success() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));
        let _result = env.assign(key.clone(), EvalResult::Boolean(true));

        assert_eq!(EvalResult::Boolean(true), *env.get(&key).unwrap());
    }

    #[test]
    fn assign_fail() {
        let mut env = Environment::new();
        let key = String::from("foo");
        let err = env.assign(key.clone(), EvalResult::Boolean(true)).unwrap_err();

        assert_eq!("Undefined variable 'foo'.", err);
    }

    #[test]
    fn get_from_parent_scope() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));
        env.push_scope();

        env.define(key.clone(), EvalResult::Number(5.0));
        env.push_scope();

        assert_eq!(EvalResult::Number(5.0), *env.get(&key).unwrap());
    }


    #[test]
    fn assign_to_parent() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));
        assert_eq!(EvalResult::Number(4.0), *env.get(&key).unwrap());

        env.push_scope();
        env.push_scope();
        let _result = env.assign(key.clone(), EvalResult::Number(5.0));

        assert_eq!(EvalResult::Number(5.0), *env.get(&key).unwrap());
    }

    #[test]
    fn shadow_var() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));
        assert_eq!(EvalResult::Number(4.0), *env.get(&key).unwrap());

        env.push_scope();
        env.define(key.clone(), EvalResult::Number(5.0));

        assert_eq!(EvalResult::Number(5.0), *env.get(&key).unwrap());

        env.pop_scope();
        assert_eq!(EvalResult::Number(4.0), *env.get(&key).unwrap());
    }
}
