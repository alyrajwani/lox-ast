use crate::interpreter::*;
use crate::error::*;
use crate::callable::*;
use crate::token::*;
use crate::lox_instance::*;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Object>
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Object>) -> LoxClass {
        LoxClass { name: name.to_owned(), methods }
    }

    pub fn instantiate(&self, _interpreter: &Interpreter, _arguments: Vec<Object>, klass: Rc<LoxClass>) -> Result<Object, LoxResult> {
        Ok(Object::Instance(Rc::new(LoxInstance::new(klass))))
    }

    pub fn find_method(&self, name: &str) -> Option<Object> {
        self.methods.get(name).cloned()
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, _interpreter: &Interpreter, _arguments: Vec<Object>) -> Result<Object, LoxResult> {
        Err(LoxResult::system_error("Tried to call a class."))
    }

    fn arity(&self) -> usize {
        0
    } 
}

impl std::string::ToString for LoxClass {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
