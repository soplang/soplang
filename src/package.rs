//! Project manifest (`soplang.toml`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{codes, file_read_error, runtime_error_ex, ErrorMeta, SoplangError};

pub const MANIFEST_FILE: &str = "soplang.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub soplang: String,
    pub source: String,
}

pub fn load_beside_source(source: &Path) -> Result<Option<Manifest>, SoplangError> {
    let Some(dir) = source.parent() else {
        return Ok(None);
    };
    let path = dir.join(MANIFEST_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|error| file_read_error(&path, &error))?;
    parse_manifest(&text).map(Some)
}

pub fn parse_manifest(text: &str) -> Result<Manifest, SoplangError> {
    let mut name = None;
    let mut version = None;
    let mut soplang = None;
    let mut source = None;
    for (number, raw) in text.lines().enumerate() {
        let line_no = number + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            return Err(manifest_error("Qaybaha xirmada weli lama taageero.", line_no));
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(manifest_error("Safka xirmada waa inuu noqdaa furaha = \"qiime\".", line_no));
        };
        let key = key.trim();
        let value = unquote(value.trim(), line_no)?;
        let slot = match key {
            "magac" => &mut name,
            "nusxa" => &mut version,
            "soplang" => &mut soplang,
            "gelid" => &mut source,
            _ => return Err(manifest_error(format!("Furaha '{key}' lama aqoonsan."), line_no)),
        };
        if slot.is_some() {
            return Err(manifest_error(format!("Furaha '{key}' waa la soo celiyey."), line_no));
        }
        *slot = Some(value);
    }
    let name = name.ok_or_else(|| manifest_error("magac waa ka maqan yahay xirmada.", 1))?;
    let version = version.ok_or_else(|| manifest_error("nusxa waa ka maqan yahay xirmada.", 1))?;
    let soplang = soplang.ok_or_else(|| manifest_error("soplang waa ka maqan yahay xirmada.", 1))?;
    validate_name(&name)?;
    validate_package_version(&version)?;
    validate_language_requirement(&soplang)?;
    let source = source.unwrap_or_else(|| ".".to_string());
    validate_source(&source)?;
    Ok(Manifest { name, version, soplang, source })
}

pub fn manifest_path(dir: &Path) -> PathBuf {
    dir.join(MANIFEST_FILE)
}

fn validate_name(name: &str) -> Result<(), SoplangError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(manifest_error("magac ma madhnaan karo.", 1));
    };
    if !first.is_ascii_lowercase()
        || !name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        || name.len() > 64
    {
        return Err(manifest_error(
            "magac waa inuu ka bilaabmaa xaraf yar oo ka kooban yahay xarfaha, lambarada, '-' ama '_'.",
            1,
        ));
    }
    Ok(())
}

fn validate_package_version(version: &str) -> Result<(), SoplangError> {
    let (core, _) = version.split_once('-').unwrap_or((version, ""));
    if !is_numeric_version(core, true) {
        return Err(manifest_error("nusxa waa inay noqotaa sida 1.2.3.", 1));
    }
    Ok(())
}

fn validate_language_requirement(requirement: &str) -> Result<(), SoplangError> {
    if !is_numeric_version(requirement, false) {
        return Err(manifest_error("soplang waa inuu noqdaa sida 3 ama 3.1.", 1));
    }
    let compiler = parse_version(env!("CARGO_PKG_VERSION"), true).expect("compiler version");
    let required = parse_version(requirement, false).expect("checked version");
    let compiler_minor = compiler.minor.unwrap_or(0);
    let compiler_patch = compiler.patch.unwrap_or(0);
    if required.major != compiler.major
        || required.minor.is_some_and(|minor| compiler_minor < minor)
        || (required.minor == Some(compiler_minor) && required.patch.is_some_and(|patch| compiler_patch < patch))
    {
        return Err(manifest_error(
            format!("Xirmadani waxay u baahan tahay Soplang {requirement}, laakiin kan waa {}.", env!("CARGO_PKG_VERSION")),
            1,
        ));
    }
    Ok(())
}

fn validate_source(source: &str) -> Result<(), SoplangError> {
    let path = Path::new(source);
    if path.is_absolute() || path.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
        return Err(manifest_error("gelid waa inuu noqdaa jid ku dhex jira xirmada.", 1));
    }
    Ok(())
}

fn is_numeric_version(text: &str, require_three: bool) -> bool {
    parse_version(text, require_three).is_some()
}

struct Version {
    major: u64,
    minor: Option<u64>,
    patch: Option<u64>,
}

fn parse_version(text: &str, require_three: bool) -> Option<Version> {
    let mut parts = text.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = match parts.next() {
        Some(part) => Some(part.parse().ok()?),
        None if require_three => return None,
        None => None,
    };
    let patch = match parts.next() {
        Some(part) => Some(part.parse().ok()?),
        None if require_three => return None,
        None => None,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(Version { major, minor, patch })
}

fn unquote(value: &str, line: usize) -> Result<String, SoplangError> {
    let Some(text) = value.strip_prefix('"').and_then(|text| text.strip_suffix('"')) else {
        return Err(manifest_error("Qiimaha xirmada waa inuu noqdaa qoraal.", line));
    };
    if text.contains(['\\', '"', '\n']) {
        return Err(manifest_error("Qoraalka xirmada ma aqbalo ka baxis.", line));
    }
    Ok(text.to_string())
}

fn manifest_error(message: impl Into<String>, line: usize) -> SoplangError {
    runtime_error_ex(
        message,
        line,
        1,
        ErrorMeta::default().with_code(codes::E057_PACKAGE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_package_identity() {
        let manifest = parse_manifest(
            r#"
# xirmo
magac = "xisaab"
nusxa = "0.1.0-alpha"
soplang = "3.1"
gelid = "src"
"#,
        )
        .unwrap();
        assert_eq!(manifest.name, "xisaab");
        assert_eq!(manifest.version, "0.1.0-alpha");
        assert_eq!(manifest.soplang, "3.1");
        assert_eq!(manifest.source, "src");
    }

    #[test]
    fn rejects_a_newer_language_requirement_and_unknown_keys() {
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"9\"\n").is_err());
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n[ku-tiirsan]\n").is_err());
        assert!(parse_manifest("magac = \"X\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n").is_err());
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\ngelid = \"../out\"\n").is_err());
    }
}
