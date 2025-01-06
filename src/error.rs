use crate::token::*;
use crate::token_type::*;

pub enum LoxResult {
    LoxParseError { token: Token, message: String },
    LoxRuntimeError { token: Token, message: String },
    LoxError { line: usize, message: String },
    LoxSystemError { message: String},
    Break,
}

impl LoxResult {
    pub fn error(line: usize, message: &str) -> LoxResult {
        // scanning error; tokens don't exist at this point
        let e = LoxResult::LoxError {
            line,
            message: message.to_string(),
        };
        e.report("");
        e
    }

    pub fn parse_error(token: &Token, message: &str) -> LoxResult {
        // parsing error; cite the incorrect token in error message
        let e = LoxResult::LoxParseError {
            token: token.duplicate(),
            message: message.to_string(),
        };
        e.report("");
        e
    }

    pub fn runtime_error(token: &Token, message: &str) -> LoxResult {
        // runtime error; cite in correct expression in error message
        let e = LoxResult::LoxRuntimeError {
            token: token.duplicate(),
            message: message.to_string(),
        };
        e.report("");
        e
    }

    pub fn system_error(message: &str) -> LoxResult {
        let e = LoxResult::LoxSystemError {
            message: message.to_string(),
        };
        e.report("");
        e
    }

    fn report(&self, loc: &str) {
        // print the appropriate error message
        match self {
            LoxResult::LoxParseError { token, message }
            | LoxResult::LoxRuntimeError { token, message } => {
                if token.is(TokenType::Eof) {
                    eprintln!("[line {}] at end: {}", token.line, message);
                } else {
                    eprintln!(
                        "[line {}] at '{}': {}",
                        token.line,
                        token.as_string(),
                        message
                    );
                }
            }
            LoxResult::LoxError { line, message } => {
                eprintln!("[line {}] Error{}: {}", *line, loc, message);
            }
            LoxResult::LoxSystemError { message } => {
                eprintln!("System Error: {message}.")
            }
            LoxResult::Break => {}
        };
    }
}
