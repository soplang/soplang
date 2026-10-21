//! Standard and URL-safe Base64.

use super::value::Value;
use crate::error::{codes, runtime_error_ex, type_error, ErrorMeta, SoplangError};

const STANDARD: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E048_BASE64),
    )
}

pub fn encode(data: &[u8], url: bool) -> String {
    let alphabet = if url { URL_SAFE } else { STANDARD };
    let mut out = String::new();
    let mut index = 0;
    while index + 3 <= data.len() {
        push_group(&data[index..index + 3], 3, alphabet, &mut out);
        index += 3;
    }
    let rest = data.len() - index;
    if rest > 0 {
        let mut group = [0u8; 3];
        group[..rest].copy_from_slice(&data[index..]);
        push_group(&group, rest, alphabet, &mut out);
        if !url {
            for _ in rest..3 {
                out.push('=');
            }
        }
    }
    out
}

pub fn decode(text: &str, url: bool) -> Result<Vec<u8>, SoplangError> {
    let bytes = text.as_bytes();
    if bytes.iter().any(u8::is_ascii_whitespace) {
        return Err(error("Base64 ma aqbalo meel bannaan."));
    }
    let mut payload = bytes;
    let mut padding = 0;
    if !url {
        while payload.last() == Some(&b'=') {
            padding += 1;
            payload = &payload[..payload.len() - 1];
        }
        if padding > 2 || (bytes.len() % 4) != 0 {
            return Err(error("Dhererka Base64 ma saxna."));
        }
    } else if payload.contains(&b'=') {
        return Err(error("Base64 url ma isticmaalo calaamadda =."));
    }
    if payload.len() % 4 == 1 {
        return Err(error("Dhererka Base64 ma saxna."));
    }
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;
    for byte in payload {
        let value = decode_char(*byte, url)?;
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    if !url {
        let expected_padding = match payload.len() % 4 {
            0 => 0,
            2 => 2,
            3 => 1,
            _ => return Err(error("Dhererka Base64 ma saxna.")),
        };
        if padding != expected_padding {
            return Err(error("Calaamadda Base64 ma saxna."));
        }
    }
    Ok(out)
}

pub fn url_mode(value: Option<&Value>, name: &str) -> Result<bool, SoplangError> {
    match value {
        None => Ok(false),
        Some(Value::Str(mode)) if mode == "url" => Ok(true),
        Some(Value::Str(_)) => Err(error(format!(
            "{name}() wuxuu aqbalo habka \"url\" kaliya."
        ))),
        Some(_) => Err(type_error(
            format!("{name}() wuxuu u baahan yahay hab qoraal ah."),
            0,
            0,
        )),
    }
}

fn push_group(group: &[u8], significant: usize, alphabet: &[u8], out: &mut String) {
    let packed = (u32::from(group[0]) << 16) | (u32::from(group[1]) << 8) | u32::from(group[2]);
    let chars = significant + 1;
    for step in 0..chars {
        let shift = 3 - step;
        let index = ((packed >> (6 * shift)) & 0x3F) as usize;
        out.push(alphabet[index] as char);
    }
}

fn decode_char(byte: u8, url: bool) -> Result<u8, SoplangError> {
    let value = match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' if !url => 62,
        b'/' if !url => 63,
        b'-' if url => 62,
        b'_' if url => 63,
        _ => return Err(error("Xarafka Base64 ma saxna.")),
    };
    Ok(value)
}
