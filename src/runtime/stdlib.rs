//! Standard library: built-in functions and method dispatch (Phase 5).
//! Built-in functions and methods for Soplang.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

use super::contracts::{mutate_list, mutate_object};
use super::value::{value_to_string, Value};
use crate::error::{input_error, io_error, runtime_error, type_error, SoplangError};

fn err_type(msg: impl Into<String>) -> SoplangError {
    type_error(msg, 0, 0)
}
fn err_runtime(msg: impl Into<String>) -> SoplangError {
    runtime_error(msg, 0, 0)
}

pub(crate) fn expect_arity(
    args: &[Value],
    name: &str,
    expected: usize,
) -> Result<(), SoplangError> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(err_type(format!(
            "Hawsha '{}()' waxay qaadataa {} dood, laakiin {} ayaa la bixiyay",
            name,
            expected,
            args.len()
        )))
    }
}

pub(crate) fn expect_arity_range(
    args: &[Value],
    name: &str,
    minimum: usize,
    maximum: usize,
) -> Result<(), SoplangError> {
    if (minimum..=maximum).contains(&args.len()) {
        Ok(())
    } else {
        Err(err_type(format!(
            "Hawsha '{}()' waxay qaadataa {} ilaa {} dood, laakiin {} ayaa la bixiyay",
            name,
            minimum,
            maximum,
            args.len()
        )))
    }
}

fn to_number(v: &Value) -> Result<f64, SoplangError> {
    match v {
        Value::Int(n) => Ok(*n as f64),
        Value::Float(x) => Ok(*x),
        _ => Err(err_type(format!(
            "{} ma ahan abn ama jajab",
            value_to_string(v)
        ))),
    }
}

fn to_integer(v: &Value, name: &str) -> Result<i64, SoplangError> {
    match v {
        Value::Int(n) => Ok(*n),
        _ => Err(err_type(format!("{}() wuxuu u baahan yahay abn", name))),
    }
}

/// Built-in qor: print space-separated values and return the text without its newline.
pub fn builtin_qor(args: Vec<Value>) -> Result<Value, SoplangError> {
    let s = args
        .iter()
        .map(value_to_string)
        .collect::<Vec<_>>()
        .join(" ");
    writeln!(io::stdout().lock(), "{}", s)
        .map_err(|e| io_error("Waxsoosaarka lama qori karin.", &e))?;
    Ok(Value::Str(s))
}

/// Built-in gelin: read line from stdin (optional prompt).
pub fn builtin_gelin(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "gelin", 0, 1)?;
    let prompt = args.first().map(|v| value_to_string(v)).unwrap_or_default();
    {
        let mut output = io::stdout().lock();
        write!(output, "{}", prompt)
            .and_then(|_| output.flush())
            .map_err(|e| io_error("Waxsoosaarka lama qori karin.", &e))?;
    }
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| input_error(&e))?;
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Value::Str(line))
}

/// Construct an immutable structured error without raising it.
pub fn builtin_khalad(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "khalad", 1, 3)?;
    let Value::Str(message) = &args[0] else {
        return Err(err_type("Fariinta khalad() waa inay noqotaa qoraal."));
    };
    let code = match args.get(1) {
        None | Some(Value::Null) => None,
        Some(Value::Str(code)) => Some(code.clone()),
        _ => {
            return Err(err_type(
                "Summadda khalad() waa inay noqotaa qoraal ama maran.",
            ))
        }
    };
    let cause = match args.get(2) {
        None | Some(Value::Null) => None,
        Some(Value::Error(error)) => Some(Box::new((**error).clone())),
        _ => {
            return Err(err_type(
                "Sababta khalad() waa inay noqotaa khalad ama maran.",
            ))
        }
    };
    let mut error = err_runtime(message.clone());
    error.meta_mut().user_code = code;
    error.meta_mut().cause = cause;
    Ok(Value::Error(Rc::new(error)))
}

/// Format a value with an optional static format descriptor.
pub fn builtin_qaabee(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "qaabee", 1, 2)?;
    let descriptor = match args.get(1) {
        None => "", Some(Value::Str(value)) => value,
        _ => return Err(err_type("Qaabka qaabee() waa inuu noqdaa qoraal.")),
    };
    super::formatting::value(&args[0], descriptor).map(Value::Str)
}

pub fn string_qaabee(text: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::formatting::template(text, args).map(Value::Str)
}

/// Built-in nooc: return type name string.
pub fn builtin_nooc(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "nooc", 1)?;
    Ok(Value::Str(args[0].type_name().to_string()))
}

/// Built-in abn: convert to integer.
pub fn builtin_abn(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "abn", 1)?;
    let value = &args[0];
    let integer = match value {
        Value::Int(n) => *n,
        Value::Float(x) if x.is_finite() => *x as i64,
        Value::Str(s) => s.trim().parse::<i64>().map_err(|_| {
            err_type(format!(
                "Qiimaha '{}' looma beddeli karo abn",
                value_to_string(value)
            ))
        })?,
        _ => {
            return Err(err_type(format!(
                "Qiimaha '{}' looma beddeli karo abn",
                value_to_string(value)
            )))
        }
    };
    Ok(Value::Int(integer))
}

/// Built-in jajab: convert to float.
pub fn builtin_jajab(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "jajab", 1)?;
    let value = &args[0];
    let number = match value {
        Value::Int(n) => *n as f64,
        Value::Float(x) => *x,
        Value::Str(s) => s.trim().parse::<f64>().map_err(|_| {
            err_type(format!(
                "Qiimaha '{}' looma beddeli karo jajab",
                value_to_string(value)
            ))
        })?,
        _ => {
            return Err(err_type(format!(
                "Qiimaha '{}' looma beddeli karo jajab",
                value_to_string(value)
            )))
        }
    };
    if !number.is_finite() {
        return Err(err_type(format!(
            "Qiimaha '{}' looma beddeli karo jajab",
            value_to_string(value)
        )));
    }
    Ok(Value::Float(number))
}

/// Built-in qoraal: convert to string (value_to_string).
pub fn builtin_qoraal(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "qoraal", 1)?;
    match &args[0] {
        Value::Bytes(bytes) => super::bytes::decode(bytes).map(Value::Str),
        value => Ok(Value::Str(value_to_string(value))),
    }
}

/// Built-in bool: truthiness (0, "", false, null, "false", "False" -> false).
pub fn builtin_bool(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "bool", 1)?;
    let v = &args[0];
    let b = match v {
        Value::Null => false,
        Value::Bool(x) => *x,
        Value::Int(n) => *n != 0,
        Value::Float(x) => *x != 0.0,
        Value::Str(s) => !s.is_empty() && s != "false" && s != "False",
        Value::List(l) => !l.borrow().is_empty(),
        Value::Tuple(tuple) => !tuple.is_empty(),
        Value::Set(set) => !set.is_empty(),
        Value::Bytes(bytes) => !super::bytes::read(bytes)?.is_empty(),
        Value::Object(o) => !o.borrow().is_empty(),
        _ => true,
    };
    Ok(Value::Bool(b))
}

/// Built-in teed: create list from arguments.
pub fn builtin_teed(args: Vec<Value>) -> Result<Value, SoplangError> {
    Ok(Value::List(Rc::new(RefCell::new(args))))
}

/// Built-in walax: create object from key-value pairs. Soplang has no kwargs; we accept 0 args = {}.
pub fn builtin_walax(args: Vec<Value>) -> Result<Value, SoplangError> {
    if args.len() % 2 != 0 {
        return Err(err_type(
            "walax() wuxuu u baahan yahay fure iyo qiime is raaca",
        ));
    }
    let mut m = HashMap::new();
    for pair in args.chunks_exact(2) {
        let Value::Str(key) = &pair[0] else {
            return Err(err_type("Furaha walax() waa in uu noqdo qoraal"));
        };
        m.insert(key.clone(), pair[1].clone());
    }
    Ok(Value::Object(Rc::new(RefCell::new(m))))
}

/// Built-in daji: floor.
pub fn builtin_daji(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "daji", 1)?;
    let v = args.into_iter().next().unwrap();
    let n = to_number(&v)?;
    Ok(Value::Int(n.floor() as i64))
}

/// Built-in kor: ceil.
pub fn builtin_kor(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "kor", 1)?;
    let v = args.into_iter().next().unwrap();
    let n = to_number(&v)?;
    Ok(Value::Int(n.ceil() as i64))
}

/// Built-in dherer: collection, byte, or Unicode scalar length.
pub fn builtin_dherer(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "dherer", 1)?;
    let v = args.into_iter().next().unwrap();
    let len = match &v {
        Value::List(l) => l.borrow().len(),
        Value::Tuple(tuple) => tuple.len(),
        Value::Set(set) => set.len(),
        Value::Bytes(bytes) => super::bytes::read(bytes)?.len(),
        Value::Str(s) => s.chars().count(),
        Value::Object(o) => o.borrow().len(),
        _ => {
            return Err(err_type(
                "Qiimaha ma ahan teed, qoraal, walax, baytyo, xirmo, ama urur",
            ))
        }
    };
    Ok(Value::Int(len as i64))
}

/// Built-in xul: random float [0,1), or random in [a,b], or random choice from list.
pub fn builtin_xul(args: Vec<Value>) -> Result<Value, SoplangError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 32) as f64 / (1u64 << 32) as f64
    };
    match args.len() {
        0 => Ok(Value::Float(next())),
        1 => {
            let list = match &args[0] {
                Value::List(l) => l.borrow().clone(),
                _ => return Err(err_type("Qiimaha ma ahan teed")),
            };
            if list.is_empty() {
                return Err(err_runtime("teedku waa madhan yahay"));
            }
            let i = (next() * list.len() as f64).floor() as usize;
            Ok(list[i].clone())
        }
        2 => {
            let a = to_number(&args[0])?;
            let b = to_number(&args[1])?;
            if a > b {
                return Err(err_runtime(
                    "Qiimaha koowaad waa in uu ka yar yahay ama la mid yahay qiimaha labaad",
                ));
            }
            let x = a + next() * (b - a);
            Ok(Value::Float(x))
        }
        _ => Err(err_type("Hawsha 'xul()' waxay qaadataa 0, 1, ama 2 dood")),
    }
}

/// Built-in baaxad: range as list (1, 2, or 3 numeric args).
pub fn builtin_baaxad(args: Vec<Value>) -> Result<Value, SoplangError> {
    let (start, stop, step) = match args.len() {
        1 => (0i64, to_integer(&args[0], "baaxad")?, 1i64),
        2 => (
            to_integer(&args[0], "baaxad")?,
            to_integer(&args[1], "baaxad")?,
            1i64,
        ),
        3 => (
            to_integer(&args[0], "baaxad")?,
            to_integer(&args[1], "baaxad")?,
            to_integer(&args[2], "baaxad")?,
        ),
        _ => return Err(err_type("Hawsha 'baaxad()' waxay qaadataa 1 ilaa 3 dood")),
    };
    if step == 0 {
        return Err(err_runtime("Tallaabada baaxad() eber ma noqon karto"));
    }
    let mut out = Vec::new();
    let mut i = start;
    while (step > 0 && i < stop) || (step < 0 && i > stop) {
        out.push(Value::Int(i));
        // Once the next value leaves i64, it is also past every valid bound.
        let Some(next) = i.checked_add(step) else { break };
        i = next;
    }
    Ok(Value::List(Rc::new(RefCell::new(out))))
}

pub fn builtin_dhammeystir(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "dhammeystir", 1)?;
    Ok(Value::Int(to_number(&args[0])?.round() as i64))
}

pub fn builtin_sugan(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "sugan", 1)?;
    match args[0] {
        Value::Int(n) => n
            .checked_abs()
            .map(Value::Int)
            .ok_or_else(|| err_runtime("abnku aad buu u weyn yahay")),
        Value::Float(x) => Ok(Value::Float(x.abs())),
        _ => Err(err_type("sugan() wuxuu u baahan yahay abn ama jajab")),
    }
}

fn numeric_extreme(args: Vec<Value>, name: &str, minimum: bool) -> Result<Value, SoplangError> {
    if args.is_empty() {
        return Err(err_type(format!(
            "Hawsha '{}()' waxay u baahan tahay ugu yaraan 1 dood",
            name
        )));
    }
    let all_int = args.iter().all(|v| matches!(v, Value::Int(_)));
    let mut best = to_number(&args[0])?;
    for value in &args[1..] {
        let number = to_number(value)?;
        if (minimum && number < best) || (!minimum && number > best) {
            best = number;
        }
    }
    if all_int {
        Ok(Value::Int(best as i64))
    } else {
        Ok(Value::Float(best))
    }
}

pub fn builtin_ugu_yar(args: Vec<Value>) -> Result<Value, SoplangError> {
    numeric_extreme(args, "ugu_yar", true)
}

pub fn builtin_ugu_weyn(args: Vec<Value>) -> Result<Value, SoplangError> {
    numeric_extreme(args, "ugu_weyn", false)
}

pub fn builtin_awood(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "awood", 2)?;
    Ok(Value::Float(
        to_number(&args[0])?.powf(to_number(&args[1])?),
    ))
}

pub fn builtin_duub(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "duub", 1)?;
    builtin_dhammeystir(args)
}

pub fn builtin_yar(args: Vec<Value>) -> Result<Value, SoplangError> {
    numeric_extreme(args, "yar", true)
}

pub fn builtin_weyn(args: Vec<Value>) -> Result<Value, SoplangError> {
    numeric_extreme(args, "weyn", false)
}

pub fn builtin_jibbaar(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "jibbaar", 2)?;
    builtin_awood(args)
}

pub fn builtin_xidid(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "xidid", 1)?;
    let value = to_number(&args[0])?;
    if value < 0.0 {
        return Err(err_runtime("xidid() ma qaadan karo tiro taban"));
    }
    Ok(Value::Float(value.sqrt()))
}

/// Native callback argument contract. A stride represents pairwise arguments.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CallbackArity {
    pub minimum: usize,
    pub maximum: usize,
    pub stride: usize,
}

impl CallbackArity {
    const fn range(minimum: usize, maximum: usize) -> Self {
        Self {
            minimum,
            maximum,
            stride: 1,
        }
    }

    const fn pairs() -> Self {
        Self {
            minimum: 0,
            maximum: usize::MAX,
            stride: 2,
        }
    }

    pub fn accepts(self, count: usize) -> bool {
        count >= self.minimum && count <= self.maximum && (count - self.minimum) % self.stride == 0
    }

    pub fn description(self) -> String {
        if self.stride == 2 {
            "tiro dhaban ah oo doodo ah".to_string()
        } else {
            format!("{} dood", crate::error::argument_count(self.minimum, self.maximum))
        }
    }
}

// Registration and callback metadata use the same list, including aliases.
const BUILTINS: &[(&str, super::NativeFn, CallbackArity)] = &[
    (
        "urur",
        super::sets::construct,
        CallbackArity::range(0, usize::MAX),
    ),
    (
        "xirmo",
        super::tuples::construct,
        CallbackArity::range(0, usize::MAX),
    ),
    (
        "baytyo",
        super::bytes::construct,
        CallbackArity::range(0, 1),
    ),
    ("qor", builtin_qor, CallbackArity::range(0, usize::MAX)),
    ("gelin", builtin_gelin, CallbackArity::range(0, 1)),
    ("nooc", builtin_nooc, CallbackArity::range(1, 1)),
    ("khalad", builtin_khalad, CallbackArity::range(1, 3)),
    ("qaabee", builtin_qaabee, CallbackArity::range(1, 2)),
    (
        "wareeg",
        super::iterators::builtin_wareeg,
        CallbackArity::range(1, 1),
    ),
    (
        "xiga",
        super::iterators::builtin_xiga,
        CallbackArity::range(1, 1),
    ),
    (
        "taxane",
        super::iterators::builtin_taxane,
        CallbackArity::range(1, 3),
    ),
    (
        "tirso",
        super::iterators::builtin_tirso,
        CallbackArity::range(1, 2),
    ),
    (
        "lamaane",
        super::iterators::builtin_lamaane,
        CallbackArity::range(0, usize::MAX),
    ),
    ("abn", builtin_abn, CallbackArity::range(1, 1)),
    ("jajab", builtin_jajab, CallbackArity::range(1, 1)),
    ("qoraal", builtin_qoraal, CallbackArity::range(1, 1)),
    ("bool", builtin_bool, CallbackArity::range(1, 1)),
    ("teed", builtin_teed, CallbackArity::range(0, usize::MAX)),
    ("walax", builtin_walax, CallbackArity::pairs()),
    ("daji", builtin_daji, CallbackArity::range(1, 1)),
    ("kor", builtin_kor, CallbackArity::range(1, 1)),
    ("dherer", builtin_dherer, CallbackArity::range(1, 1)),
    ("xul", builtin_xul, CallbackArity::range(0, 2)),
    ("baaxad", builtin_baaxad, CallbackArity::range(1, 3)),
    (
        "dhammeystir",
        builtin_dhammeystir,
        CallbackArity::range(1, 1),
    ),
    ("sugan", builtin_sugan, CallbackArity::range(1, 1)),
    (
        "ugu_yar",
        builtin_ugu_yar,
        CallbackArity::range(1, usize::MAX),
    ),
    (
        "ugu_weyn",
        builtin_ugu_weyn,
        CallbackArity::range(1, usize::MAX),
    ),
    ("yar", builtin_yar, CallbackArity::range(1, usize::MAX)),
    ("weyn", builtin_weyn, CallbackArity::range(1, usize::MAX)),
    ("awood", builtin_awood, CallbackArity::range(2, 2)),
    ("jibbaar", builtin_jibbaar, CallbackArity::range(2, 2)),
    ("duub", builtin_duub, CallbackArity::range(1, 1)),
    ("xidid", builtin_xidid, CallbackArity::range(1, 1)),
];

pub(crate) fn builtin_callback_arity(function: super::NativeFn) -> Option<CallbackArity> {
    BUILTINS
        .iter()
        .find(|(_, candidate, _)| std::ptr::fn_addr_eq(*candidate, function))
        .map(|(_, _, arity)| *arity)
}

pub fn get_builtin_functions() -> HashMap<String, Value> {
    BUILTINS
        .iter()
        .map(|(name, function, _)| (name.to_string(), Value::NativeFunction(*function)))
        .collect()
}

// ---------- List methods (kasaar=pop, dherer=length, kudar=concat/push, leeyahay, nuqul, nadiifi, rog, habee, jar, muuji) ----------

pub fn list_kasaar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kasaar", 0)?;
    mutate_list(&lst, |items| {
        items
            .pop()
            .ok_or_else(|| err_runtime("Ma saari kartid teed madhan"))
    })
}

pub fn list_dherer(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "dherer", 0)?;
    Ok(Value::Int(lst.borrow().len() as i64))
}

fn lookup_position(value: &Value, len: usize) -> Result<Option<usize>, SoplangError> {
    let index = to_integer(value, "hel")?;
    let len = len as i64;
    let index = if index < 0 { len + index } else { index };
    Ok((index >= 0 && index < len).then_some(index as usize))
}

pub fn list_hel(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "hel", 1, 2)?;
    let items = lst.borrow();
    Ok(lookup_position(&args[0], items.len())?
        .and_then(|index| items.get(index).cloned())
        .unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::Null)))
}

pub fn list_kudar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kudar", 1)?;
    let second = args[0].clone();
    if let Value::List(other) = &second {
        let mut v = lst.borrow().clone();
        v.extend(other.borrow().iter().cloned());
        Ok(Value::List(Rc::new(RefCell::new(v))))
    } else {
        mutate_list(&lst, |items| {
            items.push(second);
            Ok(())
        })?;
        Ok(Value::List(Rc::clone(&lst)))
    }
}

pub fn list_midee(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "midee", 1)?;
    let Value::List(other) = &args[0] else {
        return Err(err_type("midee() waxay u baahan tahay teed"));
    };
    let mut result = lst.borrow().clone();
    result.extend(other.borrow().iter().cloned());
    Ok(Value::List(Rc::new(RefCell::new(result))))
}

pub fn list_leeyahay(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "leeyahay", 1)?;
    let item = args[0].clone();
    let v = lst.borrow();
    Ok(Value::Bool(v.contains(&item)))
}

pub fn list_nuqul(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "nuqul", 0)?;
    Ok(Value::List(Rc::new(RefCell::new(lst.borrow().clone()))))
}

pub fn list_nadiifi(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "nadiifi", 0)?;
    mutate_list(&lst, |items| {
        items.clear();
        Ok(())
    })?;
    Ok(Value::List(Rc::clone(&lst)))
}

pub fn list_rog(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "rog", 0)?;
    mutate_list(&lst, |items| {
        items.reverse();
        Ok(())
    })?;
    Ok(Value::List(Rc::clone(&lst)))
}

fn value_cmp_for_sort(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.total_cmp(b),
        (Value::Int(a), Value::Float(b)) => (*a as f64).total_cmp(b),
        (Value::Float(a), Value::Int(b)) => a.total_cmp(&(*b as f64)),
        (Value::Str(a), Value::Str(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        _ => value_to_string(a).cmp(&value_to_string(b)),
    }
}

pub fn list_habee(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "habee", 0)?;
    let mut sorted = lst
        .try_borrow()
        .map_err(|_| err_runtime("Teedka hadda lama akhriyi karo."))?
        .clone();
    sorted.sort_by(value_cmp_for_sort);
    mutate_list(&lst, |items| {
        *items = sorted;
        Ok(())
    })?;
    Ok(Value::List(Rc::clone(&lst)))
}

/// Shared character/item slicing: integer bounds, negative offsets, clamped
/// endpoints, and an empty result when the normalized end precedes the start.
pub(super) fn slice_bounds(args: &[Value], len: usize) -> Result<(usize, usize), SoplangError> {
    expect_arity_range(args, "jar", 0, 2)?;
    let normalize = |bound: &Value| -> Result<usize, SoplangError> {
        let index = to_integer(bound, "jar")?;
        let len = len as i64;
        Ok((if index < 0 { len + index } else { index }).clamp(0, len) as usize)
    };
    let start = args.first().map(normalize).transpose()?.unwrap_or(0);
    let end = args.get(1).map(normalize).transpose()?.unwrap_or(len);
    Ok((start, end.max(start)))
}

pub fn list_jar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    let items = lst.borrow();
    let (start, end) = slice_bounds(args, items.len())?;
    Ok(Value::List(Rc::new(RefCell::new(
        items[start..end].to_vec(),
    ))))
}

pub fn list_tiri(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "tiri", 1)?;
    Ok(Value::Int(
        lst.borrow().iter().filter(|item| *item == &args[0]).count() as i64,
    ))
}

pub fn list_muuji(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "muuji", 1)?;
    let item = args[0].clone();
    let v = lst.borrow();
    for (i, x) in v.iter().enumerate() {
        if x == &item {
            return Ok(Value::Int(i as i64));
        }
    }
    Ok(Value::Null)
}

fn list_position(
    value: &Value,
    len: usize,
    allow_end: bool,
    name: &str,
) -> Result<usize, SoplangError> {
    let raw = to_integer(value, name)?;
    let len = len as i64;
    let position = if raw < 0 { len + raw } else { raw };
    let upper = if allow_end { len } else { len - 1 };
    if position < 0 || position > upper {
        return Err(err_runtime(format!(
            "Tusmada {}() way ka baxsan tahay teedka",
            name
        )));
    }
    Ok(position as usize)
}

pub fn list_geli(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "geli", 1, 2)?;
    mutate_list(&lst, |items| {
        if args.len() == 1 {
            items.push(args[0].clone());
        } else {
            let index = list_position(&args[0], items.len(), true, "geli")?;
            items.insert(index, args[1].clone());
        }
        Ok(())
    })?;
    Ok(Value::List(Rc::clone(&lst)))
}

pub fn list_dhig(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "dhig", 2)?;
    mutate_list(&lst, |items| {
        let index = list_position(&args[0], items.len(), false, "dhig")?;
        items[index] = args[1].clone();
        Ok(())
    })?;
    Ok(Value::List(lst))
}

pub fn list_ka_saar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "ka_saar", 1)?;
    mutate_list(&lst, |items| {
        let index = list_position(&args[0], items.len(), false, "ka_saar")?;
        Ok(items.remove(index))
    })
}

pub fn list_saar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "saar", 0, 1)?;
    if args.is_empty() {
        return list_kasaar(lst, args);
    }
    mutate_list(&lst, |items| {
        let index = list_position(&args[0], items.len(), false, "saar")?;
        Ok(items.remove(index))
    })
}

pub fn list_koowaad(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "koowaad", 0)?;
    Ok(lst.borrow().first().cloned().unwrap_or(Value::Null))
}

pub fn list_ugu_dambeeya(
    lst: Rc<RefCell<Vec<Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "ugu_dambeeya", 0)?;
    Ok(lst.borrow().last().cloned().unwrap_or(Value::Null))
}

// ---------- Object methods ----------

pub fn object_dherer(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "dherer", 0)?;
    Ok(Value::Int(obj.borrow().len() as i64))
}

pub fn object_fure(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "fure", 0)?;
    let mut keys: Vec<String> = obj.borrow().keys().cloned().collect();
    keys.sort();
    let keys: Vec<Value> = keys.into_iter().map(Value::Str).collect();
    Ok(Value::List(Rc::new(RefCell::new(keys))))
}

pub fn object_leeyahay(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "leeyahay", 1)?;
    let key = args
        .first()
        .and_then(|v| {
            if let Value::Str(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| err_type("Furaha waa in uu noqdo qoraal"))?;
    Ok(Value::Bool(obj.borrow().contains_key(&key)))
}

pub fn object_tir(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "tir", 1)?;
    let key = args
        .first()
        .and_then(|v| {
            if let Value::Str(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| err_type("Furaha waa in uu noqdo qoraal"))?;
    mutate_object(&obj, |items| {
        items.remove(&key);
        Ok(())
    })?;
    Ok(Value::Object(Rc::clone(&obj)))
}

pub fn object_kudar(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "kudar", 1)?;
    let other = args
        .first()
        .and_then(|v| {
            if let Value::Object(o) = v {
                Some(Rc::clone(o))
            } else {
                None
            }
        })
        .ok_or_else(|| err_type("Qiimaha labaad ma ahan walax"))?;
    let mut m = obj.borrow().clone();
    for (k, v) in other.borrow().iter() {
        m.insert(k.clone(), v.clone());
    }
    Ok(Value::Object(Rc::new(RefCell::new(m))))
}

pub fn object_nuqul(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "nuqul", 0)?;
    Ok(Value::Object(Rc::new(RefCell::new(obj.borrow().clone()))))
}

pub fn object_nadiifi(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "nadiifi", 0)?;
    mutate_object(&obj, |items| {
        items.clear();
        Ok(())
    })?;
    Ok(Value::Object(Rc::clone(&obj)))
}

pub fn object_qiime(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "qiime", 0)?;
    let m = obj.borrow();
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    let vals: Vec<Value> = keys.iter().map(|k| m.get(k).cloned().unwrap()).collect();
    Ok(Value::List(Rc::new(RefCell::new(vals))))
}

pub fn object_lamaane(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "lamaane", 0)?;
    let m = obj.borrow();
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    let pairs: Vec<Value> = keys
        .iter()
        .map(|k| {
            Value::List(Rc::new(RefCell::new(vec![
                Value::Str(k.clone()),
                m.get(k).cloned().unwrap(),
            ])))
        })
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(pairs))))
}

fn object_key(value: &Value) -> Result<&str, SoplangError> {
    match value {
        Value::Str(key) => Ok(key),
        _ => Err(err_type("Furaha waa in uu noqdo qoraal")),
    }
}

pub fn object_hel(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity_range(args, "hel", 1, 2)?;
    let key = object_key(&args[0])?;
    Ok(obj
        .borrow()
        .get(key)
        .cloned()
        .unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::Null)))
}

pub fn object_dhig(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    expect_arity(args, "dhig", 2)?;
    let key = object_key(&args[0])?.to_string();
    mutate_object(&obj, |items| {
        items.insert(key, args[1].clone());
        Ok(())
    })?;
    Ok(Value::Object(Rc::clone(&obj)))
}

pub fn object_ka_qaad(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    object_remove(obj, args, "ka_qaad")
}

pub fn object_saar(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
) -> Result<Value, SoplangError> {
    object_remove(obj, args, "saar")
}

fn object_remove(
    obj: Rc<RefCell<HashMap<String, Value>>>,
    args: &[Value],
    name: &str,
) -> Result<Value, SoplangError> {
    expect_arity(args, name, 1)?;
    let key = object_key(&args[0])?;
    mutate_object(&obj, |items| Ok(items.remove(key).unwrap_or(Value::Null)))
}

// ---------- String methods ----------

pub fn string_qeybi(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "qeybi", 1)?;
    let delim = value_to_string(&args[0]);
    let parts: Vec<Value> = s.split(&delim).map(|p| Value::Str(p.to_string())).collect();
    Ok(Value::List(Rc::new(RefCell::new(parts))))
}

pub fn string_leeyahay(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "leeyahay", 1)?;
    let sub = value_to_string(&args[0]);
    Ok(Value::Bool(s.contains(&sub)))
}

pub fn string_dhamaad(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "dhamaad", 1)?;
    let suffix = value_to_string(&args[0]);
    Ok(Value::Bool(s.ends_with(&suffix)))
}

pub fn string_bilow(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "bilow", 1)?;
    let prefix = value_to_string(&args[0]);
    Ok(Value::Bool(s.starts_with(&prefix)))
}

pub fn string_beddel(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "beddel", 2, 3)?;
    let (target, repl) = (value_to_string(&args[0]), value_to_string(&args[1]));
    let new_s = match args.get(2) {
        None => s.replacen(&target, &repl, 1),
        Some(Value::Null) => s.replace(&target, &repl),
        Some(count) => {
            let count = to_integer(count, "beddel")?;
            if count < 0 {
                return Err(err_type("beddel() tirada beddelku ma noqon karto taban"));
            }
            s.replacen(&target, &repl, count as usize)
        }
    };
    Ok(Value::Str(new_s))
}

pub fn string_beddel_dhammaan(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "beddel_dhammaan", 2)?;
    let (target, repl) = (value_to_string(&args[0]), value_to_string(&args[1]));
    Ok(Value::Str(s.replace(&target, &repl)))
}

pub fn string_kudar(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kudar", 1)?;
    let list = args
        .first()
        .and_then(|v| {
            if let Value::List(l) = v {
                Some(l.borrow().clone())
            } else {
                None
            }
        })
        .ok_or_else(|| err_type("Qiimaha labaad ma ahan teed"))?;
    let parts: Vec<String> = list.iter().map(|v| value_to_string(v)).collect();
    Ok(Value::Str(parts.join(&s)))
}

pub fn string_jar(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    let chars: Vec<char> = s.chars().collect();
    let (start, end) = slice_bounds(args, chars.len())?;
    Ok(Value::Str(chars[start..end].iter().collect()))
}

pub fn string_hore(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "hore", 0)?;
    Ok(s.chars()
        .next()
        .map(|ch| Value::Str(ch.to_string()))
        .unwrap_or(Value::Null))
}

pub fn string_dambe(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "dambe", 0)?;
    Ok(s.chars()
        .next_back()
        .map(|ch| Value::Str(ch.to_string()))
        .unwrap_or(Value::Null))
}

pub fn string_xarfaha_waaweyn(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "xarfaha_waaweyn", 0)?;
    Ok(Value::Str(s.to_uppercase()))
}

pub fn string_xarfaha_yaryar(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "xarfaha_yaryar", 0)?;
    Ok(Value::Str(s.to_lowercase()))
}

pub fn string_masax(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "masax", 0, 1)?;
    let result = match args.first() {
        None => s.trim(),
        Some(Value::Str(direction)) if direction == "hore" => s.trim_start(),
        Some(Value::Str(direction)) if direction == "dambe" => s.trim_end(),
        _ => {
            return Err(err_type(
                "masax() jihadu waa inay noqotaa 'hore' ama 'dambe'",
            ))
        }
    };
    Ok(Value::Str(result.to_string()))
}

pub fn string_raadi(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "raadi", 1, 2)?;
    let sub = value_to_string(&args[0]);
    let found = match args.get(1) {
        None => s.find(&sub),
        Some(Value::Str(direction)) if direction == "hore" => s.find(&sub),
        Some(Value::Str(direction)) if direction == "dambe" => s.rfind(&sub),
        _ => {
            return Err(err_type(
                "raadi() jihadu waa inay noqotaa 'hore' ama 'dambe'",
            ))
        }
    };
    let pos = found
        .map(|byte_index| s[..byte_index].chars().count() as i64)
        .unwrap_or(-1);
    Ok(Value::Int(pos))
}

pub fn string_dherer(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "dherer", 0)?;
    Ok(Value::Int(s.chars().count() as i64))
}

pub fn string_hel(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity_range(args, "hel", 1, 2)?;
    Ok(lookup_position(&args[0], s.chars().count())?
        .and_then(|index| s.chars().nth(index))
        .map(|ch| Value::Str(ch.to_string()))
        .unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::Null)))
}

pub fn string_masax_bilow(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "masax_bilow", 0)?;
    Ok(Value::Str(s.trim_start().to_string()))
}

pub fn string_masax_dhamaad(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "masax_dhamaad", 0)?;
    Ok(Value::Str(s.trim_end().to_string()))
}

pub fn string_tiri(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "tiri", 1)?;
    let needle = value_to_string(&args[0]);
    if needle.is_empty() {
        return Err(err_type(
            "tiri() qoraalka la raadinayo madhan ma noqon karo",
        ));
    }
    Ok(Value::Int(s.matches(&needle).count() as i64))
}

pub fn string_kuceli(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    repeat_string(s, args, "kuceli")
}

pub fn string_ku_celi(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    repeat_string(s, args, "ku_celi")
}

fn repeat_string(s: String, args: &[Value], name: &str) -> Result<Value, SoplangError> {
    expect_arity(args, name, 1)?;
    let count = to_integer(&args[0], name)?;
    if count < 0 {
        return Err(err_runtime(format!(
            "{}() tiradiisu taban ma noqon karto",
            name
        )));
    }
    Ok(Value::Str(s.repeat(count as usize)))
}

pub fn string_raadi_dambe(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "raadi_dambe", 1)?;
    let needle = value_to_string(&args[0]);
    let position = s
        .rfind(&needle)
        .map(|byte_index| s[..byte_index].chars().count() as i64)
        .unwrap_or(-1);
    Ok(Value::Int(position))
}
