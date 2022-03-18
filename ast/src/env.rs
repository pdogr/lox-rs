use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::EnvErrorKind;
use crate::Object;
use crate::Result;

#[derive(Debug)]
pub struct EnvInner {
    pub(crate) values: HashMap<String, Rc<RefCell<Object>>>,
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

    pub fn init_variable(&mut self, id: Identifier, o: Object) {
        self.values
            .insert(id.token.lexeme, Rc::new(RefCell::new(o)));
    }

    pub(crate) fn _get(env: &EnvInner, id: &Identifier, up: usize) -> Result<Rc<RefCell<Object>>> {
        match up {
            0 => match env.values.get(&id.token.lexeme) {
                Some(o) => Ok(Rc::clone(o)),
                None => Err(EnvErrorKind::UndefinedVariable(id.clone())),
            },
            _ => match &env.enclosing {
                Some(ref enclosing) => EnvInner::_get(&enclosing.borrow(), id, up - 1),
                None => Err(EnvErrorKind::NoEnclosingEnv),
            },
        }
    }
}
