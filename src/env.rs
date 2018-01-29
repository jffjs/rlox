use std::collections::HashMap;
use eval::EvalResult;

pub struct Environment<'a> {
    values: HashMap<String, EvalResult>,
    enclosing: Option<Box<&'a mut Environment<'a>>>
}

impl<'a> Environment<'a> {
    pub fn root() -> Environment<'a> {
        Environment { values: HashMap::new(), enclosing: None }
    }

    pub fn new_child(&'a mut self) -> Environment<'a> {
        Environment { values: HashMap::new(), enclosing: Some(Box::new(self)) }
    }

    pub fn assign(&mut self, name: String, val: EvalResult) -> Result<(), String> {
        match self.enclosing {
            Some(ref mut env) => env.assign(name, val),
            None => {
                if self.values.contains_key(&name) {
                    self.values.insert(name, val);
                    Ok(())
                } else {
                    Err(format!("Undefined variable '{}'.", name))
                }
            }
        }
    }

    pub fn define(&mut self, name: String, val: EvalResult) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<&EvalResult> {
        match self.enclosing {
            Some(ref env) => env.get(name),
            None => self.values.get(name)
        }
    }
}

#[cfg(test)]
mod env_tests {
    use env::Environment;
    use eval::EvalResult;

    #[test]
    fn define_and_get() {
        let mut env = Environment::root();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));

        assert_eq!(EvalResult::Number(4.0), *env.get(&key).unwrap());
    }

    #[test]
    fn assign_success() {
        let mut env = Environment::root();
        let key = String::from("foo");
        env.define(key.clone(), EvalResult::Number(4.0));
        let _result = env.assign(key.clone(), EvalResult::Boolean(true));

        assert_eq!(EvalResult::Boolean(true), *env.get(&key).unwrap());
    }

    #[test]
    fn assign_fail() {
        let mut env = Environment::root();
        let key = String::from("foo");
        let err = env.assign(key.clone(), EvalResult::Boolean(true)).unwrap_err();

        assert_eq!("Undefined variable 'foo'.", err);
    }

    #[test]
    fn get_from_parent() {
        let mut root = Environment::root();
        let key = String::from("foo");
        root.define(key.clone(), EvalResult::Number(4.0));
        let child = root.new_child();

        assert_eq!(EvalResult::Number(4.0), *child.get(&key).unwrap());
    }

    #[test]
    fn assign_to_parent() {
        let mut root = Environment::root();
        let key = String::from("foo");
        root.define(key.clone(), EvalResult::Number(4.0));
        assert_eq!(EvalResult::Number(4.0), *root.get(&key).unwrap());

        let mut child = root.new_child();
        let _result = child.assign(key.clone(), EvalResult::Number(5.0));
        assert_eq!(EvalResult::Number(5.0), *child.get(&key).unwrap());
    }
}
