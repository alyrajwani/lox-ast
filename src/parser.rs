// imports
use crate::error::*;
use crate::expr::*;
use crate::token::*;
use crate::token_type::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => None,
        }
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.equality()
    } 

    fn equality(&mut self) -> Result<Expr, LoxError> {
        // equality => comparison ( ( != | == ) comparison )*
        let mut expr  = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().duplicate();
            let right = self.comparison()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right), });
        }
        
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        // comparison => term ( ( > | >= | < | <= ) term )*
        let mut expr = self.term()?;
        
        while self.is_match(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous().duplicate();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right), });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        // term => factor ( ( - | + ) factor )*
        let mut expr = self.factor()?;

        while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().duplicate();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right), });
        }
    
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        // factor => unary ( ( * | \ ) unary )*
        let mut expr = self.unary()?;
        
        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().duplicate();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right), });
        }
    
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        // unary => ( - | ! ) unary 
        //       |  primary
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().duplicate();
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right) }));
        }

        Ok(self.primary()?)
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        // primary => NUMBER | STRING | true | false | nil | ( expression ) 
        if self.is_match(&[TokenType::False]) {
            return Ok(Expr::Literal(LiteralExpr { value: Some(Object::False) } ));
        }
        if self.is_match(&[TokenType::True]) {
            return Ok(Expr::Literal(LiteralExpr { value: Some(Object::True) } ));
        }
        if self.is_match(&[TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralExpr { value: Some(Object::Nil) } ));
        }

        if self.is_match(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(LiteralExpr { value: self.previous().literal.clone() } )); 
        }
        if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression".to_string())?;
            return Ok(Expr::Grouping(GroupingExpr { expression: Box::new(expr) } ));
        }

        Err(LoxError::error(0, "Expect expression.".to_string()))
    }

    fn consume(&mut self, ttype: TokenType, message: String) -> Result<Token, LoxError> {
        if self.check(ttype) {
            Ok(self.advance().duplicate())
        } else {
            Err(Parser::error(self.peek(), message))
        }   
    }

    fn is_match(&mut self, types: &[TokenType]) -> bool {
        for &t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&self, ttype: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        } else {
            return self.peek().is(ttype);
        }
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        } 
        return self.previous();
    }   

    fn is_at_end(&self) -> bool {
        return self.peek().is(TokenType::Eof);
    }

    fn peek(&self) -> &Token {
        return self.tokens.get(self.current).unwrap();
    }

    fn previous(&self) -> &Token {
        return self.tokens.get(self.current - 1).unwrap();
    }
    
    fn error(token: &Token, message: String) -> LoxError {
        LoxError::parse_error(token, message)
    }

    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if self.previous().is(TokenType::Semicolon) {
                return;
            }
            
            if matches!(self.peek().token_type(),
                TokenType::Class    | TokenType::Fun      | TokenType::Var      | TokenType::For      |
                TokenType::If       | TokenType::While    | TokenType::Print    | TokenType::Return) {
                    return; 
            }

            self.advance();
        }
    }
}
