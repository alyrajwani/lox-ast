use crate::token_type::*;
use crate::token::*;

#[derive(Debug)]
pub struct LoxError {
    token: Option<Token>,
    line: usize,
    message: String,
}

impl LoxError {
    pub fn error(line: usize, message: &str) -> LoxError {
        // scanning error; tokens don't exist at this point
        let err = LoxError { token: None, line, message: message.to_string() };
        err        
    }   

    pub fn parse_error(token: &Token, message: &str) -> LoxError {
        // parsing error; cite the incorrect token in error message
        let err = LoxError { token: Some(token.duplicate()), line: token.line, message: message.to_string() };
        err
    }

    pub fn runtime_error(token: &Token, message: &str) -> LoxError {
        // runtime error; cite in correct expression in error message
        let err = LoxError { token: Some(token.duplicate()), line: token.line, message: message.to_string() };
        err
    }

    pub fn report(&self, loc: String) {
        // print the appropriate error message
        if let Some(token) = &self.token {
            if token.is(TokenType::Eof) {
                eprintln!("[line {}] at end: {}", token.line, self.message);
            } else {
                eprintln!("[line {}] at '{}': {}", token.line, token.as_string(), self.message);
            }
        } else {
            eprintln!("[line {}] Error{}: {}", self.line, loc, self.message);
        }
    }
}
