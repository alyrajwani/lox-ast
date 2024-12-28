// imports 
use crate::token::*;
use crate::token_type::*;
use crate::error::*;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
} 

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner { 
            source: source.chars().collect(), 
            tokens: Vec::new(),
            start: 0,
            current: 0, 
            line: 1,
        }
    }
    
    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
        let mut had_error: Option<LoxError> = None;

        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => {},
                Err(e) => {
                    e.report("".to_string());
                    had_error = Some(e);
                }    
            }
        }

        self.tokens.push(Token::eof(self.line));
    
        match had_error {
            Some(e) => Err(e),
            None => Ok(&self.tokens),
        }
    }

    fn is_at_end(&self) -> bool {
        !self.peek().is_some()        
    } 

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();        
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let tok = if self.is_match('=') {
                    TokenType::BangEqual 
                } else { 
                    TokenType::Bang
                };
                self.add_token(tok);
            }
            '=' => {
                let tok = if self.is_match('=') {
                    TokenType::EqualEqual 
                } else {
                    TokenType::Equal
                };
                self.add_token(tok);
            }
            '<' => {
                let tok = if self.is_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less 
                };
                self.add_token(tok);
            }
            '>' => {
                let tok = if self.is_match('=') {
                    TokenType::GreaterEqual 
                } else {
                    TokenType::Greater
                };
                self.add_token(tok);
            }
            '/' => {
                if self.is_match('/') {
                    // A comment goes until the end of the line.
                    while let Some(ch) = self.peek() {
                        if ch != '\n' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                } else if self.is_match('*') {
                   self.block_comment()?; 
                } else {
                   self.add_token(TokenType::Slash);
                };
            }
            ' ' | '\t' | '\r' => {},
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string()?;
            }
            '0'..='9' => {
                self.number();
            }
            _ => {
                if Scanner::is_alpha(Some(c)) {
                    self.identifier();
                } else {
                    return Err(LoxError::error(
                        self.line,
                        "Unexpected character.".to_string()
                    ));
                };   
            }
        }
        Ok(())
    }
    
    fn advance(&mut self) -> char {
        let result = *self.source.get(self.current).unwrap();
        self.current += 1;
        result
    }

    fn add_token(&mut self, ttype: TokenType) {
        self.add_token_object(ttype, None);
    }

    fn add_token_object(&mut self, ttype: TokenType, literal: Option<Object>) { 
        let text: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(ttype, text, literal, self.line));   
    }

    fn is_match(&mut self, expected: char) -> bool {
        match self.source.get(self.current) {
            Some(ch) if *ch == expected => {
                self.current += 1;
                true
            }
            _ => false,
        }
    }

    fn is_digit(ch: Option<char>) -> bool {
        match ch {
            Some(c) => {
                c >= '0' && c <= '9'
            }
            None => false
        }
    }    

    fn is_alpha(ch: Option<char>) -> bool {
        match ch {
            Some(c) => {
                (c >= 'a' && c <= 'z') ||
                (c >= 'A' && c <= 'Z') ||
                (c == '_')
            }
            None => false
        } 
    }
        
    fn is_alpha_numeric(ch: Option<char>) -> bool {
        return Scanner::is_digit(ch) || Scanner::is_alpha(ch)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).copied()
    } 
    
    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).copied()
    }

    fn keyword(check: &str) -> Option<TokenType> {
        match check {
            "and"       => Some(TokenType::And),
            "class"     => Some(TokenType::Class),
            "else"      => Some(TokenType::Else),
            "false"     => Some(TokenType::False),
            "for"       => Some(TokenType::For),
            "fun"       => Some(TokenType::Fun),
            "if"        => Some(TokenType::If),
            "nil"       => Some(TokenType::Nil),
            "or"        => Some(TokenType::Or),
            "print"     => Some(TokenType::Print),
            "return"    => Some(TokenType::Return),
            "super"     => Some(TokenType::Super),
            "this"      => Some(TokenType::This),
            "true"      => Some(TokenType::True),
            "var"       => Some(TokenType::Var),
            "while"     => Some(TokenType::While),
            _           => None,
        }
    }
    
    fn string(&mut self) -> Result<(), LoxError> {
        while let Some(ch) = self.peek()  {
            match ch {
                '"' => {
                    break;
                }
                '\n' => {
                    self.line += 1;
                }
                _ => {}
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(LoxError::error(
                self.line,
                "Unterminated string.".to_string()
            ));
        }
        self.advance();
       
        // Ignore quotes when making string object 
        let value: String = self.source[self.start + 1 .. self.current - 1].iter().collect();
        self.add_token_object(TokenType::String, Some(Object::Str(value)));
        
        Ok(())
    }
    
    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }
        
        if self.peek() == Some('.') && Scanner::is_digit(self.peek_next()) {
            self.advance();
                    
            while Scanner::is_digit(self.peek()) {
                self.advance(); 
            }
        }

        // Parse string type into f64 when making num object
        let value: String = self.source[self.start .. self.current].iter().collect();
        let num: f64 = value.parse::<f64>().unwrap();
        self.add_token_object(TokenType::Number, Some(Object::Num(num)));
    }
    
    fn identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }
        
        let text: String = self.source[self.start .. self.current].iter().collect();
        if let Some(ttype) = Scanner::keyword(text.as_str()) {
            self.add_token(ttype)
        } else {
            self.add_token(TokenType::Identifier)
        }
    }
    
    fn block_comment(&mut self) -> Result<(), LoxError> {
        let mut nest_count: u8 = 1; 
        while let Some(ch) = self.peek() {
            match ch {
                '\n' => {
                    self.line += 1;
                }
                '/' if self.peek_next() == Some('*') => {
                    nest_count += 1;
                    self.advance();
                }
                '*' if self.peek_next() == Some('/') => {
                    nest_count -= 1;
                    if nest_count == 0 {
                        break;
                    }
                    self.advance();
                }
                _ => {}
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(LoxError::error(
                self.line,
                "Unterminated block comment.".to_string()
            ));
        }
        self.advance();
        self.advance();
        Ok(())
    }
}
