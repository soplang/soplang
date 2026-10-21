//! Context-owned iterators. Mutable collections use shallow snapshots; adapters
//! and Unicode/range cursors advance lazily without borrowing across callbacks.
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::abi;
use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E037_INVALID_ITERATION),
    )
}

pub(super) fn borrowed_error() -> SoplangError {
    error("Qiimaha hadda la beddelayo laguma wareegi karo.")
}

thread_local! { static DEPTH: Cell<usize> = const { Cell::new(0) }; }

struct DepthGuard(usize);
impl DepthGuard {
    fn enter() -> Result<Self, SoplangError> {
        DEPTH.with(|depth| {
            let previous = depth.get();
            if previous >= 32 {
                return Err(error("Wareegyada aad bay isugu dhex jiraan."));
            }
            depth.set(previous + 1);
            Ok(Self(previous))
        })
    }
}
impl Drop for DepthGuard {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(self.0));
    }
}

/// Opaque iterator state; values and their cursors belong to their creating context.
pub struct IteratorData {
    id: Cell<i64>,
    state: RefCell<Cursor>,
}

enum Cursor {
    Items(VecDeque<Value>),
    Bytes(VecDeque<u8>),
    Tuple {
        data: Rc<super::TupleData>,
        index: usize,
    },
    Text {
        text: String,
        offset: usize,
    },
    Range {
        next: Option<i64>,
        stop: i64,
        step: i64,
    },
    Enumerate {
        source: Value,
        index: Option<i64>,
    },
    Zip(Vec<Value>),
    Protocol(Value),
    Busy,
    Done,
}

impl IteratorData {
    pub fn handle(&self) -> i64 {
        self.id.get()
    }

    pub(super) fn children(&self) -> Option<Vec<Value>> {
        let cursor = self.state.try_borrow_mut().ok()?;
        Some(match &*cursor {
            Cursor::Items(items) => items
                .iter()
                .filter(|value| {
                    matches!(
                        value,
                        Value::Set(_)
                            | Value::Tuple(_)
                            | Value::List(_)
                            | Value::Bytes(_)
                            | Value::Object(_)
                            | Value::Closure(_)
                            | Value::Iterator(_)
                    )
                })
                .cloned()
                .collect(),
            Cursor::Enumerate { source, .. } | Cursor::Protocol(source) => vec![source.clone()],
            Cursor::Zip(sources) => sources.clone(),
            Cursor::Tuple { data, .. } => vec![Value::Tuple(Rc::clone(data))],
            _ => Vec::new(),
        })
    }

    pub(super) fn clear(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            *state = Cursor::Done;
        }
    }
}

fn create(cursor: Cursor) -> Value {
    let bytes = std::mem::size_of::<IteratorData>().saturating_add(match &cursor {
        Cursor::Items(items) => items.len().saturating_mul(std::mem::size_of::<Value>()),
        Cursor::Text { text, .. } => text.len(),
        Cursor::Bytes(bytes) => bytes.len(),
        Cursor::Zip(sources) => sources.len().saturating_mul(std::mem::size_of::<Value>()),
        _ => 0,
    });
    let data = Rc::new(IteratorData {
        id: Cell::new(0),
        state: RefCell::new(cursor),
    });
    STATE.with(|state| {
        let id = state.iterators.borrow_mut().insert(Rc::clone(&data));
        data.id.set(id);
        state.collector.allocated(bytes);
    });
    Value::Iterator(data)
}

pub(super) fn validate(data: &Rc<IteratorData>) -> Result<(), SoplangError> {
    STATE.with(|state| {
        if state
            .iterators
            .borrow()
            .get(data.handle())
            .is_some_and(|entry| Rc::ptr_eq(entry, data))
        {
            Ok(())
        } else {
            Err(abi::invalid_handle())
        }
    })
}

pub(super) fn begin(value: &Value) -> Result<Value, SoplangError> {
    let _depth = DepthGuard::enter()?;
    match value {
        Value::Iterator(data) => {
            validate(data)?;
            Ok(value.clone())
        }
        Value::List(items) => Ok(create(Cursor::Items(
            items
                .try_borrow()
                .map_err(|_| borrowed_error())?
                .iter()
                .cloned()
                .collect(),
        ))),
        Value::Bytes(bytes) => Ok(create(Cursor::Bytes(
            super::bytes::read(bytes)?.iter().copied().collect(),
        ))),
        Value::Set(data) => Ok(create(Cursor::Items(data.values().into()))),
        Value::Tuple(data) => Ok(create(Cursor::Tuple {
            data: Rc::clone(data),
            index: 0,
        })),
        Value::Str(text) => Ok(create(Cursor::Text {
            text: text.clone(),
            offset: 0,
        })),
        Value::Object(object) => {
            object.try_borrow().map_err(|_| borrowed_error())?;
            if abi::iteration_has_method(value, "wareeg") {
                let iterator = abi::iteration_call_method(value, "wareeg")?;
                if let Value::Iterator(data) = &iterator {
                    validate(data)?;
                    return Ok(iterator);
                }
                if abi::iteration_has_method(&iterator, "xiga") {
                    return Ok(create(Cursor::Protocol(iterator)));
                }
                return Err(error(
                    "Hawsha wareeg() waa inay celisaa wax leh hawl xiga().",
                ));
            }
            if abi::iteration_has_method(value, "xiga") {
                return Ok(create(Cursor::Protocol(value.clone())));
            }
            if abi::iteration_is_instance(value) {
                return Err(error("Qaabkani ma laha hawl wareeg() ama xiga()."));
            }
            let mut keys: Vec<_> = object.borrow().keys().cloned().collect();
            keys.sort();
            Ok(create(Cursor::Items(
                keys.into_iter().map(Value::Str).collect(),
            )))
        }
        _ => Err(error(format!(
            "Nooca '{}' laguma wareegi karo.",
            value.type_name()
        ))),
    }
}

fn decode_step(value: Value) -> Result<Option<Value>, SoplangError> {
    let Value::Object(object) = value else {
        return Err(error(
            "Hawsha xiga() waa inay celisaa walax leh dhammaad iyo qiime.",
        ));
    };
    let object = object.try_borrow().map_err(|_| borrowed_error())?;
    match object.get("dhammaad") {
        Some(Value::Bool(true)) => Ok(None),
        Some(Value::Bool(false)) => object
            .get("qiime")
            .cloned()
            .map(Some)
            .ok_or_else(|| error("Natiijada xiga() waxaa ka maqan qiime.")),
        _ => Err(error("Sifada dhammaad ee xiga() waa inay noqotaa bool.")),
    }
}

pub(super) fn advance(data: &Rc<IteratorData>) -> Result<Option<Value>, SoplangError> {
    let _depth = DepthGuard::enter()?;
    validate(data)?;
    let mut cursor = {
        let mut state = data
            .state
            .try_borrow_mut()
            .map_err(|_| error("Wareeggan mar hore ayaa la wadaa."))?;
        if matches!(*state, Cursor::Busy) {
            return Err(error("Wareeggan mar hore ayaa la wadaa."));
        }
        std::mem::replace(&mut *state, Cursor::Busy)
    };
    // No RefCell borrow survives an invocation of user code. While Busy, values
    // moved into this Rust frame retain their own references through nested GC.
    let result = (|| match &mut cursor {
        Cursor::Items(items) => Ok(items.pop_front()),
        Cursor::Tuple { data, index } => {
            let value = data.values().get(*index).cloned();
            if value.is_some() {
                *index += 1;
            }
            Ok(value)
        }
        Cursor::Bytes(bytes) => Ok(bytes.pop_front().map(|byte| Value::Int(byte as i64))),
        Cursor::Text { text, offset } => {
            let next = text[*offset..].chars().next();
            if let Some(ch) = next {
                *offset += ch.len_utf8();
            }
            Ok(next.map(|ch| Value::Str(ch.to_string())))
        }
        Cursor::Range { next, stop, step } => {
            let Some(current) = *next else {
                return Ok(None);
            };
            if (*step > 0 && current >= *stop) || (*step < 0 && current <= *stop) {
                return Ok(None);
            }
            *next = current.checked_add(*step);
            Ok(Some(Value::Int(current)))
        }
        Cursor::Enumerate { source, index } => {
            let Some(value) = next_value(source)? else {
                return Ok(None);
            };
            let current =
                index.ok_or_else(|| error("Tusmada tirso() waxay ka baxday xadka abn."))?;
            *index = current.checked_add(1);
            Ok(Some(row(vec![Value::Int(current), value])))
        }
        Cursor::Zip(sources) => {
            if sources.is_empty() {
                return Ok(None);
            }
            let mut values = Vec::with_capacity(sources.len());
            for source in sources {
                match next_value(source)? {
                    Some(value) => values.push(value),
                    None => return Ok(None),
                }
            }
            Ok(Some(row(values)))
        }
        Cursor::Protocol(source) => decode_step(abi::iteration_call_method(source, "xiga")?),
        Cursor::Done => Ok(None),
        Cursor::Busy => unreachable!("reentrancy checked before moving cursor"),
    })();
    *data.state.borrow_mut() = if matches!(result, Ok(Some(_))) {
        cursor
    } else {
        Cursor::Done
    };
    result
}

pub(super) fn next_value(iterator: &Value) -> Result<Option<Value>, SoplangError> {
    match iterator {
        Value::Iterator(data) => advance(data),
        _ => Err(error("xiga() wuxuu u baahan yahay wareeg.")),
    }
}

pub(super) fn row(values: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(values)))
}

pub(super) fn next_step(iterator: &Value, bindings: usize) -> Result<Value, SoplangError> {
    let value = next_value(iterator)?;
    if bindings > 1 {
        if let Some(value) = &value {
            let valid = match value {
                Value::Tuple(tuple) => tuple.len() == bindings,
                Value::List(items) => {
                    items.try_borrow().map_err(|_| borrowed_error())?.len() == bindings
                }
                _ => false,
            };
            if !valid {
                return Err(error(format!("Wareeggu wuxuu u baahan yahay teed ama xirmo ka kooban {} qiime si loo qaybiyo magacyada.", bindings)));
            }
        }
    }
    Ok(Value::Object(Rc::new(RefCell::new(HashMap::from([
        ("dhammaad".into(), Value::Bool(value.is_none())),
        ("qiime".into(), value.unwrap_or(Value::Null)),
    ])))))
}

pub(super) fn builtin_wareeg(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(&args, "wareeg", 1)?;
    begin(&args[0])
}
pub(super) fn builtin_xiga(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(&args, "xiga", 1)?;
    next_step(&args[0], 1)
}
pub(super) fn builtin_tirso(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(&args, "tirso", 1, 2)?;
    let index = match args.get(1) {
        None => 0,
        Some(Value::Int(index)) => *index,
        _ => return Err(error("Tusmada bilowga ee tirso() waa inay noqotaa abn.")),
    };
    Ok(create(Cursor::Enumerate {
        source: begin(&args[0])?,
        index: Some(index),
    }))
}
pub(super) fn builtin_lamaane(args: Vec<Value>) -> Result<Value, SoplangError> {
    let sources = args.iter().map(begin).collect::<Result<_, _>>()?;
    Ok(create(Cursor::Zip(sources)))
}
pub(super) fn builtin_taxane(args: Vec<Value>) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(&args, "taxane", 1, 3)?;
    let values = args
        .iter()
        .map(|value| match value {
            Value::Int(value) => Ok(*value),
            _ => Err(error("Doodaha taxane() waa inay noqdaan abn.")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (start, stop, step) = match values.as_slice() {
        [stop] => (0, *stop, 1),
        [start, stop] => (*start, *stop, 1),
        [start, stop, step] => (*start, *stop, *step),
        _ => unreachable!(),
    };
    if step == 0 {
        return Err(error("Tallaabada taxane() eber ma noqon karto."));
    }
    Ok(create(Cursor::Range {
        next: Some(start),
        stop,
        step,
    }))
}
