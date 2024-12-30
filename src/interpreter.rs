use crate::expr::*;
use crate::token::*;
use crate::error::*;
use crate::token_type::*;

pub struct Interpreter;

impl ExprVisitor<Object> for Interpreter {
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Object, LoxError> {
        Ok(expr.value.clone().unwrap())
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxError> {
        Ok(self.evaluate(&expr.expression)?)
    }

    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        let result = match expr.operator.token_type() {
            TokenType::Minus => left - right,
            TokenType::Slash => left / right,
            TokenType::Star  => left * right,
            TokenType::Plus  => left + right,
            TokenType::Greater => Object::compare(left, expr.operator.clone(), right),
            TokenType::GreaterEqual => Object::compare(left, expr.operator.clone(), right),
            TokenType::Less => Object::compare(left, expr.operator.clone(), right),
            TokenType::LessEqual => Object::compare(left, expr.operator.clone(), right),
            TokenType::BangEqual => match self.is_equal(&left, &right) {
                    Ok(b) => Object::Bool(!b),
                    Err(_) => Object::ClassCastException,
                }
            TokenType::EqualEqual => match self.is_equal(&left, &right) {
                    Ok(b) => Object::Bool(b),
                    Err(_) => Object::ClassCastException,
                }
            _ => Object::ClassCastException,
        };

        if result == Object::ClassCastException {
            Err(LoxError::runtime_error(&expr.operator, "Illegal expression.".to_string()))
        } else {
            Ok(result)
        }
    }
    
    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxError> {
        let right = self.evaluate(&expr.right)?;
        let result = match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => Object::Num(-n),
                _ => Object::ClassCastException
            },
            TokenType::Bang => {
                Object::Bool(!self.is_truthy(&right))
            }, 
            _ => Object::ClassCastException,
        };

        if result == Object::ClassCastException {
            Err(LoxError::runtime_error(&expr.operator, "Illegal expression.".to_string()))
        } else {
            Ok(result)
        }
    }
}

impl Interpreter {
    fn evaluate(&self, expr: &Expr) -> Result<Object, LoxError> {
        expr.accept(self)
    }

    fn is_truthy(&self, object: &Object) -> bool {
        // False/Nil are false, anything else is true
        match object {
            Object::Nil => false,
            Object::Bool(false) => false,
            _ => true,
        }
    }

    fn is_equal(&self, left: &Object, right: &Object) -> Result<bool, ()> {
        // Nil is only equal to itself, otherwise equality requires same type
        match (left, right) {
            (Object::Nil, Object::Nil) => Ok(true),
            (Object::Nil, _) => Ok(false),
            (_, Object::Nil) => Ok(false),
            (Object::Num(x), Object::Num(y)) => Ok(x == y),
            (Object::Str(x), Object::Str(y)) => Ok(x == y),
            _ => Err(()),
        }
    }

    pub fn interpret(&self, expr: &Expr) -> bool {
        match self.evaluate(expr) {
            Ok(val) => {
                println!("{val}");
                true
            },
            Err(e) => {
                e.report("".to_string());
                false
            }
        }
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};

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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
        let terp = Interpreter {};
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
