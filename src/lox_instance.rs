use std::rc::Rc;
use crate::lox_class::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    pub klass: Rc<LoxClass>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> LoxInstance {
        LoxInstance { klass: Rc::clone(&klass) }
    }
}