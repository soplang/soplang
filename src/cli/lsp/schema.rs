//! The database schema has its own syntax; do not parse it as executable code.
use serde_json::{json, Value};

const WORDS: &[(&str, &str)] = &[
    ("jadwal", "Declare a database table."),
    ("abn", "Signed 64-bit integer."),
    ("jajab", "Floating-point number."),
    ("qoraal", "Unicode text."),
    ("bool", "Boolean: run or been."),
    ("furaha", "The table's single non-nullable abn primary key."),
];
pub(super) fn completion(text: &str, offset: usize) -> Value {
    let start = text[..offset].rfind('\n').map(|n| n + 1).unwrap_or(0);
    let line = &text[start..offset];
    if line.contains('#') {
        return json!([]);
    }
    let words: Vec<_> = line.split_whitespace().collect();
    let inside = text[..start].lines().fold(false, |inside, line| {
        let code = line.split('#').next().unwrap_or("").trim();
        if code.ends_with('{') {
            true
        } else if code == "}" {
            false
        } else {
            inside
        }
    });
    if !inside {
        return if words.len() <= 1 {
            json!([{"label":"jadwal","kind":14,"detail":WORDS[0].1}])
        } else {
            json!([])
        };
    }
    let trailing_space = line.ends_with(char::is_whitespace);
    if words.len() == 1 && trailing_space || words.len() == 2 && !trailing_space {
        let choices: Vec<_> = WORDS[1..5].iter().flat_map(|(word, help)| [
            json!({"label":word,"kind":14,"detail":help}),
            json!({"label":format!("{word}?"),"kind":14,"detail":format!("{help} Optional; may be maran.")}),
        ]).collect();
        return json!(choices);
    }
    if words.get(1) == Some(&"abn") && (words.len() == 2 && trailing_space || words.len() == 3) {
        return json!([{"label":"furaha","kind":14,"detail":WORDS[5].1}]);
    }
    json!([])
}
pub(super) fn hover(text: &str, offset: usize) -> Value {
    let line_start = text[..offset].rfind('\n').map(|n| n + 1).unwrap_or(0);
    if text[line_start..offset].contains('#') {
        return Value::Null;
    }
    let bytes = text.as_bytes();
    let mut start = offset.min(bytes.len());
    let mut end = start;
    let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'?';
    while start > 0 && word(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && word(bytes[end]) {
        end += 1;
    }
    let name = &text[start..end];
    let base = name.strip_suffix('?').unwrap_or(name);
    WORDS.iter().find(|(word,_)|*word==base).map(|(_,help)|json!({"contents":{"kind":"plaintext","value":format!("{name}: {help}{}",if name.ends_with('?'){" Optional; may be maran."}else{""})}})).unwrap_or(Value::Null)
}
