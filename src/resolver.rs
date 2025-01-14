use crate::interpreter::*;
use crate::stmt::*;
use crate::expr::*;
use crate::error::*;
use crate::token::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
    current_function: RefCell<FunctionType>,   
    in_loop: RefCell<bool>,
    had_error: RefCell<bool>,
}

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
}

impl StmtVisitor<()> for Resolver<'_> {
    fn visit_class_stmt(&self, _: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        Ok(())
    }
    
    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if *self.current_function.borrow() == FunctionType::None {
            self.error(&stmt.keyword, "Can't return from top-level code.");
        }
        if let Some(value) = stmt.value.clone() {
            self.resolve_expr(value)?;
        }
        Ok(())
    }
    
    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function)?;

        Ok(())

    }
    
    fn visit_break_stmt(&self, _: Rc<Stmt>, stmt: &BreakStmt) -> Result<(), LoxResult> {
        if !*self.in_loop.borrow() {
            self.error(&stmt.token, "Can't break from top-level code.");
        }
        Ok(())
    }
    
    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(stmt.statements.clone())?;
        self.end_scope();
        Ok(())
    }
    
    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())?;
        Ok(())
    }
    
    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.then_branch.clone())?;
        if let Some(else_branch) = stmt.else_branch.clone() {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())?;
        Ok(())
    }
    
    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        if let Some(initializer) = &stmt.initializer {
            self.resolve_expr(initializer.clone())?;
        }
        self.define(&stmt.name);
        Ok(())
    }
    
    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        let previous_nesting = self.in_loop.replace(true);
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.body.clone())?;
        self.in_loop.replace(previous_nesting);
        Ok(())
    }
}

impl ExprVisitor<()> for Resolver<'_> {
    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<(), LoxResult> { 
        self.resolve_expr(expr.callee.clone())?;
        for argument in expr.arguments.iter() {
            self.resolve_expr(argument.clone())?;
        }
        Ok(()) 
    }

    fn visit_get_expr(&self, _: Rc<Expr>, expr: &GetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.object.clone())?;
        Ok(())
    }

    fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr) -> Result<(), LoxResult> { 
        self.resolve_expr(expr.value.clone())?;
        self.resolve_local(wrapper, &expr.name)?;
        Ok(())
    }

    fn visit_literal_expr(&self, _: Rc<Expr>, _: &LiteralExpr) -> Result<(), LoxResult> { 
        Ok(()) 
    }

    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<(), LoxResult> { 
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(()) 
    }

    fn visit_set_expr(&self, _: Rc<Expr>, expr: &SetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_expr(expr.object.clone())?;
        Ok(())
    }

    fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<(), LoxResult> { 
        self.resolve_expr(expr.expression.clone())?;
        Ok(()) 
    }

    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<(), LoxResult> { 
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(()) 
    }

    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<(), LoxResult> { 
        self.resolve_expr(expr.right.clone())?;
        Ok(()) 
    }

    fn visit_variable_expr(&self, wrapper: Rc<Expr>, expr: &VariableExpr) -> Result<(), LoxResult> {
        if !self.scopes.borrow().is_empty() 
            && self.scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(expr.name.as_string())
                == Some(&false) 
        {
            Err(LoxResult::resolver_error(&expr.name, "Can't read local variable in its own initializer."))
        } else { 
            self.resolve_local(wrapper, &expr.name)?;
            Ok(())
        }
    }
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Resolver<'a> {
        Resolver { 
            interpreter, 
            scopes: RefCell::new(Vec::new()),
            current_function: RefCell::new(FunctionType::None),
            in_loop: RefCell::new(false),
            had_error: RefCell::new(false),
        }
    }

    pub fn resolve(&self, statements: Rc<Vec<Rc<Stmt>>>) -> Result<(), LoxResult> {
        for statement in statements.iter() {
            self.resolve_stmt(statement.clone())?;
        }
        Ok(())
    }

    pub fn success(&self) -> bool {
        !*self.had_error.borrow()
    }

    fn resolve_stmt(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self)
    }

    fn begin_scope(&self) {
        self.scopes.borrow_mut().push(RefCell::new(HashMap::new()));
    }

    fn end_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    fn declare(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow().last() {
            if scope.borrow().contains_key(name.as_string()) {
                self.error(name, "Already a variable with this name in this scope");
            }
            scope.borrow_mut().insert(name.as_string().into(), false);
        }
    }

    fn define(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow().last() {
            scope.borrow_mut().insert(name.as_string().into(), true);
        }
    }

    fn resolve_local(&self, expr: Rc<Expr>, name: &Token) -> Result<(), LoxResult> {
        for (scope_level, map) in self.scopes.borrow().iter().rev().enumerate() {
            if map.borrow().contains_key(name.as_string()) {
                self.interpreter.resolve(expr.clone(), scope_level)?;
                return Ok(());
            }
        }
        Ok(())
    }

    fn resolve_expr(&self, expr: Rc<Expr>) -> Result<(), LoxResult> {
        expr.accept(expr.clone(), self)
    }

    fn resolve_function(&self, function: &FunctionStmt, function_type: FunctionType) -> Result<(), LoxResult> {
        let enclosing_function = self.current_function.replace(function_type);

        self.begin_scope();

        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }

        self.resolve(function.body.clone())?;

        self.end_scope();
        self.current_function.replace(enclosing_function);

        Ok(())
    }

    fn error(&self, token: &Token, message: &str) {
        self.had_error.replace(true);
        LoxResult::runtime_error(token, message);
    }
}
