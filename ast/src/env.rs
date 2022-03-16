use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::EnvErrorKind;
use crate::Object;
use crate::Result;

#[derive(Debug)]
pub struct EnvInner {
    values: HashMap<String, Rc<RefCell<Object>>>,
    pub enclosing: Option<Rc<RefCell<EnvInner>>>,
}

impl Default for EnvInner {
    fn default() -> Self {
        Self::new()
    }
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

    fn _get(env: &EnvInner, id: &Identifier, up: usize) -> Result<Rc<RefCell<Object>>> {
        match up {
            0 => match env.values.get(&id.ident) {
                Some(val) => Ok(Rc::clone(val)),
                None => Err(EnvErrorKind::UndefinedVariable(id.clone())),
            },
            _ => match &env.enclosing {
                Some(enclosing) => EnvInner::_get(&enclosing.borrow(), id, up - 1),
                None => Err(EnvErrorKind::NoEnclosingEnv),
            },
        }
    }

    pub fn get(&self, id: &Identifier, up: usize) -> Result<Rc<RefCell<Object>>> {
        EnvInner::_get(self, id, up)
    }

    pub fn insert(&mut self, id: Identifier, o: Object) -> Option<Rc<RefCell<Object>>> {
        self.values.insert(id.ident, Rc::new(RefCell::new(o)))
    }

    pub fn insert_fail_if_present(&mut self, name: Identifier, object: Object) -> Result<()> {
        match self
            .values
            .insert(name.ident.clone(), Rc::new(RefCell::new(object)))
        {
            // TODO: make variable definition with an Option, to not have this workaround.
            Some(tok) if *tok.borrow() != Object::Nil => {
                Err(EnvErrorKind::VariableExists(name))
            }
            _ => Ok(()),
        }
    }
}
