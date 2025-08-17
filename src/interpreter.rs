use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    builtin_funcs::{ClockFunction, LoxCallable},
    class::LoxClass,
    environment::Environment,
    error::{RuntimeError, RuntimeException, RuntimeReturn},
    expr::{
        AssignExpr, BinaryExpr, CallExpr, Expr, ExprVisitor, GetExpr, GroupingExpr, LambdaExpr,
        LiteralExpr, LogicalExpr, SetExpr, SuperExpr, TernaryExpr, ThisExpr, UnaryExpr,
        VariableExpr,
    },
    function::{FunctionType, LambdaFunction, LoxFunction},
    object::Object,
    stmt::{
        BlockStmt, ClassStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
        StmtVisitor, VarStmt, WhileStmt,
    },
    token::{Token, TokenIdentity, TokenValue},
};

pub struct Interpreter {
    pub global: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
    pub locals: HashMap<u64, usize>,
    pub writer: Rc<RefCell<dyn std::io::Write>>,
}

impl Interpreter {
    pub fn new(writer: Rc<RefCell<impl std::io::Write + 'static>>) -> Self {
        let global = Rc::new(RefCell::new(Environment::new(None)));
        global
            .borrow_mut()
            .define("clock", Object::Function(Rc::new(ClockFunction)));
        Self {
            global: global.clone(),
            environment: global,
            locals: HashMap::new(),
            writer,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<Object, RuntimeException> {
        let mut ret = Object::Undefined;
        for stmt in statements {
            ret = self.execute(stmt)?;
        }
        Ok(ret)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeException> {
        ExprVisitor::accept(self, expr)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Object, RuntimeException> {
        StmtVisitor::accept(self, stmt)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.to_hash(), depth);
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<Object, RuntimeException> {
        let previous = self.environment.clone();
        self.environment = environment;

        let mut ret = Object::Undefined;
        for stmt in statements {
            ret = self.execute(stmt)?;
        }

        self.environment = previous;

        Ok(ret)
    }

    fn lookup_variable(&mut self, name: &Token, expr: &Expr) -> Result<&Object, RuntimeException> {
        if let Some(distance) = self.locals.get(&expr.to_hash()) {
            unsafe {
                self.environment
                    .as_ptr()
                    .as_mut()
                    .unwrap()
                    .get_at(*distance, name)
            }
        } else {
            unsafe { self.global.as_ptr().as_ref().unwrap().get(name) }
        }
    }
}

impl ExprVisitor for Interpreter {
    type Output = Result<Object, RuntimeException>;

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Self::Output {
        let value = self.evaluate(&expr.value)?;
        if let Some(distance) = self
            .locals
            .get(&Expr::Assign(Box::new(expr.to_owned())).to_hash())
        {
            self.environment
                .borrow_mut()
                .assign_at(*distance, &expr.name, value.clone())?;
        } else {
            self.global.borrow_mut().assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Output {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        match expr.operator.id {
            TokenIdentity::Greater => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Boolean(left > right)),
                _ => Ok(Object::Boolean(false)),
            },
            TokenIdentity::GreaterEqual => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Boolean(left >= right)),
                _ => Ok(Object::Boolean(false)),
            },
            TokenIdentity::Less => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Boolean(left < right)),
                _ => Ok(Object::Boolean(false)),
            },
            TokenIdentity::LessEqual => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Boolean(left <= right)),
                _ => Ok(Object::Boolean(false)),
            },
            TokenIdentity::BangEqual => Ok(Object::Boolean(left != right)),
            TokenIdentity::EqualEqual => Ok(Object::Boolean(left == right)),
            TokenIdentity::Minus => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Number(left - right)),
                _ => Err(RuntimeException::Error(RuntimeError::new(
                    expr.operator.clone(),
                    "Only support number operands.",
                ))),
            },
            TokenIdentity::Plus => match (left.clone(), right.clone()) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Number(left + right)),
                (Object::String(left), Object::String(right)) => Ok(Object::String(left + &right)),
                (Object::String(left), Object::Number(right)) => {
                    Ok(Object::String(left + &right.to_string()))
                }
                _ => Err(RuntimeException::Error(RuntimeError::new(
                    expr.operator.clone(),
                    &format!("Invalid operands {left} and {right} for + operator."),
                ))),
            },
            TokenIdentity::Slash => match (left, right) {
                (Object::Number(_), Object::Number(0.0)) => Err(RuntimeException::Error(
                    RuntimeError::new(expr.operator.clone(), "Divided by zero."),
                )),
                (Object::Number(left), Object::Number(right)) => Ok(Object::Number(left / right)),
                _ => Err(RuntimeException::Error(RuntimeError::new(
                    expr.operator.clone(),
                    "Only support number operands.",
                ))),
            },
            TokenIdentity::Star => match (left, right) {
                (Object::Number(left), Object::Number(right)) => Ok(Object::Number(left * right)),
                _ => Err(RuntimeException::Error(RuntimeError::new(
                    expr.operator.clone(),
                    "Only support number operands.",
                ))),
            },
            _ => Err(RuntimeException::Error(RuntimeError::new(
                expr.operator.clone(),
                "Unsupported operator.",
            ))),
        }
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Self::Output {
        let callee = self.evaluate(&expr.callee)?;
        let mut arguments = Vec::new();

        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument)?);
        }
        match callee {
            Object::Function(function) => function.call(self, arguments),
            Object::Class(lox_class) => lox_class.call(self, arguments),
            _ => Err(RuntimeException::Error(RuntimeError::new(
                expr.paren.clone(),
                "Can only call functions and classes.",
            ))),
        }
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> Self::Output {
        let object = self.evaluate(&expr.object)?;
        match object {
            Object::Instance(instance) => instance.borrow().get_getter(&expr.name).map_or(
                instance.borrow().get(&expr.name),
                |getter| {
                    // We bind the the getter to the instance to be able to call `this` keyword
                    // Check Test3 in class2.lox test
                    getter
                        .bind(Object::Instance(instance.clone()))
                        .call(self, Vec::new())
                },
            ),
            Object::Class(class) => class.find_method(&expr.name.value.to_string()).map_or(
                Err(RuntimeException::Error(RuntimeError::new(
                    expr.name.clone(),
                    &format!(
                        "Class {} doesn't have a method named '{}'.",
                        class.name, expr.name.value
                    ),
                ))),
                |method| Ok(Object::Function(method.to_owned())),
            ),
            _ => Err(RuntimeException::Error(RuntimeError::new(
                expr.name.clone(),
                "Only instances have properties.",
            ))),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Self::Output {
        self.evaluate(&expr.expression)
    }

    fn visit_lambda_expr(&mut self, expr: &LambdaExpr) -> Self::Output {
        Ok(Object::Function(Rc::new(LambdaFunction::new(
            expr.to_owned(),
        ))))
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Self::Output {
        Ok(expr.value.to_owned())
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Self::Output {
        let left = self.evaluate(&expr.left)?;

        if left.is_truthy() && expr.operator.id == TokenIdentity::Or {
            return Ok(left);
        }
        if !left.is_truthy() && expr.operator.id == TokenIdentity::And {
            return Ok(left);
        }

        self.evaluate(&expr.right)
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> Self::Output {
        let object = self.evaluate(&expr.object)?;
        match object {
            Object::Instance(instance) => {
                let value = self.evaluate(&expr.value)?;
                instance
                    .borrow_mut()
                    .set(expr.name.clone(), value.clone())?;
                Ok(value)
            }
            _ => Err(RuntimeException::Error(RuntimeError::new(
                expr.name.clone(),
                "Only instances have properties.",
            ))),
        }
    }

    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Self::Output {
        let distance = *self
            .locals
            .get(&Expr::Super(expr.to_owned()).to_hash())
            .unwrap();
        let superclass = self
            .environment
            .borrow_mut()
            .get_at(distance, &expr.keyword)?
            .maybe_to_class()
            .unwrap();
        let object = self
            .environment
            .borrow_mut()
            .get_at(
                distance - 1,
                &Token::new(
                    TokenIdentity::This,
                    TokenValue::String("this".to_string()),
                    0,
                    0,
                ),
            )?
            .to_owned();

        if let Some(method) = superclass.find_method(&expr.method.value.to_string()) {
            Ok(Object::Function(Rc::new(method.bind(object))))
        } else {
            Err(RuntimeException::Error(RuntimeError::new(
                expr.method.clone(),
                "Undefined property.",
            )))
        }
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Self::Output {
        self.lookup_variable(&expr.keyword, &Expr::This(expr.to_owned()))
            .map(|r| r.to_owned())
    }

    fn visit_ternary_expr(&mut self, expr: &TernaryExpr) -> Self::Output {
        let condition = self.evaluate(&expr.condition)?;
        if condition.is_truthy() {
            self.evaluate(&expr.then_branch)
        } else {
            self.evaluate(&expr.else_branch)
        }
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Output {
        let right = self.evaluate(&expr.right)?;
        Ok(match expr.operator.id {
            TokenIdentity::Bang => (!right.is_truthy()).into(),
            TokenIdentity::Minus => Object::Number(-right.maybe_to_number().unwrap()),
            _ => Object::Nil,
        })
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Self::Output {
        self.lookup_variable(&expr.name, &Expr::Variable(expr.to_owned()))
            .map(|r| r.to_owned())
    }
}

impl StmtVisitor for Interpreter {
    type Output = Result<Object, RuntimeException>;

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Self::Output {
        self.execute_block(
            &stmt.statements,
            Rc::new(RefCell::new(Environment::new(Some(
                self.environment.clone(),
            )))),
        )
    }

    fn visit_break_stmt(&self) -> Self::Output {
        Err(RuntimeException::Break)
    }

    fn visit_continue_stmt(&self) -> Self::Output {
        Err(RuntimeException::Continue)
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Self::Output {
        let superclass = if let Some(superclass) = &stmt.superclass {
            match self.evaluate(&Expr::Variable(superclass.to_owned()))? {
                Object::Class(lox_class) => Some(lox_class),
                _ => {
                    return Err(RuntimeException::Error(RuntimeError::new(
                        superclass.name.clone(),
                        "Superclass must be a class.",
                    )));
                }
            }
        } else {
            None
        };

        if stmt.superclass.is_some() {
            if let Some(superclass) = superclass.clone() {
                self.environment = Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                ))));
                self.environment
                    .borrow_mut()
                    .define("super", Object::Class(superclass));
            }
        }

        let mut methods = HashMap::new();
        for method in &stmt.methods {
            let function = LoxFunction::new(method.clone(), self.environment.clone(), method.kind);
            methods.insert(method.name.value.to_string(), Rc::new(function));
        }

        for method in &stmt.getter_methods {
            let function = LoxFunction::new(
                method.clone(),
                self.environment.clone(),
                FunctionType::GetterMethod,
            );
            methods.insert(method.name.value.to_string(), Rc::new(function));
        }

        for method in &stmt.static_methods {
            let function = LoxFunction::new(
                method.clone(),
                Rc::new(RefCell::new(Environment::new(None))),
                FunctionType::StaticMethod,
            );
            methods.insert(method.name.value.to_string(), Rc::new(function));
        }

        let kclass = LoxClass::new(stmt.name.value.to_string(), superclass.clone(), methods);

        if superclass.is_some() {
            self.environment = self
                .environment
                .clone()
                .borrow()
                .enclosing
                .as_ref()
                .unwrap()
                .clone();
        }

        self.environment
            .borrow_mut()
            .define(&stmt.name.value.to_string(), Object::Class(Rc::new(kclass)));

        Ok(Object::Undefined)
    }

    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Self::Output {
        self.evaluate(&stmt.expr)
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> Self::Output {
        let lox = LoxFunction::new(
            stmt.to_owned(),
            self.environment.clone(),
            FunctionType::Function,
        );
        self.environment
            .borrow_mut()
            .define(&stmt.name.value.to_string(), Object::Function(Rc::new(lox)));
        Ok(Object::Undefined)
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Self::Output {
        if self.evaluate(&stmt.condition)?.is_truthy() {
            self.visit_block_stmt(&stmt.then_branch)
        } else if let Some(else_branch) = &stmt.else_branch {
            self.visit_block_stmt(else_branch)
        } else {
            Ok(Object::Undefined)
        }
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> Self::Output {
        let value = self.evaluate(&stmt.expr)?;
        writeln!(self.writer.borrow_mut(), "{value}").unwrap();
        Ok(Object::Undefined)
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Self::Output {
        match &stmt.value {
            Some(value) => Err(RuntimeException::Return(RuntimeReturn::new(
                self.evaluate(value)?,
            ))),
            None => Err(RuntimeException::Return(RuntimeReturn::new(Object::Nil))),
        }
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> Self::Output {
        if let Some(initializer) = &stmt.initializer {
            let value = self.evaluate(initializer)?;
            self.environment
                .borrow_mut()
                .define(&stmt.name.value.to_string(), value);
        } else {
            self.environment
                .borrow_mut()
                .define(&stmt.name.value.to_string(), Object::Undefined);
        }
        Ok(Object::Undefined)
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Self::Output {
        while self.evaluate(&stmt.condition)?.is_truthy() {
            match self.visit_block_stmt(&stmt.body) {
                Ok(_) => continue,
                Err(error) => match error {
                    RuntimeException::Break => break,
                    RuntimeException::Continue => continue,
                    _ => return Err(error),
                },
            }
        }
        Ok(Object::Undefined)
    }
}
