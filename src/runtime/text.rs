//! Unicode text operations shared by both compiled execution paths.
use std::cell::RefCell;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

use super::{unicode_case, TupleData, Value};
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};

const MAX_WIDTH: i64 = 1_000_000;
fn error(message: impl Into<String>) -> SoplangError {
    type_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E042_TEXT),
    )
}
fn cased(ch: char) -> bool {
    ch.is_lowercase() || ch.is_uppercase() || unicode_case::is_titlecase(ch)
}
fn capitalize(text: &str, first_cased: bool) -> String {
    let first = text
        .char_indices()
        .find(|(_, ch)| !first_cased || cased(*ch));
    let Some((index, ch)) = first else {
        return text.into();
    };
    // Lowercase the complete original segment so contextual final sigma keeps
    // its preceding cased character. The uncased prefix is unchanged by casing.
    let lowered = text.to_lowercase();
    let first_len: usize = ch.to_lowercase().map(char::len_utf8).sum();
    format!(
        "{}{}{}",
        &text[..index],
        unicode_case::title(ch),
        &lowered[index + first_len..]
    )
}
fn title(text: &str) -> String {
    text.split_word_bounds()
        .map(|word| capitalize(word, true))
        .collect()
}
pub(super) fn uppercase(text: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, "weynee", 0, 1)?;
    let result = match args.first() {
        None => text.to_uppercase(),
        Some(Value::Str(mode)) if mode == "hore" => capitalize(text, false),
        Some(Value::Str(mode)) if mode == "eray" => title(text),
        _ => return Err(error("weynee() habku waa inuu noqdaa 'hore' ama 'eray'.")),
    };
    Ok(Value::Str(result))
}
fn classify(text: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity(args, "hubi", 1)?;
    let Some(Value::Str(kind)) = args.first() else {
        return Err(error("hubi() nooca baaristu waa inuu noqdaa qoraal."));
    };
    let result = match kind.as_str() {
        "xarfo" => !text.is_empty() && text.chars().all(char::is_alphabetic),
        "tiro" => !text.is_empty() && text.chars().all(char::is_numeric),
        "xarfo iyo tiro" => !text.is_empty() && text.chars().all(char::is_alphanumeric),
        "bannaan" => !text.is_empty() && text.chars().all(char::is_whitespace),
        "waaweyn" => {
            text.chars().any(cased) && text.chars().filter(|ch| cased(*ch)).all(char::is_uppercase)
        }
        "yaryar" => {
            text.chars().any(cased) && text.chars().filter(|ch| cased(*ch)).all(char::is_lowercase)
        }
        "cinwaan" => text.chars().any(cased) && title(text) == text,
        "ascii" => text.is_ascii(),
        _ => {
            return Err(error(format!(
                "hubi() ma aqoonsana nooca baarista '{}'.",
                kind
            )))
        }
    };
    Ok(Value::Bool(result))
}
fn pad(text: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, "buuxi", 1, 3)?;
    let Value::Int(width) = args[0] else {
        return Err(error("buuxi() ballacu waa inuu noqdaa abn."));
    };
    if !(0..=MAX_WIDTH).contains(&width) {
        return Err(error("buuxi() ballacu waa inuu u dhexeeyaa 0 iyo 1000000."));
    }
    let fill = match args.get(1) {
        None => ' ',
        Some(Value::Str(fill)) if fill.chars().count() == 1 => fill.chars().next().unwrap(),
        _ => return Err(error("buuxi() buuxiyuhu waa inuu noqdaa hal xaraf.")),
    };
    let direction = match args.get(2) {
        None => "dambe",
        Some(Value::Str(direction)) if matches!(direction.as_str(), "hore" | "dambe" | "dhexe") => {
            direction
        }
        _ => {
            return Err(error(
                "buuxi() jihadu waa inay noqotaa 'hore', 'dambe', ama 'dhexe'.",
            ))
        }
    };
    let count = (width as usize).saturating_sub(text.chars().count());
    let left = match direction {
        "hore" => count,
        "dhexe" => count / 2,
        _ => 0,
    };
    let bytes = count
        .checked_mul(fill.len_utf8())
        .and_then(|bytes| bytes.checked_add(text.len()))
        .ok_or_else(|| error("Qoraalka la buuxinayo aad buu u weyn yahay."))?;
    let mut output = String::new();
    output
        .try_reserve(bytes)
        .map_err(|_| error("Qoraalka la buuxinayo aad buu u weyn yahay."))?;
    output.extend(std::iter::repeat_n(fill, left));
    output.push_str(text);
    output.extend(std::iter::repeat_n(fill, count - left));
    Ok(Value::Str(output))
}
fn partition(text: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, "kalaqaad", 1, 2)?;
    let Value::Str(separator) = &args[0] else {
        return Err(error("kalaqaad() kalaqaybiyuhu waa inuu noqdaa qoraal."));
    };
    if separator.is_empty() {
        return Err(error("kalaqaad() kalaqaybiyuhu madhan ma noqon karo."));
    }
    let reverse = match args.get(1) {
        None => false,
        Some(Value::Str(direction)) if direction == "hore" => false,
        Some(Value::Str(direction)) if direction == "dambe" => true,
        _ => {
            return Err(error(
                "kalaqaad() jihadu waa inay noqotaa 'hore' ama 'dambe'.",
            ))
        }
    };
    let found = if reverse {
        text.rfind(separator)
    } else {
        text.find(separator)
    };
    let parts = match found {
        Some(index) => [&text[..index], separator, &text[index + separator.len()..]],
        None if reverse => ["", "", text],
        None => [text, "", ""],
    };
    Ok(Value::Tuple(TupleData::new(
        parts
            .into_iter()
            .map(|part| Value::Str(part.into()))
            .collect(),
    )))
}
fn lines(text: &str, args: &[Value]) -> Result<Value, SoplangError> {
    super::stdlib::expect_arity_range(args, "sadarro", 0, 1)?;
    let keep = match args.first() {
        None => false,
        Some(Value::Bool(keep)) => *keep,
        _ => {
            return Err(error(
                "sadarro() ilaalinta dhammaadka safku waa inay noqotaa bool.",
            ))
        }
    };
    let mut start = 0;
    let mut result = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if matches!(
            ch,
            '\n' | '\r' | '\u{B}' | '\u{C}' | '\u{1C}'
                ..='\u{1E}' | '\u{85}' | '\u{2028}' | '\u{2029}'
        ) {
            let mut end = index + ch.len_utf8();
            if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                chars.next();
                end += 1;
            }
            result.push(Value::Str(
                text[start..if keep { end } else { index }].into(),
            ));
            start = end;
        }
    }
    if start < text.len() {
        result.push(Value::Str(text[start..].into()));
    }
    Ok(Value::List(Rc::new(RefCell::new(result))))
}
pub(super) fn dispatch(text: &str, method: &str, args: &[Value]) -> Result<Value, SoplangError> {
    match method {
        "hubi" => classify(text, args),
        "buuxi" => pad(text, args),
        "kalaqaad" => partition(text, args),
        "sadarro" => lines(text, args),
        _ => unreachable!("text method routing"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_casing_versions_and_expanding_first_scalars_are_consistent() {
        assert_eq!(
            char::UNICODE_VERSION,
            (17, 0, 0),
            "regenerate title mappings when the Rust Unicode data changes"
        );
        assert_eq!(unicode_segmentation::UNICODE_VERSION, (17, 0, 0));
        for (input, expected) in [
            ("ßABC", "Ssabc"),
            ("ǳABC", "ǲabc"),
            ("ᾀBC", "ᾈbc"),
            ("ﬃABC", "Ffiabc"),
            ("ΟΣ", "Ος"),
            ("İABC", "İabc"),
        ] {
            assert_eq!(capitalize(input, false), expected);
        }
        // Word boundaries are defined on the original text. Turning a leading
        // combining ypogegrammeni into a letter can join words on a later pass.
        assert_eq!(title("\u{345}ΣA"), "ΙΣa");
        for code in 0..=0x10FFFF {
            let Some(ch) = char::from_u32(code).filter(|ch| cased(*ch)) else {
                continue;
            };
            for prefix in ["", " ", "123", "\u{301}"] {
                let input = format!("{prefix}{ch}ΣA");
                let output = title(&input);
                assert!(output.starts_with(prefix), "{code:X}: {input:?}");
                assert!(!output.is_empty());
                // Exercise lowercase-expansion byte boundaries even when the
                // first scalar has no case mapping or occupies several bytes.
                capitalize(&input, false);
            }
        }
    }

    #[test]
    fn padding_bounds_and_fill_validation_do_not_depend_on_needing_padding() {
        let output = pad("", &[Value::Int(MAX_WIDTH), Value::Str("🙂".into())]).unwrap();
        let Value::Str(output) = output else {
            panic!("text")
        };
        assert_eq!(output.chars().count(), MAX_WIDTH as usize);
        assert_eq!(output.len(), MAX_WIDTH as usize * 4);
        for args in [
            vec![Value::Int(i64::MAX)],
            vec![Value::Int(i64::MIN)],
            vec![Value::Float(1.0)],
            vec![Value::Int(0), Value::Str("".into())],
            vec![Value::Int(0), Value::Str("e\u{301}".into())],
            vec![Value::Int(0), Value::Str(" ".into()), Value::Bool(true)],
        ] {
            assert_eq!(pad("already long", &args).unwrap_err().code(), Some("E042"));
        }
    }

    #[test]
    fn unicode_line_splitting_preserves_original_bytes_when_requested() {
        let input = "α\r\nβ\n\rγ\u{B}δ\u{C}ε\u{1C}ζ\u{1D}η\u{1E}θ\u{85}ι\u{2028}κ\u{2029}λ";
        let Value::List(kept) = lines(input, &[Value::Bool(true)]).unwrap() else {
            panic!("list")
        };
        let restored: String = kept
            .borrow()
            .iter()
            .map(|value| match value {
                Value::Str(text) => text.as_str(),
                _ => panic!("text"),
            })
            .collect();
        assert_eq!(restored, input);
        let Value::List(stripped) = lines(input, &[]).unwrap() else {
            panic!("list")
        };
        assert_eq!(stripped.borrow().len(), 12);
        assert!(stripped.borrow().iter().all(|value| matches!(value, Value::Str(text) if !text.contains(['\r','\n','\u{2028}','\u{2029}']))));
    }
}
