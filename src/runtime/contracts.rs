//! Shared collection contracts. Guards retain weak roots and index the mutable
//! identities read by validation; writes validate candidate data before commit.
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::{Rc, Weak};

use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|s| s);
type List = Rc<RefCell<Vec<Value>>>;
type Object = Rc<RefCell<HashMap<String, Value>>>;
type Key = (u8, usize);

fn error(message: impl Into<String>, mutation: bool) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(if mutation {
            codes::E039_COLLECTION_CONTRACT
        } else {
            codes::E022_TYPE_MISMATCH
        }),
    )
}
fn invalid() -> SoplangError {
    error("Qaabka noocyadu ma saxna.", false)
}

#[derive(Clone, PartialEq, Eq)]
enum Contract {
    Int,
    Float,
    Str,
    Bool,
    List,
    Tuple,
    Set,
    SetOf(Box<Contract>),
    TupleOf(Vec<Contract>),
    Object,
    Bytes,
    Function,
    Null,
    Class(String),
    ListOf(Box<Contract>),
    ObjectOf(Box<Contract>),
    Union(Vec<Contract>),
}
impl Contract {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ListOf(a), Self::ListOf(b))
            | (Self::ObjectOf(a), Self::ObjectOf(b))
            | (Self::SetOf(a), Self::SetOf(b)) => a.same(b),
            (Self::TupleOf(a), Self::TupleOf(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.same(b))
            }
            (Self::Union(a), Self::Union(b)) => {
                a.len() == b.len() && a.iter().all(|ty| b.iter().any(|other| ty.same(other)))
            }
            _ => self == other,
        }
    }
    fn has_elements(&self) -> bool {
        match self {
            Self::ListOf(_) | Self::ObjectOf(_) | Self::TupleOf(_) | Self::SetOf(_) => true,
            Self::Union(types) => types.iter().any(Self::has_elements),
            _ => false,
        }
    }
    fn can_be_set_key(&self) -> bool {
        match self {
            Self::Int | Self::Float | Self::Str | Self::Bool | Self::Null | Self::Tuple => true,
            Self::TupleOf(types) => types.iter().all(Self::can_be_set_key),
            Self::Union(types) => types.iter().any(Self::can_be_set_key),
            _ => false,
        }
    }
    fn permits(&self, scalar: &Self) -> bool {
        self == scalar || matches!(self,Self::Union(types) if types.iter().any(|ty|ty==scalar))
    }
}
impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => f.write_str("abn"),
            Self::Float => f.write_str("jajab"),
            Self::Str => f.write_str("qoraal"),
            Self::Bool => f.write_str("bool"),
            Self::List => f.write_str("teed"),
            Self::Tuple => f.write_str("xirmo"),
            Self::Set => f.write_str("urur"),
            Self::TupleOf(types) => {
                f.write_str("xirmo[")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                f.write_str("]")
            }
            Self::Object => f.write_str("walax"),
            Self::Bytes => f.write_str("baytyo"),
            Self::Function => f.write_str("hawl"),
            Self::Null => f.write_str("maran"),
            Self::Class(name) => f.write_str(name),
            Self::SetOf(inner) => write!(f, "urur[{inner}]"),
            Self::ListOf(inner) => write!(f, "teed[{inner}]"),
            Self::ObjectOf(inner) => write!(f, "walax[qoraal, {inner}]"),
            Self::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        f.write_str(" | ")?;
                    }
                    write!(f, "{ty}")?;
                }
                Ok(())
            }
        }
    }
}

struct Reader<'a> {
    text: &'a str,
    pos: usize,
    nodes: usize,
}
impl Reader<'_> {
    fn skip(&mut self) {
        while self.text[self.pos..].starts_with(char::is_whitespace) {
            self.pos += self.text[self.pos..].chars().next().unwrap().len_utf8();
        }
    }
    fn take(&mut self, ch: char) -> bool {
        self.skip();
        if self.text[self.pos..].starts_with(ch) {
            self.pos += ch.len_utf8();
            true
        } else {
            false
        }
    }
    fn ty(&mut self, depth: usize) -> Result<Contract, SoplangError> {
        if depth >= 32 {
            return Err(invalid());
        }
        let mut types = Vec::new();
        loop {
            self.nodes += 1;
            if self.nodes > 4096 || types.len() >= 64 {
                return Err(invalid());
            }
            self.skip();
            let class = self.take('@');
            let start = self.pos;
            while let Some(ch) = self.text[self.pos..].chars().next() {
                if !(ch.is_alphanumeric() || ch == '_') {
                    break;
                }
                self.pos += ch.len_utf8();
            }
            let name = &self.text[start..self.pos];
            if name.is_empty() {
                return Err(invalid());
            }
            let mut ty = if class {
                Contract::Class(name.into())
            } else {
                match name {
                    "abn" => Contract::Int,
                    "jajab" => Contract::Float,
                    "qoraal" => Contract::Str,
                    "bool" => Contract::Bool,
                    "teed" => Contract::List,
                    "xirmo" => Contract::Tuple,
                    "urur" => Contract::Set,
                    "walax" => Contract::Object,
                    "baytyo" => Contract::Bytes,
                    "hawl" => Contract::Function,
                    "maran" => Contract::Null,
                    _ => return Err(invalid()),
                }
            };
            if self.take('[') {
                if ty == Contract::Tuple {
                    let mut values = Vec::new();
                    self.skip();
                    if !self.text[self.pos..].starts_with(']') {
                        loop {
                            values.push(self.ty(depth + 1)?);
                            if !self.take(',') {
                                break;
                            }
                        }
                    }
                    ty = Contract::TupleOf(values);
                } else {
                    let inner = self.ty(depth + 1)?;
                    ty = match ty {
                        Contract::List => Contract::ListOf(Box::new(inner)),
                        Contract::Set => {
                            if !inner.can_be_set_key() {
                                return Err(invalid());
                            }
                            Contract::SetOf(Box::new(inner))
                        }
                        Contract::Object => {
                            let value = if self.take(',') {
                                if inner != Contract::Str {
                                    return Err(invalid());
                                }
                                self.ty(depth + 1)?
                            } else {
                                inner
                            };
                            Contract::ObjectOf(Box::new(value))
                        }
                        _ => return Err(invalid()),
                    };
                }
                if !self.take(']') {
                    return Err(invalid());
                }
            }
            types.push(ty);
            if !self.take('|') {
                break;
            }
        }
        Ok(if types.len() == 1 {
            types.remove(0)
        } else {
            Contract::Union(types)
        })
    }
}

#[derive(Clone)]
enum WeakRoot {
    List(Weak<RefCell<Vec<Value>>>),
    Tuple(Weak<super::TupleData>),
    Set(Weak<super::SetData>),
    Object(Weak<RefCell<HashMap<String, Value>>>),
}
impl WeakRoot {
    fn from_value(value: &Value) -> Option<(Key, Self)> {
        match value {
            Value::List(value) => Some((
                (5, Rc::as_ptr(value) as usize),
                Self::List(Rc::downgrade(value)),
            )),
            Value::Set(value) => Some((
                (14, Rc::as_ptr(value) as usize),
                Self::Set(Rc::downgrade(value)),
            )),
            Value::Tuple(value) => Some((
                (13, Rc::as_ptr(value) as usize),
                Self::Tuple(Rc::downgrade(value)),
            )),
            Value::Object(value) => Some((
                (6, Rc::as_ptr(value) as usize),
                Self::Object(Rc::downgrade(value)),
            )),
            _ => None,
        }
    }
    fn upgrade(&self) -> Option<Value> {
        match self {
            Self::List(value) => value.upgrade().map(Value::List),
            Self::Tuple(value) => value.upgrade().map(Value::Tuple),
            Self::Set(value) => value.upgrade().map(Value::Set),
            Self::Object(value) => value.upgrade().map(Value::Object),
        }
    }
}
struct Guard {
    root: WeakRoot,
    checks: Vec<Rc<Contract>>,
    dependencies: HashSet<Key>,
}
#[derive(Default)]
pub(super) struct ContractStore {
    cache: HashMap<String, Rc<Contract>>,
    roots: HashMap<Key, Guard>,
    dependents: HashMap<Key, HashSet<Key>>,
}
impl ContractStore {
    pub fn len(&self) -> usize {
        self.roots.len()
    }
    fn schema(&mut self, text: &str) -> Result<Rc<Contract>, SoplangError> {
        if let Some(schema) = self.cache.get(text) {
            return Ok(Rc::clone(schema));
        }
        let mut reader = Reader {
            text,
            pos: 0,
            nodes: 0,
        };
        let schema = Rc::new(reader.ty(0)?);
        reader.skip();
        if reader.pos != text.len() {
            return Err(invalid());
        }
        if self.cache.len() >= 256 {
            self.cache.clear();
        }
        self.cache.insert(text.into(), Rc::clone(&schema));
        Ok(schema)
    }
    fn unlink(&mut self, root: Key, deps: &HashSet<Key>) {
        for key in deps {
            if let Some(roots) = self.dependents.get_mut(key) {
                roots.remove(&root);
                if roots.is_empty() {
                    self.dependents.remove(key);
                }
            }
        }
    }
    fn dependencies(&mut self, key: Key, dependencies: HashSet<Key>) {
        let old = std::mem::take(&mut self.roots.get_mut(&key).unwrap().dependencies);
        self.unlink(key, &old);
        for dep in &dependencies {
            self.dependents.entry(*dep).or_default().insert(key);
        }
        self.roots.get_mut(&key).unwrap().dependencies = dependencies;
    }
    pub fn prune(&mut self) {
        let dead: Vec<_> = self
            .roots
            .iter()
            .filter(|(_, guard)| guard.root.upgrade().is_none())
            .map(|(key, _)| *key)
            .collect();
        for key in dead {
            let guard = self.roots.remove(&key).unwrap();
            self.unlink(key, &guard.dependencies);
        }
    }
}

enum Candidate {
    Set(Key, super::sets::State),
    List(Key, Vec<Value>),
    Object(Key, HashMap<String, Value>),
}
#[derive(Default)]
struct Validator<'a> {
    candidate: Option<&'a Candidate>,
    mutation: bool,
    dependencies: HashSet<Key>,
    memo: HashMap<(Key, usize), bool>,
}
impl Validator<'_> {
    fn matches(&mut self, value: &Value, ty: &Contract) -> Result<bool, SoplangError> {
        let key = match value {
            Value::List(value) => Some((5, Rc::as_ptr(value) as usize)),
            Value::Tuple(value) => Some((13, Rc::as_ptr(value) as usize)),
            Value::Set(value) => Some((14, Rc::as_ptr(value) as usize)),
            Value::Object(value) => Some((6, Rc::as_ptr(value) as usize)),
            _ => None,
        };
        let memo = key.map(|key| (key, ty as *const Contract as usize));
        if let Some(found) = memo.and_then(|key| self.memo.get(&key)) {
            return Ok(*found);
        }
        let result = match (value, ty) {
            (_, Contract::Union(types)) => {
                let mut found = false;
                for ty in types {
                    if self.matches(value, ty)? {
                        found = true;
                        break;
                    }
                }
                found
            }
            (Value::Tuple(tuple), Contract::TupleOf(types)) => {
                let values = tuple.values();
                if values.len() != types.len() {
                    false
                } else {
                    let mut valid = true;
                    for (value, ty) in values.iter().zip(types) {
                        if !self.matches(value, ty)? {
                            valid = false;
                            break;
                        }
                    }
                    valid
                }
            }
            (Value::Tuple(_), Contract::Tuple) => true,
            (Value::Set(_), Contract::Set) => true,
            (Value::Set(set), Contract::SetOf(inner)) => {
                let key = key.unwrap();
                self.dependencies.insert(key);
                let borrowed;
                let values = match self.candidate {
                    Some(Candidate::Set(target, values)) if *target == key => values,
                    _ => {
                        borrowed = set
                            .state
                            .try_borrow()
                            .map_err(|_| error("Ururka hadda lama akhriyi karo.", self.mutation))?;
                        &*borrowed
                    }
                };
                let mut valid = true;
                for value in values.values() {
                    if !self.matches(value, inner)? {
                        valid = false;
                        break;
                    }
                }
                valid
            }
            (Value::List(list), Contract::ListOf(inner)) => {
                let key = key.unwrap();
                self.dependencies.insert(key);
                let borrowed;
                let values = match self.candidate {
                    Some(Candidate::List(target, values)) if *target == key => values.as_slice(),
                    _ => {
                        borrowed = list
                            .try_borrow()
                            .map_err(|_| error("Teedka hadda lama akhriyi karo.", self.mutation))?;
                        borrowed.as_slice()
                    }
                };
                let mut valid = true;
                for value in values {
                    if !self.matches(value, inner)? {
                        valid = false;
                        break;
                    }
                }
                valid
            }
            (Value::Object(object), Contract::ObjectOf(inner)) => {
                let key = key.unwrap();
                self.dependencies.insert(key);
                let borrowed;
                let values = match self.candidate {
                    Some(Candidate::Object(target, values)) if *target == key => values,
                    _ => {
                        borrowed = object.try_borrow().map_err(|_| {
                            error("Walaxda hadda lama akhriyi karo.", self.mutation)
                        })?;
                        &*borrowed
                    }
                };
                let mut valid = true;
                for value in values.values() {
                    if !self.matches(value, inner)? {
                        valid = false;
                        break;
                    }
                }
                valid
            }
            (Value::Object(object), Contract::Class(expected)) => {
                let key = key.unwrap();
                self.dependencies.insert(key);
                let borrowed;
                let values = match self.candidate {
                    Some(Candidate::Object(target, values)) if *target == key => values,
                    _ => {
                        borrowed = object.try_borrow().map_err(|_| {
                            error("Walaxda hadda lama akhriyi karo.", self.mutation)
                        })?;
                        &*borrowed
                    }
                };
                match (
                    super::abi::get_instance_class(object),
                    values.get("__class__"),
                ) {
                    (Some(class), Some(Value::Str(found))) if class == *found => {
                        super::abi::class_name_matches(&class, expected)
                    }
                    _ => false,
                }
            }
            (
                Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_),
                Contract::Function,
            ) => true,
            (Value::Int(_), Contract::Int)
            | (Value::Float(_), Contract::Float)
            | (Value::Str(_), Contract::Str)
            | (Value::Bool(_), Contract::Bool)
            | (Value::List(_), Contract::List)
            | (Value::Object(_), Contract::Object)
            | (Value::Bytes(_), Contract::Bytes)
            | (Value::Null, Contract::Null) => true,
            _ => false,
        };
        if let Some(key) = memo {
            self.memo.insert(key, result);
        }
        Ok(result)
    }
}

pub(super) fn check(value: Value, descriptor: &str, name: &str) -> Result<Value, SoplangError> {
    let schema = STATE.with(|state| state.contracts.borrow_mut().schema(descriptor))?;
    let mut validator = Validator::default();
    if !validator.matches(&value, &schema)? {
        // Scalar annotations retain existing conversion rules. Mutable element
        // contracts require exact numeric kinds to keep all aliases invariant.
        let converted = match &value {
            Value::Int(_) if schema.permits(&Contract::Float) => {
                Some(super::stdlib::builtin_jajab(vec![value.clone()]))
            }
            Value::Float(number)
                if number.is_finite()
                    && number.fract() == 0.0
                    && schema.permits(&Contract::Int) =>
            {
                Some(super::stdlib::builtin_abn(vec![value.clone()]))
            }
            _ => None,
        };
        if let Some(Ok(converted)) = converted {
            return Ok(converted);
        }
        return Err(error(
            format!(
                "'{}' wuxuu u baahan yahay nooca {}, laakiin waxaa la helay {}.",
                name,
                schema,
                value.type_name()
            ),
            false,
        ));
    }
    if schema.has_elements() {
        if let Some((key, root)) = WeakRoot::from_value(&value) {
            STATE.with(|state| {
                let mut store = state.contracts.borrow_mut();
                store.prune();
                let guard = store.roots.entry(key).or_insert_with(|| Guard {
                    root,
                    checks: Vec::new(),
                    dependencies: HashSet::new(),
                });
                if !guard.checks.iter().any(|ty| ty.same(&schema)) {
                    guard.checks.push(schema);
                }
                let mut deps = guard.dependencies.clone();
                deps.extend(validator.dependencies);
                store.dependencies(key, deps);
            });
        }
    }
    Ok(value)
}

fn affected(key: Key) -> Vec<(Key, WeakRoot, Vec<Rc<Contract>>)> {
    STATE.with(|state| {
        let store = state.contracts.borrow();
        let mut guards: Vec<_> = store
            .dependents
            .get(&key)
            .into_iter()
            .flatten()
            .filter_map(|key| {
                let guard = store.roots.get(key)?;
                Some((*key, guard.root.clone(), guard.checks.clone()))
            })
            .collect();
        // Stable schema order keeps diagnostics independent of allocation
        // addresses and randomized hash iteration in either backend.
        if guards.len() > 1 {
            guards.sort_by_cached_key(|(_, _, checks)| {
                checks
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\0")
            });
        }
        guards
    })
}
fn validate(
    candidate: &Candidate,
    guards: Vec<(Key, WeakRoot, Vec<Rc<Contract>>)>,
) -> Result<Vec<(Key, HashSet<Key>)>, SoplangError> {
    let mut updates = Vec::new();
    for (key, root, checks) in guards {
        let Some(value) = root.upgrade() else {
            continue;
        };
        let mut validator = Validator {
            candidate: Some(candidate),
            mutation: true,
            ..Validator::default()
        };
        for ty in checks {
            if !validator.matches(&value, &ty)? {
                return Err(error(
                    format!("Beddelkani ma waafaqsana nooca ururka {}.", ty),
                    true,
                ));
            }
        }
        updates.push((key, validator.dependencies));
    }
    Ok(updates)
}
fn commit(updates: Vec<(Key, HashSet<Key>)>) {
    STATE.with(|state| {
        let mut store = state.contracts.borrow_mut();
        for (key, deps) in updates {
            store.dependencies(key, deps);
        }
    });
}

/// The operation must only edit candidate contents; it must not invoke user code.
pub(super) fn mutate_list<R>(
    list: &List,
    operation: impl FnOnce(&mut Vec<Value>) -> Result<R, SoplangError>,
) -> Result<R, SoplangError> {
    let key = (5, Rc::as_ptr(list) as usize);
    let guards = affected(key);
    if guards.is_empty() {
        return operation(
            &mut *list
                .try_borrow_mut()
                .map_err(|_| error("Teedka hadda lama beddeli karo.", true))?,
        );
    }
    let mut values = list
        .try_borrow()
        .map_err(|_| error("Teedka hadda lama akhriyi karo.", true))?
        .clone();
    let result = operation(&mut values)?;
    let candidate = Candidate::List(key, values);
    let updates = validate(&candidate, guards)?;
    let Candidate::List(_, values) = candidate else {
        unreachable!()
    };
    *list
        .try_borrow_mut()
        .map_err(|_| error("Teedka hadda lama beddeli karo.", true))? = values;
    commit(updates);
    Ok(result)
}
pub(super) fn mutate_object<R>(
    object: &Object,
    operation: impl FnOnce(&mut HashMap<String, Value>) -> Result<R, SoplangError>,
) -> Result<R, SoplangError> {
    let key = (6, Rc::as_ptr(object) as usize);
    let guards = affected(key);
    if guards.is_empty() {
        return operation(
            &mut *object
                .try_borrow_mut()
                .map_err(|_| error("Walaxda hadda lama beddeli karo.", true))?,
        );
    }
    let mut values = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo.", true))?
        .clone();
    let result = operation(&mut values)?;
    let candidate = Candidate::Object(key, values);
    let updates = validate(&candidate, guards)?;
    let Candidate::Object(_, values) = candidate else {
        unreachable!()
    };
    *object
        .try_borrow_mut()
        .map_err(|_| error("Walaxda hadda lama beddeli karo.", true))? = values;
    commit(updates);
    Ok(result)
}

/// Apply set mutations only after all shared contracts accept the candidate.
pub(super) fn mutate_set<R>(
    set: &Rc<super::SetData>,
    operation: impl FnOnce(&mut super::sets::State) -> Result<R, SoplangError>,
) -> Result<R, SoplangError> {
    let key = (14, Rc::as_ptr(set) as usize);
    let guards = affected(key);
    if guards.is_empty() {
        return operation(
            &mut *set
                .state
                .try_borrow_mut()
                .map_err(|_| error("Ururka hadda lama beddeli karo.", true))?,
        );
    }
    let mut values = set
        .state
        .try_borrow()
        .map_err(|_| error("Ururka hadda lama akhriyi karo.", true))?
        .clone();
    let result = operation(&mut values)?;
    let candidate = Candidate::Set(key, values);
    let updates = validate(&candidate, guards)?;
    let Candidate::Set(_, values) = candidate else {
        unreachable!()
    };
    *set.state
        .try_borrow_mut()
        .map_err(|_| error("Ururka hadda lama beddeli karo.", true))? = values;
    commit(updates);
    Ok(result)
}
