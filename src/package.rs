//! Project manifest (`soplang.toml`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{codes, file_read_error, runtime_error_ex, ErrorMeta, SoplangError};

pub const MANIFEST_FILE: &str = "soplang.toml";
pub const LOCK_FILE: &str = "soplang.lock";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub soplang: String,
    pub source: String,
    pub dependencies: Vec<Dependency>,
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
    load_manifest(dir, &text).map(Some)
}

pub fn load_manifest(dir: &Path, text: &str) -> Result<Manifest, SoplangError> {
    let manifest = parse_manifest(text)?;
    let mut locked = Vec::new();
    check_dependencies(dir, &manifest, &mut Vec::new(), "", &mut locked)?;
    verify_lock(dir, &locked)?;
    Ok(manifest)
}

pub fn parse_manifest(text: &str) -> Result<Manifest, SoplangError> {
    let mut name = None;
    let mut version = None;
    let mut soplang = None;
    let mut source = None;
    let mut dependencies: Vec<Dependency> = Vec::new();
    let mut section = None;
    for (number, raw) in text.lines().enumerate() {
        let line_no = number + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            let header = header.trim();
            if header != "ku-tiirsan" {
                return Err(manifest_error(format!("Qaybta '{header}' lama aqoonsan."), line_no));
            }
            section = Some(header.to_string());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(manifest_error("Safka xirmada waa inuu noqdaa furaha = \"qiime\".", line_no));
        };
        let key = key.trim();
        let value = unquote(value.trim(), line_no)?;
        if section.as_deref() == Some("ku-tiirsan") {
            validate_name_at(&key, line_no)?;
            validate_dependency_path(&value, line_no)?;
            if dependencies.iter().any(|dependency| dependency.name == key) {
                return Err(manifest_error(format!("Ku-tiirsiga '{key}' waa la soo celiyey."), line_no));
            }
            dependencies.push(Dependency { name: key.to_string(), path: value });
            continue;
        }
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
    Ok(Manifest { name, version, soplang, source, dependencies })
}

pub fn manifest_path(dir: &Path) -> PathBuf {
    dir.join(MANIFEST_FILE)
}

fn validate_name(name: &str) -> Result<(), SoplangError> {
    validate_name_at(name, 1)
}

fn validate_name_at(name: &str, line: usize) -> Result<(), SoplangError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(manifest_error("magac ma madhnaan karo.", line));
    };
    if !first.is_ascii_lowercase()
        || !name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        || name.len() > 64
    {
        return Err(manifest_error(
            "magac waa inuu ka bilaabmaa xaraf yar oo ka kooban yahay xarfaha, lambarada, '-' ama '_'.",
            line,
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

fn validate_dependency_path(path: &str, line: usize) -> Result<(), SoplangError> {
    let path_buf = Path::new(path);
    if path.is_empty()
        || path_buf.is_absolute()
        || path_buf.components().all(|component| matches!(component, std::path::Component::CurDir))
    {
        return Err(manifest_error("Jidka ku-tiirsiga waa inuu noqdaa jid isku xiran.", line));
    }
    Ok(())
}

fn check_dependencies(
    dir: &Path,
    manifest: &Manifest,
    stack: &mut Vec<PathBuf>,
    prefix: &str,
    locked: &mut Vec<LockEntry>,
) -> Result<(), SoplangError> {
    let canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    if stack.iter().any(|seen| seen == &canon) {
        return Err(manifest_error("Ku-tiirsanayaasha xirmada waa wareeg.", 1));
    }
    stack.push(canon);
    for dependency in &manifest.dependencies {
        let child = dir.join(&dependency.path);
        let manifest_path = child.join(MANIFEST_FILE);
        if !manifest_path.is_file() {
            return Err(manifest_error(
                format!("Ku-tiirsiga '{}' lama helin.", dependency.name),
                1,
            ));
        }
        let text = fs::read_to_string(&manifest_path).map_err(|error| file_read_error(&manifest_path, &error))?;
        let child_manifest = parse_manifest(&text)?;
        if child_manifest.name != dependency.name {
            return Err(manifest_error(
                format!(
                    "Xirmada '{}' waxay leedahay magaca '{}', laakiin waxaa loo yaqaan '{}'.",
                    dependency.path, child_manifest.name, dependency.name
                ),
                1,
            ));
        }
        let path = join_relative(prefix, &dependency.path);
        if let Some(previous) = locked.iter().find(|entry| entry.name == dependency.name) {
            if previous.version != child_manifest.version || previous.path != path {
                return Err(manifest_error(
                    format!("Ku-tiirsiga '{}' waa la soo celiyey jid kale.", dependency.name),
                    1,
                ));
            }
        } else {
            locked.push(LockEntry {
                name: dependency.name.clone(),
                version: child_manifest.version.clone(),
                path,
            });
        }
        check_dependencies(&child, &child_manifest, stack, &join_relative(prefix, &dependency.path), locked)?;
    }
    stack.pop();
    Ok(())
}

#[derive(Clone)]
struct LockEntry {
    name: String,
    version: String,
    path: String,
}

fn verify_lock(dir: &Path, locked: &[LockEntry]) -> Result<(), SoplangError> {
    let path = dir.join(LOCK_FILE);
    if !path.is_file() {
        return Ok(());
    }
    let text = fs::read_to_string(&path).map_err(|error| file_read_error(&path, &error))?;
    if text.trim_end() != render_lock(locked).trim_end() {
        return Err(manifest_error("soplang.lock ma waafaqsana xirmada.", 1));
    }
    Ok(())
}

fn render_lock(locked: &[LockEntry]) -> String {
    let mut entries = locked.to_vec();
    entries.sort_by(|left, right| left.name.cmp(&right.name).then(left.path.cmp(&right.path)));
    let mut out = String::from("# soplang.lock\n");
    for entry in entries {
        out.push('\n');
        out.push_str("[[xirmo]]\n");
        out.push_str(&format!("magac = \"{}\"\n", entry.name));
        out.push_str(&format!("nusxa = \"{}\"\n", entry.version));
        out.push_str(&format!("jid = \"{}\"\n", entry.path));
    }
    out
}

fn join_relative(prefix: &str, next: &str) -> String {
    let combined = if prefix.is_empty() { next.to_string() } else { format!("{prefix}/{next}") };
    let mut parts = Vec::new();
    for part in combined.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." && parts.last().is_some_and(|last| *last != "..") {
            parts.pop();
        } else {
            parts.push(part);
        }
    }
    if parts.is_empty() { ".".to_string() } else { parts.join("/") }
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
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn parses_local_dependency_paths() {
        let manifest = parse_manifest(
            "magac = \"app\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n\n[ku-tiirsan]\nxisaab = \"../xisaab\"\n",
        )
        .unwrap();
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.dependencies[0].name, "xisaab");
        assert_eq!(manifest.dependencies[0].path, "../xisaab");
    }

    #[test]
    fn rejects_a_newer_language_requirement_and_unknown_keys() {
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"9\"\n").is_err());
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n[kale]\n").is_err());
        assert!(parse_manifest("magac = \"X\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n").is_err());
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\ngelid = \"../out\"\n").is_err());
    }
}
