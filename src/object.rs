use std::{
    cell::RefCell,
    fmt::{self, Debug},
    rc::Rc,
};

use crate::{
    builtin_funcs::LoxCallable,
    class::{LoxClass, LoxInstance},
};

#[derive(Clone, Debug)]
pub enum Object {
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Rc<dyn LoxCallable>),
    Instance(Rc<RefCell<LoxInstance>>),
    Class(Rc<LoxClass>),
    Nil,
    Undefined,
}

impl Object {
    pub fn maybe_to_string(&self) -> Option<String> {
        match self {
            Object::String(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn maybe_to_number(&self) -> Option<f64> {
        match self {
            Object::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn maybe_to_boolean(&self) -> Option<bool> {
        match self {
            Object::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    pub fn maybe_to_function(&self) -> Option<Rc<dyn LoxCallable>> {
        match self {
            Object::Function(value) => Some(value.to_owned()),
            _ => None,
        }
    }

    pub fn maybe_to_instance(&self) -> Option<Rc<RefCell<LoxInstance>>> {
        match self {
            Object::Instance(value) => Some(value.to_owned()),
            _ => None,
        }
    }

    pub fn maybe_to_class(&self) -> Option<Rc<LoxClass>> {
        match self {
            Object::Class(value) => Some(value.to_owned()),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(value) => *value,
            Object::Nil => false,
            Object::Undefined => false,
            _ => true,
        }
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object::Boolean(value)
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Nil, Object::Nil) => true,
            (Object::Undefined, Object::Undefined) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Boolean(value) => write!(f, "{value}"),
            Object::Number(value) => write!(f, "{value}"),
            Object::String(value) => write!(f, "{value}"),
            Object::Function(value) => write!(f, "{value}"),
            Object::Instance(value) => write!(f, "{}", value.borrow()),
            Object::Class(value) => write!(f, "{value}"),
            Object::Nil => write!(f, "nil"),
            Object::Undefined => write!(f, "undefined"),
        }
    }
}
