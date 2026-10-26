//! Cookie serialization and identifiers for the bundled web framework.

use std::collections::HashMap;

use super::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

fn error(message: &str) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E054_HTTP_SERVER),
    )
}

pub fn identifier() -> Result<Value, SoplangError> {
    let mut bytes = [0; 32];
    rustls::crypto::ring::default_provider()
        .secure_random
        .fill(&mut bytes)
        .map_err(|_| error("Aqoonsi qarsoodi ah lama abuuri karin."))?;
    Ok(Value::Str(super::base64::encode(&bytes, true)))
}

pub fn cookie(id: &str, options: Option<&Value>) -> Result<Value, SoplangError> {
    if id.len() > 1024
        || id
            .bytes()
            .any(|b| !b.is_ascii_alphanumeric() && !b"-_".contains(&b))
    {
        return Err(error(
            "Aqoonsiga kalfadhigu wuxuu leeyahay xaraf aan la oggolayn.",
        ));
    }
    let fields = match options {
        None => HashMap::new(),
        Some(Value::Object(fields)) => fields
            .try_borrow()
            .map_err(|_| error("Xulashada kukiga lama akhriyi karo."))?
            .clone(),
        _ => return Err(error("Xulashada kukigu waa inay noqotaa walax.")),
    };
    for name in fields.keys() {
        if !matches!(
            name.as_str(),
            "jid" | "sugan" | "http_keliya" | "isla_goob" | "cimri"
        ) {
            return Err(error("Doorashada kukiga lama aqoonsan."));
        }
    }
    let boolean = |name: &str, default| match fields.get(name) {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(error("Calanka kukigu waa inuu noqdaa bool.")),
    };
    let path = match fields.get("jid") {
        None => "/",
        Some(Value::Str(path))
            if path.starts_with('/')
                && path.bytes().all(|b| (0x21..0x7f).contains(&b) && b != b';') =>
        {
            path
        }
        _ => return Err(error(
            "Jidka kukigu waa inuu ka bilaabmaa / oo aanu lahayn kalaqaybiye ama xaraf xakameeye.",
        )),
    };
    let secure = boolean("sugan", false)?;
    let http_only = boolean("http_keliya", true)?;
    let same_site = match fields.get("isla_goob") {
        None => "Lax",
        Some(Value::Str(value)) if matches!(value.as_str(), "Lax" | "Strict" | "None") => value,
        _ => return Err(error("isla_goob waa inay noqotaa Lax, Strict ama None.")),
    };
    if same_site == "None" && !secure {
        return Err(error(
            "Kukiga SameSite=None wuxuu u baahan yahay sugan: run.",
        ));
    }
    let mut text = format!("kalfadhi={id}; Path={path}; SameSite={same_site}");
    if secure {
        text.push_str("; Secure");
    }
    if http_only {
        text.push_str("; HttpOnly");
    }
    if let Some(value) = fields.get("cimri") {
        match value {
            Value::Int(age) if *age >= 0 => text.push_str(&format!("; Max-Age={age}")),
            _ => return Err(error("Cimriga kukigu waa inuu noqdaa abn aan tabnayn.")),
        }
    }
    Ok(Value::Str(text))
}
