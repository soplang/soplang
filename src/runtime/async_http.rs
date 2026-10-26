//! Cancellable HTTP requests on the I/O executor.

use std::collections::HashMap;

use reqwest::redirect::Policy;

use super::http;
use super::stdlib::expect_arity_range;
use super::tasks::{self, ExternalKind, OwnedFailure, OwnedResult};
use super::value::Value;
use crate::error::SoplangError;

const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

fn failure(error: reqwest::Error) -> OwnedFailure {
    let message = if error.is_timeout() {
        "Waqtigii codsiga HTTP wuu dhammaaday."
    } else if error.is_redirect() {
        "Leexashada HTTP way fashilantay."
    } else if error.is_builder() || error.is_request() {
        "Codsiga HTTP ma saxna."
    } else if error.is_body() || error.is_decode() {
        "Jawaabta HTTP ma saxna."
    } else {
        "Xidhiidhka serverka wuu fashilmay."
    };
    OwnedFailure::Http(message.to_string(), error.to_string())
}

pub(super) fn request(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "http_codso_shaqo", 2, 3)?;
    let Value::Str(method) = &args[0] else {
        return Err(http::error(
            "Habka http_codso_shaqo() waa inuu noqdaa qoraal.",
        ));
    };
    let Value::Str(url) = &args[1] else {
        return Err(http::error(
            "Cinwaanka http_codso_shaqo() waa inuu noqdaa qoraal.",
        ));
    };
    let method = http::normalize_method(method)?;
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(http::error(
            "Cinwaanka HTTP waa inuu ka bilaabmaa http:// ama https://.",
        ));
    }
    let url = url.clone();
    let options = http::parse_options(args.get(2))?;
    if options.body.len() > MAX_BODY_BYTES {
        return Err(http::error(
            "Jidhka codsiga HTTP wuxuu ka weyn yahay 16 MiB.",
        ));
    }

    tasks::spawn_external(
        async move {
            let redirects = options.redirects as usize;
            let policy = Policy::custom(move |attempt| {
                if attempt.previous().len() > redirects
                    || (attempt.url().scheme() == "http"
                        && attempt.previous().iter().any(|url| url.scheme() == "https"))
                {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            });
            let client = reqwest::Client::builder()
                .timeout(options.timeout)
                .connect_timeout(options.timeout)
                .redirect(policy)
                .user_agent("Soplang")
                .build()
                .map_err(failure)?;
            let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| {
                OwnedFailure::Http("Codsiga HTTP ma saxna.".into(), error.to_string())
            })?;
            let mut request = client.request(method, &url);
            for (name, value) in options.headers {
                request = request.header(name, value);
            }
            if !options.body.is_empty() {
                request = request.body(options.body);
            }
            let mut response = request.send().await.map_err(failure)?;
            let status = i64::from(response.status().as_u16());
            let mut headers = HashMap::new();
            for (name, value) in response.headers() {
                let text = value.to_str().unwrap_or("");
                headers
                    .entry(name.as_str().to_ascii_lowercase())
                    .and_modify(|existing: &mut String| {
                        existing.push_str(", ");
                        existing.push_str(text);
                    })
                    .or_insert_with(|| text.to_string());
            }
            let mut body = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(failure)? {
                if chunk.len() > options.limit as usize - body.len() {
                    return Err(OwnedFailure::HttpTooLarge);
                }
                body.extend_from_slice(&chunk);
            }
            Ok(OwnedResult::Http {
                status,
                headers,
                body,
            })
        },
        ExternalKind::Http,
    )
}
