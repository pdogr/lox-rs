use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::EnvErrorKind;
use crate::Object;
use crate::Result;

#[derive(Debug)]
pub struct EnvInner {
    pub(crate) values: HashMap<String, Option<Rc<RefCell<Object>>>>,
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

    pub fn declare_variable(&mut self, id: Identifier) -> Result<()> {
        if self.values.get(&id.ident).is_some() {
            return Err(EnvErrorKind::VariableExists(id));
        }
        self.values.insert(id.ident, None);
        Ok(())
    }

    pub fn declare_init_variable(&mut self, id: Identifier, o: Object) -> Result<()> {
        if self.values.get(&id.ident).is_some() {
            return Err(EnvErrorKind::VariableExists(id));
        }
        self.values.insert(id.ident, Some(Rc::new(RefCell::new(o))));
        Ok(())
    }

    pub fn init_variable(&mut self, id: Identifier, o: Object) {
        self.values.insert(id.ident, Some(Rc::new(RefCell::new(o))));
    }

    pub fn contains_variable(&self, id: &Identifier) -> bool {
        self.values.contains_key(&id.ident)
    }

    pub(crate) fn _get_env(
        env: Rc<RefCell<EnvInner>>,
        id: &Identifier,
        up: usize,
    ) -> Result<Rc<RefCell<EnvInner>>> {
        match up {
            0 => match env.borrow().values.get(&id.ident) {
                Some(_) => Ok(Rc::clone(&env)),
                None => Err(EnvErrorKind::UndefinedVariable(id.clone())),
            },
            _ => match &env.borrow().enclosing {
                Some(ref enclosing) => EnvInner::_get_env(Rc::clone(enclosing), id, up - 1),
                None => Err(EnvErrorKind::NoEnclosingEnv),
            },
        }
    }
}
