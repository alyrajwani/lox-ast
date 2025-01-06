use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::native_functions::*;
use crate::lox_function::*;
use crate::stmt::*;
use crate::token::*;
use crate::token_type::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: RefCell<Rc<RefCell<Environment>>>,
    nest_level: RefCell<usize>,
}

impl StmtVisitor<()> for Interpreter {
    fn visit_function_stmt(&self, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        let function = LoxFunction::new(&Rc::new(stmt));
        self.environment.borrow().borrow_mut().define(stmt.name.as_string(), Object::Function(Callable { func: Rc::new(function) }));
        Ok(())
    }

    fn visit_break_stmt(&self, stmt: &BreakStmt) -> Result<(), LoxResult> {
        if *self.nest_level.borrow() == 0 {
            Err(LoxResult::runtime_error(
                &stmt.token,
                "Cannot break outside scope of loop.",
            ))
        } else {
            Err(LoxResult::Break)
        }
    }

    fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), LoxResult> {
        let e = Environment::new_with_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)
    }

    fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }

    fn visit_if_stmt(&self, stmt: &IfStmt) -> Result<(), LoxResult> {
        if self.is_truthy(&self.evaluate(&stmt.condition)?) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }

        Ok(())
    }

    fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<(), LoxResult> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{value}");
        Ok(())
    }

    fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), LoxResult> {
        let value = if let Some(initializer) = &stmt.initializer {
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

    fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result<(), LoxResult> {
        *self.nest_level.borrow_mut() += 1;
        while self.is_truthy(&self.evaluate(&stmt.condition)?) {
            match self.execute(&stmt.body) {
                Err(LoxResult::Break) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }

        *self.nest_level.borrow_mut() -= 1;

        Ok(())
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_call_expr(&self, expr: &CallExpr) -> Result<Object, LoxResult> {
        let callee = self.evaluate(&expr.callee)?;

        let mut arguments = Vec::new();
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument)?);
        }

        if let Object::Function(function) = callee {
            if arguments.len() != function.func.arity() {
                return Err(LoxResult::runtime_error(
                    &expr.paren,
                    &format!("Expected {} arguments but got {}.", function.func.arity(), arguments.len()),
                ));
            }
            function.func.call(self, arguments)
        } else {
            Err(LoxResult::runtime_error(
                &expr.paren,
                "Can only call functions and classes.",
            ))
        }
    }

    fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<Object, LoxResult> {
        let value = self.evaluate(&expr.value)?;
        self.environment
            .borrow()
            .borrow_mut()
            .assign(&expr.name, value.clone())?;
        Ok(value)
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Object, LoxResult> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(&expr.left)?;

        if expr.operator.token_type() == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }

        self.evaluate(&expr.right)
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxResult> {
        self.evaluate(&expr.expression)
    }

    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

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

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxResult> {
        let right = self.evaluate(&expr.right)?;
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

    fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<Object, LoxResult> {
        self.environment.borrow().borrow().get(&expr.name)
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new()));

        globals.borrow_mut().define("clock", Object::Function(Callable {
            func: Rc::new(NativeClock {} ),
        }));

        Interpreter {
            globals: Rc::clone(&globals),
            environment: RefCell::new(Rc::clone(&globals)),
            nest_level: RefCell::new(0),
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object, LoxResult> {
        expr.accept(self)
    }

    fn execute(&self, stmt: &Stmt) -> Result<(), LoxResult> {
        stmt.accept(self)
    }

    pub fn execute_block(
        &self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), LoxResult> {
        let previous = self.environment.replace(Rc::new(RefCell::new(environment)));
        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));
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

    pub fn interpret(&self, statements: &[Stmt]) -> bool {
        let mut success = false;
        *self.nest_level.borrow_mut() = 0;
        for stmt in statements {
            if let Err(_) = self.execute(stmt) {
                success = false;
                break;
            }
        }
        success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_literal(o: Object) -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr { value: Some(o) }))
    }

    fn make_literal_string(s: &str) -> Box<Expr> {
        make_literal(Object::Str(s.to_string()))
    }

    #[test]
    fn test_unary_minus() {
        let terp = Interpreter::new();
        let unary_expr = UnaryExpr {
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Num(123.0)),
        };
        let result = terp.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(-123.0)));
    }

    #[test]
    fn test_unary_not() {
        let terp = Interpreter::new();
        let unary_expr = UnaryExpr {
            operator: Token::new(TokenType::Bang, "!".to_string(), None, 123),
            right: make_literal(Object::Bool(false)),
        };
        let result = terp.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_subtraction() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(8.0)));
    }

    #[test]
    fn test_multiplication() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Star, "*".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(105.0)));
    }

    #[test]
    fn test_division() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(21.0)),
            operator: Token::new(TokenType::Slash, "/".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(3.0)));
    }

    #[test]
    fn test_addition() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(21.0)),
            operator: Token::new(TokenType::Plus, "+".to_string(), None, 123),
            right: make_literal(Object::Num(7.0)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Num(28.0)));
    }

    #[test]
    fn test_string_concatination() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal_string("hello, "),
            operator: Token::new(TokenType::Plus, "+".to_string(), None, 123),
            right: make_literal_string("world!"),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Str("hello, world!".to_string())));
    }

    #[test]
    fn test_arithmetic_error_for_subtration() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 123),
            right: make_literal(Object::Bool(true)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_arithmetic_error_for_greater() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Num(15.0)),
            operator: Token::new(TokenType::Greater, ">".to_string(), None, 123),
            right: make_literal(Object::Bool(true)),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_err());
    }

    fn run_comparison_test(tok: &Token, cmps: Vec<bool>) {
        let nums = vec![14.0, 15.0, 16.0];
        let terp = Interpreter::new();

        for (c, nums) in cmps.iter().zip(nums) {
            let binary_expr = BinaryExpr {
                left: make_literal(Object::Num(nums)),
                operator: tok.duplicate(),
                right: make_literal(Object::Num(15.0)),
            };
            let result = terp.visit_binary_expr(&binary_expr);
            assert!(result.is_ok());
            assert_eq!(
                result.ok(),
                Some(Object::Bool(*c)),
                "Testing {} {} 15.0",
                nums,
                tok.as_string()
            );
        }
    }

    #[test]
    fn test_less_than() {
        run_comparison_test(
            &Token::new(TokenType::Less, "<".to_string(), None, 123),
            vec![true, false, false],
        );
    }

    #[test]
    fn test_less_than_or_equal_to() {
        run_comparison_test(
            &Token::new(TokenType::LessEqual, "<=".to_string(), None, 123),
            vec![true, true, false],
        );
    }

    #[test]
    fn test_greater_than() {
        run_comparison_test(
            &Token::new(TokenType::Greater, ">".to_string(), None, 123),
            vec![false, false, true],
        );
    }

    #[test]
    fn test_greater_than_or_equal_to() {
        run_comparison_test(
            &Token::new(TokenType::GreaterEqual, ">=".to_string(), None, 123),
            vec![false, true, true],
        );
    }

    #[test]
    fn test_equals_nums() {
        run_comparison_test(
            &Token::new(TokenType::EqualEqual, "==".to_string(), None, 123),
            vec![false, true, false],
        );
    }

    #[test]
    fn test_not_equals_nums() {
        run_comparison_test(
            &Token::new(TokenType::BangEqual, "!=".to_string(), None, 123),
            vec![true, false, true],
        );
    }

    #[test]
    fn test_not_equals_string() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal_string("hello"),
            operator: Token::new(TokenType::EqualEqual, "==".to_string(), None, 123),
            right: make_literal_string("hellx"),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    #[test]
    fn test_equals_string() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal_string("world"),
            operator: Token::new(TokenType::EqualEqual, "==".to_string(), None, 123),
            right: make_literal_string("world"),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_equals_nil() {
        let terp = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Nil),
            operator: Token::new(TokenType::EqualEqual, "==".to_string(), None, 123),
            right: make_literal(Object::Nil),
        };
        let result = terp.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }
}
