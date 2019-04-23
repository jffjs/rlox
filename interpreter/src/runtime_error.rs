#[derive(Debug)]
pub struct RuntimeError {
    msg: String,
    line: u32,
}

impl RuntimeError {
    fn new(line: u32, msg: String) -> RuntimeError {
        RuntimeError { msg, line }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.description())
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

fn runtime_error(token: &token::Token, msg: &str) -> Result<Value, RuntimeError> {
    Result::Err(RuntimeError::new(token.line, String::from(msg)))
}
