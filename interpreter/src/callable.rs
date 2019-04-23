pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, env: Rc<Environment>, args: Vec<Value>) -> Result<Value, RuntimeError>;
}

pub fn call<T: Callable>(
    paren: &token::Token,
    callee: T,
    env: Rc<Environment>,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if callee.arity() != args.len() {
        return runtime_error(
            paren,
            &format!(
                "Expected {} arguments but got {}.",
                callee.arity(),
                args.len()
            ),
        );
    }
    callee.call(env, args)
}
