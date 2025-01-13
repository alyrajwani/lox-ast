use crate::interpreter::*;
use crate::error::*;
use crate::callable::*;
use crate::token::*;
use crate::lox_instance::*;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: &String) -> LoxClass {
        LoxClass { name: name.clone()  }
    }

    pub fn instantiate(&self, _interpreter: &Interpreter, _arguments: Vec<Object>, klass: Rc<LoxClass>) -> Result<Object, LoxResult> {
        Ok(Object::Instance(LoxInstance::new(klass)))
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, _interpreter: &Interpreter, _arguments: Vec<Object>) -> Result<Object, LoxResult> {
        Err(LoxResult::system_error("Tried to call a class."))
    }

    fn arity(&self) -> usize {
        0
    } 

    fn to_string(&self) -> String {
        self.name.clone()
    }
}

