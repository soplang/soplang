//! List operations: snapshot inputs and publish mutations through contracts.
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::{abi, contracts, iterators, value::ordering_text, Value};
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};
type List = Rc<RefCell<Vec<Value>>>;

fn error(message: impl Into<String>) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E043_COLLECTION_OPERATION),
    )
}
fn snapshot(list: &List) -> Result<Vec<Value>, SoplangError> {
    list.try_borrow()
        .map(|items| items.clone())
        .map_err(|_| error("Teedka hadda lama akhriyi karo."))
}
pub(super) fn remove_value(list: &List, args: &[Value]) -> Result<Value, SoplangError> {
    if !matches!(&args[1], Value::Str(mode) if mode == "qiime") {
        return Err(error("saar() doodda labaad waa inay noqotaa 'qiime'."));
    }
    // Compare before the mutable borrow, including keys that refer to the list.
    let index = snapshot(list)?.iter().position(|value| value == &args[0]);
    contracts::mutate_list(list, |values| {
        Ok(index
            .map(|index| values.remove(index))
            .unwrap_or(Value::Null))
    })
}
fn extend(list: &List, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(args, "ballaadhi", 1)?;
    let iterator = iterators::begin(&args[0])?;
    let mut added = Vec::new();
    while let Some(value) = iterators::next_value(&iterator)? {
        added.push(value);
    }
    contracts::mutate_list(list, |values| {
        values.extend(added);
        Ok(())
    })?;
    Ok(Value::List(Rc::clone(list)))
}
fn unique(list: &List, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(args, "keli", 0)?;
    let mut keys = HashSet::new();
    let mut other = Vec::new();
    let mut values = Vec::new();
    for value in snapshot(list)? {
        let fresh = match super::sets::Key::of(&value, 0) {
            Ok(key) => keys.insert(key),
            Err(_) if other.contains(&value) => false,
            Err(_) => {
                other.push(value.clone());
                true
            }
        };
        if fresh {
            values.push(value);
        }
    }
    Ok(Value::List(Rc::new(RefCell::new(values))))
}
fn sum(list: &List, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, "wadar", 0, 1)?;
    let mut result = args.first().cloned().unwrap_or(Value::Int(0));
    if !matches!(result, Value::Int(_) | Value::Float(_)) {
        return Err(error(
            "wadar() qiimaha bilowgu waa inuu noqdaa abn ama jajab.",
        ));
    }
    for value in snapshot(list)? {
        result = match (result, value) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a.wrapping_add(b)),
            (Value::Int(a), Value::Float(b)) => Value::Float(a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + b as f64),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            _ => {
                return Err(error(
                    "wadar() xubnaha teedka waa inay noqdaan abn ama jajab.",
                ))
            }
        };
    }
    Ok(result)
}

// Compare integers to floats without first rounding the integer to f64.
fn int_float(integer: i64, float: f64) -> Ordering {
    if float.is_nan() || float >= 9223372036854775808.0 {
        return Ordering::Less;
    }
    if float < -9223372036854775808.0 {
        return Ordering::Greater;
    }
    integer.cmp(&(float as i64)).then_with(|| {
        if float.fract() > 0.0 {
            Ordering::Less
        } else if float.fract() < 0.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    })
}
fn float_float(a: f64, b: f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => a.total_cmp(&b),
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => a.partial_cmp(&b).unwrap(),
    }
}
pub(super) fn numeric_compare(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
        (Value::Int(a), Value::Float(b)) => Some(int_float(*a, *b)),
        (Value::Float(a), Value::Int(b)) => Some(int_float(*b, *a).reverse()),
        (Value::Float(a), Value::Float(b)) => Some(float_float(*a, *b)),
        _ => None,
    }
}
enum SortKey {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Other(u8, String),
}
impl SortKey {
    fn rank(&self) -> u8 {
        match self {
            Self::Null => 0,
            Self::Bool(_) => 1,
            Self::Int(_) | Self::Float(_) => 2,
            Self::Text(_) => 3,
            Self::Other(rank, _) => *rank,
        }
    }
    fn compare(&self, other: &Self) -> Ordering {
        let rank = self.rank().cmp(&other.rank());
        if rank != Ordering::Equal {
            return rank;
        }
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a.cmp(b),
            (Self::Int(a), Self::Int(b)) => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => float_float(*a, *b),
            (Self::Int(a), Self::Float(b)) => int_float(*a, *b),
            (Self::Float(a), Self::Int(b)) => int_float(*b, *a).reverse(),
            (Self::Text(a), Self::Text(b)) | (Self::Other(_, a), Self::Other(_, b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }
    fn from(value: &Value) -> Result<Self, SoplangError> {
        Ok(match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Bool(*value),
            Value::Int(value) => Self::Int(*value),
            Value::Float(value) => Self::Float(*value),
            Value::Str(value) => Self::Text(value.clone()),
            Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_) => {
                Self::Other(9, String::new())
            }
            Value::Iterator(_) => Self::Other(11, String::new()),
            value => {
                readable(value)?;
                let rank = match value {
                    Value::List(_) => 4,
                    Value::Tuple(_) => 5,
                    Value::Set(_) => 6,
                    Value::Bytes(_) => 7,
                    Value::Object(_) => 8,
                    _ => 10,
                };
                Self::Other(rank, ordering_text(value))
            }
        })
    }
}
// Formatting uses bounded cycle-safe traversal. Check the same accessible graph
// before rendering so a conflicting native borrow becomes a language error.
fn readable(value: &Value) -> Result<(), SoplangError> {
    let mut pending = vec![(value.clone(), 0)];
    let mut depths = HashMap::new();
    while let Some((value, depth)) = pending.pop() {
        if depth >= 64 {
            continue;
        }
        let key = match &value {
            Value::List(data) => (5, Rc::as_ptr(data) as usize),
            Value::Object(data) => (6, Rc::as_ptr(data) as usize),
            Value::Tuple(data) => (13, Rc::as_ptr(data) as usize),
            Value::Set(data) => (14, Rc::as_ptr(data) as usize),
            _ => (0, 0),
        };
        if key.0 != 0 {
            if depths.get(&key).is_some_and(|previous| *previous <= depth) {
                continue;
            }
            depths.insert(key, depth);
        }
        let children: Vec<Value> = match value {
            Value::List(data) => data
                .try_borrow()
                .map_err(|_| error("Furaha habaynta hadda lama akhriyi karo."))?
                .clone(),
            Value::Object(data) => data
                .try_borrow()
                .map_err(|_| error("Furaha habaynta hadda lama akhriyi karo."))?
                .values()
                .cloned()
                .collect(),
            Value::Tuple(data) => data.values().clone(),
            Value::Set(data) => data.values(),
            Value::Bytes(data) => {
                data.try_borrow()
                    .map_err(|_| error("Furaha habaynta hadda lama akhriyi karo."))?;
                Vec::new()
            }
            _ => Vec::new(),
        };
        pending.extend(children.into_iter().map(|value| (value, depth + 1)));
    }
    Ok(())
}
fn same_slot(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
        (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
        (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
        (Value::Tuple(a), Value::Tuple(b)) => Rc::ptr_eq(a, b),
        (Value::Set(a), Value::Set(b)) => Rc::ptr_eq(a, b),
        (Value::Bytes(a), Value::Bytes(b)) => Rc::ptr_eq(a, b),
        _ => a == b,
    }
}
pub(super) fn sort(list: &List, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, "habee", 0, 2)?;
    let reverse = match args.get(1) {
        None => false,
        Some(Value::Bool(reverse)) => *reverse,
        _ => return Err(error("habee() rogistu waa inay noqotaa bool.")),
    };
    let callback = match args.first() {
        None | Some(Value::Null) => None,
        Some(value) => Some(abi::collection_callback(value, "habee", 1)?),
    };
    let original = snapshot(list)?;
    let mut decorated = Vec::new();
    for value in original.iter().cloned() {
        let key = match callback {
            None => value.clone(),
            Some(callback) => {
                abi::call_collection_callback(callback, &[abi::value_to_soplang(&value)])?
            }
        };
        // Freeze each key after all callbacks finish: keys can be shared mutable
        // values, and no comparison may invoke user code or keep a native borrow.
        decorated.push((value, key));
    }
    {
        let current = list
            .try_borrow()
            .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
        if current.len() != original.len()
            || !current.iter().zip(&original).all(|(a, b)| same_slot(a, b))
        {
            return Err(error("habee() hawsha furaha ayaa beddeshay teedka."));
        }
    }
    let mut sorted = decorated
        .into_iter()
        .map(|(value, key)| Ok((value, SortKey::from(&key)?)))
        .collect::<Result<Vec<_>, SoplangError>>()?;
    sorted.sort_by(|(_, a), (_, b)| {
        let order = a.compare(b);
        if reverse {
            order.reverse()
        } else {
            order
        }
    });
    let values = sorted.into_iter().map(|(value, _)| value).collect();
    contracts::mutate_list(list, |items| {
        *items = values;
        Ok(())
    })?;
    Ok(Value::List(Rc::clone(list)))
}
pub(super) fn dispatch(list: &List, method: &str, args: &[Value]) -> Result<Value, SoplangError> {
    match method {
        "ballaadhi" => extend(list, args),
        "keli" => unique(list, args),
        "wadar" => sum(list, args),
        "tirso" => {
            super::stdlib::expect_arity_range(args, method, 0, 1)?;
            let mut values = vec![Value::List(Rc::clone(list))];
            values.extend_from_slice(args);
            iterators::builtin_tirso(values)
        }
        "lamaane" => {
            let mut values = vec![Value::List(Rc::clone(list))];
            values.extend_from_slice(args);
            iterators::builtin_lamaane(values)
        }
        _ => unreachable!("list operation routing"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_sort_order_is_exact_symmetric_and_transitive() {
        let values = [
            Value::Int(i64::MIN),
            Value::Int(i64::MIN + 1),
            Value::Int(-9007199254740993),
            Value::Float(-9007199254740992.0),
            Value::Float(-1.5),
            Value::Int(-1),
            Value::Float(-0.0),
            Value::Int(0),
            Value::Float(0.0),
            Value::Float(f64::from_bits(1)),
            Value::Float(0.5),
            Value::Int(1),
            Value::Float(1.0),
            Value::Float(1.5),
            Value::Int(9007199254740992),
            Value::Float(9007199254740992.0),
            Value::Int(9007199254740993),
            Value::Int(i64::MAX),
            Value::Float(9223372036854775808.0),
            Value::Float(f64::NEG_INFINITY),
            Value::Float(f64::INFINITY),
            Value::Float(f64::NAN),
            Value::Float(-f64::NAN),
            Value::Float(f64::from_bits(0x7ff8000000000001)),
        ];
        let keys = values
            .iter()
            .map(SortKey::from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for a in &keys {
            for b in &keys {
                assert_eq!(a.compare(b), b.compare(a).reverse());
                for c in &keys {
                    if a.compare(b) != Ordering::Greater && b.compare(c) != Ordering::Greater {
                        assert_ne!(a.compare(c), Ordering::Greater);
                    }
                }
            }
        }
        assert_eq!(
            int_float(9007199254740993, 9007199254740992.0),
            Ordering::Greater
        );
        assert_eq!(int_float(i64::MAX, 9223372036854775808.0), Ordering::Less);
        assert_eq!(int_float(i64::MIN, -9223372036854775808.0), Ordering::Equal);
        assert_eq!(float_float(-0.0, 0.0), Ordering::Equal);
    }

    #[test]
    fn conflicting_nested_key_borrows_fail_before_sort_publication() {
        let child = Rc::new(RefCell::new(vec![Value::Int(1)]));
        let mut deep = Value::List(Rc::clone(&child));
        for _ in 0..70 {
            deep = Value::Tuple(super::super::TupleData::new(vec![deep]));
        }
        let root = Value::Object(Rc::new(RefCell::new(HashMap::from([
            ("deep".into(), deep),
            ("shallow".into(), Value::List(Rc::clone(&child))),
        ]))));
        let list = Rc::new(RefCell::new(vec![root.clone(), Value::Null]));
        let borrowed = child.borrow_mut();
        assert_eq!(sort(&list, &[]).unwrap_err().code(), Some("E043"));
        assert!(
            matches!(&list.borrow()[0],Value::Object(current) if matches!(&root,Value::Object(original) if Rc::ptr_eq(current,original)))
        );
        drop(borrowed);
        sort(&list, &[]).unwrap();
        assert_eq!(list.borrow()[0], Value::Null);
        let first = Value::Object(Rc::new(RefCell::new(HashMap::from([(
            "f".into(),
            Value::Function(super::super::value::FunctionId(1)),
        )]))));
        let second = Value::Object(Rc::new(RefCell::new(HashMap::from([(
            "f".into(),
            Value::Function(super::super::value::FunctionId(100)),
        )]))));
        assert_eq!(
            SortKey::from(&first)
                .unwrap()
                .compare(&SortKey::from(&second).unwrap()),
            Ordering::Equal
        );
    }

    #[test]
    fn primitive_uniqueness_uses_hash_keys_and_preserves_first_occurrence() {
        let source = Rc::new(RefCell::new(
            (0..20000).map(|index| Value::Int(index % 10000)).collect(),
        ));
        let Value::List(result) = unique(&source, &[]).unwrap() else {
            panic!("list")
        };
        assert_eq!(result.borrow().len(), 10000);
        assert!(result
            .borrow()
            .iter()
            .enumerate()
            .all(|(index, value)| *value == Value::Int(index as i64)));
        let source = Rc::new(RefCell::new(vec![
            Value::Float(f64::NAN),
            Value::Float(f64::NAN),
            Value::Float(-0.0),
            Value::Float(0.0),
        ]));
        let Value::List(result) = unique(&source, &[]).unwrap() else {
            panic!("list")
        };
        assert_eq!(result.borrow().len(), 3);
    }
}
