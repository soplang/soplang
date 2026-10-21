//! Project manifest (`soplang.toml`), lockfile, and package registry.

use std::cmp::Ordering;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};
use ureq::tls::{RootCerts, TlsConfig};
use ureq::Agent;

use crate::error::{codes, file_read_error, io_reason, runtime_error_ex, ErrorMeta, SoplangError};

mod document;

pub const MANIFEST_FILE: &str = "soplang.toml";
pub const LOCK_FILE: &str = "soplang.lock";

const MAX_PACKAGE: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub spec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub soplang: String,
    pub source: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockEntry {
    name: String,
    version: String,
    path: String,
    sumad: Option<String>,
}

enum Spec {
    Path(String),
    Registry(String),
    Archive(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bound {
    Exact,
    Caret,
    Tilde,
}

#[derive(Clone, PartialEq, Eq)]
struct PackageVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
    text: String,
}

pub fn load_beside_source(source: &Path) -> Result<Option<Manifest>, SoplangError> {
    let Some(dir) = find_manifest_dir(source) else {
        return Ok(None);
    };
    let path = dir.join(MANIFEST_FILE);
    let text = fs::read_to_string(&path).map_err(|error| file_read_error(&path, &error))?;
    load_manifest(&dir, &text)
        .map(Some)
        .map_err(|error| attach_document_error(error, &text, &path))
}

pub fn load_manifest(dir: &Path, text: &str) -> Result<Manifest, SoplangError> {
    let manifest = parse_manifest(text)?;
    let hints = read_lock_hints(dir)?;
    let mut locked = Vec::new();
    resolve_dependencies(
        dir,
        &manifest,
        &mut Vec::new(),
        "",
        &mut locked,
        &hints,
        false,
    )?;
    verify_lock(dir, &locked)?;
    Ok(manifest)
}

pub fn parse_manifest(text: &str) -> Result<Manifest, SoplangError> {
    document::manifest(text)
}

pub fn manifest_path(dir: &Path) -> PathBuf {
    dir.join(MANIFEST_FILE)
}

pub fn add_dependency(dir: &Path, name: &str, spec: &str) -> Result<String, SoplangError> {
    validate_name(name)?;
    classify_at(spec, 1)?;
    let path = manifest_path(dir);
    let text = fs::read_to_string(&path).map_err(|error| file_read_error(&path, &error))?;
    let mut manifest = parse_manifest(&text)?;
    if manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.name == name)
    {
        return Err(manifest_error(
            format!("Ku-tiirsiga '{name}' waa la soo celiyey."),
            1,
        ));
    }
    manifest.dependencies.push(Dependency {
        name: name.to_string(),
        spec: spec.to_string(),
    });
    let hints = read_lock_hints(dir)?;
    let locked = resolve_and_lock(dir, &manifest, &hints)?;
    write_text(&path, &render_manifest(&manifest))?;
    write_text(&dir.join(LOCK_FILE), &render_lock(&locked))?;
    Ok(format!("Waa lagu daray {name}."))
}

pub fn remove_dependency(dir: &Path, name: &str) -> Result<String, SoplangError> {
    let path = manifest_path(dir);
    let text = fs::read_to_string(&path).map_err(|error| file_read_error(&path, &error))?;
    let mut manifest = parse_manifest(&text)?;
    if !manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.name == name)
    {
        return Err(manifest_error(
            format!("Ku-tiirsiga '{name}' lama helin."),
            1,
        ));
    }
    manifest
        .dependencies
        .retain(|dependency| dependency.name != name);
    let hints = read_lock_hints(dir)?;
    let locked = resolve_and_lock(dir, &manifest, &hints)?;
    write_text(&path, &render_manifest(&manifest))?;
    write_text(&dir.join(LOCK_FILE), &render_lock(&locked))?;
    Ok(format!("Waa laga saaray {name}."))
}

pub fn update_dependencies(dir: &Path) -> Result<String, SoplangError> {
    let manifest = read_manifest_dir(dir)?;
    let locked = resolve_and_lock(dir, &manifest, &[])?;
    write_text(&dir.join(LOCK_FILE), &render_lock(&locked))?;
    Ok("Waa la cusbooneysiiyey soplang.lock.".to_string())
}

pub fn publish_package(dir: &Path) -> Result<String, SoplangError> {
    let manifest = read_manifest_dir(dir)?;
    for dependency in &manifest.dependencies {
        if matches!(classify_at(&dependency.spec, 1)?, Spec::Path(_)) {
            return Err(manifest_error(
                "Daabacaaddu ma qaadan karto jid isku xiran. Isticmaal kayd: ama http.",
                1,
            ));
        }
    }
    let registry = registry_root()?;
    let dest = registry.join(&manifest.name).join(&manifest.version);
    if dest.exists() {
        return Err(manifest_error(
            format!(
                "Nusxada {} ee '{}' hore ayay u daabacan tahay.",
                manifest.version, manifest.name
            ),
            1,
        ));
    }
    let files = collect_package(dir, &manifest)?;
    write_tree(&dest, &files)?;
    let archive = registry
        .join(&manifest.name)
        .join(format!("{}-{}.sopkg", manifest.name, manifest.version));
    write_bytes(&archive, &pack(&files))?;
    Ok(format!(
        "Waa la daabacay {} {}.",
        manifest.name, manifest.version
    ))
}

pub fn import_file(base_dir: &Path, spec: &str) -> Result<Option<PathBuf>, SoplangError> {
    let Some((name, rest)) = package_spec(spec) else {
        return Ok(None);
    };
    let Some(manifest_dir) = find_manifest_dir(base_dir) else {
        return Ok(None);
    };
    let manifest = read_manifest_dir(&manifest_dir)?;
    let Some(dependency) = manifest
        .dependencies
        .iter()
        .find(|dependency| dependency.name == name)
    else {
        return Ok(None);
    };
    let hints = read_lock_hints(&manifest_dir)?;
    let root = materialize(&manifest_dir, dependency, &hints)?;
    let child = read_manifest_dir(&root)?;
    let relative = entry_path(&child.source, name, rest)?;
    let library = root.join(&child.source).join("lib.sop");
    let file = if rest.is_none() && library.is_file() {
        library
    } else {
        root.join(relative)
    };
    if !file.is_file() {
        return Err(manifest_error(
            format!("Faylka xirmada '{name}' lama helin."),
            1,
        ));
    }
    Ok(Some(file))
}

fn resolve_and_lock(
    dir: &Path,
    manifest: &Manifest,
    hints: &[LockEntry],
) -> Result<Vec<LockEntry>, SoplangError> {
    let mut locked = Vec::new();
    resolve_dependencies(
        dir,
        manifest,
        &mut Vec::new(),
        "",
        &mut locked,
        hints,
        false,
    )?;
    Ok(locked)
}

fn resolve_dependencies(
    dir: &Path,
    manifest: &Manifest,
    stack: &mut Vec<PathBuf>,
    prefix: &str,
    locked: &mut Vec<LockEntry>,
    hints: &[LockEntry],
    from_cache: bool,
) -> Result<(), SoplangError> {
    let canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    if stack.iter().any(|seen| seen == &canon) {
        return Err(manifest_error("Ku-tiirsanayaasha xirmada waa wareeg.", 1));
    }
    stack.push(canon);
    for dependency in &manifest.dependencies {
        match classify_at(&dependency.spec, 1)? {
            Spec::Path(path) => {
                if from_cache {
                    return Err(manifest_error(
                        "Xirmada la daabacay ma yeelan karto jid isku xiran.",
                        1,
                    ));
                }
                let child = dir.join(&path);
                if !child.join(MANIFEST_FILE).is_file() {
                    return Err(manifest_error(
                        format!("Ku-tiirsiga '{}' lama helin.", dependency.name),
                        1,
                    ));
                }
                let child_manifest = read_manifest_dir(&child)?;
                if child_manifest.name != dependency.name {
                    return Err(manifest_error(
                        format!(
                            "Xirmada '{}' waxay leedahay magaca '{}', laakiin waxaa loo yaqaan '{}'.",
                            path, child_manifest.name, dependency.name
                        ),
                        1,
                    ));
                }
                let lock_path = join_relative(prefix, &path);
                remember(
                    locked,
                    LockEntry {
                        name: dependency.name.clone(),
                        version: child_manifest.version.clone(),
                        path: lock_path.clone(),
                        sumad: None,
                    },
                )?;
                resolve_dependencies(
                    &child,
                    &child_manifest,
                    stack,
                    &lock_path,
                    locked,
                    hints,
                    false,
                )?;
            }
            Spec::Registry(constraint) => {
                let (entry, cached) = resolve_registry(&dependency.name, &constraint, hints)?;
                remember(locked, entry)?;
                let child_manifest = read_manifest_dir(&cached)?;
                resolve_dependencies(&cached, &child_manifest, stack, "", locked, hints, true)?;
            }
            Spec::Archive(url) => {
                let (entry, cached) = resolve_archive(&dependency.name, &url, hints)?;
                remember(locked, entry)?;
                let child_manifest = read_manifest_dir(&cached)?;
                resolve_dependencies(&cached, &child_manifest, stack, "", locked, hints, true)?;
            }
        }
    }
    stack.pop();
    Ok(())
}

fn remember(locked: &mut Vec<LockEntry>, entry: LockEntry) -> Result<(), SoplangError> {
    if let Some(previous) = locked.iter().find(|item| item.name == entry.name) {
        if previous.version != entry.version
            || previous.path != entry.path
            || previous.sumad != entry.sumad
        {
            return Err(manifest_error(
                format!("Ku-tiirsiga '{}' waa la soo celiyey jid kale.", entry.name),
                1,
            ));
        }
        return Ok(());
    }
    locked.push(entry);
    Ok(())
}

fn materialize(
    base: &Path,
    dependency: &Dependency,
    hints: &[LockEntry],
) -> Result<PathBuf, SoplangError> {
    match classify_at(&dependency.spec, 1)? {
        Spec::Path(path) => {
            let dir = base.join(path);
            if !dir.join(MANIFEST_FILE).is_file() {
                return Err(manifest_error(
                    format!("Ku-tiirsiga '{}' lama helin.", dependency.name),
                    1,
                ));
            }
            Ok(dir)
        }
        Spec::Registry(constraint) => Ok(resolve_registry(&dependency.name, &constraint, hints)?.1),
        Spec::Archive(url) => Ok(resolve_archive(&dependency.name, &url, hints)?.1),
    }
}

fn resolve_registry(
    name: &str,
    constraint: &str,
    hints: &[LockEntry],
) -> Result<(LockEntry, PathBuf), SoplangError> {
    let (bound, required) = parse_constraint(constraint)?;
    if let Some(hint) = hints
        .iter()
        .find(|hint| hint.name == name && hint.path == "kayd")
    {
        let Some(sumad) = hint.sumad.clone() else {
            return Err(manifest_error("soplang.lock ma waafaqsana xirmada.", 1));
        };
        let version = parse_package_version(&hint.version)
            .ok_or_else(|| manifest_error("soplang.lock ma waafaqsana xirmada.", 1))?;
        if !allows(bound, &required, &version) {
            return Err(manifest_error("soplang.lock ma waafaqsana xirmada.", 1));
        }
        let source = registry_root()?.join(name).join(&hint.version);
        let files = read_package_tree(&source)?;
        if hash_files(&files) != sumad {
            return Err(manifest_error("soplang.lock ma waafaqsana xirmada.", 1));
        }
        let cached = store_tree(&sumad, &files)?;
        return Ok((hint.clone(), cached));
    }
    let chosen = highest_version(name, bound, &required)?;
    let files = read_package_tree(&chosen)?;
    let manifest = manifest_from_files(&files)?;
    if manifest.name != name {
        return Err(manifest_error(
            format!(
                "Xirmada kaydka waxay leedahay magaca '{}', laakiin waxaa loo yaqaan '{name}'.",
                manifest.name
            ),
            1,
        ));
    }
    let sumad = hash_files(&files);
    let cached = store_tree(&sumad, &files)?;
    Ok((
        LockEntry {
            name: name.to_string(),
            version: manifest.version,
            path: "kayd".to_string(),
            sumad: Some(sumad),
        },
        cached,
    ))
}

fn resolve_archive(
    name: &str,
    url: &str,
    hints: &[LockEntry],
) -> Result<(LockEntry, PathBuf), SoplangError> {
    if let Some(hint) = hints
        .iter()
        .find(|hint| hint.name == name && hint.path == url)
    {
        let Some(sumad) = hint.sumad.clone() else {
            return Err(manifest_error("soplang.lock ma waafaqsana xirmada.", 1));
        };
        let cached = cache_root()?.join(&sumad);
        if cached.is_dir() && hash_dir(&cached).ok().as_ref() == Some(&sumad) {
            return Ok((hint.clone(), cached));
        }
        let files = unpack(&download(url)?)?;
        if hash_files(&files) != sumad {
            return Err(manifest_error("soplang.lock ma waafaqsana xirmada.", 1));
        }
        return Ok((hint.clone(), store_tree(&sumad, &files)?));
    }
    let files = unpack(&download(url)?)?;
    let manifest = manifest_from_files(&files)?;
    if manifest.name != name {
        return Err(manifest_error(
            format!("Xirmada la soo dejiyey waxay leedahay magaca '{}', laakiin waxaa loo yaqaan '{name}'.", manifest.name),
            1,
        ));
    }
    let sumad = hash_files(&files);
    let cached = store_tree(&sumad, &files)?;
    Ok((
        LockEntry {
            name: name.to_string(),
            version: manifest.version,
            path: url.to_string(),
            sumad: Some(sumad),
        },
        cached,
    ))
}

fn highest_version(
    name: &str,
    bound: Bound,
    required: &PackageVersion,
) -> Result<PathBuf, SoplangError> {
    let dir = registry_root()?.join(name);
    let entries = fs::read_dir(&dir).map_err(|_| {
        manifest_error(
            format!(
                "Ku-tiirsiga '{name}' lama helin nusxa u dhiganta '{}'.",
                required.text
            ),
            1,
        )
    })?;
    let mut best: Option<(PackageVersion, PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join(MANIFEST_FILE).is_file() {
            continue;
        }
        let manifest = read_manifest_dir(&path)?;
        if manifest.name != name {
            continue;
        }
        let Some(version) = parse_package_version(&manifest.version) else {
            continue;
        };
        if path.file_name().and_then(|name| name.to_str()) != Some(version.text.as_str()) {
            continue;
        }
        if !allows(bound, required, &version) {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|(current, _)| cmp_version(&version, current) == Ordering::Greater)
        {
            best = Some((version, path));
        }
    }
    best.map(|(_, path)| path).ok_or_else(|| {
        manifest_error(
            format!(
                "Ku-tiirsiga '{name}' lama helin nusxa u dhiganta '{}'.",
                required.text
            ),
            1,
        )
    })
}

fn verify_lock(dir: &Path, locked: &[LockEntry]) -> Result<(), SoplangError> {
    if !dir.join(LOCK_FILE).exists() {
        return Ok(());
    }
    let actual = read_lock_hints(dir)?;
    let mut expected = locked.to_vec();
    expected.sort_by(|left, right| left.name.cmp(&right.name));
    if actual != expected {
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
        out.push_str(&format!("magac = {}\n", document::quote(&entry.name)));
        out.push_str(&format!("nusxa = {}\n", document::quote(&entry.version)));
        out.push_str(&format!("jid = {}\n", document::quote(&entry.path)));
        if let Some(sumad) = &entry.sumad {
            out.push_str(&format!("sumad = \"{sumad}\"\n"));
        }
    }
    out
}

fn render_manifest(manifest: &Manifest) -> String {
    document::render_manifest(manifest)
}

fn read_lock_hints(dir: &Path) -> Result<Vec<LockEntry>, SoplangError> {
    let path = dir.join(LOCK_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if fs::symlink_metadata(&path).is_ok() {
                return Err(manifest_error(
                    "soplang.lock lama akhrin karo; jidku wuxuu ku xidhan yahay fayl maqan.",
                    1,
                ));
            }
            return Ok(Vec::new());
        }
        Err(error) => return Err(file_read_error(&path, &error)),
    };
    parse_lock(&text).map_err(|error| attach_document_error(error, &text, &path))
}

fn parse_lock(text: &str) -> Result<Vec<LockEntry>, SoplangError> {
    document::lock(text)
}

fn attach_document_error(error: SoplangError, text: &str, path: &Path) -> SoplangError {
    crate::frontend::source_error(error, text, Some(path))
}

fn find_manifest_dir(start: &Path) -> Option<PathBuf> {
    let mut dir = start.canonicalize().ok()?;
    if !dir.is_dir() {
        dir.pop();
    }
    let cwd = std::env::current_dir()
        .ok()
        .and_then(|path| path.canonicalize().ok());
    let inside_cwd = cwd.as_ref().is_some_and(|cwd| dir.starts_with(cwd));
    loop {
        if dir.join(MANIFEST_FILE).is_file() {
            return Some(dir);
        }
        if inside_cwd && cwd.as_ref().is_some_and(|cwd| cwd == &dir) {
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn read_manifest_dir(dir: &Path) -> Result<Manifest, SoplangError> {
    let path = dir.join(MANIFEST_FILE);
    let text = fs::read_to_string(&path).map_err(|error| file_read_error(&path, &error))?;
    parse_manifest(&text).map_err(|error| attach_document_error(error, &text, &path))
}

fn manifest_from_files(files: &[(String, Vec<u8>)]) -> Result<Manifest, SoplangError> {
    let text = files
        .iter()
        .find(|(path, _)| path == "soplang.toml")
        .ok_or_else(|| manifest_error("soplang.toml waa ka maqan yahay xirmada.", 1))?;
    let text = std::str::from_utf8(&text.1)
        .map_err(|_| manifest_error("soplang.toml ma aha qoraal UTF-8 ah.", 1))?;
    parse_manifest(text)
}

fn package_spec(spec: &str) -> Option<(&str, Option<&str>)> {
    if spec.ends_with(".sop") && !spec.contains('/') && !spec.contains('\\') {
        return None;
    }
    let (name, rest) = match spec.split_once('/') {
        Some((name, rest)) => (name, Some(rest)),
        None => (spec, None),
    };
    is_package_name(name).then_some((name, rest))
}

fn entry_path(source: &str, name: &str, rest: Option<&str>) -> Result<String, SoplangError> {
    let tail = match rest {
        None => format!("{name}.sop"),
        Some(rest) => {
            validate_archive_path(rest)?;
            if !rest.ends_with(".sop") {
                return Err(manifest_error(
                    "Keen-ka xirmada waa inuu noqdaa fayl .sop.",
                    1,
                ));
            }
            rest.to_string()
        }
    };
    if source == "." {
        Ok(tail)
    } else {
        Ok(format!("{source}/{tail}"))
    }
}

pub(crate) fn is_package_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        && name.len() <= 64
}

fn classify_at(spec: &str, line: usize) -> Result<Spec, SoplangError> {
    if let Some(constraint) = spec.strip_prefix("kayd:") {
        parse_constraint(constraint)?;
        return Ok(Spec::Registry(constraint.to_string()));
    }
    if spec.starts_with("https://") || spec.starts_with("http://") {
        if spec.contains([' ', '"']) || spec.len() > 2048 {
            return Err(manifest_error("Cinwaanka xirmada ma saxna.", line));
        }
        return Ok(Spec::Archive(spec.to_string()));
    }
    validate_dependency_path(spec, line)?;
    Ok(Spec::Path(spec.to_string()))
}

fn parse_constraint(text: &str) -> Result<(Bound, PackageVersion), SoplangError> {
    let (bound, rest) = if let Some(rest) = text.strip_prefix('^') {
        (Bound::Caret, rest)
    } else if let Some(rest) = text.strip_prefix('~') {
        (Bound::Tilde, rest)
    } else {
        (Bound::Exact, text)
    };
    let version = parse_package_version(rest)
        .ok_or_else(|| manifest_error("Nusxada ku-tiirsiga ma saxna.", 1))?;
    if version.pre.is_some() && bound != Bound::Exact {
        return Err(manifest_error(
            "Nusxa horudhac ah waxay u baahan tahay isku mid.",
            1,
        ));
    }
    Ok((bound, version))
}

fn allows(bound: Bound, required: &PackageVersion, candidate: &PackageVersion) -> bool {
    if bound == Bound::Exact {
        return required == candidate;
    }
    if required.pre.is_some()
        || candidate.pre.is_some()
        || cmp_version(candidate, required) == Ordering::Less
    {
        return false;
    }
    match bound {
        Bound::Tilde => candidate.major == required.major && candidate.minor == required.minor,
        Bound::Caret if required.major > 0 => candidate.major == required.major,
        Bound::Caret if required.minor > 0 => {
            candidate.major == 0 && candidate.minor == required.minor
        }
        Bound::Caret => {
            candidate.major == 0 && candidate.minor == 0 && candidate.patch == required.patch
        }
        Bound::Exact => false,
    }
}

fn cmp_version(left: &PackageVersion, right: &PackageVersion) -> Ordering {
    (left.major, left.minor, left.patch)
        .cmp(&(right.major, right.minor, right.patch))
        .then_with(|| match (&left.pre, &right.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(left), Some(right)) => left.cmp(right),
        })
}

fn parse_package_version(text: &str) -> Option<PackageVersion> {
    let (core, pre) = match text.split_once('-') {
        Some((core, pre))
            if !pre.is_empty()
                && pre
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '.') =>
        {
            (core, Some(pre.to_string()))
        }
        Some(_) => return None,
        None => (text, None),
    };
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(PackageVersion {
        major,
        minor,
        patch,
        pre,
        text: text.to_string(),
    })
}

fn registry_root() -> Result<PathBuf, SoplangError> {
    soplang_dir("SOPLANG_KAYD", "kayd")
}

fn cache_root() -> Result<PathBuf, SoplangError> {
    soplang_dir("SOPLANG_CACHE", "cache")
}

fn soplang_dir(env_name: &str, folder: &str) -> Result<PathBuf, SoplangError> {
    if let Some(path) = std::env::var_os(env_name) {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return Err(manifest_error(format!("{env_name} ma madhnaan karo."), 1));
        }
        return Ok(path);
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    let Some(home) = home else {
        return Err(manifest_error(
            format!("Kaydka xirmada lama helin. Deji {env_name}."),
            1,
        ));
    };
    Ok(PathBuf::from(home).join(".soplang").join(folder))
}

fn collect_package(
    dir: &Path,
    manifest: &Manifest,
) -> Result<Vec<(String, Vec<u8>)>, SoplangError> {
    let mut files = Vec::new();
    let mut total = 0usize;
    if manifest.source == "." {
        read_tree(dir, "", &mut files, &mut total)?;
    } else {
        let manifest_path = dir.join(MANIFEST_FILE);
        let bytes =
            fs::read(&manifest_path).map_err(|error| file_read_error(&manifest_path, &error))?;
        total += bytes.len();
        files.push(("soplang.toml".to_string(), bytes));
        let source = dir.join(&manifest.source);
        if !source.is_dir() {
            return Err(manifest_error("gelid lama helin.", 1));
        }
        read_tree(&source, &manifest.source, &mut files, &mut total)?;
    }
    if !files.iter().any(|(path, _)| path == "soplang.toml") {
        return Err(manifest_error(
            "soplang.toml waa ka maqan yahay xirmada.",
            1,
        ));
    }
    Ok(files)
}

fn read_package_tree(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, SoplangError> {
    let mut files = Vec::new();
    let mut total = 0usize;
    read_tree(dir, "", &mut files, &mut total)?;
    Ok(files)
}

fn read_tree(
    dir: &Path,
    prefix: &str,
    files: &mut Vec<(String, Vec<u8>)>,
    total: &mut usize,
) -> Result<(), SoplangError> {
    let entries = fs::read_dir(dir).map_err(|error| {
        manifest_error(
            format!("Xirmada lama akhriyi karo. {}", io_reason(&error)),
            1,
        )
    })?;
    let mut entries = entries.collect::<Result<Vec<_>, _>>().map_err(|error| {
        manifest_error(
            format!("Xirmada lama akhriyi karo. {}", io_reason(&error)),
            1,
        )
    })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(manifest_error("Magaca faylka xirmada ma aha UTF-8.", 1));
        };
        if name.starts_with('.') || name == "soplang.lock" {
            continue;
        }
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            manifest_error(
                format!("Xirmada lama akhriyi karo. {}", io_reason(&error)),
                1,
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(manifest_error("Xirmada ma daabici karto xidhiidh.", 1));
        }
        let relative = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}/{name}")
        };
        if metadata.is_dir() {
            read_tree(&path, &relative, files, total)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        let bytes = fs::read(&path).map_err(|error| file_read_error(&path, &error))?;
        *total = total
            .checked_add(bytes.len())
            .ok_or_else(|| manifest_error("Xirmada aad ayay u weyn tahay.", 1))?;
        if *total > MAX_PACKAGE {
            return Err(manifest_error("Xirmada aad ayay u weyn tahay.", 1));
        }
        files.push((relative, bytes));
    }
    Ok(())
}

fn store_tree(sumad: &str, files: &[(String, Vec<u8>)]) -> Result<PathBuf, SoplangError> {
    let cache = cache_root()?;
    let dest = cache.join(sumad);
    if dest.is_dir() && hash_dir(&dest).ok().as_deref() == Some(sumad) {
        return Ok(dest);
    }
    fs::create_dir_all(&cache)
        .map_err(|error| manifest_error(format!("Kaydka lama furin. {}", io_reason(&error)), 1))?;
    let temp = cache.join(format!(".{sumad}.sopliin"));
    write_tree(&temp, files)?;
    if hash_dir(&temp)? != sumad {
        let _ = fs::remove_dir_all(&temp);
        return Err(manifest_error(
            "Summadda xirmada way isbeddeshay inta lagu kaydinayey.",
            1,
        ));
    }
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|error| {
            manifest_error(
                format!("Kaydka lama beddeli karo. {}", io_reason(&error)),
                1,
            )
        })?;
    }
    fs::rename(&temp, &dest).map_err(|error| {
        manifest_error(
            format!("Kaydka lama beddeli karo. {}", io_reason(&error)),
            1,
        )
    })?;
    Ok(dest)
}

fn hash_dir(dir: &Path) -> Result<String, SoplangError> {
    Ok(hash_files(&read_package_tree(dir)?))
}

fn hash_files(files: &[(String, Vec<u8>)]) -> String {
    let mut files = files.to_vec();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    for (path, bytes) in files {
        hasher.update(path.as_bytes());
        hasher.update([0xff]);
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn write_tree(dir: &Path, files: &[(String, Vec<u8>)]) -> Result<(), SoplangError> {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).map_err(|error| {
        manifest_error(format!("Xirmada lama qori karin. {}", io_reason(&error)), 1)
    })?;
    for (relative, bytes) in files {
        validate_archive_path(relative)?;
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|error| {
                    manifest_error(format!("Xirmada lama qori karin. {}", io_reason(&error)), 1)
                })?;
            }
        }
        write_bytes(&path, bytes)?;
    }
    Ok(())
}

fn pack(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut files = files.to_vec();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut out = b"SOPKG1\n".to_vec();
    for (path, bytes) in files {
        out.extend(path.as_bytes());
        out.push(b'\n');
        out.extend(bytes.len().to_string().as_bytes());
        out.push(b'\n');
        out.extend(&bytes);
    }
    out
}

fn unpack(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, SoplangError> {
    let magic = b"SOPKG1\n";
    if !bytes.starts_with(magic) {
        return Err(manifest_error("Kaydka xirmada ma aha SOPKG1.", 1));
    }
    let mut rest = &bytes[magic.len()..];
    let mut files = Vec::new();
    let mut total = 0usize;
    while !rest.is_empty() {
        let Some(line_end) = rest.iter().position(|byte| *byte == b'\n') else {
            return Err(manifest_error("Kaydka xirmada waa go'an yahay.", 1));
        };
        let path = std::str::from_utf8(&rest[..line_end])
            .map_err(|_| manifest_error("Jidka kaydka ma aha UTF-8.", 1))?;
        validate_archive_path(path)?;
        rest = &rest[line_end + 1..];
        let Some(len_end) = rest.iter().position(|byte| *byte == b'\n') else {
            return Err(manifest_error("Kaydka xirmada waa go'an yahay.", 1));
        };
        let len_text = std::str::from_utf8(&rest[..len_end])
            .map_err(|_| manifest_error("Cabbirka kaydka ma saxna.", 1))?;
        let len: usize = len_text
            .parse()
            .map_err(|_| manifest_error("Cabbirka kaydka ma saxna.", 1))?;
        rest = &rest[len_end + 1..];
        if rest.len() < len {
            return Err(manifest_error("Kaydka xirmada waa go'an yahay.", 1));
        }
        total = total
            .checked_add(len)
            .ok_or_else(|| manifest_error("Xirmada aad ayay u weyn tahay.", 1))?;
        if total > MAX_PACKAGE {
            return Err(manifest_error("Xirmada aad ayay u weyn tahay.", 1));
        }
        files.push((path.to_string(), rest[..len].to_vec()));
        rest = &rest[len..];
    }
    if !files.iter().any(|(path, _)| path == "soplang.toml") {
        return Err(manifest_error(
            "soplang.toml waa ka maqan yahay xirmada.",
            1,
        ));
    }
    Ok(files)
}

fn download(url: &str) -> Result<Vec<u8>, SoplangError> {
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .timeout_connect(Some(Duration::from_secs(30)))
        .max_redirects(5)
        .http_status_as_error(false)
        .user_agent("Soplang")
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into();
    let request = ureq::http::Request::builder()
        .method("GET")
        .uri(url)
        .body(Vec::new())
        .map_err(|_| manifest_error("Cinwaanka xirmada ma saxna.", 1))?;
    let mut response = agent.run(request).map_err(|error| {
        runtime_error_ex(
            "Soo dejinta xirmada way fashilantay.",
            1,
            1,
            ErrorMeta::default()
                .with_code(codes::E057_PACKAGE)
                .with_details(error.to_string()),
        )
    })?;
    if response.status().as_u16() != 200 {
        return Err(manifest_error("Soo dejinta xirmada way fashilantay.", 1));
    }
    response
        .body_mut()
        .with_config()
        .limit(MAX_PACKAGE as u64)
        .read_to_vec()
        .map_err(|_| manifest_error("Soo dejinta xirmada way fashilantay.", 1))
}

fn validate_archive_path(path: &str) -> Result<(), SoplangError> {
    let check = Path::new(path);
    if path.is_empty()
        || path.starts_with('/')
        || path.contains(['\\', ':'])
        || check.is_absolute()
        || check
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(manifest_error("Jidka xirmada ma saxna.", 1));
    }
    Ok(())
}

fn write_text(path: &Path, text: &str) -> Result<(), SoplangError> {
    write_bytes(path, text.as_bytes())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), SoplangError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                manifest_error(
                    format!(
                        "Faylka '{}' lama qori karin. {}",
                        path.display(),
                        io_reason(&error)
                    ),
                    1,
                )
            })?;
        }
    }
    let mut file = fs::File::create(path).map_err(|error| {
        manifest_error(
            format!(
                "Faylka '{}' lama qori karin. {}",
                path.display(),
                io_reason(&error)
            ),
            1,
        )
    })?;
    file.write_all(bytes).map_err(|error| {
        manifest_error(
            format!(
                "Faylka '{}' lama qori karin. {}",
                path.display(),
                io_reason(&error)
            ),
            1,
        )
    })
}

fn validate_name(name: &str) -> Result<(), SoplangError> {
    validate_name_at(name, 1)
}

fn validate_name_at(name: &str, line: usize) -> Result<(), SoplangError> {
    if is_package_name(name) {
        Ok(())
    } else if name.is_empty() {
        Err(manifest_error("magac ma madhnaan karo.", line))
    } else {
        Err(manifest_error(
            "magac waa inuu ka bilaabmaa xaraf yar oo ka kooban yahay xarfaha, lambarada, '-' ama '_'.",
            line,
        ))
    }
}

fn validate_package_version(version: &str) -> Result<(), SoplangError> {
    if parse_package_version(version).is_some() {
        Ok(())
    } else {
        Err(manifest_error("nusxa waa inay noqotaa sida 1.2.3.", 1))
    }
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
        || (required.minor == Some(compiler_minor)
            && required.patch.is_some_and(|patch| compiler_patch < patch))
    {
        return Err(manifest_error(
            format!(
                "Xirmadani waxay u baahan tahay Soplang {requirement}, laakiin kan waa {}.",
                env!("CARGO_PKG_VERSION")
            ),
            1,
        ));
    }
    Ok(())
}

fn validate_dependency_path(path: &str, line: usize) -> Result<(), SoplangError> {
    let path_buf = Path::new(path);
    if path.is_empty()
        || path_buf.is_absolute()
        || path_buf
            .components()
            .all(|component| matches!(component, Component::CurDir))
    {
        return Err(manifest_error(
            "Jidka ku-tiirsiga waa inuu noqdaa jid isku xiran.",
            line,
        ));
    }
    Ok(())
}

fn join_relative(prefix: &str, next: &str) -> String {
    let combined = if prefix.is_empty() {
        next.to_string()
    } else {
        format!("{prefix}/{next}")
    };
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
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn validate_source(source: &str) -> Result<(), SoplangError> {
    let path = Path::new(source);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(manifest_error(
            "gelid waa inuu noqdaa jid ku dhex jira xirmada.",
            1,
        ));
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
    Some(Version {
        major,
        minor,
        patch,
    })
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
            "magac = \"app\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n\n[ku-tiirsan]\nxisaab = \"../xisaab\"\nkaydka = \"kayd:^0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dependencies[0].name, "xisaab");
        assert_eq!(manifest.dependencies[0].spec, "../xisaab");
        assert_eq!(manifest.dependencies[1].spec, "kayd:^0.1.0");
    }

    #[test]
    fn rejects_a_newer_language_requirement_and_unknown_keys() {
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"9\"\n").is_err());
        assert!(
            parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n[kale]\n").is_err()
        );
        assert!(parse_manifest("magac = \"X\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n").is_err());
        assert!(parse_manifest(
            "magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\ngelid = \"../out\"\n"
        )
        .is_err());
        assert!(parse_manifest("magac = \"x\"\nnusxa = \"1.0.0\"\nsoplang = \"3\"\n\n[ku-tiirsan]\na = \"kayd:^0.1.0-alpha\"\n").is_err());
    }

    #[test]
    fn caret_and_tilde_bounds_select_compatible_versions() {
        let required = parse_package_version("0.1.2").unwrap();
        let newer = parse_package_version("0.1.9").unwrap();
        let next_minor = parse_package_version("0.2.0").unwrap();
        assert!(allows(Bound::Caret, &required, &newer));
        assert!(!allows(Bound::Caret, &required, &next_minor));
        assert!(allows(Bound::Tilde, &required, &newer));
        assert!(!allows(Bound::Tilde, &required, &next_minor));
        let major = parse_package_version("1.2.3").unwrap();
        let same_major = parse_package_version("1.9.0").unwrap();
        assert!(allows(Bound::Caret, &major, &same_major));
        assert!(!allows(Bound::Tilde, &major, &same_major));
    }

    #[test]
    fn an_archive_round_trip_keeps_the_tree_hash() {
        let files = vec![
            ("soplang.toml".to_string(), b"magac = \"xisaab\"\n".to_vec()),
            ("src/xisaab.sop".to_string(), b"qor(1)\n".to_vec()),
        ];
        let restored = unpack(&pack(&files)).unwrap();
        assert_eq!(hash_files(&files), hash_files(&restored));
        assert_eq!(restored.len(), 2);
    }
}
