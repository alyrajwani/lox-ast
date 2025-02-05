use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::native_functions::*;
use crate::lox_function::*;
use crate::lox_class::*;
use crate::stmt::*;
use crate::token::*;
use crate::token_type::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::ops::Deref;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: RefCell<Rc<RefCell<Environment>>>,
    locals: RefCell<HashMap<Rc<Expr>, usize>>,
}

impl StmtVisitor<()> for Interpreter {
    fn visit_class_stmt(&self, _: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        let superclass = if let Some(superclass_expr) = &stmt.superclass {
            let superclass = self.evaluate(superclass_expr.clone())?;
            
            if let Object::Class(c) = superclass {
                Some(c)
            } else if let Expr::Variable(v) = superclass_expr.deref() {
                return Err(LoxResult::runtime_error(
                    &v.name,
                    "Superclass must be a class."
                ));
            } else {
                panic!("Could not dereference superclass.")
            }
        } else {
            None
        };

        self.environment.borrow().borrow_mut().define(stmt.name.as_string(), Object::Nil);

        let enclosing = if let Some(ref s) = superclass {
            let mut e = Environment::new_with_enclosing(self.environment.borrow().clone());
            e.define("super", Object::Class(s.clone()));
            Some(self.environment.replace(Rc::new(RefCell::new(e))))
        } else {
            None
        };

        let mut methods = HashMap::new();
        for method in stmt.methods.deref() {
            if let Stmt::Function(method) = method.deref() {
                let is_initializer = method.name.as_string() == "init";
                let function = Object::Function(
                    Rc::new(LoxFunction::new(method, &self.environment.borrow(), is_initializer))
                );
                methods.insert(method.name.as_string().to_string(), function);
            } else {
                panic!("Class method is not a function.");
            }
        }

        let klass = Object::Class(Rc::new(LoxClass::new(stmt.name.as_string(), superclass, methods)));

        if let Some(previous) = enclosing {
            self.environment.replace(previous);
        }

        self.environment.borrow().borrow_mut().assign(&stmt.name, klass)?;
        Ok(())
    }

    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if let Some(value) = stmt.value.clone() {
            Err(LoxResult::return_value(self.evaluate(value)?))
        } else { 
            Err(LoxResult::return_value(Object::Nil))
        }
    }

    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        let function = LoxFunction::new(stmt, &self.environment.borrow(), false);
        self.environment.borrow().borrow_mut().define(
            stmt.name.as_string(), 
            Object::Function(Rc::new(function))
        );
        Ok(())
    }

    fn visit_break_stmt(&self, _: Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Err(LoxResult::Break)
    }

    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        let e = Environment::new_with_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)
    }

    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.evaluate(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        if self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.then_branch.clone())?;
        } else if let Some(else_branch) = stmt.else_branch.clone() {
            self.execute(else_branch)?;
        }

        Ok(())
    }

    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        let value = self.evaluate(stmt.expression.clone())?;
        println!("{value}");
        Ok(())
    }

    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        let value = if let Some(initializer) = stmt.initializer.clone() {
            self.evaluate(initializer)?
        } else {
            Object::Nil
        };
        self.environment
            .borrow()
            .borrow_mut()
            .define(stmt.name.as_string(), value);
        Ok(())
    }

    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        while self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
            match self.execute(stmt.body.clone()) {
                Err(LoxResult::Break) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }

        Ok(())
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_this_expr(&self, wrapper: Rc<Expr>, expr: &ThisExpr) -> Result<Object, LoxResult> {
        self.look_up_variable(&expr.keyword, wrapper)
    }

    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<Object, LoxResult> {
        let callee = self.evaluate(expr.callee.clone())?;

        let mut arguments = Vec::new();
        for argument in expr.arguments.clone() {
            arguments.push(self.evaluate(argument)?);
        }

        let (callfunc, klass): (Option<Rc<dyn LoxCallable>>, Option<Rc<LoxClass>>) = match callee {
            Object::Function(f) => (Some(f), None),
            Object::Native(n) => (Some(n.func.clone()), None),
            Object::Class(c) => {
                let klass = Rc::clone(&c);
                (Some(c), Some(klass))
            }
            _ => (None, None),
        };

        if let Some(callfunc) = callfunc {
            if arguments.len() != callfunc.arity() {
                return Err(LoxResult::runtime_error(
                        &expr.paren,
                        &format!("Expected {} arguments but got {}.", callfunc.arity(), arguments.len()),
                ))
            };
            callfunc.call(self, arguments, klass)
        } else {
            Err(LoxResult::runtime_error(
                    &expr.paren,
                    "Can only call functions and classes.",
            ))
        }
    }

    fn visit_get_expr(&self, _: Rc<Expr>, expr: &GetExpr) -> Result<Object, LoxResult> {
        let object = self.evaluate(expr.object.clone())?;
        if let Object::Instance(instance) = object {
            Ok(instance.get(&expr.name, &instance)?)
        } else {
            Err(LoxResult::runtime_error(
                    &expr.name,
                    "Only instances have properties.",
            ))
        }
    }

    fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr) -> Result<Object, LoxResult> {
        let value = self.evaluate(expr.value.clone())?;
        if let Some(distance) = self.locals.borrow().get(&wrapper) {
            self.environment
                .borrow()
                .borrow_mut()
                .assign_at(*distance, &expr.name, value.clone())?;
            } else {
                self.globals.borrow_mut().assign(&expr.name, value.clone())?
        }

        Ok(value)
    }

    fn visit_literal_expr(&self, _: Rc<Expr>, expr: &LiteralExpr) -> Result<Object, LoxResult> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;

        if expr.operator.token_type() == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else if !self.is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(expr.right.clone())
    }

    fn visit_set_expr(&self, _: Rc<Expr>, expr: &SetExpr) -> Result<Object, LoxResult> {
        let object = self.evaluate(expr.object.clone())?;

        if let Object::Instance(instance) = object {
            let value = self.evaluate(expr.value.clone())?;
            instance.set(&expr.name, value.clone());
            Ok(value)
        } else {
            Err(LoxResult::runtime_error(
                    &expr.name,
                    "Only instances have fields."
            ))
        }
    }

    fn visit_super_expr(&self, wrapper: Rc<Expr>, expr: &SuperExpr) -> Result<Object, LoxResult> {
        let distance = *self.locals.borrow().get(&wrapper).unwrap();
        let superclass = if let Some(sc) = self.environment.borrow().borrow().get_at(distance, "super").ok() {
            if let Object::Class(superclass) = sc {
                superclass
            } else {
                panic!("Can't find superclass.");
            } 
        } else {
            panic!("Can't find superclass.");
        };
        let object = self.environment.borrow().borrow().get_at(distance - 1, "this").ok().unwrap();
        if let Some(method) = superclass.find_method(expr.method.as_string()) {
            if let Object::Function(func) = method {
                Ok(func.bind(&object))
            } else {
                panic!("Method not a function.");
            }
        } else {
            Err(LoxResult::runtime_error(&expr.method, &format!("Undefined property '{}'.", expr.method.as_string())))
        }
    }

    fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<Object, LoxResult> {
        self.evaluate(expr.expression.clone())
    }

    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;
        let right = self.evaluate(expr.right.clone())?;

        let result = match expr.operator.token_type() {
            TokenType::Minus => left - right,
            TokenType::Slash => left / right,
            TokenType::Star => left * right,
            TokenType::Plus => left + right,
            TokenType::Greater => Object::compare(left, expr.operator.clone(), right),
            TokenType::GreaterEqual => Object::compare(left, expr.operator.clone(), right),
            TokenType::Less => Object::compare(left, expr.operator.clone(), right),
            TokenType::LessEqual => Object::compare(left, expr.operator.clone(), right),
            TokenType::BangEqual => match self.is_equal(&left, &right) {
                Ok(b) => Object::Bool(!b),
                Err(e) => e,
            },
            TokenType::EqualEqual => match self.is_equal(&left, &right) {
                Ok(b) => Object::Bool(b),
                Err(e) => e,
            },
            _ => Object::ErrorMessage("Invalid operator.".to_string()),
        };

        match result {
            Object::ErrorMessage(s) => Err(LoxResult::runtime_error(&expr.operator, &s)),
            _ => Ok(result),
        }
    }

    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<Object, LoxResult> {
        let right = self.evaluate(expr.right.clone())?;
        let result = match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => Object::Num(-n),
                _ => Object::ErrorMessage("Operand must be number.".to_string()),
            },
            TokenType::Bang => Object::Bool(!self.is_truthy(&right)),
            _ => Object::ErrorMessage("Invalid operator.".to_string()),
        };

        match result {
            Object::ErrorMessage(s) => Err(LoxResult::runtime_error(&expr.operator, &s)),
            _ => Ok(result),
        }
    }

    fn visit_variable_expr(&self, wrapper: Rc<Expr>, expr: &VariableExpr) -> Result<Object, LoxResult> {
        self.look_up_variable(&expr.name, wrapper)
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new()));

        globals.borrow_mut().define("clock", Object::Native(Rc::new(LoxNative { func: Rc::new(NativeClock {}) })));

        Interpreter {
            globals: Rc::clone(&globals),
            environment: RefCell::new(Rc::clone(&globals)),
            locals: RefCell::new(HashMap::new()),
        }
    }

    fn evaluate(&self, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        expr.accept(expr.clone(), self)
    }

    fn execute(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self)
    }

    pub fn resolve(&self, expr: Rc<Expr>, depth: usize) -> Result<(), LoxResult> {
        self.locals.borrow_mut().insert(expr, depth);
        Ok(())
    }

    pub fn execute_block(
        &self,
        statements: &Rc<Vec<Rc<Stmt>>>,
        environment: Environment,
    ) -> Result<(), LoxResult> {
        let previous = self.environment.replace(Rc::new(RefCell::new(environment)));
        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement.clone()));
        self.environment.replace(previous);

        result
    }

    fn is_truthy(&self, object: &Object) -> bool {
        // False/Nil are false, anything else is true
        !matches!(object, Object::Nil | Object::Bool(false))
    }

    fn is_equal(&self, left: &Object, right: &Object) -> Result<bool, Object> {
        // Nil is only equal to itself, otherwise equality requires same type
        match (left, right) {
            (Object::Nil, Object::Nil) => Ok(true),
            (Object::Nil, _) => Ok(false),
            (_, Object::Nil) => Ok(false),
            (Object::Num(x), Object::Num(y)) => Ok(x == y),
            (Object::Str(x), Object::Str(y)) => Ok(x == y),
            (Object::Bool(x), Object::Bool(y)) => Ok(x == y),
            _ => Err(Object::ErrorMessage(
                    "Cannot compare objects of different types.".to_string(),
            )),
        }
    }

    fn look_up_variable(&self, name: &Token, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        if let Some(distance) = self.locals.borrow().get(&expr) {
            self.environment.borrow().borrow().get_at(*distance, name.as_string())
        } else { 
            self.globals.borrow().get(name)
        }
    }

    pub fn interpret(&self, statements: &[Rc<Stmt>]) -> bool {
        let mut success = true;
        for stmt in statements {
            if self.execute(stmt.clone()).is_err() {
                success = false;
                break;
            }
        }
        success
    }
}
