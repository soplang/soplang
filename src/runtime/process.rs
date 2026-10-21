//! Program arguments, environment variables, and child processes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::process::Command;
use std::rc::Rc;
use std::sync::Mutex;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

static PROGRAM_ARGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E051_PROCESS),
    )
}

fn command_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E052_COMMAND),
    )
}

pub fn set_program_arguments(args: Vec<String>) {
    *PROGRAM_ARGS.lock().unwrap_or_else(|poison| poison.into_inner()) = args;
}

pub fn program_arguments() -> Vec<String> {
    PROGRAM_ARGS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone()
}

pub fn arguments_list() -> Value {
    Value::List(std::rc::Rc::new(std::cell::RefCell::new(
        program_arguments().into_iter().map(Value::Str).collect(),
    )))
}

pub fn environment(name: &str) -> Result<Value, SoplangError> {
    check_name(name)?;
    Ok(match std::env::var_os(name) {
        Some(value) => Value::Str(value.to_string_lossy().into_owned()),
        None => Value::Null,
    })
}

pub fn set_environment(name: &str, value: &str) -> Result<(), SoplangError> {
    check_name(name)?;
    // The process environment is shared mutable state. Rust 1.87 marks the
    // setter unsafe because another thread may be reading it.
    unsafe { std::env::set_var(name, value) };
    Ok(())
}

pub fn remove_environment(name: &str) -> Result<(), SoplangError> {
    check_name(name)?;
    unsafe { std::env::remove_var(name) };
    Ok(())
}

fn check_name(name: &str) -> Result<(), SoplangError> {
    if name.is_empty() || name.contains(['=', '\0']) {
        return Err(error("Magaca deegaanka ma saxna."));
    }
    Ok(())
}

/// Install argv from a C main, skipping the program name.
///
/// # Safety
/// `argv` must contain `argc` readable C strings.
pub unsafe fn install_c_arguments(argc: i32, argv: *const *const i8) {
    if argv.is_null() || argc <= 1 {
        set_program_arguments(Vec::new());
        return;
    }
    let mut args = Vec::new();
    for index in 1..argc {
        let pointer = unsafe { *argv.add(index as usize) };
        if pointer.is_null() {
            continue;
        }
        let bytes = unsafe { CStr::from_ptr(pointer) };
        args.push(bytes.to_string_lossy().into_owned());
    }
    set_program_arguments(args);
}

pub fn run_command(program: &str, args: &[String]) -> Result<Value, SoplangError> {
    if program.is_empty() {
        return Err(command_error("Magaca barnaamijka ma madhnaan karo."));
    }
    let output = Command::new(program).args(args).output().map_err(|error| {
        command_error(format!(
            "Barnaamijka '{program}' lama bilaabi karin. {}",
            crate::error::io_reason(&error)
        ))
    })?;
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|_| command_error("Soosaarka barnaamijka ma aha qoraal UTF-8 ah oo sax ah."))?;
    let stderr = std::str::from_utf8(&output.stderr)
        .map_err(|_| command_error("Qaladka barnaamijka ma aha qoraal UTF-8 ah oo sax ah."))?;
    let mut fields = HashMap::new();
    fields.insert(
        "koodh".to_string(),
        output
            .status
            .code()
            .map(|code| Value::Int(i64::from(code)))
            .unwrap_or(Value::Null),
    );
    fields.insert("soosaar".to_string(), Value::Str(stdout.to_string()));
    fields.insert("qalad".to_string(), Value::Str(stderr.to_string()));
    fields.insert("guul".to_string(), Value::Bool(output.status.success()));
    Ok(Value::Object(Rc::new(RefCell::new(fields))))
}
