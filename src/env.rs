use std::collections::HashMap;
use eval::EvalResult;

pub struct Environment {
    values: HashMap<String, EvalResult>
}

impl Environment {
    pub fn new() -> Environment {
        Environment { values: HashMap::new() }
    }

    pub fn assign(&mut self, name: String, val: EvalResult) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, val);
            Ok(())
        } else {
            Err(format!("Undefined variable '{}'.", name))
        }
    }

    pub fn define(&mut self, name: String, val: EvalResult) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<&EvalResult> {
        self.values.get(name)
    }
}
