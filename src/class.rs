use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    builtin_funcs::LoxCallable,
    error::{RuntimeError, RuntimeException},
    function::{FunctionType, LoxFunction},
    interpreter::Interpreter,
    object::Object,
    token::Token,
};

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Rc<LoxFunction>>,
    ) -> Self {
        LoxClass {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&Rc<LoxFunction>> {
        self.methods
            .get(name)
            .or(if let Some(superclass) = &self.superclass {
                superclass.find_method(name)
            } else {
                None
            })
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeException> {
        let instance = Object::Instance(Rc::new(RefCell::new(LoxInstance::new(self.clone()))));
        if let Some(initializer) = self.find_method("init") {
            initializer.bind(instance.clone()).call(interpreter, args)?;
        }

        Ok(instance)
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, Object>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        LoxInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeException> {
        if let Some(value) = self.fields.get(&name.value.to_string()) {
            return Ok(value.clone());
        }

        if let Some(method) = self.class.find_method(&name.value.to_string()) {
            return Ok(Object::Function(Rc::new(
                method.bind(Object::Instance(Rc::new(RefCell::new(self.clone())))),
            )));
        }

        Err(RuntimeException::Error(RuntimeError::new(
            name.to_owned(),
            "Undefined property.",
        )))
    }

    pub fn get_getter(&self, name: &Token) -> Option<&Rc<LoxFunction>> {
        if let Some(method) = self.class.find_method(&name.value.to_string()) {
            if method.kind == FunctionType::GetterMethod {
                return Some(method);
            }
        }
        None
    }

    pub fn set(&mut self, name: Token, value: Object) -> Result<(), RuntimeException> {
        self.fields.insert(name.value.to_string(), value);
        Ok(())
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} instance>", self.class.name)
    }
}
