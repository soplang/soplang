//! Bounded `multipart/form-data` parsing for the sequential web framework.

use super::bytes;
use super::value::Value;

pub fn parse(content_type: &str, body: &[u8]) -> Result<Value, String> {
    let boundary =
        boundary(content_type).ok_or_else(|| "Xadka foomka multipart lama helin.".to_string())?;
    if boundary.is_empty() || boundary.len() > 200 {
        return Err("Xadka foomka multipart ma saxna.".into());
    }
    let marker = format!("--{boundary}");
    let text = body;
    let mut parts = Vec::new();
    let mut rest = text;
    let first = marker.as_bytes();
    let Some(start) = find(rest, first) else {
        return Err("Foomka multipart ma bilaabmo xadka.".into());
    };
    rest = &rest[start + first.len()..];
    if rest.starts_with(b"--") {
        return Ok(list(parts));
    }
    if rest.starts_with(b"\r\n") {
        rest = &rest[2..];
    }
    while !rest.is_empty() {
        let Some(header_end) = find(rest, b"\r\n\r\n") else {
            return Err("Madaxa qaybta foomka ma dhammaystirna.".into());
        };
        let headers = std::str::from_utf8(&rest[..header_end])
            .map_err(|_| "Madaxa qaybta foomka ma aha UTF-8.".to_string())?;
        rest = &rest[header_end + 4..];
        let next = format!("\r\n{marker}");
        let Some(body_end) = find(rest, next.as_bytes()) else {
            return Err("Jidhka qaybta foomka ma dhammaystirna.".into());
        };
        let payload = rest[..body_end].to_vec();
        rest = &rest[body_end + next.len()..];
        let (name, file) = disposition(headers)?;
        let mut fields = vec![
            ("magac".to_string(), Value::Str(name)),
            (
                "fayl".to_string(),
                file.map(Value::Str).unwrap_or(Value::Null),
            ),
            ("baytyo".to_string(), bytes::from_raw_vec(payload.clone())),
        ];
        fields.push((
            "qoraal".to_string(),
            std::str::from_utf8(&payload)
                .map(|text| Value::Str(text.to_string()))
                .unwrap_or(Value::Null),
        ));
        parts.push(object(fields));
        if rest.starts_with(b"--") {
            break;
        }
        if rest.starts_with(b"\r\n") {
            rest = &rest[2..];
        }
    }
    Ok(list(parts))
}

fn boundary(content_type: &str) -> Option<String> {
    let lower = content_type.to_ascii_lowercase();
    if !lower.contains("multipart/form-data") {
        return None;
    }
    for piece in content_type.split(';') {
        let piece = piece.trim();
        let Some(value) = piece.strip_prefix("boundary=") else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        return Some(value.to_string());
    }
    None
}

fn disposition(headers: &str) -> Result<(String, Option<String>), String> {
    let mut name = None;
    let mut file = None;
    for line in headers.lines() {
        let Some(value) = line
            .split_once(':')
            .filter(|(name, _)| name.eq_ignore_ascii_case("content-disposition"))
            .map(|(_, value)| value)
        else {
            continue;
        };
        for piece in value.split(';') {
            let piece = piece.trim();
            if let Some(found) = piece.strip_prefix("name=") {
                name = Some(unquote(found));
            } else if let Some(found) = piece.strip_prefix("filename=") {
                file = Some(unquote(found));
            }
        }
    }
    name.ok_or_else(|| "Qaybta foomka ma leh magac.".to_string())
        .map(|name| (name, file))
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn object(fields: Vec<(String, Value)>) -> Value {
    Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
        fields.into_iter().collect(),
    )))
}

fn list(items: Vec<Value>) -> Value {
    Value::List(std::rc::Rc::new(std::cell::RefCell::new(items)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_text_field_and_a_file() {
        let body = b"\
--Aa\r\n\
Content-Disposition: form-data; name=\"magac\"\r\n\
\r\n\
Amina\r\n\
--Aa\r\n\
Content-Disposition: form-data; name=\"fayl\"; filename=\"a.txt\"\r\n\
Content-Type: text/plain\r\n\
\r\n\
salaan\r\n\
--Aa--\r\n";
        let value = parse("multipart/form-data; boundary=Aa", body).unwrap();
        let Value::List(items) = value else {
            panic!("list")
        };
        let items = items.borrow();
        assert_eq!(items.len(), 2);
        let Value::Object(first) = &items[0] else {
            panic!("object")
        };
        assert_eq!(
            first.borrow().get("magac"),
            Some(&Value::Str("magac".into()))
        );
        assert_eq!(
            first.borrow().get("qoraal"),
            Some(&Value::Str("Amina".into()))
        );
    }
}
