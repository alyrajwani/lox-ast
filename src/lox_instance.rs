use std::fmt;
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

    pub fn get(&self, name: &Token, this: &Rc<LoxInstance>) -> Result<Object, LoxResult> {
        if let Entry::Occupied(o) = self.fields.borrow_mut().entry(name.as_string().into()) {
            Ok(o.get().clone())
        } else if let Some(method) = self.klass.find_method(name.as_string()) { 
            if let Object::Function(func) = method {
                return Ok(func.bind(&Object::Instance(Rc::clone(this))));
            } else {
                panic!("Tried to bind 'this' incorrectly.")
            }
        } else {
            Err(LoxResult::runtime_error(name, &format!("Undefined property '{}'.", name.as_string())))
        }
    }

    pub fn set(&self, name: &Token, value: Object) {
        self.fields.borrow_mut().insert(name.as_string().into(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fields = Vec::new();

        for (k, v) in self.fields.borrow().iter() {
            fields.push(format!("{}={}", k, v))
        }

        write!(f, "<Instance of {} with fields {{ {} }}>",
            self.klass,
            fields.join(", ")
        )
    }
}

