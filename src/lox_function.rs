use crate::interpreter::*;
use crate::environment::*;
use crate::token::*;
use crate::callable::*;
use crate::error::*;
use crate::stmt::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct LoxFunction {
    name: Token, 
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: &FunctionStmt, closure: &Rc<RefCell<Environment>>) -> LoxFunction {
        LoxFunction { 
            name: declaration.name.duplicate(),
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
            closure: Rc::clone(closure),
        } 
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {

            let mut environment = Environment::new_with_enclosing(Rc::clone(&self.closure));

            for (param, arg) in self.params.iter().zip(arguments.iter()) {
                environment.define(param.as_string(), arg.clone());
            }
            
            match interpreter.execute_block(&self.body, environment) {
                Err(LoxResult::Return { value }) => Ok(value),
                Err(e) => Err(e),
                Ok(_) => Ok(Object::Nil),
            }
    }

    fn arity(&self) -> usize {
            self.params.len()
    }

    fn to_string(&self) -> String {        
            self.name.as_string().to_string()
    }
}
