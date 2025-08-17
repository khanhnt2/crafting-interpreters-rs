use std::collections::HashMap;

use crate::{
    error::RuntimeError,
    expr::{
        AssignExpr, BinaryExpr, CallExpr, Expr, ExprVisitor, GetExpr, GroupingExpr, LambdaExpr,
        LiteralExpr, LogicalExpr, SetExpr, SuperExpr, TernaryExpr, ThisExpr, UnaryExpr,
        VariableExpr,
    },
    function::FunctionType,
    interpreter::Interpreter,
    stmt::{
        BlockStmt, ClassStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
        StmtVisitor, VarStmt, WhileStmt,
    },
    token::Token,
};

#[derive(Copy, Clone, Debug, PartialEq)]
enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver<'a> {
    pub interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![HashMap::new()],
            current_function: FunctionType::default(),
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in statements {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        StmtVisitor::accept(self, stmt)
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        ExprVisitor::accept(self, expr)
    }

    fn resolve_function(&mut self, function: &FunctionStmt) -> Result<(), RuntimeError> {
        let enclosing_function = self.current_function;
        self.current_function = function.kind;
        self.begin_scope();
        for param in &function.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(&function.body.statements)?;
        self.end_scope();
        self.current_function = enclosing_function;

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), RuntimeError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.value.to_string()) {
                return Err(RuntimeError::new(
                    name.to_owned(),
                    "Already a variable with this name in this scope.",
                ));
            }
            scope.insert(name.value.to_string(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.value.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.value.to_string()) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }
}

impl<'a> ExprVisitor for Resolver<'a> {
    type Output = Result<(), RuntimeError>;

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Self::Output {
        self.resolve_expr(&expr.value)?;
        self.resolve_local(&Expr::Assign(Box::new(expr.to_owned())), &expr.name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Output {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Self::Output {
        self.resolve_expr(&expr.callee)?;

        for arg in &expr.arguments {
            self.resolve_expr(arg)?;
        }

        Ok(())
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> Self::Output {
        self.resolve_expr(&expr.object)
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Self::Output {
        self.resolve_expr(&expr.expression)
    }

    fn visit_lambda_expr(&mut self, expr: &LambdaExpr) -> Self::Output {
        let enclosing_function = self.current_function;
        self.current_function = FunctionType::Function;
        self.begin_scope();
        for param in &expr.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(&expr.body.statements)?;
        self.end_scope();
        self.current_function = enclosing_function;

        Ok(())
    }

    fn visit_literal_expr(&self, _expr: &LiteralExpr) -> Self::Output {
        Ok(())
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Self::Output {
        self.resolve_expr(&expr.right)
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> Self::Output {
        self.resolve_expr(&expr.value)?;
        self.resolve_expr(&expr.object)
    }

    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Self::Output {
        if self.current_class == ClassType::None {
            return Err(RuntimeError::new(
                expr.keyword.clone(),
                "Can't use 'super' outside of a class.",
            ));
        }
        if self.current_class != ClassType::Subclass {
            return Err(RuntimeError::new(
                expr.keyword.clone(),
                "Can't use 'super' in a class with no superclass.",
            ));
        }

        self.resolve_local(&Expr::Super(expr.to_owned()), &expr.keyword);

        Ok(())
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Self::Output {
        if self.current_class == ClassType::None {
            return Err(RuntimeError::new(
                expr.keyword.clone(),
                "Can't use 'this' outside of a class.",
            ));
        }
        self.resolve_local(&Expr::This(expr.to_owned()), &expr.keyword);
        Ok(())
    }

    fn visit_ternary_expr(&mut self, expr: &TernaryExpr) -> Self::Output {
        self.resolve_expr(&expr.condition)?;
        self.resolve_expr(&expr.then_branch)?;
        self.resolve_expr(&expr.else_branch)
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Output {
        self.resolve_expr(&expr.right)
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Self::Output {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&expr.name.value.to_string()) {
                // TODO: fix block2.lox test
                return Err(RuntimeError::new(
                    expr.name.clone(),
                    "Can't read local variable in its own initializer.",
                ));
            }
        }
        self.resolve_local(&Expr::Variable(expr.to_owned()), &expr.name);
        Ok(())
    }
}

impl<'a> StmtVisitor for Resolver<'a> {
    type Output = Result<(), RuntimeError>;

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Self::Output {
        self.begin_scope();
        self.resolve_stmts(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_break_stmt(&self) -> Self::Output {
        Ok(())
    }

    fn visit_continue_stmt(&self) -> Self::Output {
        Ok(())
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Self::Output {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&stmt.name)?;
        self.define(&stmt.name);

        if let Some(superclass) = &stmt.superclass {
            if stmt.name.value == superclass.name.value {
                return Err(RuntimeError::new(
                    superclass.name.clone(),
                    "A class cannot inherit from itself.",
                ));
            }
            self.current_class = ClassType::Subclass;
            self.resolve_expr(&Expr::Variable(superclass.to_owned()))?;
        }

        if stmt.superclass.is_some() {
            self.begin_scope();
            self.scopes
                .last_mut()
                .and_then(|scope| scope.insert("super".to_string(), true));
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .and_then(|scope| scope.insert("this".to_string(), true));
        for method in &stmt.methods {
            self.resolve_function(method)?;
        }

        for method in &stmt.getter_methods {
            self.resolve_function(method)?;
        }
        self.end_scope();

        self.begin_scope();
        for method in &stmt.static_methods {
            self.resolve_function(method)?;
        }
        self.end_scope();

        if stmt.superclass.is_some() {
            self.end_scope();
        }
        self.current_class = enclosing_class;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Self::Output {
        self.resolve_expr(&stmt.expr)
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> Self::Output {
        self.declare(&stmt.name)?;
        self.define(&stmt.name);
        self.resolve_function(stmt)
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Self::Output {
        self.resolve_expr(&stmt.condition)?;
        self.visit_block_stmt(&stmt.then_branch)?;
        if let Some(else_branch) = &stmt.else_branch {
            self.visit_block_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> Self::Output {
        self.resolve_expr(&stmt.expr)
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Self::Output {
        if self.current_function == FunctionType::None {
            return Err(RuntimeError::new(
                stmt.keyword.clone(),
                "Cannot return from top-level code.",
            ));
        }
        if let Some(value) = &stmt.value {
            if self.current_function == FunctionType::Initializer {
                return Err(RuntimeError::new(
                    stmt.keyword.clone(),
                    "Cannot return a value from an initializer.",
                ));
            }
            self.resolve_expr(value)?;
        }
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> Self::Output {
        self.declare(&stmt.name)?;
        if let Some(initializer) = &stmt.initializer {
            self.resolve_expr(initializer)?;
        }
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Self::Output {
        self.resolve_expr(&stmt.condition)?;
        self.visit_block_stmt(&stmt.body)
    }
}
