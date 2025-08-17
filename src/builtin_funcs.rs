use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{error::RuntimeException, interpreter::Interpreter, object::Object};

pub trait LoxCallable: fmt::Display + fmt::Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeException>;
}

#[derive(Debug)]
pub struct ClockFunction;

impl LoxCallable for ClockFunction {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: Vec<Object>,
    ) -> Result<Object, RuntimeException> {
        Ok(Object::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as f64,
        ))
    }
}

impl fmt::Display for ClockFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn native clock>")
    }
}
