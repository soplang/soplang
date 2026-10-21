//! Synchronous HTTP requests.

use std::collections::HashMap;
use std::time::Duration;

use ureq::tls::{RootCerts, TlsConfig};
use ureq::Agent;

use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_REDIRECTS: u32 = 5;
const DEFAULT_LIMIT: u64 = 1024 * 1024;
const MAX_LIMIT: u64 = 16 * 1024 * 1024;

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E053_HTTP),
    )
}

fn error_with_details(message: impl Into<String>, details: impl std::fmt::Display) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default()
            .with_code(codes::E053_HTTP)
            .with_details(details.to_string()),
    )
}

struct RequestOptions {
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    timeout: Duration,
    redirects: u32,
    limit: u64,
}

pub fn request(method: &str, url: &str, options: Option<&Value>) -> Result<Value, SoplangError> {
    let method = normalize_method(method)?;
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(error("Cinwaanka HTTP waa inuu ka bilaabmaa http:// ama https://."));
    }
    let options = parse_options(options)?;
    let response = send(&method, url, &options)?;
    response_value(response, options.limit)
}

fn normalize_method(method: &str) -> Result<String, SoplangError> {
    let method = method.to_ascii_uppercase();
    if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
        return Err(error("Habka HTTP waa inuu noqdaa GET, POST, PUT, PATCH, ama DELETE."));
    }
    Ok(method)
}

fn parse_options(options: Option<&Value>) -> Result<RequestOptions, SoplangError> {
    let Some(Value::Object(object)) = options else {
        if options.is_some() {
            return Err(error("Xulashada HTTP waa inay noqotaa walax."));
        }
        return Ok(RequestOptions {
            headers: Vec::new(),
            body: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
            redirects: DEFAULT_REDIRECTS,
            limit: DEFAULT_LIMIT,
        });
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    let mut body_kind = None;
    let mut body = Vec::new();
    if let Some(value) = object.get("qoraal") {
        body_kind = Some("qoraal");
        let Value::Str(text) = value else {
            return Err(error("Jidhka qoraalka waa inuu noqdaa qoraal."));
        };
        body = text.as_bytes().to_vec();
    }
    if let Some(value) = object.get("baytyo") {
        if body_kind.is_some() {
            return Err(error("Codsiga HTTP wuxuu qaadan karaa hal nooc oo jidh ah."));
        }
        body_kind = Some("baytyo");
        let Value::Bytes(bytes) = value else {
            return Err(error("Jidhka baytyada waa inuu noqdaa baytyo."));
        };
        body = super::bytes::read(bytes)?.clone();
    }
    if let Some(value) = object.get("json") {
        if body_kind.is_some() {
            return Err(error("Codsiga HTTP wuxuu qaadan karaa hal nooc oo jidh ah."));
        }
        body = super::json::stringify(value)?.into_bytes();
    }
    let mut headers = headers(object.get("cinwaan"))?;
    if object.contains_key("json") && !headers.iter().any(|(name, _)| name.eq_ignore_ascii_case("content-type")) {
        headers.push(("content-type".to_string(), "application/json".to_string()));
    }
    Ok(RequestOptions {
        headers,
        body,
        timeout: duration(object.get("waqti"))?,
        redirects: redirects(object.get("leexasho"))?,
        limit: limit(object.get("cabbir"))?,
    })
}

fn headers(value: Option<&Value>) -> Result<Vec<(String, String)>, SoplangError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Value::Object(object) = value else {
        return Err(error("Cinwaannada HTTP waa inay noqdaan walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    let mut headers = Vec::new();
    for (name, value) in object.iter() {
        let Value::Str(text) = value else {
            return Err(error("Qiimaha cinwaanka HTTP waa inuu noqdaa qoraal."));
        };
        headers.push((name.clone(), text.clone()));
    }
    Ok(headers)
}

fn duration(value: Option<&Value>) -> Result<Duration, SoplangError> {
    let Some(value) = value else {
        return Ok(DEFAULT_TIMEOUT);
    };
    let seconds = match value {
        Value::Int(seconds) if *seconds > 0 => *seconds as f64,
        Value::Float(seconds) if seconds.is_finite() && *seconds > 0.0 => *seconds,
        _ => return Err(error("Waqtiga HTTP waa inuu noqdaa tiro ka weyn eber.")),
    };
    Duration::try_from_secs_f64(seconds).map_err(|_| error("Waqtiga HTTP ma saxna."))
}

fn redirects(value: Option<&Value>) -> Result<u32, SoplangError> {
    let Some(value) = value else {
        return Ok(DEFAULT_REDIRECTS);
    };
    let Value::Int(count) = value else {
        return Err(error("Tirada leexashada waa inay noqotaa abn."));
    };
    u32::try_from(*count)
        .ok()
        .filter(|count| *count <= 10)
        .ok_or_else(|| error("Tirada leexashada waa inay u dhaxaysaa 0 iyo 10."))
}

fn limit(value: Option<&Value>) -> Result<u64, SoplangError> {
    let Some(value) = value else {
        return Ok(DEFAULT_LIMIT);
    };
    let Value::Int(size) = value else {
        return Err(error("Cabbirka jawaabta waa inuu noqdaa abn."));
    };
    u64::try_from(*size)
        .ok()
        .filter(|size| (1..=MAX_LIMIT).contains(size))
        .ok_or_else(|| error("Cabbirka jawaabta waa inuu u dhaxeeyaa 1 iyo 16777216."))
}

fn send(method: &str, url: &str, options: &RequestOptions) -> Result<ureq::http::Response<ureq::Body>, SoplangError> {
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(options.timeout))
        .timeout_connect(Some(options.timeout))
        .max_redirects(options.redirects)
        .max_redirects_will_error(false)
        .http_status_as_error(false)
        .user_agent("Soplang")
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into();
    let mut builder = ureq::http::Request::builder().method(method).uri(url);
    for (name, value) in &options.headers {
        builder = builder.header(name, value);
    }
    let request = builder
        .body(options.body.clone())
        .map_err(|error| error_with_details("Codsiga HTTP ma saxna.", error))?;
    agent
        .run(request)
        .map_err(|error| error_with_details(explain(&error), &error))
}

fn response_value(mut response: ureq::http::Response<ureq::Body>, limit: u64) -> Result<Value, SoplangError> {
    let status = i64::from(response.status().as_u16());
    let mut header_map = HashMap::new();
    for (name, value) in response.headers() {
        let text = value.to_str().unwrap_or("").to_string();
        header_map
            .entry(name.as_str().to_ascii_lowercase())
            .and_modify(|existing: &mut String| {
                existing.push_str(", ");
                existing.push_str(&text);
            })
            .or_insert(text);
    }
    let content_type = header_map.get("content-type").cloned().unwrap_or_default();
    let bytes = response
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(|error| error_with_details(explain(&error), &error))?;
    let text = std::str::from_utf8(&bytes).ok().map(str::to_owned);
    let json = if content_type.to_ascii_lowercase().contains("json") {
        match &text {
            Some(text) if !text.is_empty() => Some(super::json::parse(text)?),
            _ => None,
        }
    } else {
        None
    };
    let mut fields = HashMap::new();
    fields.insert("heer".to_string(), Value::Int(status));
    fields.insert(
        "cinwaan".to_string(),
        Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
            header_map
                .into_iter()
                .map(|(name, value)| (name, Value::Str(value)))
                .collect(),
        ))),
    );
    fields.insert(
        "qoraal".to_string(),
        text.map(Value::Str).unwrap_or(Value::Null),
    );
    fields.insert("baytyo".to_string(), super::bytes::from_raw_vec(bytes));
    fields.insert("json".to_string(), json.unwrap_or(Value::Null));
    Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(fields))))
}

fn explain(error: &ureq::Error) -> &'static str {
    match error {
        ureq::Error::HostNotFound => "Magaca serverka lama helin.",
        ureq::Error::Timeout(_) => "Waqtigii codsiga HTTP wuu dhammaaday.",
        ureq::Error::Tls(_) | ureq::Error::Rustls(_) => "Xaqiijinta TLS way fashilantay.",
        ureq::Error::Protocol(_) => "Jawaabta HTTP ma saxna.",
        ureq::Error::TooManyRedirects | ureq::Error::RedirectFailed => {
            "Leexashada HTTP way fashilantay."
        }
        ureq::Error::BodyExceedsLimit(_) => "Jawaabta HTTP way ka weyn tahay xadka.",
        ureq::Error::BadUri(_) => "Cinwaanka HTTP ma saxna.",
        ureq::Error::ConnectionFailed | ureq::Error::Io(_) => "Xidhiidhka serverka wuu fashilmay.",
        _ => "Codsiga HTTP wuu fashilmay.",
    }
}
