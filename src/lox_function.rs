use crate::interpreter::*;
use crate::environment::*;
use crate::token::*;
use crate::callable::*;
use crate::error::*;
use crate::lox_class::*;
use crate::stmt::*;
use std::rc::Rc;
use std::fmt;
use std::cell::RefCell;

pub struct LoxFunction {
    name: Token, 
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> { 
        write!(f, "{}", self)
    }
}

impl Clone for LoxFunction {
    fn clone(&self) -> Self {
        LoxFunction{ 
            name: self.name.duplicate(),
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::clone(&self.closure),
            is_initializer: self.is_initializer,
        }
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &Self) -> bool { 
        self.name.token_type() == other.name.token_type() &&
            Rc::ptr_eq(&self.params, &other.params) &&
            Rc::ptr_eq(&self.body, &other.body) &&
            Rc::ptr_eq(&self.closure, &other.closure)
    }
}

impl LoxFunction {
    pub fn new(declaration: &FunctionStmt, closure: &Rc<RefCell<Environment>>, is_initializer: bool) -> LoxFunction {
        LoxFunction { 
            name: declaration.name.duplicate(),
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
            closure: Rc::clone(closure),
            is_initializer,
        } 
    }

    pub fn bind(&self, instance: &Object) -> Object {
        let environment = RefCell::new(Environment::new_with_enclosing(Rc::clone(&self.closure)));
        environment.borrow_mut().define("this", instance.clone());
        Object::Function(Rc::new(LoxFunction {
            name: self.name.duplicate(),
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::new(environment),
            is_initializer: self.is_initializer,
        }))
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>, _: Option<Rc<LoxClass>>) -> Result<Object, LoxResult> {

        let mut environment = Environment::new_with_enclosing(Rc::clone(&self.closure));

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            environment.define(param.as_string(), arg.clone());
        }

        match interpreter.execute_block(&self.body, environment) {
            Err(LoxResult::Return { value }) => { 
                if self.is_initializer {
                    self.closure.borrow().get_at(0, "this") 
                } else {
                    Ok(value)
                }
            }
            Err(e) => Err(e),
            Ok(_) => if self.is_initializer { 
                self.closure.borrow().get_at(0, "this") 
            } else { 
                Ok(Object::Nil) 
            },
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let paramlist = self.params.iter().map(|p| p.as_string().into()).collect::<Vec<String>>().join(", ");
        write!(f, "<Function {}({})>", self.name.as_string(), paramlist)
    }
}
