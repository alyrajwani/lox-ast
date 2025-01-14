use std::rc::Rc;
use std::cell::RefCell;
use std::collections::hash_map::*;
use crate::lox_class::*;
use crate::token::*;
use crate::error::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    pub klass: Rc<LoxClass>,
    fields: RefCell<HashMap<String, Object>>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> LoxInstance {
        LoxInstance { klass: Rc::clone(&klass), fields: RefCell::new(HashMap::new()) }
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Entry::Occupied(o) = self.fields.borrow_mut().entry(name.as_string().into()) {
            Ok(o.get().clone())
        } else {
            Err(LoxResult::runtime_error(name, &format!("Undefined property '{}'.", name.as_string().to_string())))
        }
    }
}

impl std::string::ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("Instance of {}", self.klass.to_string())
    }
}
