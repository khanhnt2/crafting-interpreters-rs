use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{object::Object, stmt::BlockStmt, token::Token};

pub trait ExprVisitor {
    type Output;

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Self::Output;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Output;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> Self::Output;
    fn visit_get_expr(&mut self, expr: &GetExpr) -> Self::Output;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Self::Output;
    fn visit_lambda_expr(&mut self, expr: &LambdaExpr) -> Self::Output;
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Self::Output;
    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Self::Output;
    fn visit_set_expr(&mut self, expr: &SetExpr) -> Self::Output;
    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Self::Output;
    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Self::Output;
    fn visit_ternary_expr(&mut self, expr: &TernaryExpr) -> Self::Output;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Output;
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Self::Output;

    fn accept(&mut self, expr: &Expr) -> Self::Output {
        match expr {
            Expr::Assign(expr) => self.visit_assign_expr(expr),
            Expr::Binary(expr) => self.visit_binary_expr(expr),
            Expr::Call(expr) => self.visit_call_expr(expr),
            Expr::Get(expr) => self.visit_get_expr(expr),
            Expr::Grouping(expr) => self.visit_grouping_expr(expr),
            Expr::Lambda(expr) => self.visit_lambda_expr(expr),
            Expr::Literal(expr) => self.visit_literal_expr(expr),
            Expr::Logical(expr) => self.visit_logical_expr(expr),
            Expr::Set(expr) => self.visit_set_expr(expr),
            Expr::Super(expr) => self.visit_super_expr(expr),
            Expr::This(expr) => self.visit_this_expr(expr),
            Expr::Ternary(expr) => self.visit_ternary_expr(expr),
            Expr::Unary(expr) => self.visit_unary_expr(expr),
            Expr::Variable(expr) => self.visit_variable_expr(expr),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Expr {
    Assign(Box<AssignExpr>),
    Binary(Box<BinaryExpr>),
    Call(Box<CallExpr>),
    Get(Box<GetExpr>),
    Grouping(Box<GroupingExpr>),
    Lambda(Box<LambdaExpr>),
    Literal(LiteralExpr),
    Logical(Box<LogicalExpr>),
    Set(Box<SetExpr>),
    Super(SuperExpr),
    This(ThisExpr),
    Ternary(Box<TernaryExpr>),
    Unary(Box<UnaryExpr>),
    Variable(VariableExpr),
}

impl Expr {
    pub fn to_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        format!("{self:?}").hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Debug)]
pub struct AssignExpr {
    pub name: Token,
    pub value: Expr,
}

impl AssignExpr {
    pub fn new(name: Token, value: Expr) -> Self {
        AssignExpr { name, value }
    }
}
#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

impl BinaryExpr {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        BinaryExpr {
            left,
            operator,
            right,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CallExpr {
    pub callee: Expr,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

impl CallExpr {
    pub fn new(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Self {
        CallExpr {
            callee,
            paren,
            arguments,
        }
    }
}
#[derive(Clone, Debug)]
pub struct GetExpr {
    pub object: Expr,
    pub name: Token,
}

impl GetExpr {
    pub fn new(object: Expr, name: Token) -> Self {
        GetExpr { object, name }
    }
}
#[derive(Clone, Debug)]
pub struct GroupingExpr {
    pub expression: Expr,
}

impl GroupingExpr {
    pub fn new(expression: Expr) -> Self {
        GroupingExpr { expression }
    }
}

#[derive(Clone, Debug)]
pub struct LambdaExpr {
    pub params: Vec<Token>,
    pub body: BlockStmt,
}

impl LambdaExpr {
    pub fn new(params: Vec<Token>, body: BlockStmt) -> Self {
        LambdaExpr { params, body }
    }
}

#[derive(Clone, Debug)]
pub struct LiteralExpr {
    pub value: Object,
}

impl LiteralExpr {
    pub fn new(value: Object) -> Self {
        LiteralExpr { value }
    }
}
#[derive(Clone, Debug)]
pub struct LogicalExpr {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

impl LogicalExpr {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}
#[derive(Clone, Debug)]
pub struct SetExpr {
    pub object: Expr,
    pub name: Token,
    pub value: Expr,
}

impl SetExpr {
    pub fn new(object: Expr, name: Token, value: Expr) -> Self {
        Self {
            object,
            name,
            value,
        }
    }
}
#[derive(Clone, Debug)]
pub struct SuperExpr {
    pub keyword: Token,
    pub method: Token,
}

impl SuperExpr {
    pub fn new(keyword: Token, method: Token) -> Self {
        Self { keyword, method }
    }
}
#[derive(Clone, Debug)]
pub struct ThisExpr {
    pub keyword: Token,
}

impl ThisExpr {
    pub fn new(keyword: Token) -> Self {
        Self { keyword }
    }
}

#[derive(Clone, Debug)]
pub struct TernaryExpr {
    pub condition: Expr,
    pub then_branch: Expr,
    pub else_branch: Expr,
}

impl TernaryExpr {
    pub fn new(condition: Expr, then_branch: Expr, else_branch: Expr) -> Self {
        Self {
            condition,
            then_branch,
            else_branch,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Expr,
}

impl UnaryExpr {
    pub fn new(operator: Token, right: Expr) -> Self {
        UnaryExpr { operator, right }
    }
}
#[derive(Clone, Debug)]
pub struct VariableExpr {
    pub name: Token,
}

impl VariableExpr {
    pub fn new(name: Token) -> Self {
        Self { name }
    }
}
