use std::rc::Rc;
use crate::error::*;
use crate::token::*;
use crate::expr::*;

pub enum Stmt {
    Break(BreakStmt),
    Block(BlockStmt),
    Expression(ExpressionStmt),
    Function(FunctionStmt),
    If(IfStmt),
    Print(PrintStmt),
    Return(ReturnStmt),
    Var(VarStmt),
    While(WhileStmt),
}

impl Stmt {
    pub fn accept<T>(&self, wrapper: &Rc<Stmt>, stmt_visitor: &dyn StmtVisitor<T>) -> Result<T, LoxResult> {
        match self {
            Stmt::Break(v) => stmt_visitor.visit_break_stmt(wrapper, &v),
            Stmt::Block(v) => stmt_visitor.visit_block_stmt(wrapper, &v),
            Stmt::Expression(v) => stmt_visitor.visit_expression_stmt(wrapper, &v),
            Stmt::Function(v) => stmt_visitor.visit_function_stmt(wrapper, &v),
            Stmt::If(v) => stmt_visitor.visit_if_stmt(wrapper, &v),
            Stmt::Print(v) => stmt_visitor.visit_print_stmt(wrapper, &v),
            Stmt::Return(v) => stmt_visitor.visit_return_stmt(wrapper, &v),
            Stmt::Var(v) => stmt_visitor.visit_var_stmt(wrapper, &v),
            Stmt::While(v) => stmt_visitor.visit_while_stmt(wrapper, &v),
        }
    }
}

pub struct BreakStmt {
    pub token: Token,
}

pub struct BlockStmt {
    pub statements: Rc<Vec<Rc<Stmt>>>,
}

pub struct ExpressionStmt {
    pub expression: Rc<Expr>,
}

pub struct FunctionStmt {
    pub name: Token,
    pub params: Rc<Vec<Token>>,
    pub body: Rc<Vec<Rc<Stmt>>>,
}

pub struct IfStmt {
    pub condition: Rc<Expr>,
    pub then_branch: Rc<Stmt>,
    pub else_branch: Option<Rc<Stmt>>,
}

pub struct PrintStmt {
    pub expression: Rc<Expr>,
}

pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Rc<Expr>>,
}

pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Rc<Expr>>,
}

pub struct WhileStmt {
    pub condition: Rc<Expr>,
    pub body: Rc<Stmt>,
}

pub trait StmtVisitor<T> {
    fn visit_break_stmt(&self, wrapper: &Rc<Stmt>, stmt: &BreakStmt) -> Result<T, LoxResult>;
    fn visit_block_stmt(&self, wrapper: &Rc<Stmt>, stmt: &BlockStmt) -> Result<T, LoxResult>;
    fn visit_expression_stmt(&self, wrapper: &Rc<Stmt>, stmt: &ExpressionStmt) -> Result<T, LoxResult>;
    fn visit_function_stmt(&self, wrapper: &Rc<Stmt>, stmt: &FunctionStmt) -> Result<T, LoxResult>;
    fn visit_if_stmt(&self, wrapper: &Rc<Stmt>, stmt: &IfStmt) -> Result<T, LoxResult>;
    fn visit_print_stmt(&self, wrapper: &Rc<Stmt>, stmt: &PrintStmt) -> Result<T, LoxResult>;
    fn visit_return_stmt(&self, wrapper: &Rc<Stmt>, stmt: &ReturnStmt) -> Result<T, LoxResult>;
    fn visit_var_stmt(&self, wrapper: &Rc<Stmt>, stmt: &VarStmt) -> Result<T, LoxResult>;
    fn visit_while_stmt(&self, wrapper: &Rc<Stmt>, stmt: &WhileStmt) -> Result<T, LoxResult>;
}

