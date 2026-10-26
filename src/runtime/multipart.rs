//! Bounded `multipart/form-data` parsing for the sequential web framework.

use super::bytes;
use super::value::Value;

const MAX_BODY: usize = 16 * 1024 * 1024;
const MAX_HEADERS: usize = 8 * 1024;
const MAX_PARTS: usize = 128;

pub fn parse(content_type: &str, body: &[u8]) -> Result<Value, String> {
    if body.len() > MAX_BODY {
        return Err("Jidhka multipart aad buu u weyn yahay.".into());
    }
    let (media_type, media_parameters) = parameters(content_type)?;
    if !media_type.eq_ignore_ascii_case("multipart/form-data") {
        return Err("Nooca foomku waa inuu noqdaa multipart/form-data.".into());
    }
    let boundary = media_parameters
        .get("boundary")
        .ok_or("Xadka foomka multipart lama helin.")?;
    if boundary.is_empty()
        || boundary.len() > 70
        || boundary.ends_with(' ')
        || !boundary
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"'()+_,-./:=? ".contains(&byte))
    {
        return Err("Xadka foomka multipart ma saxna.".into());
    }
    let marker = format!("--{boundary}");
    let (offset, mut rest, closed) =
        delimiter(body, marker.as_bytes(), true).ok_or("Foomka multipart ma bilaabmo xadka.")?;
    if offset > MAX_HEADERS {
        return Err("Hordhaca multipart aad buu u dheer yahay.".into());
    }
    if closed {
        return Ok(list(Vec::new()));
    }
    let mut parts = Vec::new();
    loop {
        if parts.len() == MAX_PARTS {
            return Err("Qaybaha multipart way ka badan yihiin xadka 128.".into());
        }
        let header_end =
            find(rest, b"\r\n\r\n").ok_or("Madaxa qaybta foomka ma dhammaystirna.")? + 4;
        if header_end > MAX_HEADERS {
            return Err("Madaxa qaybta multipart aad buu u weyn yahay.".into());
        }
        let mut header_slots = [httparse::EMPTY_HEADER; 32];
        let parsed = httparse::parse_headers(&rest[..header_end], &mut header_slots)
            .map_err(|_| "Madaxa qaybta multipart ma saxna.")?;
        let httparse::Status::Complete((_, headers)) = parsed else {
            return Err("Madaxa qaybta foomka ma dhammaystirna.".into());
        };
        let mut disposition = None;
        let mut seen = std::collections::HashSet::new();
        for header in headers {
            let name = header.name.to_ascii_lowercase();
            if !seen.insert(name.clone()) {
                return Err("Cinwaan ayaa lagu celiyey qaybta multipart.".into());
            }
            if name == "content-transfer-encoding" {
                return Err("Habkan gudbinta multipart lama taageero.".into());
            }
            if name == "content-disposition" {
                disposition = Some(
                    std::str::from_utf8(header.value)
                        .map_err(|_| "Magaca qaybta multipart ma aha UTF-8.")?,
                );
            }
        }
        let (kind, params) =
            parameters(disposition.ok_or("Qaybta foomka ma leh Content-Disposition.")?)?;
        if !kind.eq_ignore_ascii_case("form-data") {
            return Err("Qaybtu waa inay noqotaa form-data.".into());
        }
        if params.contains_key("filename*") {
            return Err(
                "Magaca filename* weli lama taageero; adeegso filename oo UTF-8 ah.".into(),
            );
        }
        let name = params
            .get("name")
            .filter(|name| !name.is_empty())
            .ok_or("Qaybta foomka ma leh magac.")?
            .clone();
        let payload_start = &rest[header_end..];
        let (end, next, closed) = delimiter(payload_start, marker.as_bytes(), false)
            .ok_or("Xadka xiritaanka qaybta multipart lama helin.")?;
        let payload = payload_start[..end].to_vec();
        let text = std::str::from_utf8(&payload)
            .map(|text| Value::Str(text.into()))
            .unwrap_or(Value::Null);
        parts.push(object(vec![
            ("magac".into(), Value::Str(name)),
            (
                "fayl".into(),
                params
                    .get("filename")
                    .cloned()
                    .map(Value::Str)
                    .unwrap_or(Value::Null),
            ),
            ("baytyo".into(), bytes::from_raw_vec(payload)),
            ("qoraal".into(), text),
        ]));
        if closed {
            return Ok(list(parts));
        }
        rest = next;
    }
}

/// Find whole delimiter lines, never a boundary prefix inside file contents.
/// The returned payload end excludes the CRLF introducing the delimiter.
fn delimiter<'a>(body: &'a [u8], marker: &[u8], first: bool) -> Option<(usize, &'a [u8], bool)> {
    let mut start = 0;
    while let Some(found) = find(&body[start..], marker) {
        let index = start + found;
        let at_start = first && index == 0;
        if at_start || (index >= 2 && &body[index - 2..index] == b"\r\n") {
            let mut end = index + marker.len();
            let closed = body[end..].starts_with(b"--");
            if closed {
                end += 2;
            }
            while end < body.len() && matches!(body[end], b' ' | b'\t') {
                end += 1;
            }
            if (closed && end == body.len()) || body[end..].starts_with(b"\r\n") {
                let next = if end == body.len() { end } else { end + 2 };
                return Some((if at_start { 0 } else { index - 2 }, &body[next..], closed));
            }
        }
        start = index + marker.len();
    }
    None
}

/// Parse semicolon parameters with quoted strings and escaped quotes. Header
/// field names are case-insensitive; duplicate parameters are ambiguous errors.
fn parameters(text: &str) -> Result<(String, std::collections::HashMap<String, String>), String> {
    if text.chars().any(|ch| ch.is_control() && ch != '\t') {
        return Err("Cinwaanka multipart wuxuu leeyahay xaraf xakameeye.".into());
    }
    let mut pieces = text.splitn(2, ';');
    let kind = pieces.next().unwrap_or("").trim().to_owned();
    let mut rest = pieces.next().unwrap_or("").trim();
    let mut fields = std::collections::HashMap::new();
    while !rest.is_empty() {
        let (key, tail) = rest
            .split_once('=')
            .ok_or("Xulashada cinwaanka multipart ma saxna.")?;
        let key = key.trim();
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
        {
            return Err("Magaca xulashada multipart ma saxna.".into());
        }
        let tail = tail.trim_start();
        let (value, remaining) = if let Some(quoted) = tail.strip_prefix('"') {
            let mut value = String::new();
            let mut chars = quoted.char_indices();
            let mut end = None;
            while let Some((index, ch)) = chars.next() {
                match ch {
                    '\\' => value.push(
                        chars
                            .next()
                            .ok_or("Xigashada multipart ma dhammaystirna.")?
                            .1,
                    ),
                    '"' => {
                        end = Some(index + 1);
                        break;
                    }
                    _ => value.push(ch),
                }
            }
            let end = end.ok_or("Xigashada multipart ma dhammaystirna.")?;
            (value, quoted[end..].trim_start())
        } else {
            let end = tail.find(';').unwrap_or(tail.len());
            let value = tail[..end].trim();
            if value.is_empty()
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
            {
                return Err("Qiimaha xulashada multipart ma saxna.".into());
            }
            (value.to_owned(), &tail[end..])
        };
        if fields.insert(key.to_ascii_lowercase(), value).is_some() {
            return Err("Xulasho ayaa lagu celiyey cinwaanka multipart.".into());
        }
        rest = if remaining.is_empty() {
            ""
        } else {
            remaining
                .strip_prefix(';')
                .ok_or("Kalaqaybiyaha multipart ma saxna.")?
                .trim_start()
        };
    }
    Ok((kind, fields))
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

    #[test]
    fn boundary_prefixes_and_quoted_parameters_preserve_uploaded_bytes() {
        let payload = b"one\r\n--Aa-not-a-boundary\r\ntwo\0\xff\r\n--Aa--not-a-close";
        let mut body = b"--Aa\r\ncOnTeNt-DiSpOsItIoN: FORM-DATA; NAME=\"upload\"; FILENAME=\"a; b\\\".txt\"\r\n\r\n".to_vec();
        body.extend_from_slice(payload);
        body.extend_from_slice(b"\r\n--Aa--\r\nignored epilogue");
        let Value::List(parts) = parse("MULTIPART/FORM-DATA; BOUNDARY=\"Aa\"", &body).unwrap()
        else {
            panic!()
        };
        let parts = parts.borrow();
        let Value::Object(part) = &parts[0] else {
            panic!()
        };
        let part = part.borrow();
        assert_eq!(part["fayl"], Value::Str("a; b\".txt".into()));
        assert_eq!(part["qoraal"], Value::Null);
        let Value::Bytes(upload) = &part["baytyo"] else {
            panic!()
        };
        assert_eq!(upload.borrow().as_slice(), payload);
    }

    #[test]
    fn incomplete_ambiguous_and_excessive_multipart_is_rejected() {
        let valid = b"--Aa\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\nx\r\n--Aa--\r\n";
        for end in [valid.len() - 8, valid.len() - 5, valid.len() - 3] {
            assert!(parse("multipart/form-data; boundary=Aa", &valid[..end]).is_err());
        }
        for content_type in [
            "x-multipart/form-data; boundary=Aa",
            "multipart/form-data; boundary=Aa; boundary=Bb",
            "multipart/form-data; boundary=\"unterminated",
            "multipart/form-data; boundary=\"Aa\r\nBad\"",
        ] {
            assert!(parse(content_type, valid).is_err(), "{content_type:?}");
        }
        for disposition in [
            "attachment; name=a",
            "form-data; name=a; name=b",
            "form-data; name=\"a\"junk",
            "form-data; name=a; filename*=UTF-8''a.txt",
        ] {
            let body = format!("--Aa\r\nContent-Disposition: {disposition}\r\n\r\nx\r\n--Aa--");
            assert!(
                parse("multipart/form-data; boundary=Aa", body.as_bytes()).is_err(),
                "{disposition}"
            );
        }
        let repeated = "--Aa\r\nContent-Disposition: form-data; name=a\r\n\r\nx\r\n"
            .repeat(MAX_PARTS + 1)
            + "--Aa--\r\n";
        assert!(
            parse("multipart/form-data; boundary=Aa", repeated.as_bytes())
                .unwrap_err()
                .contains("128")
        );
        let header = format!(
            "--Aa\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\nx\r\n--Aa--",
            "x".repeat(MAX_HEADERS)
        );
        assert!(parse("multipart/form-data; boundary=Aa", header.as_bytes()).is_err());
    }
}
