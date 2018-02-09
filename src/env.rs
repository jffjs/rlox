use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use eval::Value;

type Scope = HashMap<String, Rc<RefCell<Value>>>;

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

    pub fn assign(&mut self, name: String, val: Value) -> Result<(), String> {
        let mut scope = self.current_scope;
        let value = Rc::new(RefCell::new(val));

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

    pub fn define(&mut self, name: String, val: Value) {
        let value = Rc::new(RefCell::new(val));
        self.scopes[self.current_scope].insert(name, value);
    }

    pub fn get(&self, name: &String) -> Option<Rc<RefCell<Value>>> {
        let mut scope = self.current_scope;
        while scope != 0 {
            match self.get_in_scope(name, scope) {
                Some(val) => return Some(val),
                None => scope -= 1
            }
        }
        self.get_in_scope(name, scope)
    }

    fn get_in_scope(&self, name: &String, scope: usize) -> Option<Rc<RefCell<Value>>> {
        match self.scopes[scope].get(name) {
            Some(v) => Some(v.clone()),
            None => None
        }
    }
}

#[cfg(test)]
mod env_tests {
    use std::cell::RefCell;
    use env::Environment;
    use eval::Value;

    #[test]
    fn define_and_get() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));

        let expect = RefCell::new(Value::Number(4.0));
        let actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());
    }

    #[test]
    fn assign_success() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        let _result = env.assign(key.clone(), Value::Boolean(true));

        let expect = RefCell::new(Value::Boolean(true));
        let actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());
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

        let expect = RefCell::new(Value::Number(5.0));
        let actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());
    }


    #[test]
    fn assign_to_parent() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));

        let mut expect = RefCell::new(Value::Number(4.0));
        let mut actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());

        env.push_scope();
        env.push_scope();
        let _result = env.assign(key.clone(), Value::Number(5.0));

        expect = RefCell::new(Value::Number(5.0));
        actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());
    }

    #[test]
    fn shadow_var() {
        let mut env = Environment::new();
        let key = String::from("foo");
        env.define(key.clone(), Value::Number(4.0));
        let mut expect = RefCell::new(Value::Number(4.0));
        let mut actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());

        env.push_scope();
        env.define(key.clone(), Value::Number(5.0));

        expect = RefCell::new(Value::Number(5.0));
        actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());

        env.pop_scope();
        expect = RefCell::new(Value::Number(4.0));
        actual = env.get(&key).unwrap();
        assert_eq!(*expect.borrow(), *actual.borrow());
    }
}
