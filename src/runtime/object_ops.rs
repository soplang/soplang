//! Plain-object operations with shared contracts and deterministic callbacks.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{abi, contracts, Value};
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};
type Object = Rc<RefCell<HashMap<String, Value>>>;
fn error(message: impl Into<String>) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E043_COLLECTION_OPERATION),
    )
}
pub(super) fn read_error() -> SoplangError {
    error("Walaxda hadda lama akhriyi karo.")
}
fn snapshot(object: &Object) -> Result<HashMap<String, Value>, SoplangError> {
    object
        .try_borrow()
        .map(|values| values.clone())
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))
}
fn insert(object: &Object, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(args, "geli", 2)?;
    let Value::Str(key) = &args[0] else {
        return Err(error("Furaha geli() waa inuu noqdaa qoraal."));
    };
    if object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?
        .contains_key(key)
    {
        return Ok(Value::Object(Rc::clone(object)));
    }
    contracts::mutate_object(object, |values| {
        values.entry(key.clone()).or_insert_with(|| args[1].clone());
        Ok(())
    })?;
    Ok(Value::Object(Rc::clone(object)))
}
fn extend(object: &Object, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(args, "ballaadhi", 1)?;
    let Value::Object(other) = &args[0] else {
        return Err(error("ballaadhi() waxay u baahan tahay walax."));
    };
    let added = snapshot(other)?;
    contracts::mutate_object(object, |values| {
        values.extend(added);
        Ok(())
    })?;
    Ok(Value::Object(Rc::clone(object)))
}
fn pair(value: Value) -> Result<(String, Value), SoplangError> {
    let values = match value {
        Value::List(values) => values.try_borrow().map_err(|_| error("Lammaanaha aaddin() hadda lama akhriyi karo."))?.clone(),
        Value::Tuple(values) => values.values().clone(),
        _ => return Err(error("aaddin() hawshu waa inay celisaa teed ama xirmo laba xubnood leh marka habku yahay 'lamaane'.")),
    };
    if values.len() != 2 {
        return Err(error("aaddin() hawshu waa inay celisaa teed ama xirmo laba xubnood leh marka habku yahay 'lamaane'."));
    }
    let mut values = values.into_iter();
    let Value::Str(key) = values.next().unwrap() else {
        return Err(error("aaddin() furaha lammaanuhu waa inuu noqdaa qoraal."));
    };
    Ok((key, values.next().unwrap()))
}
fn transform(object: &Object, method: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, method, 1, if method == "aaddin" { 2 } else { 1 })?;
    let rename = match args.get(1) {
        None => false,
        Some(Value::Str(mode)) if mode == "qiime" => false,
        Some(Value::Str(mode)) if mode == "lamaane" => true,
        _ => {
            return Err(error(
                "aaddin() habku waa inuu noqdaa 'qiime' ama 'lamaane'.",
            ))
        }
    };
    let callback = abi::collection_callback(&args[0], method, 2)?;
    let mut entries: Vec<_> = snapshot(object)?.into_iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut result = HashMap::new();
    for (key, value) in entries {
        let call_args = [
            abi::value_to_soplang(&Value::Str(key.clone())),
            abi::value_to_soplang(&value),
        ];
        let mapped = abi::call_collection_callback(callback, &call_args)?;
        if method == "shaandhee" {
            if mapped.is_truthy() {
                result.insert(key, value);
            }
        } else {
            let (key, value) = if rename { pair(mapped)? } else { (key, mapped) };
            if result.insert(key.clone(), value).is_some() {
                return Err(error(format!(
                    "aaddin() furaha '{}' laba jeer ayaa la soo celiyey.",
                    key
                )));
            }
        }
    }
    Ok(Value::Object(Rc::new(RefCell::new(result))))
}
fn position(segment: &Value, len: usize) -> Result<Option<usize>, SoplangError> {
    let Value::Int(index) = segment else {
        return Err(error("Tusmada jidku waa inay noqotaa abn."));
    };
    let index = if *index < 0 {
        len as i64 + index
    } else {
        *index
    };
    Ok((index >= 0 && index < len as i64).then_some(index as usize))
}
fn key(segment: &Value) -> Result<&str, SoplangError> {
    let Value::Str(key) = segment else {
        return Err(error(
            "Furaha jidku waa inuu noqdaa qoraal marka qiimuhu yahay walax ama khalad.",
        ));
    };
    Ok(key)
}
fn step(value: &Value, segment: &Value) -> Result<Option<Value>, SoplangError> {
    Ok(match value {
        Value::Object(object) => object
            .try_borrow()
            .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?
            .get(key(segment)?)
            .cloned(),
        Value::List(list) => {
            let values = list
                .try_borrow()
                .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
            position(segment, values.len())?.and_then(|index| values.get(index).cloned())
        }
        Value::Tuple(tuple) => {
            position(segment, tuple.len())?.and_then(|index| tuple.values().get(index).cloned())
        }
        Value::Bytes(bytes) => {
            let bytes = super::bytes::read(bytes)?;
            position(segment, bytes.len())?.map(|index| Value::Int(bytes[index] as i64))
        }
        Value::Str(text) => position(segment, text.chars().count())?
            .and_then(|index| text.chars().nth(index))
            .map(|ch| Value::Str(ch.to_string())),
        Value::Error(error) => {
            let key = key(segment)?;
            matches!(
                key,
                "fariin" | "summad" | "sabab" | "fayl" | "saf" | "tiir" | "raad"
            )
            .then(|| abi::error_property(error, key))
        }
        _ => None,
    })
}
pub(super) fn lookup(object: &Object, args: &[Value]) -> Result<Value, SoplangError> {
    let segments = match &args[0] {
        Value::List(values) => values
            .try_borrow()
            .map_err(|_| error("Jidka hadda lama akhriyi karo."))?
            .clone(),
        Value::Tuple(values) => values.values().clone(),
        _ => unreachable!("nested path routing"),
    };
    if segments
        .iter()
        .any(|value| !matches!(value, Value::Str(_) | Value::Int(_)))
    {
        return Err(error("Qaybaha jidku waa inay noqdaan qoraal ama abn."));
    }
    let mut current = Value::Object(Rc::clone(object));
    for segment in &segments {
        let Some(value) = step(&current, segment)? else {
            return Ok(args.get(1).cloned().unwrap_or(Value::Null));
        };
        current = value;
    }
    Ok(current)
}
pub(super) fn dispatch(
    object: &Object,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    match method {
        "geli" => insert(object, args),
        "ballaadhi" => extend(object, args),
        "aaddin" | "shaandhee" => transform(object, method, args),
        _ => unreachable!("object operation routing"),
    }
}
