use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    builtin_funcs::LoxCallable,
    environment::Environment,
    error::RuntimeException,
    expr::LambdaExpr,
    interpreter::Interpreter,
    object::Object,
    stmt::FunctionStmt,
    token::{Token, TokenIdentity, TokenValue},
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum FunctionType {
    #[default]
    None,
    Function,
    Initializer,
    Method,
    StaticMethod,
    GetterMethod,
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionType::Function => write!(f, "function"),
            FunctionType::Initializer => write!(f, "initializer"),
            FunctionType::Method => write!(f, "method"),
            FunctionType::StaticMethod => write!(f, "static method"),
            FunctionType::GetterMethod => write!(f, "getter method"),
            FunctionType::None => write!(f, "none"),
        }
    }
}

#[derive(Clone)]
pub struct LoxFunction {
    declaration: FunctionStmt,
    closure: Rc<RefCell<Environment>>,
    pub kind: FunctionType,
}

impl fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoxFunction")
            .field("declaration", &self.declaration)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl LoxFunction {
    pub fn new(
        declaration: FunctionStmt,
        closure: Rc<RefCell<Environment>>,
        kind: FunctionType,
    ) -> Self {
        Self {
            declaration,
            closure,
            kind,
        }
    }

    pub fn bind(&self, instance: Object) -> LoxFunction {
        if let Object::Instance(_) = instance {
            let mut environment = Environment::new(Some(self.closure.clone()));
            environment.define("this", instance);
            LoxFunction::new(
                self.declaration.clone(),
                Rc::new(RefCell::new(environment)),
                self.kind,
            )
        } else {
            panic!("Cannot bind non-instance object.")
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeException> {
        let mut environment = Environment::new(Some(self.closure.clone()));
        for (i, param) in self.declaration.params.iter().enumerate().take(args.len()) {
            environment.define(&param.value.to_string(), args[i].clone());
        }

        match interpreter.execute_block(
            &self.declaration.body.statements,
            Rc::new(RefCell::new(environment)),
        ) {
            Ok(_) => {
                if self.kind == FunctionType::Initializer {
                    self.closure
                        .borrow_mut()
                        .get_at(
                            0,
                            &Token::new(
                                TokenIdentity::This,
                                TokenValue::String("this".to_string()),
                                0,
                                0,
                            ),
                        )
                        .map(|r| r.to_owned())
                } else {
                    Ok(Object::Nil)
                }
            }
            Err(e) => match e {
                RuntimeException::Error(err) => Err(RuntimeException::Error(err)),
                RuntimeException::Return(ret) => {
                    if self.kind == FunctionType::Initializer {
                        self.closure
                            .borrow_mut()
                            .get_at(
                                0,
                                &Token::new(
                                    TokenIdentity::This,
                                    TokenValue::String("this".to_string()),
                                    0,
                                    0,
                                ),
                            )
                            .map(|r| r.to_owned())
                    } else {
                        Ok(ret.value)
                    }
                }
                RuntimeException::Break | RuntimeException::Continue => todo!("Why hit this?"),
            },
        }
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.value)
    }
}

#[derive(Clone, Debug)]
pub struct LambdaFunction {
    declaration: LambdaExpr,
}

impl LambdaFunction {
    pub fn new(declaration: LambdaExpr) -> Self {
        LambdaFunction { declaration }
    }
}

impl LoxCallable for LambdaFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeException> {
        let mut environment = Environment::new(Some(interpreter.global.clone()));

        for (i, param) in self.declaration.params.iter().enumerate() {
            environment.define(&param.value.to_string(), args[i].clone());
        }

        interpreter.execute_block(
            &self.declaration.body.statements,
            Rc::new(RefCell::new(environment)),
        )
    }
}

impl fmt::Display for LambdaFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn lambda>")
    }
}
