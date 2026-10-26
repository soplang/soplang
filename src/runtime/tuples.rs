//! Fixed sequences. Only the collector can clear unreachable tuple edges.
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

pub struct TupleData {
    values: RefCell<Vec<Value>>,
}
impl TupleData {
    pub fn new(values: Vec<Value>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(values),
        })
    }
    pub fn values(&self) -> Ref<'_, Vec<Value>> {
        self.values.borrow()
    }
    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }
    pub(super) fn children(&self) -> Option<Vec<Value>> {
        self.values
            .try_borrow_mut()
            .ok()
            .map(|values| values.clone())
    }
    pub(super) fn clear(&self) {
        if let Ok(mut values) = self.values.try_borrow_mut() {
            values.clear();
        }
    }
}
fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E040_TUPLE),
    )
}
pub(super) fn immutable() -> SoplangError {
    error("Xirmada lama beddeli karo.")
}
fn value(values: Vec<Value>) -> Value {
    Value::Tuple(TupleData::new(values))
}
pub(super) fn construct(args: Vec<Value>) -> Result<Value, SoplangError> {
    Ok(value(args))
}
pub(super) fn from_list(list: &Rc<RefCell<Vec<Value>>>) -> Result<Value, SoplangError> {
    Ok(value(
        list.try_borrow()
            .map_err(|_| error("Teedka hadda lama akhriyi karo."))?
            .clone(),
    ))
}
fn position(index: &Value, len: usize) -> Result<Option<usize>, SoplangError> {
    let Value::Int(raw) = index else {
        return Err(error("Tusmada xirmadu waa inay noqotaa abn."));
    };
    let index = if *raw < 0 { len as i64 + raw } else { *raw };
    Ok((index >= 0 && index < len as i64).then_some(index as usize))
}
pub(super) fn get(tuple: &TupleData, index: &Value) -> Result<Value, SoplangError> {
    let values = tuple.values();
    Ok(position(index, values.len())?
        .and_then(|index| values.get(index).cloned())
        .unwrap_or(Value::Null))
}
pub(super) fn dispatch(
    tuple: &TupleData,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    use super::stdlib::{expect_arity, expect_arity_range, slice_bounds};
    match method {
        "dherer" => {
            expect_arity(args, method, 0)?;
            Ok(Value::Int(tuple.len() as i64))
        }
        "hel" => {
            expect_arity_range(args, method, 1, 2)?;
            let values = tuple.values();
            Ok(position(&args[0], values.len())?
                .and_then(|index| values.get(index).cloned())
                .unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::Null)))
        }
        "hore" | "dambe" => {
            expect_arity(args, method, 0)?;
            let values = tuple.values();
            Ok(if method == "hore" {
                values.first()
            } else {
                values.last()
            }
            .cloned()
            .unwrap_or(Value::Null))
        }
        "nuqul" => {
            expect_arity(args, method, 0)?;
            Ok(value(tuple.values().clone()))
        }
        "urur" => {
            expect_arity(args, method, 0)?;
            super::sets::construct(tuple.values().clone())
        }
        "teed" => {
            expect_arity(args, method, 0)?;
            Ok(Value::List(Rc::new(RefCell::new(tuple.values().clone()))))
        }
        "jar" => {
            let values = tuple.values();
            let (start, end) = slice_bounds(args, values.len())?;
            Ok(value(values[start..end].to_vec()))
        }
        "midee" => {
            expect_arity(args, method, 1)?;
            let Value::Tuple(other) = &args[0] else {
                return Err(error("midee() waxay u baahan tahay xirmo."));
            };
            let mut values = tuple.values().clone();
            values.extend(other.values().iter().cloned());
            Ok(value(values))
        }
        "leeyahay" | "tiri" | "raadi" => {
            expect_arity(args, method, 1)?;
            let values = tuple.values();
            Ok(match method {
                "leeyahay" => Value::Bool(values.contains(&args[0])),
                "tiri" => {
                    Value::Int(values.iter().filter(|value| **value == args[0]).count() as i64)
                }
                _ => values
                    .iter()
                    .position(|value| *value == args[0])
                    .map(|index| Value::Int(index as i64))
                    .unwrap_or(Value::Null),
            })
        }
        "geli" | "dhig" | "saar" | "kasaar" | "ka_saar" | "nadiifi" | "rog" | "habee" | "kudar" => {
            Err(immutable())
        }
        _ => Err(error(format!(
            "Xirmadu ma laha hawl la yiraahdo '{}'.",
            method
        ))),
    }
}
