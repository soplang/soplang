//! CSV rows of text fields.

use std::cell::RefCell;
use std::rc::Rc;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E046_CSV),
    )
}

pub fn parse(text: &str) -> Result<Value, SoplangError> {
    if text.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(Vec::new()))));
    }
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = text.chars().peekable();
    let mut quoted = false;
    let mut started = false;
    while let Some(ch) = chars.next() {
        if quoted {
            match ch {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        quoted = false;
                    }
                }
                _ => field.push(ch),
            }
            continue;
        }
        match ch {
            '"' => {
                if !field.is_empty() {
                    return Err(error("Xajinta CSV waa inay bilaabato qoraal la xiray."));
                }
                quoted = true;
                started = true;
            }
            ',' => {
                row.push(Value::Str(std::mem::take(&mut field)));
                started = true;
            }
            '\n' | '\r' => {
                if ch == '\r' && chars.peek() == Some(&'\n') {
                    chars.next();
                }
                row.push(Value::Str(std::mem::take(&mut field)));
                rows.push(Value::List(Rc::new(RefCell::new(std::mem::take(&mut row)))));
                started = false;
            }
            _ => {
                field.push(ch);
                started = true;
            }
        }
    }
    if quoted {
        return Err(error("Xajinta CSV lama xidhin."));
    }
    if started {
        row.push(Value::Str(field));
        rows.push(Value::List(Rc::new(RefCell::new(row))));
    }
    Ok(Value::List(Rc::new(RefCell::new(rows))))
}

pub fn generate(value: &Value) -> Result<String, SoplangError> {
    let Value::List(rows) = value else {
        return Err(error("csv_qor() wuxuu u baahan yahay teed safaf ah."));
    };
    let rows = rows
        .try_borrow()
        .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
    let mut out = String::new();
    for (index, row) in rows.iter().enumerate() {
        let Value::List(fields) = row else {
            return Err(error("Saf kasta oo CSV ah waa inuu noqdaa teed."));
        };
        let fields = fields
            .try_borrow()
            .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
        if index > 0 {
            out.push('\n');
        }
        for (field_index, field) in fields.iter().enumerate() {
            if field_index > 0 {
                out.push(',');
            }
            write_field(&cell_text(field)?, &mut out);
        }
    }
    Ok(out)
}

fn cell_text(value: &Value) -> Result<String, SoplangError> {
    match value {
        Value::Str(text) => Ok(text.clone()),
        Value::Int(value) => Ok(value.to_string()),
        Value::Float(value) if value.is_finite() => Ok(value.to_string()),
        Value::Null => Ok(String::new()),
        _ => Err(error("Unugga CSV waa inuu noqdaa qoraal, abn, jajab, ama maran.")),
    }
}

fn write_field(text: &str, out: &mut String) {
    if text.contains([',', '"', '\n', '\r']) {
        out.push('"');
        for ch in text.chars() {
            if ch == '"' {
                out.push('"');
            }
            out.push(ch);
        }
        out.push('"');
    } else {
        out.push_str(text);
    }
}
