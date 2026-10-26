//! Bounded workspace discovery for references outside the current import graph.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::Value;

pub(super) fn roots(params: &Value) -> BTreeSet<PathBuf> {
    let mut roots = BTreeSet::new();
    if let Some(folders) = params.get("workspaceFolders").and_then(Value::as_array) {
        for folder in folders {
            if let Some(path) = folder
                .get("uri")
                .and_then(Value::as_str)
                .and_then(file_path)
            {
                roots.insert(path);
            }
        }
    }
    if roots.is_empty() {
        if let Some(path) = params
            .get("rootUri")
            .and_then(Value::as_str)
            .and_then(file_path)
        {
            roots.insert(path);
        }
    }
    roots
}

pub(super) fn file_path(uri: &str) -> Option<PathBuf> {
    url::Url::parse(uri)
        .ok()?
        .to_file_path()
        .ok()
        .map(|path| super::source_identity(&path))
}

pub(super) fn discover(roots: &BTreeSet<PathBuf>, max_files: usize) -> (BTreeSet<PathBuf>, bool) {
    let mut files = BTreeSet::new();
    let mut visited = 0;
    let mut complete = true;
    for root in roots {
        visit(root, 0, &mut visited, max_files, &mut files, &mut complete);
    }
    (files, complete)
}

fn visit(
    dir: &Path,
    depth: usize,
    visited: &mut usize,
    max_files: usize,
    files: &mut BTreeSet<PathBuf>,
    complete: &mut bool,
) {
    if depth > 32 || *visited > 100_000 || files.len() > max_files {
        *complete = false;
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        *complete = false;
        return;
    };
    for entry in entries {
        *visited += 1;
        if *visited > 100_000 || files.len() > max_files {
            *complete = false;
            break;
        }
        let Ok(entry) = entry else {
            *complete = false;
            continue;
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.')
            || matches!(name.as_ref(), "target" | "node_modules" | "dhis" | "vendor")
        {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            *complete = false;
            continue;
        };
        if kind.is_dir() {
            visit(
                &entry.path(),
                depth + 1,
                visited,
                max_files,
                files,
                complete,
            );
        } else if kind.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "sop")
            && name != "schema.sop"
        {
            files.insert(super::source_identity(&entry.path()));
        }
    }
}
