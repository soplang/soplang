//! Regular-expression search and replacement.
//!
//! Patterns use the Rust `regex` dialect: Unicode by default, without
//! backreferences or look-around.

use std::cell::RefCell;
use std::rc::Rc;

use regex::Regex;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

fn compile(pattern: &str) -> Result<Regex, SoplangError> {
    Regex::new(pattern).map_err(|error| {
        runtime_error_ex(
            "Qaabka raadinta ma saxna.",
            0,
            0,
            ErrorMeta::default()
                .with_code(codes::E049_REGEX)
                .with_details(error.to_string()),
        )
    })
}

pub fn find(pattern: &str, text: &str) -> Result<Value, SoplangError> {
    let regex = compile(pattern)?;
    Ok(regex
        .find(text)
        .map(|found| Value::Str(found.as_str().to_string()))
        .unwrap_or(Value::Null))
}

pub fn find_all(pattern: &str, text: &str) -> Result<Value, SoplangError> {
    let regex = compile(pattern)?;
    let matches = regex
        .find_iter(text)
        .map(|found| Value::Str(found.as_str().to_string()))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(matches))))
}

pub fn replace(pattern: &str, text: &str, replacement: &str) -> Result<Value, SoplangError> {
    let regex = compile(pattern)?;
    Ok(Value::Str(
        regex.replace_all(text, replacement).into_owned(),
    ))
}
