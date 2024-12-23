use crate::token_type::*;
use std::fmt;


#[derive(Debug)]
pub enum Object {
    Num(f64),
    Str(String),
    Nil,
    True, 
    False,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Num(n) => write!(f, "{n}"),
            Object::Str(s) => write!(f, "\"{s}\""),
            Object::Nil => write!(f, "nil"),
            Object::True => write!(f, "true"),
            Object::False => write!(f, "false"),
        }
    }
}


#[derive(Debug)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
    literal: Option<Object>,
    line: usize,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Token {
        Token { ttype, lexeme, literal, line } 
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} {} {}",
            self.ttype,
            self.lexeme,
            if let Some(literal) = &self.literal {
                literal.to_string()
            } else {
                "None".to_string()
            }
        )
    }
}

