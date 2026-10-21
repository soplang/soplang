//! Percent-encoding for URL components and form bodies.

use super::value::Value;
use crate::error::{codes, runtime_error_ex, type_error, ErrorMeta, SoplangError};

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E047_URL),
    )
}

pub fn encode(text: &str, form: bool) -> String {
    let mut out = String::new();
    for byte in text.as_bytes() {
        if unreserved(*byte) {
            out.push(*byte as char);
        } else if form && *byte == b' ' {
            out.push('+');
        } else {
            out.push('%');
            out.push(hex(byte >> 4));
            out.push(hex(byte & 0x0F));
        }
    }
    out
}

pub fn decode(text: &str, form: bool) -> Result<String, SoplangError> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' if form => {
                out.push(b' ');
                index += 1;
            }
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(error("Qaybta boqolkiiba ee URL ma dhammaystirna."));
                }
                let high = hex_value(bytes[index + 1])?;
                let low = hex_value(bytes[index + 2])?;
                out.push((high << 4) | low);
                index += 3;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| error("URL-ka la furay ma aha qoraal UTF-8 ah oo sax ah."))
}

pub fn form_mode(value: Option<&Value>, name: &str) -> Result<bool, SoplangError> {
    match value {
        None => Ok(false),
        Some(Value::Str(mode)) if mode == "foom" => Ok(true),
        Some(Value::Str(_)) => Err(error(format!(
            "{name}() wuxuu aqbalo habka \"foom\" kaliya."
        ))),
        Some(_) => Err(type_error(
            format!("{name}() wuxuu u baahan yahay hab qoraal ah."),
            0,
            0,
        )),
    }
}

fn unreserved(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~')
}

fn hex(value: u8) -> char {
    b"0123456789ABCDEF"[value as usize] as char
}

fn hex_value(byte: u8) -> Result<u8, SoplangError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(error("Qaybta boqolkiiba ee URL ma saxna.")),
    }
}
