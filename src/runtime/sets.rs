//! Insertion-ordered hash sets. Keys contain no mutable runtime values.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Null,
    Bool(bool),
    Int(i64),
    Float(u64),
    Str(String),
    Tuple(Vec<Key>),
}
impl Key {
    pub(super) fn of(value: &Value, depth: usize) -> Result<Self, SoplangError> {
        Ok(match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Bool(*value),
            Value::Int(value) => Self::Int(*value),
            Value::Float(value) if !value.is_nan() => {
                Self::Float(if *value == 0.0 { 0 } else { value.to_bits() })
            }
            Value::Float(_) => return Err(error("Qiimaha NaN looma adeegsan karo xubin urur.")),
            Value::Str(value) => Self::Str(value.clone()),
            Value::Tuple(value) => {
                if depth >= 64 {
                    return Err(error(
                        "Xirmada ururka lagu darayo waxay leedahay lakabyo aad u badan.",
                    ));
                }
                Self::Tuple(
                    value
                        .values()
                        .iter()
                        .map(|value| Self::of(value, depth + 1))
                        .collect::<Result<_, _>>()?,
                )
            }
            _ => return Err(error(format!(
                "Xubinta ururku waa inay noqotaa qiime aan la beddeli karin; waxaa la helay '{}'.",
                value.type_name()
            ))),
        })
    }
}

#[derive(Clone, Default)]
pub(super) struct State {
    slots: Vec<Option<(Rc<Key>, Value)>>,
    index: HashMap<Rc<Key>, usize>,
}
impl State {
    fn insert(&mut self, key: Key, value: Value) {
        if self.index.contains_key(&key) {
            return;
        }
        let key = Rc::new(key);
        self.index.insert(Rc::clone(&key), self.slots.len());
        self.slots.push(Some((key, value)));
    }
    fn remove(&mut self, key: &Key) -> Option<Value> {
        let index = self.index.remove(key)?;
        let (_, value) = self.slots[index].take()?;
        // Bound tombstones without changing surviving insertion order.
        if self.slots.len() > self.index.len().max(16).saturating_mul(2) {
            self.slots.retain(Option::is_some);
            self.index.clear();
            for (index, slot) in self.slots.iter().enumerate() {
                self.index
                    .insert(Rc::clone(&slot.as_ref().unwrap().0), index);
            }
        }
        Some(value)
    }
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.slots.iter().flatten().map(|(_, value)| value)
    }
}

/// Set contents can only be changed through checked runtime methods.
#[derive(Default)]
pub struct SetData {
    pub(super) state: RefCell<State>,
}
impl SetData {
    pub fn new(values: Vec<Value>) -> Result<Rc<Self>, SoplangError> {
        let mut state = State::default();
        for value in values {
            state.insert(Key::of(&value, 0)?, value);
        }
        Ok(Rc::new(Self {
            state: RefCell::new(state),
        }))
    }
    pub fn len(&self) -> usize {
        self.state.borrow().index.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn values(&self) -> Vec<Value> {
        self.state.borrow().values().cloned().collect()
    }
    pub(super) fn children(&self) -> Option<Vec<Value>> {
        self.state
            .try_borrow_mut()
            .ok()
            .map(|state| state.values().cloned().collect())
    }
    pub(super) fn clear(&self) {
        *self.state.borrow_mut() = State::default();
    }
    pub(super) fn equals(&self, other: &Self) -> bool {
        let left = self.state.borrow();
        let right = other.state.borrow();
        left.index.len() == right.index.len()
            && left.index.keys().all(|key| right.index.contains_key(key))
    }
}

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E041_SET),
    )
}
pub(super) fn not_indexed() -> SoplangError {
    error("Ururku ma laha tusmo; isticmaal leeyahay(), geli(), ama saar().")
}
pub(super) fn construct(values: Vec<Value>) -> Result<Value, SoplangError> {
    SetData::new(values).map(Value::Set)
}
pub(super) fn from_iterable(source: &Value) -> Result<Value, SoplangError> {
    let iterator = super::iterators::begin(source)?;
    let mut state = State::default();
    while let Some(value) = super::iterators::next_value(&iterator)? {
        state.insert(Key::of(&value, 0)?, value);
    }
    Ok(Value::Set(Rc::new(SetData {
        state: RefCell::new(state),
    })))
}
pub(super) fn dispatch(
    set: &Rc<SetData>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    use super::stdlib::expect_arity;
    match method {
        "geli" => {
            expect_arity(args, method, 1)?;
            let key = Key::of(&args[0], 0)?;
            super::contracts::mutate_set(set, |state| {
                state.insert(key, args[0].clone());
                Ok(())
            })?;
            Ok(Value::Set(Rc::clone(set)))
        }
        "saar" => {
            expect_arity(args, method, 1)?;
            let key = Key::of(&args[0], 0)?;
            super::contracts::mutate_set(set, |state| Ok(state.remove(&key).unwrap_or(Value::Null)))
        }
        "leeyahay" => {
            expect_arity(args, method, 1)?;
            Ok(Value::Bool(
                set.state
                    .borrow()
                    .index
                    .contains_key(&Key::of(&args[0], 0)?),
            ))
        }
        "dherer" => {
            expect_arity(args, method, 0)?;
            Ok(Value::Int(set.len() as i64))
        }
        "nadiifi" => {
            expect_arity(args, method, 0)?;
            super::contracts::mutate_set(set, |state| {
                *state = State::default();
                Ok(())
            })?;
            Ok(Value::Set(Rc::clone(set)))
        }
        "nuqul" | "urur" => {
            expect_arity(args, method, 0)?;
            construct(set.values())
        }
        "teed" => {
            expect_arity(args, method, 0)?;
            Ok(Value::List(Rc::new(RefCell::new(set.values()))))
        }
        "xirmo" => {
            expect_arity(args, method, 0)?;
            super::tuples::construct(set.values())
        }
        "midee" | "wadaag" | "reeb" | "kaladuwan" | "hoosaad" => {
            expect_arity(args, method, 1)?;
            let Value::Set(other) = &args[0] else {
                return Err(error(format!("{}() waxay u baahan tahay urur.", method)));
            };
            let left = set.state.borrow();
            let right = other.state.borrow();
            if method == "hoosaad" {
                return Ok(Value::Bool(
                    left.index.keys().all(|key| right.index.contains_key(key)),
                ));
            }
            let mut result = State::default();
            for (key, value) in left.slots.iter().flatten() {
                let contains = right.index.contains_key(key);
                if method == "midee"
                    || (method == "wadaag" && contains)
                    || (method != "wadaag" && !contains)
                {
                    result.insert((**key).clone(), value.clone());
                }
            }
            if method == "midee" || method == "kaladuwan" {
                for (key, value) in right.slots.iter().flatten() {
                    if method == "midee" || !left.index.contains_key(key) {
                        result.insert((**key).clone(), value.clone());
                    }
                }
            }
            Ok(Value::Set(Rc::new(SetData {
                state: RefCell::new(result),
            })))
        }
        _ => Err(error(format!(
            "Ururku ma laha hawl la yiraahdo '{}'.",
            method
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tombstone_compaction_preserves_order_and_hash_membership() {
        let mut state = State::default();
        let mut expected = Vec::new();
        let mut random = 7u64;
        for step in 0..20000 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = ((random >> 32) % 200) as i64;
            if random & 1 == 0 {
                state.insert(Key::Int(value), Value::Int(value));
                if !expected.contains(&Value::Int(value)) {
                    expected.push(Value::Int(value));
                }
            } else {
                let index = expected.iter().position(|item| *item == Value::Int(value));
                assert_eq!(
                    state.remove(&Key::Int(value)),
                    index.map(|index| expected.remove(index))
                );
            }
            assert!(state.slots.len() <= state.index.len().max(16) * 2);
            if step % 67 == 0 {
                assert_eq!(state.values().cloned().collect::<Vec<_>>(), expected);
                for value in 0..200 {
                    assert_eq!(
                        state.index.contains_key(&Key::Int(value)),
                        expected.contains(&Value::Int(value))
                    );
                }
            }
        }
        for value in 0..200 {
            state.remove(&Key::Int(value));
        }
        assert!(state.index.is_empty());
        assert!(state.slots.len() <= 32);
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn borrowed_set_state_rejects_writes_without_blocking_unrelated_sets() {
        let context = super::super::RuntimeContext::new();
        context.with(|| {
            let data = SetData::new(vec![Value::Int(1)]).unwrap();
            super::super::contracts::check(Value::Set(Rc::clone(&data)), "urur[abn]", "x").unwrap();
            let borrowed = data.state.borrow();
            assert_eq!(
                dispatch(&data, "geli", &[Value::Int(2)])
                    .unwrap_err()
                    .code(),
                Some("E039")
            );
            assert_eq!(
                borrowed.values().cloned().collect::<Vec<_>>(),
                [Value::Int(1)]
            );
            drop(borrowed);
            let borrowed = data.state.borrow_mut();
            assert_eq!(
                dispatch(&data, "nadiifi", &[]).unwrap_err().code(),
                Some("E039")
            );
            let other = SetData::new(vec![]).unwrap();
            dispatch(&other, "geli", &[Value::Int(2)]).unwrap();
            assert_eq!(other.values(), [Value::Int(2)]);
            drop(borrowed);
            dispatch(&data, "geli", &[Value::Int(2)]).unwrap();
        });
    }
}
