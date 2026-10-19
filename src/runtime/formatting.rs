//! Value formatting shared by qaabee(), templates, and interpolation.

use super::value::{value_to_string, Value};
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};

const MAX_SIZE: usize = 1_000_000;

fn error(message: impl Into<String>) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E036_INVALID_FORMAT),
    )
}

#[derive(Default)]
struct Spec {
    fill: Option<char>,
    align: Option<char>,
    sign: Option<char>,
    alternate: bool,
    zero: bool,
    width: usize,
    precision: Option<usize>,
    kind: Option<char>,
}

impl Spec {
    fn parse(text: &str) -> Result<Self, SoplangError> {
        let chars: Vec<char> = text.chars().collect();
        let mut spec = Self::default();
        let mut index = 0;
        if chars.get(1).is_some_and(|ch| matches!(ch, '<' | '>' | '^')) {
            spec.fill = chars.first().copied();
            spec.align = chars.get(1).copied();
            index = 2;
        } else if chars
            .first()
            .is_some_and(|ch| matches!(ch, '<' | '>' | '^'))
        {
            spec.align = chars.first().copied();
            index = 1;
        }
        if chars
            .get(index)
            .is_some_and(|ch| matches!(ch, '+' | '-' | ' '))
        {
            spec.sign = chars.get(index).copied();
            index += 1;
        }
        if chars.get(index) == Some(&'#') {
            spec.alternate = true;
            index += 1;
        }
        if chars.get(index) == Some(&'0') {
            spec.zero = true;
            index += 1;
        }
        let start = index;
        while chars.get(index).is_some_and(char::is_ascii_digit) {
            index += 1;
        }
        if index > start {
            spec.width = size(&chars[start..index])?;
        }
        if chars.get(index) == Some(&'.') {
            index += 1;
            let start = index;
            while chars.get(index).is_some_and(char::is_ascii_digit) {
                index += 1;
            }
            if index == start {
                return Err(invalid(text));
            }
            spec.precision = Some(size(&chars[start..index])?);
        }
        if index < chars.len() {
            spec.kind = Some(chars[index]);
            index += 1;
        }
        if index != chars.len()
            || !matches!(
                spec.kind,
                None | Some('s' | 'd' | 'x' | 'X' | 'b' | 'o' | 'f' | 'e' | 'E' | '%')
            )
        {
            return Err(invalid(text));
        }
        Ok(spec)
    }
}

fn invalid(text: &str) -> SoplangError {
    error(format!("Qaabka qoraalka '{}' lama aqoonsan.", text))
}

fn size(digits: &[char]) -> Result<usize, SoplangError> {
    let value = digits
        .iter()
        .collect::<String>()
        .parse::<usize>()
        .ok()
        .filter(|value| *value <= MAX_SIZE);
    value.ok_or_else(|| error("Qaabka qoraalku wuxuu ka badan yahay xadka la oggol yahay."))
}

pub(super) fn value(value: &Value, text: &str) -> Result<String, SoplangError> {
    if text.is_empty() {
        return Ok(value_to_string(value));
    }
    let spec = Spec::parse(text)?;
    let numeric = matches!(value, Value::Int(_) | Value::Float(_));
    let number_kind = matches!(
        spec.kind,
        Some('d' | 'x' | 'X' | 'b' | 'o' | 'f' | 'e' | 'E' | '%')
    );
    if !numeric && (spec.sign.is_some() || spec.alternate || spec.zero || number_kind) {
        return Err(error(format!(
            "Qaabka '{}' wuxuu u baahan yahay tiro.",
            text
        )));
    }
    let mut rendered = match spec.kind {
        Some('d' | 'x' | 'X' | 'b' | 'o') => {
            let Value::Int(number) = value else {
                return Err(error(format!(
                    "Qaabka '{}' wuxuu u baahan yahay abn.",
                    text
                )));
            };
            if spec.precision.is_some() {
                return Err(error("Saxnaanta laguma sheegi karo qaabka abn."));
            }
            let magnitude = number.unsigned_abs();
            let (digits, prefix) = match spec.kind {
                Some('x') => (format!("{magnitude:x}"), "0x"),
                Some('X') => (format!("{magnitude:X}"), "0X"),
                Some('b') => (format!("{magnitude:b}"), "0b"),
                Some('o') => (format!("{magnitude:o}"), "0o"),
                _ => (magnitude.to_string(), ""),
            };
            let sign = if *number < 0 {
                "-"
            } else {
                positive_sign(spec.sign)
            };
            format!("{sign}{}{digits}", if spec.alternate { prefix } else { "" })
        }
        Some('f' | 'e' | 'E' | '%') => {
            let number = match value {
                Value::Int(number) => *number as f64,
                Value::Float(number) => *number,
                _ => unreachable!(),
            };
            let precision = spec.precision.unwrap_or(6);
            let number = if spec.kind == Some('%') {
                number * 100.0
            } else {
                number
            };
            let mut output = match spec.kind {
                Some('e') => format!("{number:.precision$e}"),
                Some('E') => format!("{number:.precision$E}"),
                _ => format!("{number:.precision$}"),
            };
            if !number.is_sign_negative() {
                output.insert_str(0, positive_sign(spec.sign));
            }
            if spec.kind == Some('%') {
                output.push('%');
            }
            output
        }
        Some('s') => {
            let Value::Str(string) = value else {
                return Err(error("Qaabka 's' wuxuu u baahan yahay qoraal."));
            };
            match spec.precision {
                Some(count) => string.chars().take(count).collect(),
                None => string.clone(),
            }
        }
        None if matches!(value, Value::Str(_)) && spec.precision.is_some() => {
            let Value::Str(string) = value else {
                unreachable!()
            };
            string.chars().take(spec.precision.unwrap()).collect()
        }
        None if matches!(value, Value::Float(_)) && spec.precision.is_some() => {
            let Value::Float(number) = value else {
                unreachable!()
            };
            let precision = spec.precision.unwrap();
            let output = format!("{number:.precision$}");
            if number.is_sign_negative() {
                output
            } else {
                format!("{}{output}", positive_sign(spec.sign))
            }
        }
        None => {
            if spec.precision.is_some() {
                return Err(error(
                    "Saxnaantu waxay u baahan tahay qaab qoraal ama jajab.",
                ));
            }
            let output = value_to_string(value);
            if numeric && !output.starts_with('-') {
                format!("{}{output}", positive_sign(spec.sign))
            } else {
                output
            }
        }
        _ => unreachable!(),
    };
    if spec.alternate && !matches!(spec.kind, Some('x' | 'X' | 'b' | 'o')) {
        return Err(invalid(text));
    }
    let padding = spec.width.saturating_sub(rendered.chars().count());
    if padding == 0 {
        return Ok(rendered);
    }
    if numeric && spec.zero && spec.align.is_none() {
        let sign = usize::from(rendered.starts_with(['-', '+', ' ']));
        let prefix = if ["0x", "0X", "0b", "0o"]
            .iter()
            .any(|prefix| rendered[sign..].starts_with(prefix))
        {
            2
        } else {
            0
        };
        rendered.insert_str(sign + prefix, &"0".repeat(padding));
        return Ok(rendered);
    }
    let (left, right) = match spec.align.unwrap_or(if numeric { '>' } else { '<' }) {
        '<' => (0, padding),
        '^' => (padding / 2, padding - padding / 2),
        _ => (padding, 0),
    };
    let fill = spec.fill.unwrap_or(if spec.zero { '0' } else { ' ' });
    Ok(format!(
        "{}{}{}",
        fill.to_string().repeat(left),
        rendered,
        fill.to_string().repeat(right)
    ))
}

fn positive_sign(sign: Option<char>) -> &'static str {
    match sign {
        Some('+') => "+",
        Some(' ') => " ",
        _ => "",
    }
}

pub(super) fn template(text: &str, args: &[Value]) -> Result<String, SoplangError> {
    let mut chars = text.chars().peekable();
    let mut output = String::new();
    let mut automatic = 0;
    let mut automatic_used = false;
    let mut manual_used = false;
    while let Some(character) = chars.next() {
        if character == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                output.push('{');
                continue;
            }
            let mut field = String::new();
            let mut closed = false;
            for character in chars.by_ref() {
                if character == '}' {
                    closed = true;
                    break;
                }
                if character == '{' {
                    return Err(error(
                        "Qaabka qoraalku ma aqbalo '{' gudaha meel qiime lagu gelinayo.",
                    ));
                }
                field.push(character);
            }
            if !closed {
                return Err(error("Qaybta qiimaha lagu gelinayo qoraalka lama xidhin."));
            }
            let (name, spec) = field.split_once(':').unwrap_or((&field, ""));
            let name = name.trim();
            let missing = || error(format!("Qiime looma helin meesha '{}' ee qoraalka.", name));
            let item = if name.is_empty() {
                automatic_used = true;
                let item = args.get(automatic).cloned().ok_or_else(missing)?;
                automatic += 1;
                item
            } else if name.chars().all(|character| character.is_ascii_digit()) {
                manual_used = true;
                name.parse::<usize>()
                    .ok()
                    .and_then(|index| args.get(index))
                    .cloned()
                    .ok_or_else(missing)?
            } else {
                match args.first() {
                    Some(Value::Object(values)) => {
                        values.borrow().get(name).cloned().ok_or_else(missing)?
                    }
                    _ => return Err(missing()),
                }
            };
            if automatic_used && manual_used {
                return Err(error(
                    "Qaabka qoraalku ma isku dari karo meelgelin toos ah iyo mid lambar leh.",
                ));
            }
            output.push_str(&value(&item, spec)?);
        } else if character == '}' {
            if chars.next() != Some('}') {
                return Err(error("Calaamadda '}' ee qoraalka ma laha '{' u dhiganta."));
            }
            output.push('}');
        } else {
            output.push(character);
        }
    }
    Ok(output)
}
