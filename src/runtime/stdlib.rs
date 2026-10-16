//! Standard library: built-in functions and method dispatch (Phase 5).
//! Built-in functions and methods for Soplang.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

use crate::error::{runtime_error, type_error, SoplangError};
use super::value::{value_to_string, Value};

fn err_type(msg: impl Into<String>) -> SoplangError {
    type_error(msg, 0, 0)
}
fn err_runtime(msg: impl Into<String>) -> SoplangError {
    runtime_error(msg, 0, 0)
}

fn expect_arity(args: &[Value], name: &str, expected: usize) -> Result<(), SoplangError> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(err_type(format!(
            "{}() waxay qaadataa {} qiimo, laakiin {} ayaa la bixiyay",
            name,
            expected,
            args.len()
        )))
    }
}

fn to_number(v: &Value) -> Result<f64, SoplangError> {
    match v {
        Value::Int(n) => Ok(*n as f64),
        Value::Float(x) => Ok(*x),
        _ => Err(err_type(format!("{} ma ahan abn ama jajab", value_to_string(v)))),
    }
}

/// Built-in qor: print value as qoraal string, return that string.
pub fn builtin_qor(args: Vec<Value>) -> Result<Value, SoplangError> {
    let s = value_to_string(&args.into_iter().next().unwrap_or(Value::Null));
    println!("{}", s);
    Ok(Value::Str(s))
}

/// Built-in gelin: read line from stdin (optional prompt).
pub fn builtin_gelin(args: Vec<Value>) -> Result<Value, SoplangError> {
    let prompt = args
        .first()
        .map(|v| value_to_string(v))
        .unwrap_or_default();
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| err_runtime(e.to_string()))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| err_runtime(e.to_string()))?;
    if line.ends_with('\n') {
        line.pop();
    }
    Ok(Value::Str(line))
}

/// Built-in nooc: return type name string.
pub fn builtin_nooc(args: Vec<Value>) -> Result<Value, SoplangError> {
    let v = args.into_iter().next().unwrap_or(Value::Null);
    Ok(Value::Str(v.type_name().to_string()))
}

/// Built-in abn: convert to integer.
pub fn builtin_abn(args: Vec<Value>) -> Result<Value, SoplangError> {
    let v = args.into_iter().next().unwrap_or(Value::Null);
    let n = to_number(&v).map_err(|_| err_type(format!("{} ma badali karo abn", value_to_string(&v))))?;
    Ok(Value::Int(n as i64))
}

/// Built-in jajab: convert to float.
pub fn builtin_jajab(args: Vec<Value>) -> Result<Value, SoplangError> {
    let v = args.into_iter().next().unwrap_or(Value::Null);
    let x = to_number(&v).map_err(|_| err_type(format!("{} ma badali karo jajab", value_to_string(&v))))?;
    Ok(Value::Float(x))
}

/// Built-in qoraal: convert to string (value_to_string).
pub fn builtin_qoraal(args: Vec<Value>) -> Result<Value, SoplangError> {
    let v = args.into_iter().next().unwrap_or(Value::Null);
    Ok(Value::Str(value_to_string(&v)))
}

/// Built-in bool: truthiness (0, "", false, null, "false", "False" -> false).
pub fn builtin_bool(args: Vec<Value>) -> Result<Value, SoplangError> {
    let v = args.into_iter().next().unwrap_or(Value::Null);
    let b = match &v {
        Value::Null => false,
        Value::Bool(x) => *x,
        Value::Int(n) => *n != 0,
        Value::Float(x) => *x != 0.0,
        Value::Str(s) => !s.is_empty() && s != "false" && s != "False",
        Value::List(l) => !l.borrow().is_empty(),
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
    let mut m = HashMap::new();
    // Optional: if args are alternating key (Str) and value, build object
    let mut i = 0;
    while i + 1 < args.len() {
        if let Value::Str(k) = &args[i] {
            m.insert(k.clone(), args[i + 1].clone());
            i += 2;
        } else {
            break;
        }
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

/// Built-in dherer: length of list, string, or object.
pub fn builtin_dherer(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "dherer", 1)?;
    let v = args.into_iter().next().unwrap();
    let len = match &v {
        Value::List(l) => l.borrow().len(),
        Value::Str(s) => s.chars().count(),
        Value::Object(o) => o.borrow().len(),
        _ => return Err(err_type("Qiimaha ma ahan teed, qoraal, ama walax")),
    };
    Ok(Value::Int(len as i64))
}

/// Built-in xul: random float [0,1), or random in [a,b], or random choice from list.
pub fn builtin_xul(args: Vec<Value>) -> Result<Value, SoplangError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
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
        _ => Err(err_type("xul() waxay qaadataa 0, 1, ama 2 qiimo")),
    }
}

/// Built-in baaxad: range as list (1, 2, or 3 numeric args).
pub fn builtin_baaxad(args: Vec<Value>) -> Result<Value, SoplangError> {
    let (start, stop, step) = match args.len() {
        1 => (0i64, to_number(&args[0])? as i64, 1i64),
        2 => (
            to_number(&args[0])? as i64,
            to_number(&args[1])? as i64,
            1i64,
        ),
        3 => (
            to_number(&args[0])? as i64,
            to_number(&args[1])? as i64,
            to_number(&args[2])? as i64,
        ),
        _ => return Err(err_type("baaxad() waxay qaadataa 1 ilaa 3 qiimo")),
    };
    let mut out = Vec::new();
    let mut i = start;
    while (step > 0 && i < stop) || (step < 0 && i > stop) {
        out.push(Value::Int(i));
        i += step;
    }
    Ok(Value::List(Rc::new(RefCell::new(out))))
}

pub fn get_builtin_functions() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("qor".into(), Value::NativeFunction(builtin_qor));
    m.insert("gelin".into(), Value::NativeFunction(builtin_gelin));
    m.insert("nooc".into(), Value::NativeFunction(builtin_nooc));
    m.insert("abn".into(), Value::NativeFunction(builtin_abn));
    m.insert("jajab".into(), Value::NativeFunction(builtin_jajab));
    m.insert("qoraal".into(), Value::NativeFunction(builtin_qoraal));
    m.insert("bool".into(), Value::NativeFunction(builtin_bool));
    m.insert("teed".into(), Value::NativeFunction(builtin_teed));
    m.insert("walax".into(), Value::NativeFunction(builtin_walax));
    m.insert("daji".into(), Value::NativeFunction(builtin_daji));
    m.insert("kor".into(), Value::NativeFunction(builtin_kor));
    m.insert("dherer".into(), Value::NativeFunction(builtin_dherer));
    m.insert("xul".into(), Value::NativeFunction(builtin_xul));
    m.insert("baaxad".into(), Value::NativeFunction(builtin_baaxad));
    m
}

// ---------- List methods (kasaar=pop, dherer=length, kudar=concat/push, leeyahay, nuqul, nadiifi, rog, habee, jar, muuji) ----------

pub fn list_kasaar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kasaar", 0)?;
    let mut v = lst.borrow_mut();
    if v.is_empty() {
        return Err(err_runtime("Ma saari kartid teed madhan"));
    }
    Ok(v.pop().unwrap())
}

pub fn list_dherer(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "dherer", 0)?;
    Ok(Value::Int(lst.borrow().len() as i64))
}

pub fn list_kudar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kudar", 1)?;
    let second = args[0].clone();
    if let Value::List(other) = &second {
        let mut v = lst.borrow().clone();
        v.extend(other.borrow().iter().cloned());
        Ok(Value::List(Rc::new(RefCell::new(v))))
    } else {
        lst.borrow_mut().push(second);
        Ok(Value::List(Rc::clone(&lst)))
    }
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
    lst.borrow_mut().clear();
    Ok(Value::List(Rc::clone(&lst)))
}

pub fn list_rog(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "rog", 0)?;
    lst.borrow_mut().reverse();
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
    lst.borrow_mut().sort_by(value_cmp_for_sort);
    Ok(Value::List(Rc::clone(&lst)))
}

pub fn list_jar(lst: Rc<RefCell<Vec<Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "jar", 2)?;
    let start = to_number(&args[0]).map_err(|_| err_type("Bilowga iyo dhamaadka waa inay noqdaan abn"))? as i64;
    let end = to_number(&args[1]).map_err(|_| err_type("Bilowga iyo dhamaadka waa inay noqdaan abn"))? as i64;
    let v = lst.borrow();
    let len = v.len() as i64;
    let mut start = start;
    let mut end = end;
    if start < 0 {
        start = (len + start).max(0);
    }
    start = start.min(len);
    if end < 0 {
        end = (len + end).max(0);
    }
    end = end.min(len);
    let start = start as usize;
    let end = end as usize;
    let slice: Vec<Value> = if start >= end { Vec::new() } else { v[start..end].to_vec() };
    Ok(Value::List(Rc::new(RefCell::new(slice))))
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

// ---------- Object methods ----------

pub fn object_fure(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "fure", 0)?;
    let mut keys: Vec<String> = obj.borrow().keys().cloned().collect();
    keys.sort();
    let keys: Vec<Value> = keys.into_iter().map(Value::Str).collect();
    Ok(Value::List(Rc::new(RefCell::new(keys))))
}

pub fn object_leeyahay(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "leeyahay", 1)?;
    let key = args.first().and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None }).ok_or_else(|| err_type("Furaha waa in uu noqdo qoraal"))?;
    Ok(Value::Bool(obj.borrow().contains_key(&key)))
}

pub fn object_tir(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "tir", 1)?;
    let key = args.first().and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None }).ok_or_else(|| err_type("Furaha waa in uu noqdo qoraal"))?;
    obj.borrow_mut().remove(&key);
    Ok(Value::Object(Rc::clone(&obj)))
}

pub fn object_kudar(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kudar", 1)?;
    let other = args.first().and_then(|v| if let Value::Object(o) = v { Some(Rc::clone(o)) } else { None }).ok_or_else(|| err_type("Qiimaha labaad ma ahan walax"))?;
    let mut m = obj.borrow().clone();
    for (k, v) in other.borrow().iter() {
        m.insert(k.clone(), v.clone());
    }
    Ok(Value::Object(Rc::new(RefCell::new(m))))
}

pub fn object_nuqul(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "nuqul", 0)?;
    Ok(Value::Object(Rc::new(RefCell::new(obj.borrow().clone()))))
}

pub fn object_nadiifi(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "nadiifi", 0)?;
    obj.borrow_mut().clear();
    Ok(Value::Object(Rc::clone(&obj)))
}

pub fn object_qiime(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "qiime", 0)?;
    let m = obj.borrow();
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    let vals: Vec<Value> = keys.iter().map(|k| m.get(k).cloned().unwrap()).collect();
    Ok(Value::List(Rc::new(RefCell::new(vals))))
}

pub fn object_lamaane(obj: Rc<RefCell<HashMap<String, Value>>>, args: &[Value]) -> Result<Value, SoplangError> {
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
    expect_arity(args, "beddel", 2)?;
    let (target, repl) = (value_to_string(&args[0]), value_to_string(&args[1]));
    let new_s = s.replacen(&target, &repl, 1);
    Ok(Value::Str(new_s))
}

pub fn string_beddel_dhammaan(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "beddel_dhammaan", 2)?;
    let (target, repl) = (value_to_string(&args[0]), value_to_string(&args[1]));
    Ok(Value::Str(s.replace(&target, &repl)))
}

pub fn string_kudar(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "kudar", 1)?;
    let list = args.first().and_then(|v| if let Value::List(l) = v { Some(l.borrow().clone()) } else { None }).ok_or_else(|| err_type("Qiimaha labaad ma ahan teed"))?;
    let parts: Vec<String> = list.iter().map(|v| value_to_string(v)).collect();
    Ok(Value::Str(parts.join(&s)))
}

pub fn string_jar(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    if args.len() > 2 {
        return Err(err_type(format!(
            "jar() waxay qaadataa ugu badnaan 2 qiimo, laakiin {} ayaa la bixiyay",
            args.len()
        )));
    }
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let (start, end) = match args.len() {
        0 => (0i64, len),
        1 => (
            to_number(args.get(0).unwrap()).map(|x| x as i64).map_err(|_| err_type("Bilowga waa inuu noqdaa abn"))?,
            len,
        ),
        _ => (
            to_number(&args[0]).map(|x| x as i64).map_err(|_| err_type("Bilowga waa inuu noqdaa abn"))?,
            to_number(&args[1]).map(|x| x as i64).map_err(|_| err_type("Dhamaadka waa inuu noqdaa abn"))?,
        ),
    };
    let mut start = start;
    let mut end = end;
    if start < 0 {
        start = (len + start).max(0);
    }
    start = start.min(len);
    if end < 0 {
        end = (len + end).max(0);
    }
    end = end.min(len);
    let start = start as usize;
    let end = end as usize;
    Ok(Value::Str(chars[start..end].iter().collect()))
}

pub fn string_xarafaha_weyn(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "xarafaha_weyn", 0)?;
    Ok(Value::Str(s.to_uppercase()))
}

pub fn string_xarfaha_yaryar(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "xarfaha_yaryar", 0)?;
    Ok(Value::Str(s.to_lowercase()))
}

pub fn string_masax(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "masax", 0)?;
    Ok(Value::Str(s.trim().to_string()))
}

pub fn string_raadi(s: String, args: &[Value]) -> Result<Value, SoplangError> {
    expect_arity(args, "raadi", 1)?;
    let sub = value_to_string(&args[0]);
    let pos = s
        .find(&sub)
        .map(|byte_index| s[..byte_index].chars().count() as i64)
        .unwrap_or(-1);
    Ok(Value::Int(pos))
}
