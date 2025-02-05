use crate::interpreter::*;
use crate::error::*;
use crate::callable::*;
use crate::token::*;
use crate::lox_instance::*;
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    name: String,
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, Object>
}

impl LoxClass {
    pub fn new(name: &str, superclass: Option<Rc<LoxClass>>, methods: HashMap<String, Object>) -> LoxClass {
        LoxClass { name: name.to_owned(), superclass, methods }
    }

    pub fn instantiate(&self, interpreter: &Interpreter, arguments: Vec<Object>, klass: Rc<LoxClass>) -> Result<Object, LoxResult> {
        let instance = Object::Instance(Rc::new(LoxInstance::new(klass)));
        if let Some(Object::Function(initializer)) = self.find_method("init") {
            if let Object::Function(init) = initializer.bind(&instance) {
                init.call(interpreter, arguments, None)?;
            }
        }
        Ok(instance)
    }

    pub fn find_method(&self, name: &str) -> Option<Object> {
        if let Some(method) = self.methods.get(name) {
            Some(method.clone()) 
        } else if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
        }
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>, klass: Option<Rc<LoxClass>>) -> Result<Object, LoxResult> {
        self.instantiate(interpreter, arguments, klass.unwrap())
    }

    fn arity(&self) -> usize {
        if let Some(Object::Function(initializer)) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let methods = self
            .methods
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "<Class {} {{ {methods} }}>", self.name)
    }
}
