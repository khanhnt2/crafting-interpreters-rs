use crate::{
    expr::{Expr, VariableExpr},
    function::FunctionType,
    token::Token,
};

pub trait StmtVisitor {
    type Output;

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Self::Output;
    fn visit_break_stmt(&self) -> Self::Output;
    fn visit_continue_stmt(&self) -> Self::Output;
    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Self::Output;
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Self::Output;
    fn visit_function_stmt(&mut self, stmt: &FunctionStmt) -> Self::Output;
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Self::Output;
    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> Self::Output;
    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Self::Output;
    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> Self::Output;
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Self::Output;

    fn accept(&mut self, stmt: &Stmt) -> Self::Output {
        match stmt {
            Stmt::Block(stmt) => self.visit_block_stmt(stmt),
            Stmt::Break => self.visit_break_stmt(),
            Stmt::Continue => self.visit_continue_stmt(),
            Stmt::Class(stmt) => self.visit_class_stmt(stmt),
            Stmt::Expression(stmt) => self.visit_expression_stmt(stmt),
            Stmt::Function(stmt) => self.visit_function_stmt(stmt),
            Stmt::If(stmt) => self.visit_if_stmt(stmt),
            Stmt::Print(stmt) => self.visit_print_stmt(stmt),
            Stmt::Return(stmt) => self.visit_return_stmt(stmt),
            Stmt::Var(stmt) => self.visit_var_stmt(stmt),
            Stmt::While(stmt) => self.visit_while_stmt(stmt),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(BlockStmt),
    Break,
    Continue,
    Class(ClassStmt),
    Expression(ExpressionStmt),
    Function(FunctionStmt),
    If(IfStmt),
    Print(PrintStmt),
    Return(ReturnStmt),
    Var(VarStmt),
    While(WhileStmt),
}

#[derive(Clone, Debug)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>,
}

impl BlockStmt {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}

#[derive(Clone, Debug)]
pub struct ClassStmt {
    pub name: Token,
    pub superclass: Option<VariableExpr>,
    pub methods: Vec<FunctionStmt>,
    pub static_methods: Vec<FunctionStmt>,
    pub getter_methods: Vec<FunctionStmt>,
}

impl ClassStmt {
    pub fn new(
        name: Token,
        superclass: Option<VariableExpr>,
        methods: Vec<FunctionStmt>,
        static_methods: Vec<FunctionStmt>,
        getter_methods: Vec<FunctionStmt>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
            static_methods,
            getter_methods,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionStmt {
    pub expr: Expr,
}

impl ExpressionStmt {
    pub fn new(expr: Expr) -> Self {
        Self { expr }
    }
}
#[derive(Clone, Debug)]
pub struct FunctionStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: BlockStmt,
    pub kind: FunctionType,
}

impl FunctionStmt {
    pub fn new(name: Token, params: Vec<Token>, body: BlockStmt, kind: FunctionType) -> Self {
        Self {
            name,
            params,
            body,
            kind,
        }
    }
}
#[derive(Clone, Debug)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: BlockStmt,
    pub else_branch: Option<BlockStmt>,
}

impl IfStmt {
    pub fn new(condition: Expr, then_branch: BlockStmt, else_branch: Option<BlockStmt>) -> Self {
        Self {
            condition,
            then_branch,
            else_branch,
        }
    }
}
#[derive(Clone, Debug)]
pub struct PrintStmt {
    pub expr: Expr,
}

impl PrintStmt {
    pub fn new(expr: Expr) -> Self {
        Self { expr }
    }
}
#[derive(Clone, Debug)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Expr>,
}

impl ReturnStmt {
    pub fn new(keyword: Token, value: Option<Expr>) -> Self {
        Self { keyword, value }
    }
}
#[derive(Clone, Debug)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

impl VarStmt {
    pub fn new(name: Token, initializer: Option<Expr>) -> Self {
        Self { name, initializer }
    }
}
#[derive(Clone, Debug)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: BlockStmt,
}

impl WhileStmt {
    pub fn new(condition: Expr, body: BlockStmt) -> Self {
        Self { condition, body }
    }
}
