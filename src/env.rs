use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::anyhow;
use crate::ast::*;
use crate::ErrorOrCtxJmp;
use crate::Object;
use crate::Result;

#[derive(Debug)]
pub struct EnvInner {
    values: HashMap<Identifier, Rc<RefCell<Object>>>,
    pub enclosing: Option<Rc<RefCell<EnvInner>>>,
}

impl EnvInner {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn detach_env(enclosing: Rc<RefCell<EnvInner>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    fn _get(env: &EnvInner, id: &Identifier) -> Result<Rc<RefCell<Object>>> {
        match env.values.get(id) {
            Some(val) => Ok(Rc::clone(val)),
            None => match &env.enclosing {
                Some(enclosing) => EnvInner::_get(&enclosing.borrow(), id),
                None => return Err(ErrorOrCtxJmp::Error(anyhow!("undefined variable {}", id))),
            },
        }
    }

    pub fn get(&self, id: &Identifier) -> Result<Rc<RefCell<Object>>> {
        EnvInner::_get(&self, id)
    }

    pub fn insert(&mut self, id: Identifier, o: Object) -> Option<Rc<RefCell<Object>>> {
        self.values.insert(id, Rc::new(RefCell::new(o)))
    }
}
