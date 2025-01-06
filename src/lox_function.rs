use crate::interpreter::*;
use crate::environment::*;
use crate::token::*;
use crate::callable::*;
use crate::error::*;
use crate::stmt::*;
use std::rc::Rc;

pub struct LoxFunction {
    declaration: Rc<FunctionStmt>, 
}

impl LoxFunction {
    pub fn new(declaration: &Rc<FunctionStmt>) -> LoxFunction {
        LoxFunction { declaration: Rc::clone(declaration) } 
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {

            let mut environment = Environment::new_with_enclosing(Rc::clone(&interpreter.globals));

            for (param, arg) in self.declaration.params.iter().zip(arguments.iter()) {
                environment.define(param.as_string(), arg.clone());
            }

            interpreter.execute_block(&self.declaration.body, environment)?;

            Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
            self.declaration.params.len()
    }

    fn to_string(&self) -> String {        
            self.declaration.name.as_string().to_string()
    }
}
