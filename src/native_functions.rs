use std::time::SystemTime;
use std::fmt;
use crate::interpreter::*;
use crate::error::*;
use crate::token::*;
use crate::callable::*;
use crate::lox_class::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct LoxNative {
    pub func: Rc<dyn LoxCallable>,
}

impl fmt::Debug for LoxNative {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Native Function>")
    }
}   

impl fmt::Display for LoxNative {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Native Function>")
    }
}

impl PartialEq for LoxNative {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.func, &other.func)
    }
}

pub struct NativeClock;

impl LoxCallable for NativeClock {
    fn call(&self, _: &Interpreter, _: Vec<Object>, _: Option<Rc<LoxClass>>) -> Result<Object, LoxResult> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => Ok(Object::Num(n.as_millis() as f64)),
            Err(e) => Err(LoxResult::system_error(
                &format!("Clock returned invalid duration: {:?}.", e.duration())
            ))
        }
    }

    fn arity(&self) -> usize {
        0
    }
}
