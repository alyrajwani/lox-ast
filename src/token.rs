use crate::error::*;
use crate::interpreter::*;
use crate::token_type::*;
use std::cmp::*;
use std::fmt;
use std::ops::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Num(f64),
    Str(String),
    Bool(bool),
    Function(LoxCallable),
    Nil,
    ErrorMessage(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Num(n) => write!(f, "{n}"),
            Object::Str(s) => write!(f, "{s}"),
            Object::Bool(b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Object::Function(_) => write!(f, "<func>"),
            Object::Nil => write!(f, "nil"),
            Object::ErrorMessage(_) => panic!("Do not print upon error."),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoxCallable;

impl LoxCallable {
    pub fn call(&self, _terp: &Interpreter, _arguments: Vec<Object>) -> Result<Object, LoxResult> {
        Ok(Object::Nil)
    }
}

impl Sub for Object {
    type Output = Object;

    fn sub(self, other: Self) -> Object {
        match (self, other) {
            (Object::Num(left), Object::Num(right)) => Object::Num(left - right),
            _ => Object::ErrorMessage("Operands must be numbers.".to_string()),
        }
    }
}

impl Div for Object {
    type Output = Object;

    fn div(self, other: Self) -> Object {
        match (self, other) {
            (Object::Num(left), Object::Num(right)) => {
                if right == 0.0 {
                    Object::ErrorMessage("Cannot divide by zero.".to_string())
                } else {
                    Object::Num(left / right)
                }
            }
            _ => Object::ErrorMessage("Operands must be numbers.".to_string()),
        }
    }
}

impl Mul for Object {
    type Output = Object;

    fn mul(self, other: Self) -> Object {
        match (self, other) {
            (Object::Num(left), Object::Num(right)) => Object::Num(left * right),
            (Object::Str(s), Object::Num(n)) => Object::Str(s.repeat(n as usize)),
            _ => Object::ErrorMessage(
                "Operands must be numbers or a string and a number.".to_string(),
            ),
        }
    }
}

impl Add for Object {
    type Output = Object;

    fn add(self, other: Self) -> Object {
        match (self, other) {
            (Object::Num(left), Object::Num(right)) => Object::Num(left + right),
            (Object::Str(left), Object::Str(right)) => Object::Str(format!("{}{}", left, right)),
            (Object::Str(left), Object::Num(right)) => Object::Str(format!("{}{}", left, right)),
            (Object::Num(left), Object::Str(right)) => Object::Str(format!("{}{}", left, right)),
            _ => Object::ErrorMessage("Operants must be numbers or strings.".to_string()),
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Object::Nil, o) => {
                if o == &Object::Nil {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            (Object::Num(left), Object::Num(right)) => left.partial_cmp(right),
            _ => None,
        }
    }
}

impl Object {
    pub fn compare(left: Object, operator: Token, right: Object) -> Object {
        if !Self::are_num_objects(left.clone(), right.clone()) {
            Object::ErrorMessage("Operands must be numbers.".to_string())
        } else {
            let first = Self::deconstruct_num_object(left).unwrap();
            let second = Self::deconstruct_num_object(right).unwrap();
            match operator.ttype {
                TokenType::Greater => Object::Bool(first > second),
                TokenType::GreaterEqual => Object::Bool(first >= second),
                TokenType::Less => Object::Bool(first < second),
                TokenType::LessEqual => Object::Bool(first <= second),
                _ => Object::ErrorMessage("Invalid comparator".to_string()),
            }
        }
    }

    fn are_num_objects(left: Object, right: Object) -> bool {
        match (left, right) {
            (Object::Num(_), Object::Num(_)) => true,
            (_, _) => false,
        }
    }

    fn deconstruct_num_object(obj: Object) -> Option<f64> {
        match obj {
            Object::Num(n) => Some(n),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
    pub literal: Option<Object>,
    pub line: usize,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Option<Object>, line: usize) -> Token {
        Token {
            ttype,
            lexeme,
            literal,
            line,
        }
    }

    pub fn is(&self, ttype: TokenType) -> bool {
        self.ttype == ttype
    }

    pub fn token_type(&self) -> TokenType {
        self.ttype
    }

    pub fn as_string(&self) -> &String {
        &self.lexeme
    }

    pub fn duplicate(&self) -> Token {
        Token {
            ttype: self.ttype,
            lexeme: self.lexeme.to_string(),
            literal: self.literal.clone(),
            line: self.line,
        }
    }

    pub fn eof(line: usize) -> Token {
        Token {
            ttype: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line,
        }
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
