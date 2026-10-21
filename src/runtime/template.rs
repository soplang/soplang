//! Small HTML templates: `{{name}}` is escaped, `{{!name}}` is raw.

use std::collections::HashMap;

use super::value::Value;

pub fn render(template: &str, data: &Value) -> Result<String, String> {
    let fields = fields(data)?;
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        let Some(end) = rest.find("}}") else {
            return Err("Qaabka {{ ma dhammaystirna.".into());
        };
        let token = rest[..end].trim();
        rest = &rest[end + 2..];
        let (raw, name) = token
            .strip_prefix('!')
            .map(|name| (true, name.trim()))
            .unwrap_or((false, token));
        if name.is_empty() || name.contains(['{', '}', ' ']) {
            return Err("Magaca qaabka ma saxna.".into());
        }
        let value = fields.get(name).map(text).unwrap_or_default();
        if raw {
            out.push_str(&value);
        } else {
            out.push_str(&escape(&value));
        }
    }
    out.push_str(rest);
    Ok(out)
}

fn fields(data: &Value) -> Result<HashMap<String, Value>, String> {
    let Value::Object(object) = data else {
        return Err("Xogta qaabka waa inay noqotaa walax.".into());
    };
    let object = object
        .try_borrow()
        .map_err(|_| "Walaxda qaabka lama akhriyi karo.".to_string())?;
    Ok(object.clone())
}

fn text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Str(text) => text.clone(),
        Value::Int(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::Bool(true) => "run".into(),
        Value::Bool(false) => "been".into(),
        other => other.to_string(),
    }
}

fn escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html_and_allows_a_raw_slot() {
        let data = object(&[
            ("magac", Value::Str("<Amina>".into())),
            ("html", Value::Str("<b>ok</b>".into())),
        ]);
        let text = render("<p>{{ magac }}</p>{{!html}}", &data).unwrap();
        assert_eq!(text, "<p>&lt;Amina&gt;</p><b>ok</b>");
    }

    fn object(fields: &[(&str, Value)]) -> Value {
        Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
            fields
                .iter()
                .map(|(key, value)| ((*key).to_string(), value.clone()))
                .collect(),
        )))
    }
}
