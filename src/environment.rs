use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use crate::{
    error::{RuntimeError, RuntimeException},
    object::Object,
    token::Token,
};

#[derive(Clone, Debug)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Object>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<&Object, RuntimeException> {
        if let Some(value) = self.values.get(&name.value.to_string()) {
            if *value != Object::Undefined {
                return Ok(value);
            } else {
                return Err(RuntimeException::Error(RuntimeError::new(
                    name.to_owned(),
                    "The variable isn't initialized.",
                )));
            }
        }

        if let Some(enclosing) = &self.enclosing {
            return unsafe { enclosing.as_ptr().as_ref().unwrap().get(name) };
        }

        Err(RuntimeException::Error(RuntimeError::new(
            name.to_owned(),
            "Undefined variable.",
        )))
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeException> {
        if let Entry::Occupied(mut e) = self.values.entry(name.value.to_string()) {
            e.insert(value);
            return Ok(());
        }
        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }
        Err(RuntimeException::Error(RuntimeError::new(
            name.to_owned(),
            "Unclarified variable.",
        )))
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn ancestor(&mut self, distance: usize) -> Option<&mut Environment> {
        let mut environment = self;
        for _ in 0..distance {
            if let Some(enclosing) = &environment.enclosing {
                environment = unsafe { enclosing.as_ptr().as_mut().unwrap() };
            } else {
                return None;
            }
        }
        Some(environment)
    }

    pub fn get_at(&mut self, distance: usize, name: &Token) -> Result<&Object, RuntimeException> {
        match self.ancestor(distance) {
            Some(env) => env.get(name),
            None => Err(RuntimeException::Error(RuntimeError::new(
                name.clone(),
                "The variable isn't declared.",
            ))),
        }
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<(), RuntimeException> {
        match self.ancestor(distance) {
            Some(env) => env.assign(name, value),
            None => Err(RuntimeException::Error(RuntimeError::new(
                name.to_owned(),
                "Unclarified variable.",
            ))),
        }
    }
}
