use std::collections::HashMap;
use eval::EvalResult;

pub struct Environment {
    values: HashMap<String, EvalResult>
}

impl Environment {
    pub fn new() -> Environment {
        Environment { values: HashMap::new() }
    }

    pub fn define(&mut self, name: String, val: EvalResult) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &String) -> Option<&EvalResult> {
        self.values.get(name)
    }
}
