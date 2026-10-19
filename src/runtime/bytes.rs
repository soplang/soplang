//! Compact, mutable byte arrays and explicit, strict UTF-8 conversion.

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use super::stdlib::{expect_arity, expect_arity_range, slice_bounds};
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

type Bytes = Rc<RefCell<Vec<u8>>>;

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E038_INVALID_BYTES),
    )
}

pub(super) fn read(bytes: &Bytes) -> Result<Ref<'_, Vec<u8>>, SoplangError> {
    bytes
        .try_borrow()
        .map_err(|_| error("Baytyada hadda lama akhriyi karo."))
}

fn write(bytes: &Bytes) -> Result<RefMut<'_, Vec<u8>>, SoplangError> {
    bytes
        .try_borrow_mut()
        .map_err(|_| error("Baytyada hadda lama beddeli karo."))
}

fn byte(value: &Value) -> Result<u8, SoplangError> {
    match value {
        Value::Int(value) => u8::try_from(*value)
            .map_err(|_| error("Qiimaha baytku waa inuu noqdaa abn u dhexeeya 0 iyo 255.")),
        _ => Err(error(
            "Qiimaha baytku waa inuu noqdaa abn u dhexeeya 0 iyo 255.",
        )),
    }
}

fn integer(value: &Value) -> Result<i64, SoplangError> {
    match value {
        Value::Int(value) => Ok(*value),
        _ => Err(error("Tusmada baytyadu waa inay noqotaa abn.")),
    }
}

fn position(value: &Value, len: usize) -> Result<Option<usize>, SoplangError> {
    let raw = integer(value)?;
    let index = if raw < 0 { len as i64 + raw } else { raw };
    Ok((index >= 0 && index < len as i64).then_some(index as usize))
}

fn checked_position(value: &Value, len: usize, insert: bool) -> Result<usize, SoplangError> {
    let raw = integer(value)?;
    if insert && raw == len as i64 {
        return Ok(len);
    }
    position(value, len)?.ok_or_else(|| error("Tusmadu way ka baxsan tahay baytyada."))
}

fn from_vec(bytes: Vec<u8>) -> Value {
    Value::Bytes(Rc::new(RefCell::new(bytes)))
}

pub(super) fn construct(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "baytyo", 0, 1)?;
    let bytes = match args.first() {
        None => Vec::new(),
        Some(Value::Str(text)) => text.as_bytes().to_vec(),
        Some(Value::Bytes(bytes)) => read(bytes)?.clone(),
        Some(Value::List(values)) => values
            .try_borrow()
            .map_err(|_| error("Teedka hadda lama akhriyi karo."))?
            .iter()
            .map(byte)
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(error(
                "baytyo() waxay u baahan tahay qoraal, teed ka kooban baytyo, ama baytyo kale.",
            ))
        }
    };
    Ok(from_vec(bytes))
}

pub(super) fn decode(bytes: &Bytes) -> Result<String, SoplangError> {
    let bytes = read(bytes)?;
    std::str::from_utf8(&bytes)
        .map(str::to_owned)
        .map_err(|_| error("Baytyadu ma aha qoraal UTF-8 ah oo sax ah."))
}

pub(super) fn get(bytes: &Bytes, index: &Value) -> Result<Value, SoplangError> {
    let bytes = read(bytes)?;
    Ok(position(index, bytes.len())?
        .map(|index| Value::Int(bytes[index] as i64))
        .unwrap_or(Value::Null))
}

pub(super) fn set(bytes: &Bytes, index: &Value, value: &Value) -> Result<Value, SoplangError> {
    let value = byte(value)?;
    let mut items = write(bytes)?;
    let index = checked_position(index, items.len(), false)?;
    items[index] = value;
    Ok(Value::Bytes(Rc::clone(bytes)))
}

pub(super) fn dispatch(bytes: &Bytes, method: &str, args: &[Value]) -> Result<Value, SoplangError> {
    match method {
        "dherer" => {
            expect_arity(args, method, 0)?;
            Ok(Value::Int(read(bytes)?.len() as i64))
        }
        "qoraal" => {
            expect_arity(args, method, 0)?;
            decode(bytes).map(Value::Str)
        }
        "teed" => {
            expect_arity(args, method, 0)?;
            Ok(Value::List(Rc::new(RefCell::new(
                read(bytes)?
                    .iter()
                    .map(|byte| Value::Int(*byte as i64))
                    .collect(),
            ))))
        }
        "hel" => {
            expect_arity_range(args, method, 1, 2)?;
            let items = read(bytes)?;
            Ok(position(&args[0], items.len())?
                .map(|index| Value::Int(items[index] as i64))
                .unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::Null)))
        }
        "dhig" => {
            expect_arity(args, method, 2)?;
            set(bytes, &args[0], &args[1])
        }
        "geli" => {
            expect_arity_range(args, method, 1, 2)?;
            let value = byte(args.last().unwrap())?;
            let mut items = write(bytes)?;
            let index = if args.len() == 1 {
                items.len()
            } else {
                checked_position(&args[0], items.len(), true)?
            };
            let capacity = items.capacity();
            items.insert(index, value);
            super::abi::record_bytes_growth(items.capacity().saturating_sub(capacity));
            Ok(Value::Bytes(Rc::clone(bytes)))
        }
        "saar" => {
            expect_arity_range(args, method, 0, 1)?;
            let mut items = write(bytes)?;
            let index = if let Some(value) = args.first() {
                checked_position(value, items.len(), false)?
            } else {
                items
                    .len()
                    .checked_sub(1)
                    .ok_or_else(|| error("Ma saari kartid baytyo madhan."))?
            };
            Ok(Value::Int(items.remove(index) as i64))
        }
        "hore" | "dambe" => {
            expect_arity(args, method, 0)?;
            let items = read(bytes)?;
            let value = if method == "hore" {
                items.first()
            } else {
                items.last()
            };
            Ok(value
                .map(|byte| Value::Int(*byte as i64))
                .unwrap_or(Value::Null))
        }
        "nuqul" => {
            expect_arity(args, method, 0)?;
            Ok(from_vec(read(bytes)?.clone()))
        }
        "jar" => {
            let items = read(bytes)?;
            let (start, end) = slice_bounds(args, items.len())?;
            Ok(from_vec(items[start..end].to_vec()))
        }
        "midee" => {
            expect_arity(args, method, 1)?;
            let Value::Bytes(other) = &args[0] else {
                return Err(error("midee() waxay u baahan tahay baytyo."));
            };
            let mut result = read(bytes)?.clone();
            result.extend_from_slice(&read(other)?);
            Ok(from_vec(result))
        }
        "nadiifi" | "rog" => {
            expect_arity(args, method, 0)?;
            let mut items = write(bytes)?;
            if method == "nadiifi" {
                items.clear();
            } else {
                items.reverse();
            }
            Ok(Value::Bytes(Rc::clone(bytes)))
        }
        "leeyahay" | "tiri" | "raadi" => {
            expect_arity(args, method, 1)?;
            let value = byte(&args[0])?;
            let items = read(bytes)?;
            Ok(match method {
                "leeyahay" => Value::Bool(items.contains(&value)),
                "tiri" => Value::Int(items.iter().filter(|item| **item == value).count() as i64),
                _ => items
                    .iter()
                    .position(|item| *item == value)
                    .map(|index| Value::Int(index as i64))
                    .unwrap_or(Value::Null),
            })
        }
        _ => Err(error(format!(
            "Baytyadu ma laha hawl la yiraahdo '{}'.",
            method
        ))),
    }
}
