//! Scoped environment for variables.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::error::{runtime_error, SoplangError};
use crate::frontend::ast::TypeAnnotation;
use crate::runtime::value::Value;

pub struct Env {
    vars:   HashMap<String, Value>,
    types:  HashMap<String, TypeAnnotation>,
    consts: HashSet<String>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars:   HashMap::new(),
            types:  HashMap::new(),
            consts: HashSet::new(),
            parent: None,
        }
    }

    #[allow(dead_code)] // Phase 4: function scope
    pub fn new_child(parent: Rc<RefCell<Env>>) -> Self {
        Self {
            vars:   HashMap::new(),
            types:  HashMap::new(),
            consts: HashSet::new(),
            parent: Some(parent),
        }
    }

    pub fn define(
        &mut self,
        name: &str,
        value: Value,
        type_ann: TypeAnnotation,
        is_const: bool,
    ) {
        self.vars.insert(name.to_string(), value);
        if type_ann != TypeAnnotation::Dynamic {
            self.types.insert(name.to_string(), type_ann);
        }
        if is_const {
            self.consts.insert(name.to_string());
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|p| p.borrow().get(name))
        })
    }

    pub fn assign(&mut self, name: &str, value: Value, line: usize, col: usize) -> Result<(), SoplangError> {
        if self.consts.contains(name) {
            return Err(runtime_error(
                format!("Ma bedeli kartid qiimaha doorsamaha madoor '{}'", name),
                line,
                col,
            ));
        }
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(ref p) = self.parent {
            return p.borrow_mut().assign(name, value, line, col);
        }
        Err(runtime_error(
            format!("Doorsame aan la qeexin: '{}'", name),
            line,
            col,
        ))
    }

    #[allow(dead_code)] // Phase 4: type check on assign
    pub fn get_type(&self, name: &str) -> Option<TypeAnnotation> {
        self.types.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|p| p.borrow().get_type(name))
        })
    }

    #[allow(dead_code)] // Phase 4: reassignment check
    pub fn is_const(&self, name: &str) -> bool {
        self.consts.contains(name)
            || self
                .parent
                .as_ref()
                .map(|p| p.borrow().is_const(name))
                .unwrap_or(false)
    }
}
