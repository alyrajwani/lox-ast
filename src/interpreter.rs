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
            TokenType::Greater => Object::Bool(left > right),
            TokenType::GreaterEqual => Object::Bool(left >= right),
            TokenType::Less => Object::Bool(left < right),
            TokenType::LessEqual => Object::Bool(left <= right),
            TokenType::BangEqual => Object::Bool(left != right),
            TokenType::EqualEqual => Object::Bool(left == right),
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
        match expr.operator.token_type() {
            TokenType::Minus => match right {
                Object::Num(n) => return Ok(Object::Num(-n)),
                _ => Ok(Object::Nil)
            },
            TokenType::Bang => {
                return Ok(Object::Bool(!self.is_truthy(&right)));
            }, 
            _ => Err(LoxError::error(expr.operator.line, "Unreachable.".to_string()))
        
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
}


