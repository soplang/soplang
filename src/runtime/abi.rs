//! Runtime library: C-ABI functions called by Cranelift and LLVM backends.
//! Phase 3 of COMPILER_PLAN. Converts SoplangValue to/from Value and delegates to stdlib.

#![allow(clippy::cast_lossless, clippy::cast_possible_truncation)]

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::os::raw::c_int;
use std::rc::Rc;

use crate::error::{runtime_error, type_error, SoplangError};
use super::stdlib;
use super::value::{value_to_string, NativeFn, Value};

const TAG_THROW: u8 = 8;

fn fatal_error(e: impl std::fmt::Display) -> ! {
    raise_or_fatal(runtime_error(e.to_string(), 0, 0));
}

fn raise_or_fatal(e: SoplangError) -> ! {
    handle_runtime_error(e);
    unreachable!()
}

fn error_message(e: &SoplangError) -> String {
    match e {
        SoplangError::Runtime { msg, .. } | SoplangError::Type { msg, .. } => msg.clone(),
        _ => e.to_string(),
    }
}

fn throw_value() -> SoplangValue {
    SoplangValue {
        tag: TAG_THROW,
        _pad: [0; 7],
        payload: 0,
    }
}

fn has_pending_error() -> bool {
    PENDING_ERROR.with(|p| p.borrow().is_some())
}

fn short_circuit_if_pending() -> Option<SoplangValue> {
    if has_pending_error() {
        Some(throw_value())
    } else {
        None
    }
}

fn handle_runtime_error(e: SoplangError) -> SoplangValue {
    let msg = error_message(&e);
    if TRY_DEPTH.with(|d| d.get()) > 0 {
        PENDING_ERROR.with(|p| *p.borrow_mut() = Some(msg));
        throw_value()
    } else {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

thread_local! {
    static TRY_DEPTH: Cell<u32> = Cell::new(0);
    static PENDING_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

/// Begin a try block.
#[no_mangle]
pub extern "C" fn soplang_try_begin() -> c_int {
    TRY_DEPTH.with(|d| d.set(d.get().saturating_add(1)));
    0
}

/// End a try block (success or after catch); pop the active handler.
#[no_mangle]
pub extern "C" fn soplang_try_end() {
    TRY_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    PENDING_ERROR.with(|e| *e.borrow_mut() = None);
}

/// Returns 1 if a runtime error is pending inside an active try block.
#[no_mangle]
pub extern "C" fn soplang_has_pending_error() -> c_int {
    if has_pending_error() { 1 } else { 0 }
}

/// Return the caught error message as a Soplang string value.
#[no_mangle]
pub extern "C" fn soplang_get_caught_error() -> SoplangValue {
    let msg = PENDING_ERROR.with(|e| e.borrow().clone().unwrap_or_default());
    value_to_soplang(&Value::Str(msg))
}

// ----- Tag constants (match COMPILER_PLAN) -----
pub const TAG_NULL: u8 = 0;
pub const TAG_INT: u8 = 1;
pub const TAG_FLOAT: u8 = 2;
pub const TAG_BOOL: u8 = 3;
pub const TAG_STR: u8 = 4;
pub const TAG_LIST: u8 = 5;
pub const TAG_OBJECT: u8 = 6;
pub const TAG_FUNC: u8 = 7;

/// Tagged value for C ABI. 16 bytes: tag (u8) + padding + payload (i64).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SoplangValue {
    pub tag:   u8,
    pub _pad:  [u8; 7],
    pub payload: i64,
}

impl Default for SoplangValue {
    fn default() -> Self {
        Self::null()
    }
}

impl SoplangValue {
    pub fn null() -> Self {
        Self { tag: TAG_NULL, _pad: [0; 7], payload: 0 }
    }
}

// ----- Heaps for heap-allocated types (used across C boundary) -----
thread_local! {
    static STR_HEAP: RefCell<Vec<String>> = RefCell::new(Vec::new());
    static LIST_HEAP: RefCell<Vec<Rc<RefCell<Vec<Value>>>>> = RefCell::new(Vec::new());
    static OBJ_HEAP: RefCell<Vec<Rc<RefCell<HashMap<String, Value>>>>> = RefCell::new(Vec::new());
    static NATIVE_FN_TABLE: RefCell<Vec<NativeFn>> = RefCell::new(Vec::new());
    static BUILTIN_INDICES: RefCell<Option<HashMap<String, i64>>> = RefCell::new(None);
    static GLOBAL_VARS: RefCell<HashMap<String, SoplangValue>> = RefCell::new(HashMap::new());
    static COMPILED_FN_TABLE: RefCell<Vec<CompiledFnEntry>> = RefCell::new(Vec::new());
    static CONST_GLOBALS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static CLASS_REGISTRY: RefCell<HashMap<String, ClassRegistryEntry>> = RefCell::new(HashMap::new());
}

const INSTANCE_CLASS_FIELD: &str = "__class__";

#[derive(Clone)]
struct ClassRegistryEntry {
    parent:  Option<String>,
    methods: HashMap<String, i64>,
}

#[derive(Clone, Copy)]
struct CompiledFnEntry {
    ptr: *const u8,
    n_params: usize,
}

fn ensure_builtins() {
    BUILTIN_INDICES.with(|b| {
        if b.borrow().is_none() {
            let builtins = stdlib::get_builtin_functions();
            let mut m = HashMap::new();
            NATIVE_FN_TABLE.with(|t| {
                let mut v = t.borrow_mut();
                for (name, val) in builtins {
                    if let Value::NativeFunction(f) = val {
                        let idx = v.len() as i64;
                        v.push(f);
                        m.insert(name, idx);
                    }
                }
            });
            *b.borrow_mut() = Some(m);
        }
    });
}

fn native_fn_get(idx: i64) -> Option<NativeFn> {
    NATIVE_FN_TABLE.with(|t| {
        let v = t.borrow();
        if idx >= 0 && (idx as usize) < v.len() {
            Some(v[idx as usize])
        } else {
            None
        }
    })
}

fn str_alloc(s: String) -> i64 {
    STR_HEAP.with(|h| {
        let mut v = h.borrow_mut();
        let idx = v.len() as i64;
        v.push(s);
        idx
    })
}

fn str_get(idx: i64) -> String {
    STR_HEAP.with(|h| h.borrow()[idx as usize].clone())
}

fn list_alloc(lst: Rc<RefCell<Vec<Value>>>) -> i64 {
    LIST_HEAP.with(|h| {
        let mut v = h.borrow_mut();
        let idx = v.len() as i64;
        v.push(lst);
        idx
    })
}

fn list_get(idx: i64) -> Rc<RefCell<Vec<Value>>> {
    LIST_HEAP.with(|h| Rc::clone(&h.borrow()[idx as usize]))
}

fn obj_alloc(m: Rc<RefCell<HashMap<String, Value>>>) -> i64 {
    OBJ_HEAP.with(|h| {
        let mut v = h.borrow_mut();
        let idx = v.len() as i64;
        v.push(m);
        idx
    })
}

fn obj_get(idx: i64) -> Rc<RefCell<HashMap<String, Value>>> {
    OBJ_HEAP.with(|h| Rc::clone(&h.borrow()[idx as usize]))
}

// ----- Conversion Value <-> SoplangValue -----
pub fn value_to_soplang(v: &Value) -> SoplangValue {
    match v {
        Value::Null => SoplangValue::null(),
        Value::Int(n) => SoplangValue { tag: TAG_INT, _pad: [0; 7], payload: *n },
        Value::Float(x) => SoplangValue { tag: TAG_FLOAT, _pad: [0; 7], payload: x.to_bits() as i64 },
        Value::Bool(b) => SoplangValue { tag: TAG_BOOL, _pad: [0; 7], payload: if *b { 1 } else { 0 } },
        Value::Str(s) => SoplangValue { tag: TAG_STR, _pad: [0; 7], payload: str_alloc(s.clone()) },
        Value::List(l) => SoplangValue { tag: TAG_LIST, _pad: [0; 7], payload: list_alloc(Rc::clone(l)) },
        Value::Object(o) => SoplangValue { tag: TAG_OBJECT, _pad: [0; 7], payload: obj_alloc(Rc::clone(o)) },
        Value::Function(id) => SoplangValue { tag: TAG_FUNC, _pad: [0; 7], payload: id.0 as i64 },
        Value::NativeFunction(f) => {
            ensure_builtins();
            for (name, v) in stdlib::get_builtin_functions() {
                if let Value::NativeFunction(g) = v {
                    if std::ptr::fn_addr_eq(*f, g) {
                        let idx = BUILTIN_INDICES.with(|b| b.borrow().as_ref().unwrap().get(&name).copied());
                        if let Some(i) = idx {
                            return SoplangValue { tag: TAG_FUNC, _pad: [0; 7], payload: i };
                        }
                    }
                }
            }
            SoplangValue { tag: TAG_FUNC, _pad: [0; 7], payload: -1 }
        }
    }
}

pub fn soplang_to_value(sv: SoplangValue) -> Result<Value, SoplangError> {
    match sv.tag {
        TAG_NULL => Ok(Value::Null),
        TAG_INT => Ok(Value::Int(sv.payload)),
        TAG_FLOAT => Ok(Value::Float(f64::from_bits(sv.payload as u64))),
        TAG_BOOL => Ok(Value::Bool(sv.payload != 0)),
        TAG_STR => Ok(Value::Str(str_get(sv.payload))),
        TAG_LIST => Ok(Value::List(list_get(sv.payload))),
        TAG_OBJECT => Ok(Value::Object(obj_get(sv.payload))),
        TAG_FUNC => Ok(Value::Function(super::value::FunctionId(sv.payload as usize))),
        _ => Err(runtime_error(format!("Invalid tag {}", sv.tag), 0, 0)),
    }
}

fn run_native(name: &str, args: Vec<Value>) -> Result<Value, SoplangError> {
    let builtins = stdlib::get_builtin_functions();
    if let Some(Value::NativeFunction(f)) = builtins.get(name) {
        return f(args);
    }
    Err(runtime_error(format!("Ma aqoonsan: {}", name), 0, 0))
}

// ----- Extern "C" API -----

#[no_mangle]
pub extern "C" fn soplang_int(n: i64) -> SoplangValue {
    value_to_soplang(&Value::Int(n))
}

#[no_mangle]
pub extern "C" fn soplang_float(x: f64) -> SoplangValue {
    value_to_soplang(&Value::Float(x))
}

#[no_mangle]
pub extern "C" fn soplang_str(ptr: *const u8, len: usize) -> SoplangValue {
    let s = if ptr.is_null() || len == 0 {
        String::new()
    } else {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        String::from_utf8_lossy(slice).into_owned()
    };
    value_to_soplang(&Value::Str(s))
}

#[no_mangle]
pub extern "C" fn soplang_bool(b: bool) -> SoplangValue {
    value_to_soplang(&Value::Bool(b))
}

#[no_mangle]
pub extern "C" fn soplang_null() -> SoplangValue {
    SoplangValue::null()
}

fn binop<'a>(
    a: SoplangValue,
    b: SoplangValue,
    f: impl FnOnce(Value, Value) -> Result<Value, SoplangError>,
) -> SoplangValue {
    if let Some(v) = short_circuit_if_pending() {
        return v;
    }
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => match f(va, vb) {
            Ok(v) => value_to_soplang(&v),
            Err(e) => handle_runtime_error(e),
        },
        _ => SoplangValue::null(),
    }
}

fn val_to_str(v: &Value) -> String {
    match v {
        Value::Bool(true) => "run".to_string(),
        Value::Bool(false) => "been".to_string(),
        _ => format!("{}", v),
    }
}

fn add_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n + m)),
        (Value::Int(n), Value::Float(y)) => Ok(Value::Float(*n as f64 + y)),
        (Value::Float(x), Value::Int(m)) => Ok(Value::Float(x + *m as f64)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Str(a), b) => Ok(Value::Str(format!("{}{}", a, val_to_str(b)))),
        (a, Value::Str(b)) => Ok(Value::Str(format!("{}{}", val_to_str(a), b))),
        _ => Err(runtime_error("Isku dar waa in ay noqdaan tiro ama qoraal", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_add(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    binop(a, b, add_impl)
}

fn sub_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n - m)),
        (Value::Int(n), Value::Float(y)) => Ok(Value::Float(*n as f64 - y)),
        (Value::Float(x), Value::Int(m)) => Ok(Value::Float(x - *m as f64)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        _ => Err(runtime_error("Ka jar waa in ay noqdaan tiro", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_sub(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    binop(a, b, sub_impl)
}

fn mul_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n * m)),
        (Value::Int(n), Value::Float(y)) => Ok(Value::Float(*n as f64 * y)),
        (Value::Float(x), Value::Int(m)) => Ok(Value::Float(x * *m as f64)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        _ => Err(runtime_error("Isku dhufashada waa in ay noqdaan tiro", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_mul(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    binop(a, b, mul_impl)
}

fn div_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error("Qaybinta eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(*n as f64 / *m as f64))
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error("Qaybinta eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(x / y))
            }
        }
        (Value::Int(n), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error("Qaybinta eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(*n as f64 / y))
            }
        }
        (Value::Float(x), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error("Qaybinta eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(x / *m as f64))
            }
        }
        _ => Err(runtime_error("Qaybinta waa in ay noqdaan tiro", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_div(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    binop(a, b, div_impl)
}

fn mod_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error("Habka eber ma jirto", 0, 0))
            } else {
                Ok(Value::Int(n % m))
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error("Habka eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(x % y))
            }
        }
        (Value::Int(n), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error("Habka eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(*n as f64 % y))
            }
        }
        (Value::Float(x), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error("Habka eber ma jirto", 0, 0))
            } else {
                Ok(Value::Float(x % *m as f64))
            }
        }
        _ => Err(runtime_error("Habka waa in ay noqdaan tiro", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_mod(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    binop(a, b, mod_impl)
}

fn neg_impl(a: Value) -> Result<Value, SoplangError> {
    match &a {
        Value::Int(n) => Ok(Value::Int(-n)),
        Value::Float(x) => Ok(Value::Float(-x)),
        _ => Err(runtime_error("Tixgacaysi (-) waa in uu noqdo tiro", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_neg(a: SoplangValue) -> SoplangValue {
    match soplang_to_value(a) {
        Ok(va) => match neg_impl(va) {
            Ok(v) => value_to_soplang(&v),
            Err(_) => SoplangValue::null(),
        },
        _ => SoplangValue::null(),
    }
}

fn not_impl(a: Value) -> Value {
    Value::Bool(!a.is_truthy())
}

#[no_mangle]
pub extern "C" fn soplang_not(a: SoplangValue) -> SoplangValue {
    match soplang_to_value(a) {
        Ok(va) => value_to_soplang(&not_impl(va)),
        _ => SoplangValue::null(),
    }
}

fn cmp_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(n), Value::Int(m)) => n == m,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(n), Value::Float(y)) => *n as f64 == *y,
        (Value::Float(x), Value::Int(m)) => *x == *m as f64,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
        (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
        (Value::Function(a), Value::Function(b)) => a == b,
        _ => false,
    }
}

#[no_mangle]
pub extern "C" fn soplang_eq(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => value_to_soplang(&Value::Bool(cmp_eq(&va, &vb))),
        _ => value_to_soplang(&Value::Bool(false)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_ne(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => value_to_soplang(&Value::Bool(!cmp_eq(&va, &vb))),
        _ => value_to_soplang(&Value::Bool(true)),
    }
}

fn cmp_lt(a: &Value, b: &Value) -> Result<bool, SoplangError> {
    match (a, b) {
        (Value::Int(n), Value::Int(m)) => Ok(n < m),
        (Value::Float(x), Value::Float(y)) => Ok(x < y),
        (Value::Int(n), Value::Float(y)) => Ok((*n as f64) < *y),
        (Value::Float(x), Value::Int(m)) => Ok(*x < *m as f64),
        (Value::Str(a), Value::Str(b)) => Ok(a < b),
        _ => Err(runtime_error("Is barbar dhig waa in ay noqdaan tiro ama qoraal", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_lt(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => match cmp_lt(&va, &vb) {
            Ok(r) => value_to_soplang(&Value::Bool(r)),
            Err(_) => SoplangValue::null(),
        },
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_le(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => {
            let lt = cmp_lt(&va, &vb).unwrap_or(false);
            let eq = cmp_eq(&va, &vb);
            value_to_soplang(&Value::Bool(lt || eq))
        }
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_gt(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => match cmp_lt(&vb, &va) {
            Ok(r) => value_to_soplang(&Value::Bool(r)),
            Err(_) => SoplangValue::null(),
        },
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_ge(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => {
            let lt = cmp_lt(&vb, &va).unwrap_or(false);
            let eq = cmp_eq(&va, &vb);
            value_to_soplang(&Value::Bool(lt || eq))
        }
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_qor(v: SoplangValue) {
    if has_pending_error() || v.tag == TAG_THROW {
        return;
    }
    if let Ok(val) = soplang_to_value(v) {
        println!("{}", value_to_string(&val));
    }
}

#[no_mangle]
pub extern "C" fn soplang_gelin() -> SoplangValue {
    match run_native("gelin", vec![]) {
        Ok(v) => value_to_soplang(&v),
        Err(_) => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_nooc(v: SoplangValue) -> SoplangValue {
    match soplang_to_value(v) {
        Ok(val) => match run_native("nooc", vec![val]) {
            Ok(v) => value_to_soplang(&v),
            Err(_) => SoplangValue::null(),
        },
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_list_new() -> SoplangValue {
    match run_native("teed", vec![]) {
        Ok(v) => value_to_soplang(&v),
        Err(_) => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_list_push(list: SoplangValue, val: SoplangValue) -> SoplangValue {
    match (soplang_to_value(list), soplang_to_value(val)) {
        (Ok(Value::List(l)), Ok(v)) => {
            l.borrow_mut().push(v);
            value_to_soplang(&Value::List(l))
        }
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_object_new() -> SoplangValue {
    match run_native("walax", vec![]) {
        Ok(v) => value_to_soplang(&v),
        Err(_) => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_get_index(obj: SoplangValue, idx: SoplangValue) -> SoplangValue {
    match (soplang_to_value(obj), soplang_to_value(idx)) {
        (Ok(Value::List(l)), Ok(Value::Int(i))) => {
            let v = l.borrow();
            let i = if i < 0 {
                (v.len() as i64 + i).max(0) as usize
            } else {
                i as usize
            };
            v.get(i).cloned().map(|x| value_to_soplang(&x)).unwrap_or_else(SoplangValue::null)
        }
        (Ok(Value::Str(s)), Ok(Value::Int(i))) => {
            let len = s.len() as i64;
            let i = if i < 0 {
                (len + i).max(0).min(len - 1)
            } else {
                i.min(len - 1)
            };
            if i >= 0 && (i as usize) < s.len() {
                let ch = s.chars().nth(i as usize).map(|c| c.to_string()).unwrap_or_default();
                value_to_soplang(&Value::Str(ch))
            } else {
                SoplangValue::null()
            }
        }
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_set_index(obj: SoplangValue, idx: SoplangValue, val: SoplangValue) -> SoplangValue {
    match (soplang_to_value(obj), soplang_to_value(idx), soplang_to_value(val)) {
        (Ok(Value::List(l)), Ok(Value::Int(i)), Ok(v)) => {
            let i = if i < 0 {
                (l.borrow().len() as i64 + i).max(0) as usize
            } else {
                i as usize
            };
            if i < l.borrow().len() {
                l.borrow_mut()[i] = v;
            }
            value_to_soplang(&Value::List(l))
        }
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_get_prop(obj: SoplangValue, name: *const u8, len: usize) -> SoplangValue {
    if name.is_null() {
        return SoplangValue::null();
    }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    match soplang_to_value(obj) {
        Ok(Value::Object(o)) => o
            .borrow()
            .get(&key)
            .cloned()
            .map(|v| value_to_soplang(&v))
            .unwrap_or_else(SoplangValue::null),
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_set_prop(obj: SoplangValue, name: *const u8, len: usize, val: SoplangValue) -> SoplangValue {
    if name.is_null() {
        return SoplangValue::null();
    }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    match (soplang_to_value(obj), soplang_to_value(val)) {
        (Ok(Value::Object(o)), Ok(v)) => {
            o.borrow_mut().insert(key, v);
            value_to_soplang(&Value::Object(o))
        }
        _ => SoplangValue::null(),
    }
}

// ----- Logical operators (And / Or) -----

#[no_mangle]
pub extern "C" fn soplang_and(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match soplang_to_value(a) {
        Ok(va) => {
            if !va.is_truthy() {
                value_to_soplang(&Value::Bool(false))
            } else {
                match soplang_to_value(b) {
                    Ok(vb) => value_to_soplang(&Value::Bool(vb.is_truthy())),
                    _ => SoplangValue::null(),
                }
            }
        }
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_or(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    match soplang_to_value(a) {
        Ok(va) => {
            if va.is_truthy() {
                value_to_soplang(&Value::Bool(true))
            } else {
                match soplang_to_value(b) {
                    Ok(vb) => value_to_soplang(&Value::Bool(vb.is_truthy())),
                    _ => SoplangValue::null(),
                }
            }
        }
        _ => SoplangValue::null(),
    }
}

// ----- Class registry (OOP) -----

pub fn clear_class_registry() {
    CLASS_REGISTRY.with(|r| r.borrow_mut().clear());
}

/// Drop a leftover try-block or pending error before the next REPL line.
pub fn reset_try_state() {
    TRY_DEPTH.with(|d| d.set(0));
    PENDING_ERROR.with(|e| *e.borrow_mut() = None);
}

fn global_fn_payload(name: &str) -> Option<i64> {
    ensure_builtins();
    GLOBAL_VARS.with(|g| {
        g.borrow().get(name).and_then(|sv| {
            if sv.tag == TAG_FUNC {
                Some(sv.payload)
            } else {
                None
            }
        })
    })
}

fn compiled_fn_entry(fn_payload: i64) -> Option<CompiledFnEntry> {
    let native_count = NATIVE_FN_TABLE.with(|t| t.borrow().len()) as i64;
    let compiled_idx = fn_payload - native_count;
    COMPILED_FN_TABLE.with(|t| {
        let v = t.borrow();
        if compiled_idx >= 0 && (compiled_idx as usize) < v.len() {
            Some(v[compiled_idx as usize])
        } else {
            None
        }
    })
}

fn get_instance_class(o: &Rc<RefCell<HashMap<String, Value>>>) -> Option<String> {
    o.borrow().get(INSTANCE_CLASS_FIELD).and_then(|v| {
        if let Value::Str(s) = v {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn resolve_method_fn_index(class_name: &str, method: &str) -> Option<i64> {
    CLASS_REGISTRY.with(|r| {
        let reg = r.borrow();
        let mut current = Some(class_name.to_string());
        while let Some(cn) = current {
            if let Some(entry) = reg.get(&cn) {
                if let Some(&idx) = entry.methods.get(method) {
                    return Some(idx);
                }
                current = entry.parent.clone();
            } else {
                break;
            }
        }
        None
    })
}

fn resolve_constructor_fn_index(class_name: &str) -> Option<i64> {
    resolve_method_fn_index(class_name, "dhaw")
}

/// Register all classes from semantic analysis after compiled functions are in globals.
pub fn register_classes_from_sym(classes: &HashMap<String, crate::semantic::ClassMeta>) {
    clear_class_registry();
    merge_classes_from_sym(classes);
}

/// Add or replace classes without dropping ones registered by earlier snippets.
pub fn merge_classes_from_sym(classes: &HashMap<String, crate::semantic::ClassMeta>) {
    for (name, meta) in classes {
        let mut methods = HashMap::new();
        for (method, mangled) in &meta.methods {
            if let Some(idx) = global_fn_payload(mangled) {
                methods.insert(method.clone(), idx);
            }
        }
        CLASS_REGISTRY.with(|r| {
            r.borrow_mut().insert(
                name.clone(),
                ClassRegistryEntry {
                    parent: meta.parent.clone(),
                    methods,
                },
            );
        });
    }
}

fn call_compiled_fn_value(
    entry: CompiledFnEntry,
    args: &[SoplangValue],
) -> Result<Value, SoplangError> {
    let sv = call_compiled_fn(entry.ptr, entry.n_params, args.as_ptr(), args.len() as i32);
    soplang_to_value(sv)
}

fn dispatch_instance_method(
    o: &Rc<RefCell<HashMap<String, Value>>>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    let class_name = get_instance_class(o).ok_or_else(|| {
        runtime_error("Walaxkan ma ahan tusaale fasal", 0, 0)
    })?;
    let fn_idx = resolve_method_fn_index(&class_name, method).ok_or_else(|| {
        runtime_error(
            format!(
                "Hawl '{}' lama helin qaabka '{}'",
                method, class_name
            ),
            0,
            0,
        )
    })?;
    let entry = compiled_fn_entry(fn_idx).ok_or_else(|| {
        runtime_error(format!("Hawl '{}' lama diyaarin", method), 0, 0)
    })?;
    let instance_sv = value_to_soplang(&Value::Object(Rc::clone(o)));
    let mut call_args = vec![instance_sv];
    for a in args {
        call_args.push(value_to_soplang(a));
    }
    call_compiled_fn_value(entry, &call_args)
}

#[no_mangle]
pub extern "C" fn soplang_new_instance(
    class_ptr: *const u8,
    class_len: usize,
    args_ptr: *const SoplangValue,
    n: i32,
) -> SoplangValue {
    if class_ptr.is_null() || class_len == 0 {
        return SoplangValue::null();
    }
    let class_name = unsafe {
        String::from_utf8_lossy(std::slice::from_raw_parts(class_ptr, class_len)).into_owned()
    };
    let mut fields = HashMap::new();
    fields.insert(
        INSTANCE_CLASS_FIELD.to_string(),
        Value::Str(class_name.clone()),
    );
    let instance = Value::Object(Rc::new(RefCell::new(fields)));
    let instance_rc = match &instance {
        Value::Object(o) => Rc::clone(o),
        _ => return SoplangValue::null(),
    };

    let mut user_args = Vec::new();
    if !args_ptr.is_null() && n > 0 {
        for i in 0..n as usize {
            let sv = unsafe { *args_ptr.add(i) };
            user_args.push(soplang_to_value(sv).unwrap_or(Value::Null));
        }
    }

    if let Some(fn_idx) = resolve_constructor_fn_index(&class_name) {
        if let Some(entry) = compiled_fn_entry(fn_idx) {
            let instance_sv = value_to_soplang(&instance);
            let mut call_args = vec![instance_sv];
            for a in &user_args {
                call_args.push(value_to_soplang(a));
            }
            match call_compiled_fn_value(entry, &call_args) {
                Ok(_) => {}
                Err(e) => fatal_error(e),
            }
        }
    }

    value_to_soplang(&Value::Object(instance_rc))
}

#[no_mangle]
pub extern "C" fn soplang_register_class(
    name_ptr: *const u8,
    name_len: usize,
    parent_ptr: *const u8,
    parent_len: usize,
    method_ptr: *const u8,
    method_len: usize,
    fn_payload: i64,
) {
    if name_ptr.is_null() || name_len == 0 || method_ptr.is_null() || method_len == 0 {
        return;
    }
    let name = unsafe {
        String::from_utf8_lossy(std::slice::from_raw_parts(name_ptr, name_len)).into_owned()
    };
    let parent = if parent_ptr.is_null() || parent_len == 0 {
        None
    } else {
        Some(
            unsafe {
                String::from_utf8_lossy(std::slice::from_raw_parts(parent_ptr, parent_len))
                    .into_owned()
            },
        )
    };
    let method = unsafe {
        String::from_utf8_lossy(std::slice::from_raw_parts(method_ptr, method_len)).into_owned()
    };
    CLASS_REGISTRY.with(|r| {
        let mut reg = r.borrow_mut();
        let entry = reg.entry(name).or_insert_with(|| ClassRegistryEntry {
            parent: parent.clone(),
            methods: HashMap::new(),
        });
        if entry.parent.is_none() {
            entry.parent = parent;
        }
        entry.methods.insert(method, fn_payload);
    });
}

// ----- Method dispatch -----

#[no_mangle]
pub extern "C" fn soplang_call_method(
    obj: SoplangValue,
    method_ptr: *const u8,
    method_len: usize,
    args_ptr: *const SoplangValue,
    n: i32,
) -> SoplangValue {
    let method = if method_ptr.is_null() || method_len == 0 {
        ""
    } else {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(method_ptr, method_len))
        }
    };
    let obj_val = match soplang_to_value(obj) {
        Ok(v) => v,
        Err(_) => return SoplangValue::null(),
    };
    let mut argv = Vec::new();
    if !args_ptr.is_null() && n > 0 {
        for i in 0..n as usize {
            let sv = unsafe { *args_ptr.add(i) };
            argv.push(soplang_to_value(sv).unwrap_or(Value::Null));
        }
    }
    let result = match &obj_val {
        Value::Object(o) if get_instance_class(o).is_some() => {
            dispatch_instance_method(o, method, &argv)
        }
        Value::List(l) => dispatch_list_method(l, method, &argv),
        Value::Object(o) => dispatch_object_method(o, method, &argv),
        Value::Str(s) => dispatch_string_method(s, method, &argv),
        _ => Err(runtime_error(format!("No method '{}' on this type", method), 0, 0)),
    };
    match result {
        Ok(v) => value_to_soplang(&v),
        Err(e) => fatal_error(e),
    }
}

fn call_compiled_fn(ptr: *const u8, n_params: usize, args: *const SoplangValue, n: i32) -> SoplangValue {
    let mut raw: Vec<i64> = Vec::with_capacity(n_params * 2);
    for i in 0..n_params {
        if !args.is_null() && (i as i32) < n {
            let sv = unsafe { *args.add(i) };
            raw.push(sv.tag as i64);
            raw.push(sv.payload);
        } else {
            raw.push(0);
            raw.push(0);
        }
    }
    let (ret_tag, ret_pay): (i64, i64) = unsafe {
        match n_params {
            0 => {
                let f: extern "C" fn() -> (i64, i64) = std::mem::transmute(ptr);
                f()
            }
            1 => {
                let f: extern "C" fn(i64, i64) -> (i64, i64) = std::mem::transmute(ptr);
                f(raw[0], raw[1])
            }
            2 => {
                let f: extern "C" fn(i64, i64, i64, i64) -> (i64, i64) = std::mem::transmute(ptr);
                f(raw[0], raw[1], raw[2], raw[3])
            }
            3 => {
                let f: extern "C" fn(i64, i64, i64, i64, i64, i64) -> (i64, i64) = std::mem::transmute(ptr);
                f(raw[0], raw[1], raw[2], raw[3], raw[4], raw[5])
            }
            4 => {
                let f: extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64) -> (i64, i64) = std::mem::transmute(ptr);
                f(raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7])
            }
            _ => {
                let f: extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64,
                    i64, i64, i64, i64, i64, i64, i64, i64) -> (i64, i64) = std::mem::transmute(ptr);
                let mut a = [0i64; 16];
                for (i, v) in raw.iter().enumerate().take(16) { a[i] = *v; }
                f(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
                  a[8], a[9], a[10], a[11], a[12], a[13], a[14], a[15])
            }
        }
    };
    SoplangValue { tag: ret_tag as u8, _pad: [0; 7], payload: ret_pay }
}

fn dispatch_list_method(
    l: &Rc<RefCell<Vec<Value>>>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    match method {
        "shaandhee" => {
            let func_val = args.first().ok_or_else(|| runtime_error("shaandhee() waa in uu qaato 1 qiimo (hawl)", 0, 0))?;
            let callee_sv = value_to_soplang(func_val);
            let mut out = Vec::new();
            for item in l.borrow().iter() {
                let item_sv = value_to_soplang(item);
                let result = soplang_call(callee_sv, &item_sv as *const SoplangValue, 1);
                if let Ok(v) = soplang_to_value(result) {
                    if v.is_truthy() {
                        out.push(item.clone());
                    }
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(out))))
        }
        "aaddin" => {
            let func_val = args.first().ok_or_else(|| runtime_error("aaddin() waa in uu qaato 1 qiimo (hawl)", 0, 0))?;
            let callee_sv = value_to_soplang(func_val);
            let mut out = Vec::new();
            for item in l.borrow().iter() {
                let item_sv = value_to_soplang(item);
                let result = soplang_call(callee_sv, &item_sv as *const SoplangValue, 1);
                out.push(soplang_to_value(result).unwrap_or(Value::Null));
            }
            Ok(Value::List(Rc::new(RefCell::new(out))))
        }
        "kasaar" => super::stdlib::list_kasaar(Rc::clone(l), args),
        "dherer" => super::stdlib::list_dherer(Rc::clone(l), args),
        "kudar"  => super::stdlib::list_kudar(Rc::clone(l), args),
        "leeyahay" => super::stdlib::list_leeyahay(Rc::clone(l), args),
        "nuqul"  => super::stdlib::list_nuqul(Rc::clone(l), args),
        "nadiifi" => super::stdlib::list_nadiifi(Rc::clone(l), args),
        "rog"    => super::stdlib::list_rog(Rc::clone(l), args),
        "habee"  => super::stdlib::list_habee(Rc::clone(l), args),
        "jar"    => super::stdlib::list_jar(Rc::clone(l), args),
        "muuji"  => super::stdlib::list_muuji(Rc::clone(l), args),
        _ => Err(runtime_error(format!("Unknown list method: {}", method), 0, 0)),
    }
}

fn dispatch_object_method(
    o: &Rc<RefCell<HashMap<String, Value>>>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    match method {
        "fure"    => super::stdlib::object_fure(Rc::clone(o), args),
        "leeyahay" => super::stdlib::object_leeyahay(Rc::clone(o), args),
        "tir"     => super::stdlib::object_tir(Rc::clone(o), args),
        "kudar"   => super::stdlib::object_kudar(Rc::clone(o), args),
        "nuqul"   => super::stdlib::object_nuqul(Rc::clone(o), args),
        "nadiifi" => super::stdlib::object_nadiifi(Rc::clone(o), args),
        "qiime"   => super::stdlib::object_qiime(Rc::clone(o), args),
        "lamaane" => super::stdlib::object_lamaane(Rc::clone(o), args),
        _ => Err(runtime_error(format!("Unknown object method: {}", method), 0, 0)),
    }
}

fn dispatch_string_method(
    s: &str,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    match method {
        "qeybi"    => super::stdlib::string_qeybi(s.to_string(), args),
        "leeyahay" => super::stdlib::string_leeyahay(s.to_string(), args),
        "dhamaad"  => super::stdlib::string_dhamaad(s.to_string(), args),
        "bilow"    => super::stdlib::string_bilow(s.to_string(), args),
        "beddel"   => super::stdlib::string_beddel(s.to_string(), args),
        "beddel_dhammaan" => super::stdlib::string_beddel_dhammaan(s.to_string(), args),
        "kudar"    => super::stdlib::string_kudar(s.to_string(), args),
        "jar"      => super::stdlib::string_jar(s.to_string(), args),
        "xarafaha_weyn"   => super::stdlib::string_xarafaha_weyn(s.to_string(), args),
        "xarfaha_yaryar"  => super::stdlib::string_xarfaha_yaryar(s.to_string(), args),
        "masax"    => super::stdlib::string_masax(s.to_string(), args),
        "raadi"    => super::stdlib::string_raadi(s.to_string(), args),
        _ => Err(runtime_error(format!("Unknown string method: {}", method), 0, 0)),
    }
}

/// Store a global variable (Rust API).
pub fn store_global(name: &str, val: SoplangValue) {
    GLOBAL_VARS.with(|g| g.borrow_mut().insert(name.to_string(), val));
}

/// Store a global variable by name (C ABI).
#[no_mangle]
pub extern "C" fn soplang_store_global(name: *const u8, len: usize, tag: i64, payload: i64) {
    if name.is_null() { return; }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    let is_const = CONST_GLOBALS.with(|c| c.borrow().contains(&key));
    if is_const {
        let _ = handle_runtime_error(runtime_error(
            format!("Ma bedeli kartid qiimaha doorsamaha madoor '{}'", key),
            0,
            0,
        ));
        return;
    }
    let sv = SoplangValue { tag: tag as u8, _pad: [0; 7], payload };
    GLOBAL_VARS.with(|g| g.borrow_mut().insert(key, sv));
}

/// Mark a global as constant (C ABI).
#[no_mangle]
pub extern "C" fn soplang_mark_const(name: *const u8, len: usize) {
    if name.is_null() { return; }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    CONST_GLOBALS.with(|c| c.borrow_mut().insert(key));
}

/// The value must be an instance of `expected` or of a class that extends it.
#[no_mangle]
pub extern "C" fn soplang_check_class(
    tag: i64,
    payload: i64,
    class_ptr: *const u8,
    class_len: usize,
    name_ptr: *const u8,
    name_len: usize,
) {
    if class_ptr.is_null() || class_len == 0 {
        return;
    }
    let expected = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(class_ptr, class_len))
    };
    let sv = SoplangValue { tag: tag as u8, _pad: [0; 7], payload };
    let val = soplang_to_value(sv).unwrap_or(Value::Null);
    if instance_matches_class(&val, expected) {
        return;
    }
    let name = if name_ptr.is_null() || name_len == 0 {
        ""
    } else {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(name_ptr, name_len)) }
    };
    let err = type_error(
        format!(
            "'{}' waa {} laakin qiimaheeda '{}' ma ahan {}",
            name, expected, val, expected
        ),
        0,
        0,
    );
    if TRY_DEPTH.with(|d| d.get()) > 0 {
        PENDING_ERROR.with(|p| *p.borrow_mut() = Some(error_message(&err)));
    } else {
        fatal_error(err);
    }
}

fn instance_matches_class(val: &Value, expected: &str) -> bool {
    let Value::Object(o) = val else {
        return false;
    };
    let Some(mut current) = get_instance_class(o) else {
        return false;
    };
    let mut seen = std::collections::HashSet::new();
    loop {
        if current == expected {
            return true;
        }
        if !seen.insert(current.clone()) {
            return false;
        }
        let next = CLASS_REGISTRY.with(|r| {
            r.borrow().get(&current).and_then(|entry| entry.parent.clone())
        });
        match next {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

/// Runtime type validation for typed variable assignment (C ABI).
/// expected_type: 1=abn, 2=jajab, 3=qoraal, 4=bool, 5=teed, 6=walax, 0=dynamic(skip).
#[no_mangle]
pub extern "C" fn soplang_check_type(tag: i64, payload: i64, expected_type: i64, name_ptr: *const u8, name_len: usize) {
    if expected_type == 0 { return; }
    let sv = SoplangValue { tag: tag as u8, _pad: [0; 7], payload };
    let val = soplang_to_value(sv).unwrap_or(Value::Null);
    let ok = match expected_type {
        1 => matches!(val, Value::Int(_)) || matches!(&val, Value::Float(x) if x.fract() == 0.0 && x.is_finite()),
        2 => matches!(val, Value::Float(_)) || matches!(val, Value::Int(_)),
        3 => matches!(val, Value::Str(_)),
        4 => matches!(val, Value::Bool(_)),
        5 => matches!(val, Value::List(_)),
        6 => matches!(val, Value::Object(_)),
        _ => true,
    };
    if !ok {
        let name = if name_ptr.is_null() { "" } else {
            unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(name_ptr, name_len)) }
        };
        let type_name = match expected_type {
            1 => "abn", 2 => "jajab", 3 => "qoraal", 4 => "bool", 5 => "teed", 6 => "walax", _ => "?",
        };
        let err = type_error(
            format!("'{}' waa {} laakin qiimaheeda '{}' ma ahan {}", name, type_name, val, type_name),
            0,
            0,
        );
        if TRY_DEPTH.with(|d| d.get()) > 0 {
            PENDING_ERROR.with(|p| *p.borrow_mut() = Some(error_message(&err)));
        } else {
            fatal_error(err);
        }
    }
}

/// Resolve a builtin or global by name.
#[no_mangle]
pub extern "C" fn soplang_get_builtin(name: *const u8, len: usize) -> SoplangValue {
    if name.is_null() {
        return SoplangValue::null();
    }
    ensure_builtins();
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    let idx = BUILTIN_INDICES.with(|b| b.borrow().as_ref().and_then(|m| m.get(&key).copied()));
    if let Some(i) = idx {
        return SoplangValue { tag: TAG_FUNC, _pad: [0; 7], payload: i };
    }
    GLOBAL_VARS.with(|g| g.borrow().get(&key).copied().unwrap_or_else(SoplangValue::null))
}

/// Register a compiled function pointer so soplang_call can dispatch to it.
/// Returns an index that can be used as the payload of a TAG_FUNC SoplangValue.
/// The index is offset past native functions.
pub fn register_compiled_fn(ptr: *const u8, n_params: usize) -> i64 {
    ensure_builtins();
    let native_count = NATIVE_FN_TABLE.with(|t| t.borrow().len());
    COMPILED_FN_TABLE.with(|t| {
        let mut v = t.borrow_mut();
        let idx = native_count as i64 + v.len() as i64;
        v.push(CompiledFnEntry { ptr, n_params });
        idx
    })
}

/// Call a callee with args. Callee must be SoplangValue with TAG_FUNC (native or user fn).
#[no_mangle]
pub extern "C" fn soplang_call(callee: SoplangValue, args: *const SoplangValue, n: i32) -> SoplangValue {
    if let Some(v) = short_circuit_if_pending() {
        return v;
    }
    if callee.tag != TAG_FUNC {
        let val = soplang_to_value(callee).unwrap_or(Value::Null);
        return handle_runtime_error(runtime_error(
            format!("'{}' ma ahan hawl, lama wici karo", value_to_string(&val)),
            0,
            0,
        ));
    }
    if let Some(f) = native_fn_get(callee.payload) {
        let mut argv = Vec::new();
        if !args.is_null() && n > 0 {
            for i in 0..n as usize {
                let sv = unsafe { *args.add(i) };
                if let Ok(v) = soplang_to_value(sv) {
                    argv.push(v);
                } else {
                    argv.push(Value::Null);
                }
            }
        }
        match f(argv) {
            Ok(v) => value_to_soplang(&v),
            Err(e) => handle_runtime_error(e),
        }
    } else {
        let native_count = NATIVE_FN_TABLE.with(|t| t.borrow().len()) as i64;
        let compiled_idx = callee.payload - native_count;
        let entry = COMPILED_FN_TABLE.with(|t| {
            let v = t.borrow();
            if compiled_idx >= 0 && (compiled_idx as usize) < v.len() {
                Some(v[compiled_idx as usize])
            } else {
                None
            }
        });
        if let Some(entry) = entry {
            call_compiled_fn(entry.ptr, entry.n_params, args, n)
        } else {
            SoplangValue::null()
        }
    }
}
