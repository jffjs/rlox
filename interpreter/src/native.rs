use crate::{environment::v2::Environment, function::NativeFunction, value::Value};
use std::{
  rc::Rc,
  time::{SystemTime, UNIX_EPOCH},
};

pub fn define_native_functions(environment: &mut Environment) {
  let clock_fun = NativeFunction::new("clock".to_string(), 0, Rc::new(clock));
  environment.define(clock_fun.name.clone(), Value::NativeFunction(clock_fun));
}

fn clock(_args: Vec<Value>) -> Value {
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
  Value::Number(since_the_epoch.as_millis() as f64)
}
