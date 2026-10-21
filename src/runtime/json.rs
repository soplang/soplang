//! JSON parse and generation for Soplang values.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const MAX_DEPTH: usize = 64;

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E045_JSON),
    )
}

pub fn parse(text: &str) -> Result<Value, SoplangError> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        index: 0,
        depth: 0,
    };
    parser.skip();
    let value = parser.value()?;
    parser.skip();
    if parser.index != parser.bytes.len() {
        return Err(error("JSON-ku wuxuu leeyahay xog dheeraad ah dhamaadka."));
    }
    Ok(value)
}

pub fn stringify(value: &Value) -> Result<String, SoplangError> {
    let mut out = String::new();
    write_value(value, &mut out, &mut HashSet::new(), 0)?;
    Ok(out)
}

struct Parser<'a> {
    bytes: &'a [u8],
    index: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn skip(&mut self) {
        while let Some(byte) = self.bytes.get(self.index) {
            if byte.is_ascii_whitespace() {
                self.index += 1;
            } else {
                break;
            }
        }
    }

    fn value(&mut self) -> Result<Value, SoplangError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(error("JSON-ku aad buu u qoto dheer yahay."));
        }
        self.skip();
        let value = match self.bytes.get(self.index).copied() {
            Some(b'n') => self.literal(b"null", Value::Null),
            Some(b't') => self.literal(b"true", Value::Bool(true)),
            Some(b'f') => self.literal(b"false", Value::Bool(false)),
            Some(b'"') => self.string().map(Value::Str),
            Some(b'-') | Some(b'0'..=b'9') => self.number(),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            _ => Err(error("JSON-ku ma saxna.")),
        };
        self.depth -= 1;
        value
    }

    fn literal(&mut self, expected: &[u8], value: Value) -> Result<Value, SoplangError> {
        if self.bytes[self.index..].starts_with(expected) {
            self.index += expected.len();
            Ok(value)
        } else {
            Err(error("JSON-ku ma saxna."))
        }
    }

    fn string(&mut self) -> Result<String, SoplangError> {
        self.index += 1;
        let mut out = String::new();
        while self.index < self.bytes.len() {
            let byte = self.bytes[self.index];
            self.index += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => out.push(self.escape()?),
                0x00..=0x1F => return Err(error("Qoraalka JSON wuxuu leeyahay xaraf aan la oggolayn.")),
                _ => {
                    let start = self.index - 1;
                    let width = utf8_width(byte);
                    if self.index + width - 1 > self.bytes.len() {
                        return Err(error("JSON-ku ma saxna."));
                    }
                    let text = std::str::from_utf8(&self.bytes[start..self.index + width - 1])
                        .map_err(|_| error("JSON-ku ma aha qoraal UTF-8 ah oo sax ah."))?;
                    out.push_str(text);
                    self.index += width - 1;
                }
            }
        }
        Err(error("Qoraalka JSON lama xidhin."))
    }

    fn escape(&mut self) -> Result<char, SoplangError> {
        let byte = self.bytes.get(self.index).copied().ok_or_else(|| error("JSON-ku ma saxna."))?;
        self.index += 1;
        Ok(match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{0008}',
            b'f' => '\u{000c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => self.unicode_escape()?,
            _ => return Err(error("Ka baxista JSON ma saxna.")),
        })
    }

    fn unicode_escape(&mut self) -> Result<char, SoplangError> {
        let unit = self.hex_unit()?;
        if (0xD800..=0xDBFF).contains(&unit) {
            if !self.bytes[self.index..].starts_with(br"\u") {
                return Err(error("Lammaanaha Unicode ee JSON ma dhammaystirna."));
            }
            self.index += 2;
            let low = self.hex_unit()?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(error("Lammaanaha Unicode ee JSON ma saxna."));
            }
            let code = 0x10000 + (((unit as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
            return char::from_u32(code).ok_or_else(|| error("Xarafka Unicode ee JSON ma saxna."));
        }
        char::from_u32(unit as u32).ok_or_else(|| error("Xarafka Unicode ee JSON ma saxna."))
    }

    fn hex_unit(&mut self) -> Result<u16, SoplangError> {
        if self.index + 4 > self.bytes.len() {
            return Err(error("Xarafka Unicode ee JSON ma dhammaystirna."));
        }
        let text = std::str::from_utf8(&self.bytes[self.index..self.index + 4])
            .map_err(|_| error("Xarafka Unicode ee JSON ma saxna."))?;
        self.index += 4;
        u16::from_str_radix(text, 16).map_err(|_| error("Xarafka Unicode ee JSON ma saxna."))
    }

    fn number(&mut self) -> Result<Value, SoplangError> {
        let start = self.index;
        if self.bytes[self.index] == b'-' {
            self.index += 1;
        }
        match self.bytes.get(self.index).copied() {
            Some(b'0') => self.index += 1,
            Some(b'1'..=b'9') => {
                self.index += 1;
                while matches!(self.bytes.get(self.index), Some(b'0'..=b'9')) {
                    self.index += 1;
                }
            }
            _ => return Err(error("Lambarka JSON ma saxna.")),
        }
        let mut floating = false;
        if self.bytes.get(self.index) == Some(&b'.') {
            floating = true;
            self.index += 1;
            if !matches!(self.bytes.get(self.index), Some(b'0'..=b'9')) {
                return Err(error("Lambarka JSON ma saxna."));
            }
            while matches!(self.bytes.get(self.index), Some(b'0'..=b'9')) {
                self.index += 1;
            }
        }
        if matches!(self.bytes.get(self.index), Some(b'e' | b'E')) {
            floating = true;
            self.index += 1;
            if matches!(self.bytes.get(self.index), Some(b'+' | b'-')) {
                self.index += 1;
            }
            if !matches!(self.bytes.get(self.index), Some(b'0'..=b'9')) {
                return Err(error("Lambarka JSON ma saxna."));
            }
            while matches!(self.bytes.get(self.index), Some(b'0'..=b'9')) {
                self.index += 1;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.index])
            .map_err(|_| error("Lambarka JSON ma saxna."))?;
        if floating {
            let value = text.parse::<f64>().map_err(|_| error("Lambarka JSON ma saxna."))?;
            if !value.is_finite() {
                return Err(error("Lambarka JSON ma saxna."));
            }
            Ok(Value::Float(value))
        } else {
            text.parse::<i64>()
                .map(Value::Int)
                .map_err(|_| error("Lambarka JSON kuma habboona abn."))
        }
    }

    fn array(&mut self) -> Result<Value, SoplangError> {
        self.index += 1;
        let mut items = Vec::new();
        self.skip();
        if self.bytes.get(self.index) == Some(&b']') {
            self.index += 1;
            return Ok(Value::List(Rc::new(RefCell::new(items))));
        }
        loop {
            items.push(self.value()?);
            self.skip();
            match self.bytes.get(self.index).copied() {
                Some(b',') => {
                    self.index += 1;
                    self.skip();
                }
                Some(b']') => {
                    self.index += 1;
                    break;
                }
                _ => return Err(error("Teedka JSON ma saxna.")),
            }
        }
        Ok(Value::List(Rc::new(RefCell::new(items))))
    }

    fn object(&mut self) -> Result<Value, SoplangError> {
        self.index += 1;
        let mut map = HashMap::new();
        self.skip();
        if self.bytes.get(self.index) == Some(&b'}') {
            self.index += 1;
            return Ok(Value::Object(Rc::new(RefCell::new(map))));
        }
        loop {
            self.skip();
            if self.bytes.get(self.index) != Some(&b'"') {
                return Err(error("Furaha JSON waa inuu noqdaa qoraal."));
            }
            let key = self.string()?;
            self.skip();
            if self.bytes.get(self.index) != Some(&b':') {
                return Err(error("Walaxda JSON ma saxna."));
            }
            self.index += 1;
            map.insert(key, self.value()?);
            self.skip();
            match self.bytes.get(self.index).copied() {
                Some(b',') => self.index += 1,
                Some(b'}') => {
                    self.index += 1;
                    break;
                }
                _ => return Err(error("Walaxda JSON ma saxna.")),
            }
        }
        Ok(Value::Object(Rc::new(RefCell::new(map))))
    }
}

fn utf8_width(byte: u8) -> usize {
    if byte < 0x80 {
        1
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}

fn write_value(
    value: &Value,
    out: &mut String,
    seen: &mut HashSet<(u8, usize)>,
    depth: usize,
) -> Result<(), SoplangError> {
    if depth > MAX_DEPTH {
        return Err(error("Qiimaha JSON aad buu u qoto dheer yahay."));
    }
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Int(value) => out.push_str(&value.to_string()),
        Value::Float(value) => {
            if !value.is_finite() {
                return Err(error("JSON ma qori karo jajab aan dhammaan."));
            }
            let text = format!("{value}");
            if text.contains('.') || text.contains('e') || text.contains('E') {
                out.push_str(&text);
            } else {
                out.push_str(&text);
                out.push_str(".0");
            }
        }
        Value::Str(text) => write_string(text, out),
        Value::List(items) => {
            let key = (1, Rc::as_ptr(items) as usize);
            if !seen.insert(key) {
                return Err(error("JSON ma qori karo teed wareeg ah."));
            }
            let borrowed = items
                .try_borrow()
                .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
            out.push('[');
            for (index, item) in borrowed.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(item, out, seen, depth + 1)?;
            }
            out.push(']');
            seen.remove(&key);
        }
        Value::Object(items) => {
            let key = (2, Rc::as_ptr(items) as usize);
            if !seen.insert(key) {
                return Err(error("JSON ma qori karo walax wareeg ah."));
            }
            let borrowed = items
                .try_borrow()
                .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
            let mut keys = borrowed.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            out.push('{');
            for (index, name) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(name, out);
                out.push(':');
                write_value(&borrowed[name], out, seen, depth + 1)?;
            }
            out.push('}');
            seen.remove(&key);
        }
        _ => return Err(error("Noocan lama qori karo JSON.")),
    }
    Ok(())
}

fn write_string(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}
