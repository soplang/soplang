//! Runtime values for Soplang.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use crate::error::SoplangError;

/// Opaque native or compiled function handle in its owning runtime context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionId(pub usize);

/// Built-in function type (Phase 5 stdlib).
pub type NativeFn = fn(Vec<Value>) -> Result<Value, SoplangError>;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Rc<RefCell<Vec<Value>>>),
    Tuple(Rc<super::TupleData>),
    Set(Rc<super::SetData>),
    Bytes(Rc<RefCell<Vec<u8>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
    Function(FunctionId),
    NativeFunction(NativeFn),
    /// Context-owned closure data; host references preserve captures while its owner lives.
    Closure(Rc<super::abi::ClosureData>),
    Error(Rc<SoplangError>),
    Iterator(Rc<super::IteratorData>),
    Task(Rc<super::tasks::TaskData>),
    Channel(Rc<super::channels::ChannelData>),
    Null,
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NativeFunction(_) => write!(f, "NativeFunction(<hawl>)"),
            _ => std::fmt::Debug::fmt(&self.to_debug_enum(), f),
        }
    }
}

impl Value {
    fn to_debug_enum(&self) -> ValueDebug<'_> {
        match self {
            Value::Int(n) => ValueDebug::Int(*n),
            Value::Float(x) => ValueDebug::Float(*x),
            Value::Str(s) => ValueDebug::Str(s.as_str()),
            Value::Bool(b) => ValueDebug::Bool(*b),
            Value::List(l) => ValueDebug::List(l.borrow().len()),
            Value::Tuple(tuple) => ValueDebug::Tuple(tuple.len()),
            Value::Set(set) => ValueDebug::Set(set.len()),
            Value::Bytes(bytes) => ValueDebug::Bytes(bytes.try_borrow().map(|b| b.len()).ok()),
            Value::Object(o) => ValueDebug::Object(o.borrow().len()),
            Value::Function(id) => ValueDebug::Function(*id),
            Value::NativeFunction(_) => ValueDebug::NativeFn,
            Value::Closure(data) => ValueDebug::Closure(data.handle()),
            Value::Error(error) => ValueDebug::Error(error.message()),
            Value::Iterator(data) => ValueDebug::Iterator(data.handle()),
            Value::Task(data) => ValueDebug::Task(data.handle()),
            Value::Channel(data) => ValueDebug::Channel(data.handle()),
            Value::Null => ValueDebug::Null,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)] // fields used by Debug output
enum ValueDebug<'a> {
    Int(i64),
    Float(f64),
    Str(&'a str),
    Bool(bool),
    List(usize),
    Tuple(usize),
    Set(usize),
    Bytes(Option<usize>),
    Object(usize),
    Function(FunctionId),
    NativeFn,
    Closure(i64),
    Error(&'a str),
    Iterator(i64),
    Task(i64),
    Channel(i64),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        value_eq(self, other, &mut HashSet::new())
    }
}

fn value_eq(a: &Value, b: &Value, seen: &mut HashSet<(u8, usize, usize)>) -> bool {
    let mut pending = vec![(a.clone(), b.clone())];
    while let Some((a, b)) = pending.pop() {
        let equal = match (&a, &b) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => {
                if Rc::ptr_eq(a, b)
                    || !seen.insert((0, Rc::as_ptr(a) as usize, Rc::as_ptr(b) as usize))
                {
                    continue;
                }
                let (Ok(a), Ok(b)) = (a.try_borrow(), b.try_borrow()) else {
                    return false;
                };
                if a.len() != b.len() {
                    return false;
                }
                pending.extend(a.iter().cloned().zip(b.iter().cloned()));
                true
            }
            (Value::Object(a), Value::Object(b)) => {
                if Rc::ptr_eq(a, b)
                    || !seen.insert((1, Rc::as_ptr(a) as usize, Rc::as_ptr(b) as usize))
                {
                    continue;
                }
                let (Ok(a), Ok(b)) = (a.try_borrow(), b.try_borrow()) else {
                    return false;
                };
                if a.len() != b.len() {
                    return false;
                }
                for (key, value) in a.iter() {
                    let Some(other) = b.get(key) else {
                        return false;
                    };
                    pending.push((value.clone(), other.clone()));
                }
                true
            }
            (Value::Tuple(a), Value::Tuple(b)) => {
                if Rc::ptr_eq(a, b)
                    || !seen.insert((2, Rc::as_ptr(a) as usize, Rc::as_ptr(b) as usize))
                {
                    continue;
                }
                let a = a.values();
                let b = b.values();
                if a.len() != b.len() {
                    return false;
                }
                pending.extend(a.iter().cloned().zip(b.iter().cloned()));
                true
            }
            (Value::Set(a), Value::Set(b)) => Rc::ptr_eq(a, b) || a.equals(b),
            (Value::Bytes(a), Value::Bytes(b)) => {
                Rc::ptr_eq(a, b)
                    || match (a.try_borrow(), b.try_borrow()) {
                        (Ok(a), Ok(b)) => *a == *b,
                        _ => false,
                    }
            }
            (Value::Iterator(a), Value::Iterator(b)) => Rc::ptr_eq(a, b),
            (Value::Task(a), Value::Task(b)) => Rc::ptr_eq(a, b),
            (Value::Channel(a), Value::Channel(b)) => Rc::ptr_eq(a, b),
            (Value::Error(a), Value::Error(b)) => Rc::ptr_eq(a, b),
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Closure(a), Value::Closure(b)) => Rc::ptr_eq(a, b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => {
                std::ptr::eq(*a as *const (), *b as *const ())
            }
            _ => false,
        };
        if !equal {
            return false;
        }
    }
    true
}

impl Value {
    /// Somali type name for error messages (nooc). Phase 4+.
    #[allow(dead_code)]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "abn",
            Value::Float(_) => "jajab",
            Value::Str(_) => "qoraal",
            Value::Bool(_) => "bool",
            Value::List(_) => "teed",
            Value::Tuple(_) => "xirmo",
            Value::Set(_) => "urur",
            Value::Bytes(_) => "baytyo",
            Value::Object(_) => "walax",
            Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_) => "hawl",
            Value::Error(_) => "khalad",
            Value::Iterator(_) => "wareeg",
            Value::Task(_) => "shaqo",
            Value::Channel(_) => "marin",
            Value::Null => "maran",
        }
    }

    /// Truthiness for conditions and short-circuit.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(x) => *x != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Tuple(tuple) => !tuple.is_empty(),
            Value::Set(set) => !set.is_empty(),
            Value::Bytes(bytes) => bytes.try_borrow().map(|b| !b.is_empty()).unwrap_or(true),
            Value::Object(o) => !o.borrow().is_empty(),
            Value::Function(_)
            | Value::NativeFunction(_)
            | Value::Closure(_)
            | Value::Error(_)
            | Value::Iterator(_)
            | Value::Task(_)
            | Value::Channel(_) => true,
        }
    }
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hawl#{}", self.0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_value(self, &mut HashSet::new(), false))
    }
}

/// String conversion for stdlib and qor.
pub fn value_to_string(v: &Value) -> String {
    format_value(v, &mut HashSet::new(), false)
}

pub(super) fn ordering_text(value: &Value) -> String {
    format_value(value, &mut HashSet::new(), true)
}

fn format_value(v: &Value, seen: &mut HashSet<(u8, usize)>, opaque_functions: bool) -> String {
    // Display is bounded independently of graph depth; equality and GC walk
    // complete graphs iteratively, including immutable tuple chains.
    if seen.len() >= 64
        && matches!(
            v,
            Value::List(_) | Value::Object(_) | Value::Tuple(_) | Value::Set(_)
        )
    {
        return "...".into();
    }
    match v {
        Value::Bool(true) => "run".to_string(),
        Value::Bool(false) => "been".to_string(),
        Value::Null => "maran".to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(x) => x.to_string(),
        Value::Str(s) => s.clone(),
        Value::Error(error) => error.message().to_owned(),
        Value::Iterator(_) => "<wareeg>".into(),
        Value::Task(_) => "<shaqo>".into(),
        Value::Channel(_) => "<marin>".into(),
        Value::Bytes(bytes) => match bytes.try_borrow() {
            Ok(bytes) => format!("baytyo({:?})", &*bytes),
            Err(_) => "<baytyo>".into(),
        },
        Value::Set(set) => {
            let parts: Vec<_> = set
                .values()
                .iter()
                .map(|value| format_value(value, seen, opaque_functions))
                .collect();
            format!("urur({})", parts.join(", "))
        }
        Value::Tuple(tuple) => {
            let id = (2, Rc::as_ptr(tuple) as usize);
            if !seen.insert(id) {
                return "xirmo(...)".into();
            }
            let parts: Vec<_> = tuple
                .values()
                .iter()
                .map(|value| format_value(value, seen, opaque_functions))
                .collect();
            seen.remove(&id);
            format!("xirmo({})", parts.join(", "))
        }
        Value::List(lst) => {
            let id = (0, Rc::as_ptr(lst) as usize);
            if !seen.insert(id) {
                return "[...]".to_string();
            }
            let v = lst.borrow();
            let parts: Vec<String> = v
                .iter()
                .map(|value| format_value(value, seen, opaque_functions))
                .collect();
            seen.remove(&id);
            format!("[{}]", parts.join(", "))
        }
        Value::Object(obj) => {
            let id = (1, Rc::as_ptr(obj) as usize);
            if !seen.insert(id) {
                return "{...}".to_string();
            }
            let m = obj.borrow();
            let mut pairs: Vec<_> = m
                .iter()
                .map(|(k, val)| format!("{}: {}", k, format_value(val, seen, opaque_functions)))
                .collect();
            pairs.sort();
            seen.remove(&id);
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Function(_) if opaque_functions => "<hawl>".into(),
        Value::Function(id) => format!("{}", id),
        Value::NativeFunction(_) | Value::Closure(_) => "<hawl>".to_string(),
    }
}
