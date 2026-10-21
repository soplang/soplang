//! Runtime library: C-ABI functions called by Cranelift and LLVM backends.
//! Phase 3 of COMPILER_PLAN. Converts SoplangValue to/from Value and delegates to stdlib.

#![allow(clippy::cast_lossless, clippy::cast_possible_truncation)]

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::os::raw::c_int;
use std::rc::Rc;

use super::context::RuntimeLocal;
use super::handles::HandleArena;
use super::stdlib;
use super::value::{value_to_string, NativeFn, Value};
use crate::error::{runtime_error, type_error, SoplangError};

const TAG_THROW: u8 = 8;

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
    let mut e = CURRENT_SOURCE.with(|source| match source.borrow().as_ref() {
        Some(span) => span.attach(e),
        None => e,
    });
    if e.meta().cause.is_none() {
        e.meta_mut().cause = SUSPENDED_ERRORS.with(|errors| {
            errors.borrow().iter().rev().find_map(|error| error.as_ref())
                .map(|error| Box::new(error.clone()))
        });
    }
    if e.meta().trace.is_empty() {
        e.meta_mut().trace = CALL_STACK.with(|stack| stack.borrow().iter().rev().cloned().collect());
    }
    // Generated code propagates this state to the execution boundary without
    // unwinding Rust through a C ABI or terminating an embedding process.
    PENDING_ERROR.with(|pending| {
        if pending.borrow().is_none() {
            *pending.borrow_mut() = Some(e);
        }
    });
    throw_value()
}

const COLLECTOR: RuntimeLocal<super::gc::CollectorState> = RuntimeLocal::new(|state| &state.collector);
const STATE: RuntimeLocal<super::context::RuntimeState> = RuntimeLocal::new(|state| state);

const CALL_STACK: RuntimeLocal<RefCell<Vec<crate::error::StackFrame>>> =
    RuntimeLocal::new(|state| &state.call_stack);
const ERROR_HEAP: RuntimeLocal<RefCell<HandleArena<Rc<SoplangError>>>> =
    RuntimeLocal::new(|state| &state.errors);

const CURRENT_SOURCE: RuntimeLocal<RefCell<Option<crate::error::SourceSpan>>> =
    RuntimeLocal::new(|state| &state.current_source);
const TRY_DEPTH: RuntimeLocal<Cell<u32>> = RuntimeLocal::new(|state| &state.try_depth);
const PENDING_ERROR: RuntimeLocal<RefCell<Option<SoplangError>>> =
    RuntimeLocal::new(|state| &state.pending_error);
const SUSPENDED_ERRORS: RuntimeLocal<RefCell<Vec<Option<SoplangError>>>> =
    RuntimeLocal::new(|state| &state.suspended_errors);

#[no_mangle]
pub extern "C" fn soplang_gc_safepoint() {
    if COLLECTOR.with(|collector| collector.due()) {
        STATE.with(|state| { super::gc::collect(state, false); });
    }
}

/// Register generated slot storage until the matching leave operation.
#[no_mangle]
pub extern "C" fn soplang_enter_roots(tags: *const i64, payloads: *const i64, len: usize, stride: usize) {
    super::roots::enter(tags, payloads, len, stride);
}

#[no_mangle]
pub extern "C" fn soplang_leave_roots() {
    super::roots::leave();
}

/// Set the source token for the next runtime operation in either backend.
#[no_mangle]
pub extern "C" fn soplang_set_location(
    file_ptr: *const u8,
    file_len: usize,
    line: usize,
    col: usize,
    end_col: usize,
    text_ptr: *const u8,
    text_len: usize,
) {
    if file_ptr.is_null() || text_ptr.is_null() {
        return;
    }
    let file = unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(file_ptr, file_len)) };
    let text = unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(text_ptr, text_len)) };
    CURRENT_SOURCE.with(|source| {
        let mut source = source.borrow_mut();
        if let Some(current) = source.as_mut() {
            // Most consecutive operations share a line. Avoid allocating file
            // and line strings for every arithmetic operation in a loop.
            if current.file.as_ref().and_then(|p| p.to_str()) != Some(file.as_ref()) {
                current.file = Some(std::path::PathBuf::from(file.as_ref()));
            }
            if current.source_line != text {
                current.source_line = text.into_owned();
            }
            current.line = line;
            current.col = col;
            current.end_col = end_col;
        } else {
            *source = Some(crate::error::SourceSpan {
                file: Some(std::path::PathBuf::from(file.as_ref())),
                line,
                col,
                end_col,
                source_line: text.into_owned(),
            });
        }
    });
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
    if has_pending_error() {
        1
    } else {
        0
    }
}

/// Return the caught error message as a Soplang string value.
#[no_mangle]
pub extern "C" fn soplang_get_caught_error() -> SoplangValue {
    let msg = PENDING_ERROR.with(|error| {
        error.borrow().as_ref().map(error_message).unwrap_or_default()
    });
    value_to_soplang(&Value::Str(msg))
}

macro_rules! check_handles {
    ($($value:expr),+ $(,)?) => {
        if let Some(value) = short_circuit_if_pending() {
            return value;
        }
        $(if let Err(error) = validate_handle($value) {
            return handle_runtime_error(error);
        })+
    };
}

/// Raise a user message at the current source location.
#[no_mangle]
pub extern "C" fn soplang_raise(value: SoplangValue) -> SoplangValue {
    check_handles!(value);
    match soplang_to_value(value) {
        Ok(Value::Str(message)) => handle_runtime_error(runtime_error(message, 0, 0)),
        Ok(Value::Error(error)) => handle_runtime_error((*error).clone()),
        Ok(_) => handle_runtime_error(type_error("Qiimaha 'tuur' waa inuu noqdaa qoraal ama khalad.", 0, 0)),
        Err(error) => handle_runtime_error(error),
    }
}

/// Temporarily clear a pending error so guaranteed cleanup can execute.
#[no_mangle]
pub extern "C" fn soplang_suspend_error() {
    let error = take_pending_error();
    SUSPENDED_ERRORS.with(|errors| errors.borrow_mut().push(error));
}

/// Resume an error after successful cleanup. A new error takes precedence.
#[no_mangle]
pub extern "C" fn soplang_restore_error() {
    let previous = SUSPENDED_ERRORS.with(|errors| errors.borrow_mut().pop().flatten());
    PENDING_ERROR.with(|pending| {
        if pending.borrow().is_none() {
            *pending.borrow_mut() = previous;
        }
    });
}

/// An abrupt cleanup exit supersedes the original error or return.
#[no_mangle]
pub extern "C" fn soplang_discard_error() {
    SUSPENDED_ERRORS.with(|errors| { errors.borrow_mut().pop(); });
}

/// Retrieve the structured form without consuming the pending error.
#[no_mangle]
pub extern "C" fn soplang_get_caught_error_value() -> SoplangValue {
    let error = PENDING_ERROR.with(|error| error.borrow().as_ref().cloned());
    error
        .map(|error| value_to_soplang(&Value::Error(Rc::new(error))))
        .unwrap_or_else(SoplangValue::null)
}

#[no_mangle]
pub extern "C" fn soplang_enter_function(name: *const u8, len: usize) {
    if name.is_null() {
        return;
    }
    let name = unsafe { std::slice::from_raw_parts(name, len) };
    let function = String::from_utf8_lossy(name).into_owned();
    let frame = CURRENT_SOURCE.with(|source| {
        let source = source.borrow();
        crate::error::StackFrame {
            function,
            file: source.as_ref().and_then(|source| source.file.clone()),
            line: source.as_ref().map(|source| source.line).unwrap_or(0),
            col: source.as_ref().map(|source| source.col).unwrap_or(0),
        }
    });
    CALL_STACK.with(|stack| stack.borrow_mut().push(frame));
}

#[no_mangle]
pub extern "C" fn soplang_leave_function() {
    CALL_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
}

/// Take an uncaught error at an execution boundary, preserving its metadata.
pub fn take_pending_error() -> Option<SoplangError> {
    PENDING_ERROR.with(|error| error.borrow_mut().take())
}

/// Complete a standalone program without terminating the embedding runtime.
#[no_mangle]
pub extern "C" fn soplang_finish_execution() -> c_int {
    use std::io::Write;
    match take_pending_error() {
        Some(error) => {
            let _ = std::io::stdout().flush();
            let _ = writeln!(std::io::stderr(), "{}",
                crate::error::format_error_with_source(&error, None));
            1
        }
        None => 0,
    }
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
pub const TAG_CLOSURE: u8 = 9;
pub const TAG_ERROR: u8 = 10;
pub const TAG_ITERATOR: u8 = 11;
pub const TAG_BYTES: u8 = 12;
/// Internal argument slot marker; never a language value.
pub(crate) const TAG_MISSING: u8 = 255;

/// Tagged value for C ABI. 16 bytes: tag (u8) + padding + payload (i64).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SoplangValue {
    pub tag: u8,
    pub _pad: [u8; 7],
    pub payload: i64,
}

impl Default for SoplangValue {
    fn default() -> Self {
        Self::null()
    }
}

impl SoplangValue {
    pub fn null() -> Self {
        Self {
            tag: TAG_NULL,
            _pad: [0; 7],
            payload: 0,
        }
    }
}

// ----- Heaps for heap-allocated types (used across C boundary) -----
const STR_HEAP: RuntimeLocal<RefCell<HandleArena<String>>> = RuntimeLocal::new(|state| &state.strings);
const LIST_HEAP: RuntimeLocal<RefCell<HandleArena<Rc<RefCell<Vec<Value>>>>>> =
    RuntimeLocal::new(|state| &state.lists);
const OBJ_HEAP: RuntimeLocal<RefCell<HandleArena<Rc<RefCell<HashMap<String, Value>>>>>> =
    RuntimeLocal::new(|state| &state.objects);
const NATIVE_FN_TABLE: RuntimeLocal<RefCell<HandleArena<NativeFn>>> =
    RuntimeLocal::new(|state| &state.native_functions);
const BUILTIN_INDICES: RuntimeLocal<RefCell<Option<HashMap<String, i64>>>> =
    RuntimeLocal::new(|state| &state.builtin_indices);
const GLOBAL_VARS: RuntimeLocal<RefCell<HashMap<String, SoplangValue>>> =
    RuntimeLocal::new(|state| &state.globals);
const COMPILED_FN_TABLE: RuntimeLocal<RefCell<HandleArena<CompiledFnEntry>>> =
    RuntimeLocal::new(|state| &state.compiled_functions);
const CONST_GLOBALS: RuntimeLocal<RefCell<HashSet<String>>> =
    RuntimeLocal::new(|state| &state.constants);
const CLASS_REGISTRY: RuntimeLocal<RefCell<HashMap<String, ClassRegistryEntry>>> =
    RuntimeLocal::new(|state| &state.classes);
const CLOSURE_HEAP: RuntimeLocal<RefCell<HandleArena<Rc<ClosureData>>>> =
    RuntimeLocal::new(|state| &state.closures);

const LIST_IDS: RuntimeLocal<RefCell<HashMap<usize, i64>>> = RuntimeLocal::new(|state| &state.list_ids);
const OBJECT_IDS: RuntimeLocal<RefCell<HashMap<usize, i64>>> = RuntimeLocal::new(|state| &state.object_ids);

const INSTANCE_CLASS_FIELD: &str = "__class__";

#[derive(Clone)]
pub(super) struct ClassRegistryEntry {
    parent: Option<String>,
    methods: HashMap<String, i64>,
}

#[derive(Clone)]
pub(super) struct CompiledFnEntry {
    ptr: *const u8,
    n_params: usize,
    required_params: usize,
    variadic: bool,
    keyword_rest: bool,
    names: Option<Rc<Vec<String>>>,
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
                        let idx = v.insert(f);
                        m.insert(name, idx);
                    }
                }
            });
            *b.borrow_mut() = Some(m);
        }
    });
}

pub(super) fn invalid_handle() -> SoplangError {
    crate::error::runtime_error_ex(
        "Qiimahan ma jiro ama wuxuu ka tirsan yahay fulin kale.",
        0,
        0,
        crate::error::ErrorMeta::default().with_code(crate::error::codes::E035_INVALID_HANDLE),
    )
}

fn validate_handle(value: SoplangValue) -> Result<(), SoplangError> {
    let live = match value.tag {
        TAG_ITERATOR => STATE.with(|state| state.iterators.borrow().contains(value.payload)),
        TAG_ERROR => ERROR_HEAP.with(|heap| heap.borrow().contains(value.payload)),
        TAG_STR => STR_HEAP.with(|heap| heap.borrow().contains(value.payload)),
        TAG_LIST => LIST_HEAP.with(|heap| heap.borrow().contains(value.payload)),
        TAG_BYTES => STATE.with(|state| state.bytes.borrow().contains(value.payload)),
        TAG_OBJECT => OBJ_HEAP.with(|heap| heap.borrow().contains(value.payload)),
        TAG_FUNC => native_fn_get(value.payload).is_some() || compiled_fn_entry(value.payload).is_some(),
        TAG_CLOSURE => CLOSURE_HEAP.with(|heap| heap.borrow().contains(value.payload)),
        _ => true,
    };
    if live {
        Ok(())
    } else {
        Err(invalid_handle())
    }
}


fn native_fn_get(id: i64) -> Option<NativeFn> {
    NATIVE_FN_TABLE.with(|table| table.borrow().get(id).copied())
}

fn str_alloc(value: String) -> i64 {
    COLLECTOR.with(|collector| collector.allocated(value.len()));
    STR_HEAP.with(|heap| heap.borrow_mut().insert(value))
}

fn str_get(id: i64) -> Result<String, SoplangError> {
    STR_HEAP.with(|heap| heap.borrow().get(id).cloned().ok_or_else(invalid_handle))
}

fn list_alloc(value: Rc<RefCell<Vec<Value>>>) -> i64 {
    let key = Rc::as_ptr(&value) as usize;
    if let Some(id) = LIST_IDS.with(|ids| ids.borrow().get(&key).copied()) {
        if LIST_HEAP.with(|heap| heap.borrow().contains(id)) { return id; }
    }
    COLLECTOR.with(|collector| collector.allocated(value.try_borrow().map(|values| values.len()).unwrap_or(0) * std::mem::size_of::<Value>()));
    let id = LIST_HEAP.with(|heap| heap.borrow_mut().insert(value));
    LIST_IDS.with(|ids| ids.borrow_mut().insert(key, id));
    id
}

fn list_get(id: i64) -> Result<Rc<RefCell<Vec<Value>>>, SoplangError> {
    LIST_HEAP.with(|heap| heap.borrow().get(id).cloned().ok_or_else(invalid_handle))
}

pub(super) fn record_bytes_growth(bytes: usize) {
    if bytes > 0 {
        COLLECTOR.with(|collector| collector.allocated(bytes));
    }
}

fn bytes_alloc(value: Rc<RefCell<Vec<u8>>>) -> i64 {
    STATE.with(|state| {
        let key = Rc::as_ptr(&value) as usize;
        if let Some(id) = state.byte_ids.borrow().get(&key).copied() {
            if state.bytes.borrow().contains(id) {
                return id;
            }
        }
        state.collector.allocated(
            value
                .try_borrow()
                .map(|bytes| bytes.capacity())
                .unwrap_or(0),
        );
        let id = state.bytes.borrow_mut().insert(value);
        state.byte_ids.borrow_mut().insert(key, id);
        id
    })
}

fn obj_alloc(value: Rc<RefCell<HashMap<String, Value>>>) -> i64 {
    let key = Rc::as_ptr(&value) as usize;
    if let Some(id) = OBJECT_IDS.with(|ids| ids.borrow().get(&key).copied()) {
        if OBJ_HEAP.with(|heap| heap.borrow().contains(id)) { return id; }
    }
    COLLECTOR.with(|collector| collector.allocated(value.try_borrow().map(|values| values.len()).unwrap_or(0) * std::mem::size_of::<(String, Value)>()));
    let id = OBJ_HEAP.with(|heap| heap.borrow_mut().insert(value));
    OBJECT_IDS.with(|ids| ids.borrow_mut().insert(key, id));
    id
}

fn obj_get(id: i64) -> Result<Rc<RefCell<HashMap<String, Value>>>, SoplangError> {
    OBJ_HEAP.with(|heap| heap.borrow().get(id).cloned().ok_or_else(invalid_handle))
}

// ----- Conversion Value <-> SoplangValue -----
pub fn value_to_soplang(v: &Value) -> SoplangValue {
    match v {
        Value::Null => SoplangValue::null(),
        Value::Int(n) => SoplangValue {
            tag: TAG_INT,
            _pad: [0; 7],
            payload: *n,
        },
        Value::Float(x) => SoplangValue {
            tag: TAG_FLOAT,
            _pad: [0; 7],
            payload: x.to_bits() as i64,
        },
        Value::Bool(b) => SoplangValue {
            tag: TAG_BOOL,
            _pad: [0; 7],
            payload: if *b { 1 } else { 0 },
        },
        Value::Str(s) => SoplangValue {
            tag: TAG_STR,
            _pad: [0; 7],
            payload: str_alloc(s.clone()),
        },
        Value::Bytes(bytes) => SoplangValue {
            tag: TAG_BYTES,
            _pad: [0; 7],
            payload: bytes_alloc(Rc::clone(bytes)),
        },
        Value::List(l) => SoplangValue {
            tag: TAG_LIST,
            _pad: [0; 7],
            payload: list_alloc(Rc::clone(l)),
        },
        Value::Object(o) => SoplangValue {
            tag: TAG_OBJECT,
            _pad: [0; 7],
            payload: obj_alloc(Rc::clone(o)),
        },
        Value::Error(error) => SoplangValue {
            tag: TAG_ERROR,
            _pad: [0; 7],
            payload: {
                COLLECTOR.with(|collector| collector.allocated(error.message().len()));
                ERROR_HEAP.with(|heap| heap.borrow_mut().insert(Rc::clone(error)))
            },
        },
        Value::Function(id) => SoplangValue {
            tag: TAG_FUNC,
            _pad: [0; 7],
            payload: id.0 as i64,
        },
        Value::Iterator(data) => SoplangValue {
            tag: TAG_ITERATOR, _pad: [0; 7], payload: data.handle(),
        },
        Value::Closure(data) => SoplangValue {
            tag: TAG_CLOSURE,
            _pad: [0; 7],
            payload: data.handle(),
        },
        Value::NativeFunction(f) => {
            ensure_builtins();
            for (name, v) in stdlib::get_builtin_functions() {
                if let Value::NativeFunction(g) = v {
                    if std::ptr::fn_addr_eq(*f, g) {
                        let idx = BUILTIN_INDICES
                            .with(|b| b.borrow().as_ref().unwrap().get(&name).copied());
                        if let Some(i) = idx {
                            return SoplangValue {
                                tag: TAG_FUNC,
                                _pad: [0; 7],
                                payload: i,
                            };
                        }
                    }
                }
            }
            SoplangValue {
                tag: TAG_FUNC,
                _pad: [0; 7],
                payload: NATIVE_FN_TABLE.with(|table| {
                    let mut table = table.borrow_mut();
                    let existing = table
                        .iter()
                        .find(|(_, candidate)| std::ptr::fn_addr_eq(*f, **candidate))
                        .map(|(index, _)| index);
                    existing.unwrap_or_else(|| table.insert(*f))
                }),
            }
        }
    }
}

pub fn soplang_to_value(sv: SoplangValue) -> Result<Value, SoplangError> {
    match sv.tag {
        TAG_NULL => Ok(Value::Null),
        TAG_INT => Ok(Value::Int(sv.payload)),
        TAG_FLOAT => Ok(Value::Float(f64::from_bits(sv.payload as u64))),
        TAG_BOOL => Ok(Value::Bool(sv.payload != 0)),
        TAG_ITERATOR => STATE.with(|state| state.iterators.borrow().get(sv.payload).cloned())
            .map(Value::Iterator).ok_or_else(invalid_handle),
        TAG_ERROR => ERROR_HEAP.with(|heap| heap.borrow().get(sv.payload).cloned())
            .map(Value::Error).ok_or_else(invalid_handle),
        TAG_STR => str_get(sv.payload).map(Value::Str),
        TAG_LIST => list_get(sv.payload).map(Value::List),
        TAG_BYTES => STATE
            .with(|state| state.bytes.borrow().get(sv.payload).cloned())
            .map(Value::Bytes)
            .ok_or_else(invalid_handle),
        TAG_OBJECT => obj_get(sv.payload).map(Value::Object),
        TAG_FUNC if native_fn_get(sv.payload).is_some() || compiled_fn_entry(sv.payload).is_some() => {
            Ok(Value::Function(super::value::FunctionId(sv.payload as usize)))
        }
        TAG_CLOSURE => CLOSURE_HEAP.with(|heap| heap.borrow().get(sv.payload).cloned())
            .map(Value::Closure).ok_or_else(invalid_handle),
        TAG_FUNC => Err(invalid_handle()),
        _ => Err(runtime_error(
            format!("Summadda qiimaha ma saxna: {}.", sv.tag), 0, 0,
        )),
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
pub extern "C" fn soplang_format_value(
    value: SoplangValue,
    spec: *const u8,
    len: usize,
) -> SoplangValue {
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    let text = if spec.is_null() || len == 0 {
        std::borrow::Cow::Borrowed("")
    } else {
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(spec, len) })
    };
    match soplang_to_value(value).and_then(|value| super::formatting::value(&value, &text)) {
        Ok(text) => value_to_soplang(&Value::Str(text)),
        Err(error) => handle_runtime_error(error),
    }
}

#[no_mangle]
pub extern "C" fn soplang_join_text(parts: *const SoplangValue, len: usize) -> SoplangValue {
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    if parts.is_null() && len != 0 {
        return handle_runtime_error(invalid_handle());
    }
    let parts = if len == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(parts, len) }
    };
    let text = STR_HEAP.with(|heap| {
        let heap = heap.borrow();
        let mut result = String::new();
        for part in parts {
            if part.tag != TAG_STR {
                return Err(invalid_handle());
            }
            let text = heap.get(part.payload).ok_or_else(invalid_handle)?;
            result.push_str(text);
        }
        Ok(result)
    });
    match text {
        Ok(text) => value_to_soplang(&Value::Str(text)),
        Err(error) => handle_runtime_error(error),
    }
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
        (Err(error), _) | (_, Err(error)) => handle_runtime_error(error),
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
        (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n.wrapping_add(*m))),
        (Value::Int(n), Value::Float(y)) => Ok(Value::Float(*n as f64 + y)),
        (Value::Float(x), Value::Int(m)) => Ok(Value::Float(x + *m as f64)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Str(a), b) => Ok(Value::Str(format!("{}{}", a, val_to_str(b)))),
        (a, Value::Str(b)) => Ok(Value::Str(format!("{}{}", val_to_str(a), b))),
        _ => Err(runtime_error(
            "Isku dar waa in ay noqdaan tiro ama qoraal",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_add(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
    binop(a, b, add_impl)
}

fn sub_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n.wrapping_sub(*m))),
        (Value::Int(n), Value::Float(y)) => Ok(Value::Float(*n as f64 - y)),
        (Value::Float(x), Value::Int(m)) => Ok(Value::Float(x - *m as f64)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
        _ => Err(runtime_error(
            "Hawlgalka (-) wuxuu u baahan yahay tirooyin.",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_sub(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
    binop(a, b, sub_impl)
}

fn mul_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n.wrapping_mul(*m))),
        (Value::Int(n), Value::Float(y)) => Ok(Value::Float(*n as f64 * y)),
        (Value::Float(x), Value::Int(m)) => Ok(Value::Float(x * *m as f64)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
        _ => Err(runtime_error(
            "Isku dhufashada waa in ay noqdaan tiro",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_mul(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
    binop(a, b, mul_impl)
}

fn div_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error("Tiro looma qaybin karo eber.", 0, 0))
            } else {
                Ok(Value::Float(*n as f64 / *m as f64))
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error("Tiro looma qaybin karo eber.", 0, 0))
            } else {
                Ok(Value::Float(x / y))
            }
        }
        (Value::Int(n), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error("Tiro looma qaybin karo eber.", 0, 0))
            } else {
                Ok(Value::Float(*n as f64 / y))
            }
        }
        (Value::Float(x), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error("Tiro looma qaybin karo eber.", 0, 0))
            } else {
                Ok(Value::Float(x / *m as f64))
            }
        }
        _ => Err(runtime_error(
            "Hawlgalka (/) wuxuu u baahan yahay tirooyin.",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_div(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
    binop(a, b, div_impl)
}

fn mod_impl(a: Value, b: Value) -> Result<Value, SoplangError> {
    match (&a, &b) {
        (Value::Int(n), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error(
                    "Hawlgalka (%) ma aqbalo qaybiye eber ah.",
                    0,
                    0,
                ))
            } else {
                Ok(Value::Int(n.wrapping_rem(*m)))
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error(
                    "Hawlgalka (%) ma aqbalo qaybiye eber ah.",
                    0,
                    0,
                ))
            } else {
                Ok(Value::Float(x % y))
            }
        }
        (Value::Int(n), Value::Float(y)) => {
            if *y == 0.0 {
                Err(runtime_error(
                    "Hawlgalka (%) ma aqbalo qaybiye eber ah.",
                    0,
                    0,
                ))
            } else {
                Ok(Value::Float(*n as f64 % y))
            }
        }
        (Value::Float(x), Value::Int(m)) => {
            if *m == 0 {
                Err(runtime_error(
                    "Hawlgalka (%) ma aqbalo qaybiye eber ah.",
                    0,
                    0,
                ))
            } else {
                Ok(Value::Float(x % *m as f64))
            }
        }
        _ => Err(runtime_error(
            "Hawlgalka (%) wuxuu u baahan yahay tirooyin.",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_mod(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
    binop(a, b, mod_impl)
}

fn neg_impl(a: Value) -> Result<Value, SoplangError> {
    match &a {
        Value::Int(n) => Ok(Value::Int(n.wrapping_neg())),
        Value::Float(x) => Ok(Value::Float(-x)),
        _ => Err(runtime_error(
            "Hawlgalka (-) wuxuu u baahan yahay tiro.",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_neg(a: SoplangValue) -> SoplangValue {
    check_handles!(a);
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
    check_handles!(a);
    match soplang_to_value(a) {
        Ok(va) => value_to_soplang(&not_impl(va)),
        _ => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_truthy(value: SoplangValue) -> SoplangValue {
    check_handles!(value);
    match soplang_to_value(value) {
        Ok(value) => value_to_soplang(&Value::Bool(value.is_truthy())),
        Err(error) => handle_runtime_error(error),
    }
}

#[no_mangle]
pub extern "C" fn soplang_range_continue(
    current: SoplangValue,
    end: SoplangValue,
    step: SoplangValue,
) -> SoplangValue {
    check_handles!(current, end, step);
    fn number(value: SoplangValue) -> Result<f64, SoplangError> {
        match soplang_to_value(value)? {
            Value::Int(value) => Ok(value as f64),
            Value::Float(value) => Ok(value),
            _ => Err(runtime_error("kuceli wuxuu u baahan yahay tirooyin", 0, 0)),
        }
    }

    if current.tag == TAG_INT && end.tag == TAG_INT && step.tag == TAG_INT {
        if step.payload == 0 {
            return handle_runtime_error(runtime_error(
                "tallaabada kuceli ma noqon karto eber",
                0,
                0,
            ));
        }
        return value_to_soplang(&Value::Bool(if step.payload > 0 {
            current.payload <= end.payload
        } else {
            current.payload >= end.payload
        }));
    }
    let result = (|| {
        let current = number(current)?;
        let end = number(end)?;
        let step = number(step)?;
        if !current.is_finite() || !end.is_finite() || !step.is_finite() {
            return Err(runtime_error("kuceli ma aqbalo tiro aan xad lahayn", 0, 0));
        }
        if step == 0.0 {
            return Err(runtime_error("tallaabada kuceli ma noqon karto eber", 0, 0));
        }
        Ok(Value::Bool(if step > 0.0 {
            current <= end
        } else {
            current >= end
        }))
    })();

    match result {
        Ok(value) => value_to_soplang(&value),
        Err(error) => handle_runtime_error(error),
    }
}

#[no_mangle]
pub extern "C" fn soplang_range_advance(
    current: SoplangValue,
    end: SoplangValue,
    step: SoplangValue,
) -> SoplangValue {
    check_handles!(current, end, step);
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    if current.tag == TAG_INT && step.tag == TAG_INT {
        return current
            .payload
            .checked_add(step.payload)
            .map(|next| value_to_soplang(&Value::Int(next)))
            .unwrap_or_else(SoplangValue::null);
    }
    let number = |value: SoplangValue| match value.tag {
        TAG_INT => Ok(value.payload as f64),
        TAG_FLOAT => Ok(f64::from_bits(value.payload as u64)),
        _ => Err(runtime_error("kuceli wuxuu u baahan yahay tirooyin", 0, 0)),
    };
    let result = (|| {
        let value = number(current)?;
        let end = number(end)?;
        let step = number(step)?;
        if !value.is_finite() || !end.is_finite() || !step.is_finite() {
            return Err(runtime_error("kuceli ma aqbalo tiro aan xad lahayn", 0, 0));
        }
        let next = value + step;
        if !next.is_finite() || (next == value && value == end) {
            return Ok(Value::Null);
        }
        if next == value {
            return Err(runtime_error(
                "Tallaabada kuceli ma beddelayso qiimaha hadda.",
                0,
                0,
            ));
        }
        Ok(Value::Float(next))
    })();
    match result {
        Ok(value) => value_to_soplang(&value),
        Err(error) => handle_runtime_error(error),
    }
}

fn cmp_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(n), Value::Int(m)) => n == m,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(n), Value::Float(y)) => *n as f64 == *y,
        (Value::Float(x), Value::Int(m)) => *x == *m as f64,
        _ => a == b,
    }
}

#[no_mangle]
pub extern "C" fn soplang_eq(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
    match (soplang_to_value(a), soplang_to_value(b)) {
        (Ok(va), Ok(vb)) => value_to_soplang(&Value::Bool(cmp_eq(&va, &vb))),
        _ => value_to_soplang(&Value::Bool(false)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_ne(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
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
        _ => Err(runtime_error(
            "Is barbar dhig waa in ay noqdaan tiro ama qoraal",
            0,
            0,
        )),
    }
}

#[no_mangle]
pub extern "C" fn soplang_lt(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
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
    check_handles!(a, b);
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
    check_handles!(a, b);
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
    check_handles!(a, b);
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
    if let Err(error) = validate_handle(v) {
        handle_runtime_error(error);
        return;
    }
    if let Ok(val) = soplang_to_value(v) {
        println!("{}", value_to_string(&val));
    }
}

#[no_mangle]
pub extern "C" fn soplang_gelin() -> SoplangValue {
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    match run_native("gelin", vec![]) {
        Ok(v) => value_to_soplang(&v),
        Err(_) => SoplangValue::null(),
    }
}

#[no_mangle]
pub extern "C" fn soplang_nooc(v: SoplangValue) -> SoplangValue {
    check_handles!(v);
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
    check_handles!(list, val);
    match (soplang_to_value(list), soplang_to_value(val)) {
        (Ok(Value::List(l)), Ok(v)) => {
            match super::contracts::mutate_list(&l, |items| {
                items.push(v);
                Ok(())
            }) {
                Ok(()) => value_to_soplang(&Value::List(l)),
                Err(error) => handle_runtime_error(error),
            }
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
    check_handles!(obj, idx);
    match (soplang_to_value(obj), soplang_to_value(idx)) {
        (Ok(Value::Bytes(bytes)), Ok(index)) => match super::bytes::get(&bytes, &index) {
            Ok(value) => value_to_soplang(&value),
            Err(error) => handle_runtime_error(error),
        },
        (Ok(Value::List(l)), Ok(Value::Int(i))) => {
            let v = l.borrow();
            let position = if i < 0 { v.len() as i64 + i } else { i };
            if position < 0 {
                return SoplangValue::null();
            }
            v.get(position as usize)
                .cloned()
                .map(|value| value_to_soplang(&value))
                .unwrap_or_else(SoplangValue::null)
        }
        (Ok(Value::Str(s)), Ok(Value::Int(i))) => {
            let len = s.chars().count() as i64;
            let position = if i < 0 { len + i } else { i };
            s.chars()
                .nth(position.max(0) as usize)
                .filter(|_| position >= 0 && position < len)
                .map(|ch| value_to_soplang(&Value::Str(ch.to_string())))
                .unwrap_or_else(SoplangValue::null)
        }
        (Ok(Value::Error(error)), Ok(Value::Str(key))) => value_to_soplang(&error_property(&error, &key)),
        (Ok(Value::Error(_)), Ok(_)) => handle_runtime_error(type_error("Furaha khaladku waa inuu noqdaa qoraal.", 0, 0)),
        (Ok(Value::Object(o)), Ok(Value::Str(key))) => o
            .borrow()
            .get(&key)
            .cloned()
            .map(|value| value_to_soplang(&value))
            .unwrap_or_else(SoplangValue::null),
        (Ok(Value::List(_)) | Ok(Value::Str(_)), Ok(_)) => handle_runtime_error(type_error(
            "Tusmada teed ama qoraal waa in ay noqoto abn",
            0,
            0,
        )),
        (Ok(Value::Object(_)), Ok(_)) => {
            handle_runtime_error(type_error("Furaha walaxdu waa in uu noqdo qoraal", 0, 0))
        }
        _ => handle_runtime_error(type_error("Qiimahan lama tusi karo", 0, 0)),
    }
}

#[no_mangle]
pub extern "C" fn soplang_set_index(
    obj: SoplangValue,
    idx: SoplangValue,
    val: SoplangValue,
) -> SoplangValue {
    check_handles!(obj, idx, val);
    match (
        soplang_to_value(obj),
        soplang_to_value(idx),
        soplang_to_value(val),
    ) {
        (Ok(Value::Bytes(bytes)), Ok(index), Ok(value)) => {
            match super::bytes::set(&bytes, &index, &value) {
                Ok(value) => value_to_soplang(&value),
                Err(error) => handle_runtime_error(error),
            }
        }
        (Ok(Value::List(l)), Ok(Value::Int(i)), Ok(v)) => {
            match super::contracts::mutate_list(&l, |items| {
                let len = items.len() as i64;
                let index = if i < 0 { len + i } else { i };
                if index < 0 || index >= len {
                    return Err(runtime_error("Tusmadu way ka baxsan tahay teedka", 0, 0));
                }
                items[index as usize] = v;
                Ok(())
            }) {
                Ok(()) => value_to_soplang(&Value::List(l)),
                Err(error) => handle_runtime_error(error),
            }
        }
        (Ok(Value::Object(o)), Ok(Value::Str(key)), Ok(v)) => {
            match super::contracts::mutate_object(&o, |items| {
                items.insert(key, v);
                Ok(())
            }) {
                Ok(()) => value_to_soplang(&Value::Object(o)),
                Err(error) => handle_runtime_error(error),
            }
        }
        (Ok(Value::List(_)), Ok(_), Ok(_)) => {
            handle_runtime_error(type_error("Tusmada teedku waa in ay noqoto abn", 0, 0))
        }
        (Ok(Value::Object(_)), Ok(_), Ok(_)) => {
            handle_runtime_error(type_error("Furaha walaxdu waa in uu noqdo qoraal", 0, 0))
        }
        (Ok(Value::Error(_)), _, _) => handle_runtime_error(type_error("Xogta khaladka lama beddeli karo.", 0, 0)),
        (Ok(Value::Str(_)), _, _) => handle_runtime_error(type_error(
            "Qoraal toos looma beddeli karo tusmo ahaan",
            0,
            0,
        )),
        _ => handle_runtime_error(type_error("Qiimahan tusmo laguma beddeli karo", 0, 0)),
    }
}

fn error_property(error: &SoplangError, key: &str) -> Value {
    let meta = error.meta();
    match key {
        "fariin" => Value::Str(error.message().into()),
        "summad" => error
            .code()
            .map(|code| Value::Str(code.into()))
            .unwrap_or(Value::Null),
        "sabab" => meta
            .cause
            .as_ref()
            .map(|cause| Value::Error(Rc::new((**cause).clone())))
            .unwrap_or(Value::Null),
        "fayl" => meta
            .file
            .as_ref()
            .map(|file| Value::Str(file.display().to_string()))
            .unwrap_or(Value::Null),
        "saf" => match error.location().0 {
            0 => Value::Null,
            line => Value::Int(line as i64),
        },
        "tiir" => match error.location().1 {
            0 => Value::Null,
            col => Value::Int(col as i64),
        },
        "raad" => Value::List(Rc::new(RefCell::new(
            meta.trace
                .iter()
                .map(|frame| {
                    Value::Object(Rc::new(RefCell::new(HashMap::from([
                        ("magac".into(), Value::Str(frame.function.clone())),
                        (
                            "fayl".into(),
                            frame
                                .file
                                .as_ref()
                                .map(|file| Value::Str(file.display().to_string()))
                                .unwrap_or(Value::Null),
                        ),
                        (
                            "saf".into(),
                            if frame.line == 0 {
                                Value::Null
                            } else {
                                Value::Int(frame.line as i64)
                            },
                        ),
                        (
                            "tiir".into(),
                            if frame.col == 0 {
                                Value::Null
                            } else {
                                Value::Int(frame.col as i64)
                            },
                        ),
                    ]))))
                })
                .collect(),
        ))),
        _ => Value::Null,
    }
}

#[no_mangle]
pub extern "C" fn soplang_get_prop(obj: SoplangValue, name: *const u8, len: usize) -> SoplangValue {
    check_handles!(obj);
    if name.is_null() {
        return SoplangValue::null();
    }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    match soplang_to_value(obj) {
        Ok(Value::Error(error)) => value_to_soplang(&error_property(&error, &key)),
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
pub extern "C" fn soplang_set_prop(
    obj: SoplangValue,
    name: *const u8,
    len: usize,
    val: SoplangValue,
) -> SoplangValue {
    check_handles!(obj, val);
    if name.is_null() {
        return SoplangValue::null();
    }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    match (soplang_to_value(obj), soplang_to_value(val)) {
        (Ok(Value::Object(o)), Ok(v)) => {
            match super::contracts::mutate_object(&o, |items| {
                items.insert(key, v);
                Ok(())
            }) {
                Ok(()) => value_to_soplang(&Value::Object(o)),
                Err(error) => handle_runtime_error(error),
            }
        }
        (Ok(Value::Error(_)), _) => handle_runtime_error(type_error("Xogta khaladka lama beddeli karo.", 0, 0)),
        _ => SoplangValue::null(),
    }
}

// ----- Logical operators (And / Or) -----

#[no_mangle]
pub extern "C" fn soplang_and(a: SoplangValue, b: SoplangValue) -> SoplangValue {
    check_handles!(a, b);
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
    check_handles!(a, b);
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

fn compiled_fn_entry(id: i64) -> Option<CompiledFnEntry> {
    COMPILED_FN_TABLE.with(|table| table.borrow().get(id).cloned())
}

pub(super) fn iteration_is_instance(value: &Value) -> bool {
    matches!(value, Value::Object(object) if get_instance_class(object).is_some())
}

pub(super) fn iteration_has_method(value: &Value, method: &str) -> bool {
    let Value::Object(object) = value else {
        return false;
    };
    let Ok(fields) = object.try_borrow() else {
        return false;
    };
    if let Some(Value::Str(class)) = fields.get(INSTANCE_CLASS_FIELD) {
        if resolve_method_fn_index(class, method).is_some() {
            return true;
        }
    }
    matches!(
        fields.get(method),
        Some(Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_))
    )
}

pub(super) fn iteration_call_method(value: &Value, method: &str) -> Result<Value, SoplangError> {
    let Value::Object(object) = value else {
        return Err(type_error("Wareeggu wuxuu u baahan yahay walax.", 0, 0));
    };
    object
        .try_borrow()
        .map_err(|_| super::iterators::borrowed_error())?;
    let source = CURRENT_SOURCE.with(|current| current.borrow().clone());
    let result = if get_instance_class(object).is_some() {
        dispatch_instance_method(object, method, &[])
    } else {
        dispatch_object_method(object, method, &[])
    };
    // Callback failures already own their inner span. Validation performed
    // after a successful callback belongs to the caller's iteration operation.
    CURRENT_SOURCE.with(|current| {
        *current.borrow_mut() = source;
    });
    // Preserve the first callback failure instead of replacing THROW with an
    // invalid-tag conversion error when an inner compiled function failed.
    PENDING_ERROR.with(|pending| match pending.borrow().as_ref() {
        Some(error) => Err(error.clone()),
        None => result,
    })
}

#[no_mangle]
pub extern "C" fn soplang_iter_begin(value: SoplangValue) -> SoplangValue {
    check_handles!(value);
    match soplang_to_value(value).and_then(|value| super::iterators::begin(&value)) {
        Ok(value) => value_to_soplang(&value),
        Err(error) => handle_runtime_error(error),
    }
}

#[no_mangle]
pub extern "C" fn soplang_iter_next(value: SoplangValue, bindings: usize) -> SoplangValue {
    check_handles!(value);
    match soplang_to_value(value).and_then(|value| super::iterators::next_step(&value, bindings)) {
        Ok(value) => value_to_soplang(&value),
        Err(error) => handle_runtime_error(error),
    }
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
                if let Some(&idx) = entry.methods.get(method).or_else(|| match method {
                    "dhis" => entry.methods.get("dhaw"),
                    "dhaw" => entry.methods.get("dhis"),
                    _ => None,
                }) {
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
    resolve_method_fn_index(class_name, "dhis")
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
    let sv = call_compiled_fn(entry, args.as_ptr(), args.len() as i32);
    soplang_to_value(sv)
}

fn dispatch_instance_method(
    o: &Rc<RefCell<HashMap<String, Value>>>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    let class_name = get_instance_class(o)
        .ok_or_else(|| runtime_error("Walaxdani ma aha nuqul qaab laga sameeyay.", 0, 0))?;
    let fn_idx = match resolve_method_fn_index(&class_name, method) {
        Some(index) => index,
        None => {
            let field = o.borrow().get(method).cloned();
            return match field {
                Some(value) => call_value(&value, args),
                None => Err(runtime_error(format!("Hawl '{}' lama helin qaabka '{}'", method, class_name), 0, 0)),
            };
        }
    };
    let entry = compiled_fn_entry(fn_idx)
        .ok_or_else(|| runtime_error(format!("Hawl '{}' lama diyaarin", method), 0, 0))?;
    let expected = entry
        .n_params
        .saturating_sub(1 + usize::from(entry.keyword_rest));
    if args.len() < entry.required_params.saturating_sub(1)
        || (!entry.variadic && args.len() > expected)
    {
        return Err(type_error(
            format!(
                "hawl '{}' waxay u baahan tahay {} dood, laakiin {} ayaa la bixiyay",
                method,
                crate::error::argument_count(
                    entry.required_params.saturating_sub(1),
                    if entry.variadic { usize::MAX } else { expected }
                ),
                args.len()
            ),
            0,
            0,
        ));
    }
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
    if class_ptr.is_null() || class_len == 0 || has_pending_error() {
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
            match soplang_to_value(sv) {
                Ok(value) => user_args.push(value),
                Err(error) => return handle_runtime_error(error),
            }
        }
    }

    if let Some(fn_idx) = resolve_constructor_fn_index(&class_name) {
        if let Some(entry) = compiled_fn_entry(fn_idx) {
            let expected = entry
                .n_params
                .saturating_sub(1 + usize::from(entry.keyword_rest));
            if user_args.len() < entry.required_params.saturating_sub(1)
                || (!entry.variadic && user_args.len() > expected)
            {
                return handle_runtime_error(type_error(
                    format!(
                        "dhaw() ee qaabka '{}' wuxuu u baahan yahay {} dood, laakiin {} ayaa la bixiyay",
                        class_name,
                        crate::error::argument_count(entry.required_params.saturating_sub(1), if entry.variadic { usize::MAX } else { expected }),
                        user_args.len()
                    ),
                    0,
                    0,
                ));
            }
            let instance_sv = value_to_soplang(&instance);
            let mut call_args = vec![instance_sv];
            for a in &user_args {
                call_args.push(value_to_soplang(a));
            }
            match call_compiled_fn_value(entry, &call_args) {
                Ok(_) => {}
                Err(e) => return handle_runtime_error(e),
            }
        }
    } else if !user_args.is_empty() {
        return handle_runtime_error(type_error(
            format!("Qaabka '{}' ma laha dhaw() qaata doodo", class_name),
            0,
            0,
        ));
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
    if name_ptr.is_null() || name_len == 0 {
        return;
    }
    let name = unsafe {
        String::from_utf8_lossy(std::slice::from_raw_parts(name_ptr, name_len)).into_owned()
    };
    let parent = if parent_ptr.is_null() || parent_len == 0 {
        None
    } else {
        Some(unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(parent_ptr, parent_len)).into_owned()
        })
    };
    let method = if method_ptr.is_null() || method_len == 0 {
        None
    } else {
        Some(unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(method_ptr, method_len)).into_owned()
        })
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
        if let Some(method) = method {
            entry.methods.insert(method, fn_payload);
        }
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
    check_handles!(obj);
    let method = if method_ptr.is_null() || method_len == 0 {
        ""
    } else {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(method_ptr, method_len)) }
    };
    let obj_val = match soplang_to_value(obj) {
        Ok(v) => v,
        Err(_) => return SoplangValue::null(),
    };
    let mut argv = Vec::new();
    if !args_ptr.is_null() && n > 0 {
        for i in 0..n as usize {
            let sv = unsafe { *args_ptr.add(i) };
            match soplang_to_value(sv) {
                Ok(value) => argv.push(value),
                Err(error) => return handle_runtime_error(error),
            }
        }
    }
    let result = if method == "wareeg" && !iteration_has_method(&obj_val, method) {
        super::stdlib::expect_arity(&argv, method, 0)
            .and_then(|_| super::iterators::begin(&obj_val))
    } else {
        match &obj_val {
            Value::Object(o) if get_instance_class(o).is_some() => {
                dispatch_instance_method(o, method, &argv)
            }
            Value::Iterator(_) => {
                super::stdlib::expect_arity(&argv, method, 0).and_then(|_| match method {
                    "wareeg" => super::iterators::begin(&obj_val),
                    "xiga" => super::iterators::next_step(&obj_val, 1),
                    _ => Err(runtime_error(
                        format!("Wareeggu ma laha hawl la yiraahdo '{}'.", method),
                        0,
                        0,
                    )),
                })
            }
            Value::List(l) => dispatch_list_method(l, method, &argv),
            Value::Bytes(bytes) => super::bytes::dispatch(bytes, method, &argv),
            Value::Object(o) => dispatch_object_method(o, method, &argv),
            Value::Str(s) => dispatch_string_method(s, method, &argv),
            _ => Err(runtime_error(
                format!(
                    "Nooca '{}' ma laha hawl la yiraahdo '{}'.",
                    obj_val.type_name(),
                    method
                ),
                0,
                0,
            )),
        }
    };
    match result {
        Ok(v) => value_to_soplang(&v),
        Err(e) => handle_runtime_error(e),
    }
}

#[repr(C)]
struct CompiledReturn {
    tag: i64,
    payload: i64,
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
type CompiledEntry = extern "sysv64" fn(*const SoplangValue, usize) -> CompiledReturn;
#[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
type CompiledEntry = extern "C" fn(*const SoplangValue, usize) -> CompiledReturn;

fn call_compiled_fn(function: CompiledFnEntry, args: *const SoplangValue, n: i32) -> SoplangValue {
    let CompiledFnEntry {
        ptr,
        n_params,
        required_params,
        variadic,
        keyword_rest,
        ..
    } = function;
    let maximum = n_params - usize::from(keyword_rest);
    if n < 0 || (n as usize) < required_params || (!variadic && n as usize > maximum) {
        return handle_runtime_error(type_error(
            format!(
                "Hawshu waxay u baahan tahay {} dood, laakiin {} ayaa la bixiyay",
                crate::error::argument_count(
                    required_params,
                    if variadic { usize::MAX } else { maximum }
                ),
                n.max(0)
            ),
            0,
            0,
        ));
    }
    if ptr.is_null() || (n > 0 && args.is_null()) {
        return handle_runtime_error(runtime_error("Liiska doodaha hawsha ma saxna.", 0, 0));
    }
    // Both code generators expose one vector entry, independent of arity.
    let entry: CompiledEntry = unsafe { std::mem::transmute(ptr) };
    // Entry code loads every parameter slot; omitted slots are initialized but
    // the original count tells the prologue which defaults to evaluate.
    let mut padded = Vec::new();
    let arguments = if variadic || keyword_rest || (n as usize) < n_params {
        let fixed = n_params - usize::from(variadic) - usize::from(keyword_rest);
        let supplied = if n > 0 {
            unsafe { std::slice::from_raw_parts(args, n as usize) }
        } else {
            &[]
        };
        padded.extend_from_slice(&supplied[..supplied.len().min(fixed)]);
        padded.resize(fixed, missing_argument());
        if variadic {
            let mut rest = Vec::new();
            for value in supplied.iter().skip(fixed) {
                match soplang_to_value(*value) {
                    Ok(value) => rest.push(value),
                    Err(error) => return handle_runtime_error(error),
                }
            }
            padded.push(value_to_soplang(&Value::List(Rc::new(RefCell::new(rest)))));
        }
        if keyword_rest {
            padded.push(value_to_soplang(&Value::Object(Rc::new(RefCell::new(
                HashMap::new(),
            )))));
        }
        padded.as_ptr()
    } else {
        args
    };
    let result = entry(arguments, n as usize);
    SoplangValue {
        tag: result.tag as u8,
        _pad: [0; 7],
        payload: result.payload,
    }
}

fn call_value(callee: &Value, args: &[Value]) -> Result<Value, SoplangError> {
    let callee = value_to_soplang(callee);
    let args: Vec<_> = args.iter().map(value_to_soplang).collect();
    call_collection_callback(callee, &args)
}

fn collection_callback(
    value: &Value,
    name: &str,
    supplied: usize,
) -> Result<SoplangValue, SoplangError> {
    let callee = match value {
        Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_) => {
            value_to_soplang(value)
        }
        _ => {
            return Err(type_error(
                format!("Hawsha '{}()' waxay u baahan tahay dood hawl ah", name),
                0,
                0,
            ))
        }
    };
    validate_handle(callee)?;
    let (entry, hidden) = if callee.tag == TAG_CLOSURE {
        let index = CLOSURE_HEAP.with(|heap| {
            heap.borrow()
                .get(callee.payload)
                .map(|closure| closure.fn_index)
        });
        (index.and_then(compiled_fn_entry), 1)
    } else {
        (compiled_fn_entry(callee.payload), 0)
    };
    let arity = if let Some(entry) = entry {
        Some(super::stdlib::CallbackArity {
            minimum: entry.required_params.saturating_sub(hidden),
            maximum: if entry.variadic {
                usize::MAX
            } else {
                entry
                    .n_params
                    .saturating_sub(hidden + usize::from(entry.keyword_rest))
            },
            stride: 1,
        })
    } else {
        native_fn_get(callee.payload).and_then(super::stdlib::builtin_callback_arity)
    };
    if let Some(arity) = arity {
        if !arity.accepts(supplied) {
            return Err(crate::error::type_error_ex(
                format!(
                    "Hawsha la siiyay '{}()' ma aqbasho {} dood; waxay u baahan tahay {}.",
                    name,
                    supplied,
                    arity.description()
                ),
                0,
                0,
                crate::error::ErrorMeta::default()
                    .with_code(crate::error::codes::E022_TYPE_MISMATCH),
            ));
        }
    }
    Ok(callee)
}

fn call_collection_callback(
    callee: SoplangValue,
    args: &[SoplangValue],
) -> Result<Value, SoplangError> {
    let result = soplang_call(callee, args.as_ptr(), args.len() as i32);
    if let Some(error) = PENDING_ERROR.with(|error| error.borrow().clone()) {
        return Err(error);
    }
    soplang_to_value(result)
}

fn dispatch_list_method(
    l: &Rc<RefCell<Vec<Value>>>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    // Report the spelling used at the call site when validating aliases.
    match method {
        "raadi" => super::stdlib::expect_arity(args, method, 1)?,
        "hore" | "dambe" => super::stdlib::expect_arity(args, method, 0)?,
        _ => {}
    }
    match method {
        "shaandhee" => {
            if args.len() != 1 {
                return Err(runtime_error(
                    "Hawsha 'shaandhee()' waxay u baahan tahay 1 dood oo hawl ah.",
                    0,
                    0,
                ));
            }
            let callee_sv = collection_callback(&args[0], method, 1)?;
            let mut out = Vec::new();
            let items = l.borrow().clone();
            for item in items {
                let item_sv = value_to_soplang(&item);
                if call_collection_callback(callee_sv, &[item_sv])?.is_truthy() {
                    out.push(item);
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(out))))
        }
        "aaddin" => {
            if args.len() != 1 {
                return Err(runtime_error(
                    "Hawsha 'aaddin()' waxay u baahan tahay 1 dood oo hawl ah.",
                    0,
                    0,
                ));
            }
            let callee_sv = collection_callback(&args[0], method, 1)?;
            let mut out = Vec::new();
            let items = l.borrow().clone();
            for item in items {
                let item_sv = value_to_soplang(&item);
                out.push(call_collection_callback(callee_sv, &[item_sv])?);
            }
            Ok(Value::List(Rc::new(RefCell::new(out))))
        }
        "ururi" | "isku_gee" => {
            if args.len() != 2 {
                return Err(type_error(
                    format!("{}() wuxuu u baahan yahay hawl iyo qiime bilow ah", method),
                    0,
                    0,
                ));
            }
            let callee = collection_callback(&args[0], method, 2)?;
            let mut accumulator = args[1].clone();
            let items = l.borrow().clone();
            for item in items {
                let call_args = [value_to_soplang(&accumulator), value_to_soplang(&item)];
                accumulator = call_collection_callback(callee, &call_args)?;
            }
            Ok(accumulator)
        }
        "dhammaan" | "qaar" => {
            if args.len() != 1 {
                return Err(type_error(
                    format!("{}() wuxuu u baahan yahay 1 hawl", method),
                    0,
                    0,
                ));
            }
            let callee = collection_callback(&args[0], method, 1)?;
            let want_every = method == "dhammaan";
            let items = l.borrow().clone();
            for item in items {
                let item = value_to_soplang(&item);
                let truthy = call_collection_callback(callee, &[item])?.is_truthy();
                if (want_every && !truthy) || (!want_every && truthy) {
                    return Ok(Value::Bool(!want_every));
                }
            }
            Ok(Value::Bool(want_every))
        }
        "kasaar" => super::stdlib::list_kasaar(Rc::clone(l), args),
        "dherer" => super::stdlib::list_dherer(Rc::clone(l), args),
        "hel" => super::stdlib::list_hel(Rc::clone(l), args),
        "dhig" => super::stdlib::list_dhig(Rc::clone(l), args),
        "saar" => super::stdlib::list_saar(Rc::clone(l), args),
        "kudar" => super::stdlib::list_kudar(Rc::clone(l), args),
        "midee" => super::stdlib::list_midee(Rc::clone(l), args),
        "leeyahay" => super::stdlib::list_leeyahay(Rc::clone(l), args),
        "nuqul" => super::stdlib::list_nuqul(Rc::clone(l), args),
        "nadiifi" => super::stdlib::list_nadiifi(Rc::clone(l), args),
        "rog" => super::stdlib::list_rog(Rc::clone(l), args),
        "habee" => super::stdlib::list_habee(Rc::clone(l), args),
        "jar" => super::stdlib::list_jar(Rc::clone(l), args),
        "tiri" => super::stdlib::list_tiri(Rc::clone(l), args),
        "raadi" | "muuji" => super::stdlib::list_muuji(Rc::clone(l), args),
        "geli" => super::stdlib::list_geli(Rc::clone(l), args),
        "ka_saar" => super::stdlib::list_ka_saar(Rc::clone(l), args),
        "hore" | "koowaad" => super::stdlib::list_koowaad(Rc::clone(l), args),
        "dambe" | "ugu_dambeeya" => super::stdlib::list_ugu_dambeeya(Rc::clone(l), args),
        _ => Err(runtime_error(
            format!("Teedku ma laha hawl la yiraahdo '{}'.", method),
            0,
            0,
        )),
    }
}

fn dispatch_object_method(
    o: &Rc<RefCell<HashMap<String, Value>>>,
    method: &str,
    args: &[Value],
) -> Result<Value, SoplangError> {
    // Report the spelling used at the call site when validating aliases.
    match method {
        "furayaal" | "qiimayaal" | "lammaanayaal" => super::stdlib::expect_arity(args, method, 0)?,
        "midee" => super::stdlib::expect_arity(args, method, 1)?,
        _ => {}
    }
    match method {
        "furayaal" | "fure" => super::stdlib::object_fure(Rc::clone(o), args),
        "dherer" => super::stdlib::object_dherer(Rc::clone(o), args),
        "saar" => super::stdlib::object_saar(Rc::clone(o), args),
        "leeyahay" => super::stdlib::object_leeyahay(Rc::clone(o), args),
        "tir" => super::stdlib::object_tir(Rc::clone(o), args),
        "kudar" => super::stdlib::object_kudar(Rc::clone(o), args),
        "midee" => super::stdlib::object_kudar(Rc::clone(o), args),
        "nuqul" => super::stdlib::object_nuqul(Rc::clone(o), args),
        "nadiifi" => super::stdlib::object_nadiifi(Rc::clone(o), args),
        "qiimayaal" | "qiime" => super::stdlib::object_qiime(Rc::clone(o), args),
        "lammaanayaal" | "lamaane" => super::stdlib::object_lamaane(Rc::clone(o), args),
        "hel" => super::stdlib::object_hel(Rc::clone(o), args),
        "dhig" => super::stdlib::object_dhig(Rc::clone(o), args),
        "ka_qaad" => super::stdlib::object_ka_qaad(Rc::clone(o), args),
        _ => {
            // Release the object borrow before invoking code that may mutate it.
            let field = o.borrow().get(method).cloned();
            match field {
                Some(value) => call_value(&value, args),
                None => Err(runtime_error(format!("Walaxdu ma laha hawl la yiraahdo '{}'.", method), 0, 0)),
            }
        }
    }
}

fn dispatch_string_method(s: &str, method: &str, args: &[Value]) -> Result<Value, SoplangError> {
    // Report the spelling used at the call site when validating aliases.
    match method {
        "weynee" | "yaree" | "masax_dhammaad" => super::stdlib::expect_arity(args, method, 0)?,
        "xiriiri" | "dhammaad" => super::stdlib::expect_arity(args, method, 1)?,
        _ => {}
    }
    match method {
        "qaabee" => super::stdlib::string_qaabee(s, args),
        "qeybi" => super::stdlib::string_qeybi(s.to_string(), args),
        "leeyahay" => super::stdlib::string_leeyahay(s.to_string(), args),
        "dhammaad" | "dhamaad" => super::stdlib::string_dhamaad(s.to_string(), args),
        "bilow" => super::stdlib::string_bilow(s.to_string(), args),
        "beddel" => super::stdlib::string_beddel(s.to_string(), args),
        "beddel_dhammaan" => super::stdlib::string_beddel_dhammaan(s.to_string(), args),
        "xiriiri" | "kudar" => super::stdlib::string_kudar(s.to_string(), args),
        "jar" => super::stdlib::string_jar(s.to_string(), args),
        "weynee" | "xarfaha_waaweyn" | "xarafaha_weyn" => {
            super::stdlib::string_xarfaha_waaweyn(s.to_string(), args)
        }
        "yaree" | "xarfaha_yaryar" => super::stdlib::string_xarfaha_yaryar(s.to_string(), args),
        "masax" => super::stdlib::string_masax(s.to_string(), args),
        "raadi" => super::stdlib::string_raadi(s.to_string(), args),
        "dherer" => super::stdlib::string_dherer(s.to_string(), args),
        "hel" => super::stdlib::string_hel(s.to_string(), args),
        "hore" => super::stdlib::string_hore(s.to_string(), args),
        "dambe" => super::stdlib::string_dambe(s.to_string(), args),
        "masax_bilow" => super::stdlib::string_masax_bilow(s.to_string(), args),
        "masax_dhammaad" | "masax_dhamaad" => {
            super::stdlib::string_masax_dhamaad(s.to_string(), args)
        }
        "tiri" => super::stdlib::string_tiri(s.to_string(), args),
        "kuceli" => super::stdlib::string_kuceli(s.to_string(), args),
        "ku_celi" => super::stdlib::string_ku_celi(s.to_string(), args),
        "raadi_dambe" => super::stdlib::string_raadi_dambe(s.to_string(), args),
        _ => Err(runtime_error(
            format!("Qoraalku ma laha hawl la yiraahdo '{}'.", method),
            0,
            0,
        )),
    }
}

/// Store a global variable (Rust API).
pub fn store_global(name: &str, val: SoplangValue) {
    GLOBAL_VARS.with(|g| g.borrow_mut().insert(name.to_string(), val));
}

/// Store a global variable by name (C ABI).
#[no_mangle]
pub extern "C" fn soplang_store_global(name: *const u8, len: usize, tag: i64, payload: i64) {
    // A failed check or RHS must not overwrite the previous global value.
    if name.is_null() || has_pending_error() {
        return;
    }
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
    let sv = SoplangValue {
        tag: tag as u8,
        _pad: [0; 7],
        payload,
    };
    GLOBAL_VARS.with(|g| g.borrow_mut().insert(key, sv));
}

/// Mark a global as constant (C ABI).
#[no_mangle]
pub extern "C" fn soplang_mark_const(name: *const u8, len: usize) {
    if name.is_null() {
        return;
    }
    let key = unsafe { std::slice::from_raw_parts(name, len) };
    let key = String::from_utf8_lossy(key).into_owned();
    CONST_GLOBALS.with(|c| c.borrow_mut().insert(key));
}

fn contract_error(message: impl Into<String>, line: usize, col: usize) -> SoplangError {
    crate::error::type_error_ex(
        message,
        line,
        col,
        crate::error::ErrorMeta::default().with_code(crate::error::codes::E022_TYPE_MISMATCH),
    )
}

/// Validate a flat union descriptor and return its accepted representation.
/// Generated descriptors use canonical type names and @Class alternatives.
#[no_mangle]
pub extern "C" fn soplang_check_contract(
    tag: i64,
    payload: i64,
    contract_ptr: *const u8,
    contract_len: usize,
    name_ptr: *const u8,
    name_len: usize,
) -> SoplangValue {
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    let result = (|| {
        let invalid = || contract_error("Qaabka noocyadu ma saxna.", 0, 0);
        let text = |ptr: *const u8, len: usize| -> Result<&str, SoplangError> {
            if len == 0 {
                return Ok("");
            }
            if ptr.is_null() {
                return Err(invalid());
            }
            std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, len) })
                .map_err(|_| invalid())
        };
        let contract = text(contract_ptr, contract_len)?;
        let name = text(name_ptr, name_len)?;
        let value = soplang_to_value(SoplangValue {
            tag: u8::try_from(tag).map_err(|_| invalid())?,
            _pad: [0; 7],
            payload,
        })?;
        super::contracts::check(value, contract, name).map(|value| value_to_soplang(&value))
    })();
    match result {
        Ok(value) => value,
        Err(error) => handle_runtime_error(error),
    }
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
    let expected =
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(class_ptr, class_len)) };
    let sv = SoplangValue {
        tag: tag as u8,
        _pad: [0; 7],
        payload,
    };
    let val = match soplang_to_value(sv) {
        Ok(value) => value,
        Err(error) => { handle_runtime_error(error); return; }
    };
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
    handle_runtime_error(err);
}

fn instance_matches_class(val: &Value, expected: &str) -> bool {
    let Value::Object(o) = val else {
        return false;
    };
    let Some(current) = get_instance_class(o) else {
        return false;
    };
    class_name_matches(&current, expected)
}

pub(super) fn class_name_matches(found: &str, expected: &str) -> bool {
    let mut current = found.to_string();
    let mut seen = std::collections::HashSet::new();
    loop {
        if current == expected {
            return true;
        }
        if !seen.insert(current.clone()) {
            return false;
        }
        let next = CLASS_REGISTRY.with(|r| {
            r.borrow()
                .get(&current)
                .and_then(|entry| entry.parent.clone())
        });
        match next {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

/// Runtime type validation for typed variable assignment (C ABI).
/// expected_type: 1=abn, 2=jajab, 3=qoraal, 4=bool, 5=teed, 6=walax, 7=baytyo, 0=dynamic(skip).
#[no_mangle]
pub extern "C" fn soplang_check_type(
    tag: i64,
    payload: i64,
    expected_type: i64,
    name_ptr: *const u8,
    name_len: usize,
) {
    if expected_type == 0 || has_pending_error() {
        return;
    }
    let sv = SoplangValue {
        tag: tag as u8,
        _pad: [0; 7],
        payload,
    };
    let val = match soplang_to_value(sv) {
        Ok(value) => value,
        Err(error) => { handle_runtime_error(error); return; }
    };
    let ok = match expected_type {
        1 => {
            matches!(val, Value::Int(_))
                || matches!(&val, Value::Float(x) if x.fract() == 0.0 && x.is_finite())
        }
        2 => matches!(val, Value::Float(_)) || matches!(val, Value::Int(_)),
        3 => matches!(val, Value::Str(_)),
        4 => matches!(val, Value::Bool(_)),
        5 => matches!(val, Value::List(_)),
        6 => matches!(val, Value::Object(_)),
        7 => matches!(val, Value::Bytes(_)),
        _ => true,
    };
    if !ok {
        let name = if name_ptr.is_null() {
            ""
        } else {
            unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(name_ptr, name_len)) }
        };
        let type_name = match expected_type {
            1 => "abn",
            2 => "jajab",
            3 => "qoraal",
            4 => "bool",
            5 => "teed",
            6 => "walax",
            7 => "baytyo",
            _ => "?",
        };
        let err = type_error(
            format!(
                "'{}' waa {} laakin qiimaheeda '{}' ma ahan {}",
                name, type_name, val, type_name
            ),
            0,
            0,
        );
        handle_runtime_error(err);
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
    if let Some(value) = GLOBAL_VARS.with(|g| g.borrow().get(&key).copied()) {
        return value;
    }
    let idx = BUILTIN_INDICES.with(|b| b.borrow().as_ref().and_then(|m| m.get(&key).copied()));
    if let Some(i) = idx {
        return SoplangValue {
            tag: TAG_FUNC,
            _pad: [0; 7],
            payload: i,
        };
    }
    handle_runtime_error(crate::error::type_error_ex(
        format!("Magaca '{}' lama aqoonsan", key),
        0,
        0,
        crate::error::ErrorMeta::default()
            .with_code(crate::error::codes::E021_UNDECLARED)
            .with_hint("Magacan qeex ka hor intaadan isticmaalin."),
    ))
}

/// Register a compiled function pointer in the active runtime context.
/// Returns an opaque handle for the payload of a TAG_FUNC value.
pub fn register_compiled_fn(ptr: *const u8, n_params: usize) -> i64 {
    register_compiled_fn_with_defaults(ptr, n_params, n_params)
}

/// Register a vector entry with a required positional prefix.
pub fn register_compiled_fn_with_defaults(
    ptr: *const u8,
    n_params: usize,
    required_params: usize,
) -> i64 {
    ensure_builtins();
    COMPILED_FN_TABLE.with(|table| {
        table.borrow_mut().insert(CompiledFnEntry {
            ptr,
            n_params,
            required_params: required_params.min(n_params),
            variadic: false,
            keyword_rest: false,
            names: None,
        })
    })
}

/// Configure an AOT function after registration, before it can be called.
#[no_mangle]
pub extern "C" fn soplang_set_fn_required(index: i64, required: usize) {
    COMPILED_FN_TABLE.with(|table| {
        let mut table = table.borrow_mut();
        if let Some(entry) = table.get_mut(index) {
            entry.required_params = required.min(entry.n_params);
        }
    });
}

/// Mark the final parameter slot as a list of remaining positional values.
#[no_mangle]
pub extern "C" fn soplang_set_fn_variadic(index: i64) {
    COMPILED_FN_TABLE.with(|table| {
        if let Some(entry) = table.borrow_mut().get_mut(index) {
            if entry.n_params > 0 {
                entry.variadic = true;
                entry.required_params = entry.required_params.min(entry.n_params - 1);
            }
        }
    });
}

/// Record parameter names (empty for implicit slots) and an optional keyword collector.
#[no_mangle]
pub extern "C" fn soplang_set_fn_signature(
    index: i64,
    names: *const u8,
    len: usize,
    keyword_rest: i64,
) {
    if names.is_null() && len > 0 {
        return;
    }
    let bytes = if len > 0 {
        unsafe { std::slice::from_raw_parts(names, len) }
    } else {
        &[]
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return;
    };
    COMPILED_FN_TABLE.with(|table| {
        if let Some(entry) = table.borrow_mut().get_mut(index) {
            let names: Vec<_> = if entry.n_params == 0 {
                Vec::new()
            } else {
                text.split('\0').map(str::to_owned).collect()
            };
            if names.len() == entry.n_params {
                entry.names = Some(Rc::new(names));
                entry.keyword_rest =
                    keyword_rest != 0 && entry.n_params > usize::from(entry.variadic);
            }
        }
    });
}

fn missing_argument() -> SoplangValue {
    SoplangValue {
        tag: TAG_MISSING,
        _pad: [0; 7],
        payload: 0,
    }
}

fn named_argument_error(message: impl Into<String>) -> SoplangError {
    crate::error::type_error_ex(
        message,
        0,
        0,
        crate::error::ErrorMeta::default().with_code(crate::error::codes::E022_TYPE_MISMATCH),
    )
}

fn invoke_named_entry(
    entry: CompiledFnEntry,
    args: &[SoplangValue],
    labels: &[String],
) -> Result<SoplangValue, SoplangError> {
    let names = entry
        .names
        .as_ref()
        .ok_or_else(|| named_argument_error("Hawshani ma taageerto doodaha magacyada leh."))?;
    let fixed = entry.n_params - usize::from(entry.variadic) - usize::from(entry.keyword_rest);
    let mut bound = vec![missing_argument(); fixed];
    let mut rest = Vec::new();
    let mut keywords = HashMap::new();
    let mut position = 0;
    let mut named = false;
    for (&value, label) in args.iter().zip(labels) {
        if label.is_empty() {
            if named {
                return Err(named_argument_error(
                    "Dood boos leh ma raaci karto dood magac leh.",
                ));
            }
            if position < fixed {
                bound[position] = value;
            } else if entry.variadic {
                rest.push(soplang_to_value(value)?);
            } else {
                return Err(named_argument_error(
                    "Doodaha boosaska leh ayaa ka badan intii hawshu aqbali lahayd.",
                ));
            }
            position += 1;
        } else {
            named = true;
            if let Some(index) = names[..fixed]
                .iter()
                .position(|name| name == label && !name.is_empty())
            {
                if bound[index].tag != TAG_MISSING {
                    return Err(named_argument_error(format!(
                        "Doodda '{}' laba jeer ayaa la bixiyay.",
                        label
                    )));
                }
                bound[index] = value;
            } else if entry.keyword_rest {
                if keywords
                    .insert(label.clone(), soplang_to_value(value)?)
                    .is_some()
                {
                    return Err(named_argument_error(format!(
                        "Doodda '{}' laba jeer ayaa la bixiyay.",
                        label
                    )));
                }
            } else {
                return Err(named_argument_error(format!(
                    "Doodda magaca '{}' leh hawshani ma aqbasho.",
                    label
                )));
            }
        }
    }
    for index in 0..entry.required_params {
        if bound
            .get(index)
            .is_none_or(|value| value.tag == TAG_MISSING)
        {
            return Err(named_argument_error(format!(
                "Doodda '{}' ayaa ka maqan wicitaankan.",
                names[index]
            )));
        }
    }
    if entry.variadic {
        bound.push(value_to_soplang(&Value::List(Rc::new(RefCell::new(rest)))));
    }
    if entry.keyword_rest {
        bound.push(value_to_soplang(&Value::Object(Rc::new(RefCell::new(
            keywords,
        )))));
    }
    if entry.ptr.is_null() {
        return Err(named_argument_error("Hawsha lama diyaarin."));
    }
    let function: CompiledEntry = unsafe { std::mem::transmute(entry.ptr) };
    let result = function(bound.as_ptr(), args.len());
    Ok(SoplangValue {
        tag: result.tag as u8,
        _pad: [0; 7],
        payload: result.payload,
    })
}

fn named_callable(
    callee: SoplangValue,
    args: &[SoplangValue],
    labels: &[String],
) -> Result<SoplangValue, SoplangError> {
    validate_handle(callee)?;
    if callee.tag == TAG_CLOSURE {
        let data = CLOSURE_HEAP
            .with(|heap| heap.borrow().get(callee.payload).cloned())
            .ok_or_else(invalid_handle)?;
        let entry = compiled_fn_entry(data.fn_index).ok_or_else(invalid_handle)?;
        let mut full = vec![SoplangValue {
            tag: TAG_NULL,
            _pad: [0; 7],
            payload: callee.payload,
        }];
        full.extend_from_slice(args);
        let mut names = vec![String::new()];
        names.extend_from_slice(labels);
        invoke_named_entry(entry, &full, &names)
    } else if callee.tag == TAG_FUNC {
        let entry = compiled_fn_entry(callee.payload).ok_or_else(|| {
            named_argument_error("Hawshani waxay aqbashaa doodaha boosaska leh oo keliya.")
        })?;
        invoke_named_entry(entry, args, labels)
    } else {
        Err(named_argument_error("Qiimahan hawl ahaan looma wici karo."))
    }
}

fn named_instance_call(
    index: i64,
    instance: Value,
    args: &[SoplangValue],
    labels: &[String],
) -> Result<SoplangValue, SoplangError> {
    let entry = compiled_fn_entry(index).ok_or_else(invalid_handle)?;
    let mut full = vec![value_to_soplang(&instance)];
    full.extend_from_slice(args);
    let mut names = vec![String::new()];
    names.extend_from_slice(labels);
    invoke_named_entry(entry, &full, &names)
}

/// Named call descriptor: empty labels denote positional arguments. Kinds are
/// ordinary callable (0), object method (1), and constructor (2).
#[no_mangle]
pub extern "C" fn soplang_call_named(
    callee: SoplangValue,
    kind: i64,
    name: *const u8,
    name_len: usize,
    args: *const SoplangValue,
    n: i32,
    labels: *const u8,
    labels_len: usize,
) -> SoplangValue {
    if has_pending_error() {
        return SoplangValue::null();
    }
    let execute = || -> Result<SoplangValue, SoplangError> {
        if n < 0
            || (n > 0 && args.is_null())
            || (name_len > 0 && name.is_null())
            || (labels_len > 0 && labels.is_null())
        {
            return Err(named_argument_error("Liiska doodaha hawsha ma saxna."));
        }
        let args = if n > 0 {
            unsafe { std::slice::from_raw_parts(args, n as usize) }
        } else {
            &[]
        };
        for value in args {
            validate_handle(*value)?;
            soplang_to_value(*value)?;
        }
        let read = |pointer, len| {
            if len == 0 {
                Ok("")
            } else {
                std::str::from_utf8(unsafe { std::slice::from_raw_parts(pointer, len) }).map_err(
                    |_| named_argument_error("Magacyada dooduhu ma aha qoraal UTF-8 ah oo sax ah."),
                )
            }
        };
        let name = read(name, name_len)?;
        let text = read(labels, labels_len)?;
        let labels: Vec<String> = if n == 0 {
            Vec::new()
        } else {
            text.split('\0').map(str::to_owned).collect()
        };
        if labels.len() != args.len() {
            return Err(named_argument_error(
                "Magacyada iyo qiimayaasha dooduhu isma laha.",
            ));
        }
        match kind {
            0 => named_callable(callee, args, &labels),
            1 => {
                let Value::Object(object) = soplang_to_value(callee)? else {
                    return Err(named_argument_error(
                        "Hawshani waxay aqbashaa doodaha boosaska leh oo keliya.",
                    ));
                };
                let (class, field) = {
                    let fields = object
                        .try_borrow()
                        .map_err(|_| named_argument_error("Walaxda hadda lama akhriyi karo."))?;
                    let class = match fields.get(INSTANCE_CLASS_FIELD) {
                        Some(Value::Str(name)) => Some(name.clone()),
                        _ => None,
                    };
                    (class, fields.get(name).cloned())
                };
                if let Some(class) = &class {
                    if let Some(index) = resolve_method_fn_index(class, name) {
                        return named_instance_call(index, Value::Object(object), args, &labels);
                    }
                } else if matches!(
                    name,
                    "furayaal"
                        | "fure"
                        | "dherer"
                        | "saar"
                        | "leeyahay"
                        | "tir"
                        | "kudar"
                        | "midee"
                        | "nuqul"
                        | "nadiifi"
                        | "qiimayaal"
                        | "qiime"
                        | "lammaanayaal"
                        | "lamaane"
                        | "hel"
                        | "dhig"
                        | "ka_qaad"
                ) {
                    return Err(named_argument_error(
                        "Hawshani waxay aqbashaa doodaha boosaska leh oo keliya.",
                    ));
                }
                let field = field.ok_or_else(|| {
                    named_argument_error(format!("Hawl '{}' lama helin walaxda.", name))
                })?;
                named_callable(value_to_soplang(&field), args, &labels)
            }
            2 => {
                let mut fields = HashMap::new();
                fields.insert(
                    INSTANCE_CLASS_FIELD.to_string(),
                    Value::Str(name.to_string()),
                );
                let instance = Value::Object(Rc::new(RefCell::new(fields)));
                if let Some(index) = resolve_constructor_fn_index(name) {
                    let result = named_instance_call(index, instance.clone(), args, &labels)?;
                    if has_pending_error() {
                        return Ok(result);
                    }
                    Ok(value_to_soplang(&instance))
                } else {
                    Err(named_argument_error(format!(
                        "Qaabka '{}' ma laha dhise aqbala doodahan.",
                        name
                    )))
                }
            }
            _ => Err(named_argument_error("Nooca wicitaanku ma saxna.")),
        }
    };
    match execute() {
        Ok(value) => value,
        Err(error) => handle_runtime_error(error),
    }
}

/// Record an AOT function pointer under `name` so calls, methods, and closures can find it.
#[no_mangle]
pub extern "C" fn soplang_register_user_fn(
    name: *const u8,
    name_len: usize,
    ptr: *const u8,
    n_params: i64,
) -> i64 {
    let idx = register_compiled_fn(ptr, n_params.max(0) as usize);
    if !name.is_null() && name_len > 0 {
        let key = unsafe {
            String::from_utf8_lossy(std::slice::from_raw_parts(name, name_len)).into_owned()
        };
        store_global(
            &key,
            SoplangValue {
                tag: TAG_FUNC,
                _pad: [0; 7],
                payload: idx,
            },
        );
    }
    idx
}

pub struct ClosureData {
    id: Cell<i64>,
    pub(super) fn_index: i64,
    pub(super) env: Vec<SoplangValue>,
}

impl ClosureData {
    pub fn handle(&self) -> i64 { self.id.get() }
}

/// Build a closure from a compiled function name and the captured values.
#[no_mangle]
pub extern "C" fn soplang_make_closure(
    name: *const u8,
    name_len: usize,
    n: i64,
    args: *const SoplangValue,
) -> SoplangValue {
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    let callee = soplang_get_builtin(name, name_len);
    if callee.tag != TAG_FUNC {
        let val = soplang_to_value(callee).unwrap_or(Value::Null);
        return handle_runtime_error(runtime_error(
            format!("'{}' ma ahan hawl, lama xidhi karo", value_to_string(&val)),
            0,
            0,
        ));
    }
    let mut env = Vec::new();
    if !args.is_null() && n > 0 {
        for i in 0..n as usize {
            let value = unsafe { *args.add(i) };
            check_handles!(value);
            env.push(value);
        }
    }
    let data = Rc::new(ClosureData {
        id: Cell::new(0),
        fn_index: callee.payload,
        env,
    });
    COLLECTOR.with(|collector| collector.allocated(data.env.len() * 16));
    let payload = CLOSURE_HEAP.with(|heap| heap.borrow_mut().insert(Rc::clone(&data)));
    data.id.set(payload);
    SoplangValue {
        tag: TAG_CLOSURE,
        _pad: [0; 7],
        payload,
    }
}

/// Read one captured value. `env_ptr` is the owning context's closure handle.
#[no_mangle]
pub extern "C" fn soplang_capture_get(env_ptr: i64, index: i64) -> SoplangValue {
    if let Some(value) = short_circuit_if_pending() {
        return value;
    }
    let Some(data) = CLOSURE_HEAP.with(|heap| heap.borrow().get(env_ptr).cloned()) else {
        return handle_runtime_error(invalid_handle());
    };
    if index < 0 {
        return SoplangValue::null();
    }
    data.env
        .get(index as usize)
        .copied()
        .unwrap_or_else(SoplangValue::null)
}

fn call_closure(callee: SoplangValue, args: *const SoplangValue, n: i32) -> SoplangValue {
    if callee.payload == 0 {
        return SoplangValue::null();
    }
    let Some(data) = CLOSURE_HEAP.with(|heap| heap.borrow().get(callee.payload).cloned()) else {
        return handle_runtime_error(invalid_handle());
    };
    let mut full = Vec::with_capacity(1 + n.max(0) as usize);
    full.push(SoplangValue {
        tag: TAG_NULL,
        _pad: [0; 7],
        payload: callee.payload,
    });
    if !args.is_null() && n > 0 {
        for i in 0..n as usize {
            full.push(unsafe { *args.add(i) });
        }
    }
    let entry = compiled_fn_entry(data.fn_index);
    if let Some(entry) = entry {
        let expected = entry
            .n_params
            .saturating_sub(1 + usize::from(entry.keyword_rest));
        if n < 0
            || (n as usize) < entry.required_params.saturating_sub(1)
            || (!entry.variadic && n as usize > expected)
        {
            return handle_runtime_error(type_error(
                format!(
                    "Hawshu waxay u baahan tahay {} dood, laakiin {} ayaa la bixiyay",
                    crate::error::argument_count(
                        entry.required_params.saturating_sub(1),
                        if entry.variadic { usize::MAX } else { expected }
                    ),
                    n.max(0)
                ),
                0,
                0,
            ));
        }
        call_compiled_fn(entry, full.as_ptr(), full.len() as i32)
    } else {
        SoplangValue::null()
    }
}

/// Call a callee with args. Callee is TAG_FUNC (native or user) or TAG_CLOSURE.
#[no_mangle]
pub extern "C" fn soplang_call(
    callee: SoplangValue,
    args: *const SoplangValue,
    n: i32,
) -> SoplangValue {
    check_handles!(callee);
    if let Err(error) = soplang_to_value(callee) {
        return handle_runtime_error(error);
    }
    if !args.is_null() && n > 0 {
        for index in 0..n as usize {
            check_handles!(unsafe { *args.add(index) });
        }
    }
    if callee.tag == TAG_CLOSURE {
        return call_closure(callee, args, n);
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
                match soplang_to_value(sv) {
                    Ok(value) => argv.push(value),
                    Err(error) => return handle_runtime_error(error),
                }
            }
        }
        match f(argv) {
            Ok(v) => value_to_soplang(&v),
            Err(e) => handle_runtime_error(e),
        }
    } else {
        let entry = compiled_fn_entry(callee.payload);
        if let Some(entry) = entry {
            call_compiled_fn(entry, args, n)
        } else {
            SoplangValue::null()
        }
    }
}
