//! Fetch committed Git objects as data; no checkout, hooks, filters, or submodules.

use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{
    cache::TempDirectory, cache_root, hash_dir, hash_files, manifest_error, manifest_from_files,
    store_tree, validate_archive_path, LockEntry, MAX_PACKAGE,
};
use crate::error::{codes, io_reason, runtime_error_ex, ErrorMeta, SoplangError};

const MAX_FILES: usize = 16384;
const INDEX_LIMIT: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(super) struct GitSource {
    pub url: String,
    pub revision: String,
}

impl GitSource {
    pub fn new(remote: &str, revision: &str) -> Result<Self, SoplangError> {
        validate_revision(revision)?;
        if remote.len() > 4096 {
            return Err(manifest_error("Cinwaanka Git aad ayuu u dheer yahay.", 1));
        }
        let remote = if remote.starts_with("git@") && !remote.contains("://") {
            let (host, path) = remote
                .split_once(':')
                .ok_or_else(|| manifest_error("Cinwaanka Git ma saxna.", 1))?;
            format!("ssh://{host}/{path}")
        } else if ["github.com/", "gitlab.com/", "codeberg.org/"]
            .iter()
            .any(|prefix| remote.starts_with(prefix))
        {
            format!("https://{remote}")
        } else {
            remote.to_string()
        };
        let mut url =
            url::Url::parse(&remote).map_err(|_| manifest_error("Cinwaanka Git ma saxna.", 1))?;
        if !matches!(url.scheme(), "https" | "http" | "ssh" | "file")
            || url.query().is_some()
            || url.fragment().is_some()
            || url.password().is_some()
            || remote.chars().any(char::is_control)
        {
            return Err(manifest_error("Cinwaanka Git ma saxna.", 1));
        }
        if url.scheme() == "file" {
            if url.to_file_path().is_err() {
                return Err(manifest_error("Jidka Git ma saxna.", 1));
            }
        } else if url.host_str().is_none() || url.path().trim_matches('/').is_empty() {
            return Err(manifest_error(
                "Cinwaanka Git wuxuu u baahan yahay martigeliye iyo jid.",
                1,
            ));
        }
        let mut path = url.path().trim_end_matches('/').to_string();
        if matches!(
            url.host_str(),
            Some("github.com" | "gitlab.com" | "codeberg.org")
        ) && !path.ends_with(".git")
        {
            path.push_str(".git");
        }
        url.set_path(&path);
        Ok(Self {
            url: url.to_string(),
            revision: revision.to_string(),
        })
    }

    pub fn spec(&self) -> String {
        format!("git+{}#{}", self.url, self.revision)
    }
    pub fn identity(&self) -> String {
        format!("git+{}", self.url)
    }
}

pub(super) fn parse(text: &str) -> Result<Option<GitSource>, SoplangError> {
    if let Some(text) = text.strip_prefix("git+") {
        let (url, revision) = text.split_once('#').unwrap_or((text, "HEAD"));
        return GitSource::new(url, revision).map(Some);
    }
    if ["github.com/", "gitlab.com/", "codeberg.org/"]
        .iter()
        .any(|prefix| text.starts_with(prefix))
    {
        let (url, revision) = text.split_once('@').unwrap_or((text, "HEAD"));
        return GitSource::new(url, revision).map(Some);
    }
    let (url, revision) = text.split_once('#').unwrap_or((text, "HEAD"));
    if text.starts_with("git@")
        || text.starts_with("ssh://")
        || (url.ends_with(".git")
            && (url.starts_with("https://")
                || url.starts_with("http://")
                || url.starts_with("file://")))
    {
        return GitSource::new(url, revision).map(Some);
    }
    Ok(None)
}

fn validate_revision(revision: &str) -> Result<(), SoplangError> {
    if revision.is_empty()
        || revision.len() > 1024
        || revision.starts_with('-')
        || revision.ends_with('.')
        || revision.contains("..")
        || revision.contains("@{")
        || revision == "@"
        || revision
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || "~^:?*[\\".contains(ch))
        || revision
            .split('/')
            .any(|part| part.is_empty() || part.starts_with('.') || part.ends_with(".lock"))
    {
        return Err(manifest_error(
            "Tixraaca Git waa inuu noqdaa laan, summad, ama aqoonsi kaydin oo sax ah.",
            1,
        ));
    }
    Ok(())
}

pub(super) fn valid_oid(text: &str) -> bool {
    matches!(text.len(), 40 | 64)
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn offline() -> bool {
    std::env::var_os("SOPLANG_OFFLINE")
        .is_some_and(|value| value == OsStr::new("1") || value == OsStr::new("true"))
}

pub(super) fn resolve(
    name: &str,
    source: &GitSource,
    hints: &[LockEntry],
) -> Result<(LockEntry, std::path::PathBuf), SoplangError> {
    let identity = source.identity();
    let pinned = hints
        .iter()
        .find(|hint| hint.name == name && hint.path == identity);
    if let Some(pin) = pinned {
        if pin
            .commit
            .as_deref()
            .is_none_or(|commit| !valid_oid(commit))
            || pin.sumad.is_none()
            || !pin.selectors.contains(&source.revision)
        {
            return Err(manifest_error(
                "soplang.lock ma waafaqsana tixraaca Git; adeegso --cusbooneysii.",
                1,
            ));
        }
        let cached = cache_root()?.join(pin.sumad.as_ref().unwrap());
        if cached.is_dir() && hash_dir(&cached).ok().as_ref() == pin.sumad.as_ref() {
            let manifest = super::read_manifest_dir(&cached)?;
            if manifest.name != name || manifest.version != pin.version {
                return Err(manifest_error(
                    "Aqoonsiga xirmada Git ma waafaqsana soplang.lock.",
                    1,
                ));
            }
            let mut entry = pin.clone();
            entry.selectors = vec![source.revision.clone()];
            return Ok((entry, cached));
        }
    }
    if offline() {
        return Err(manifest_error(
            "Xirmada Git kama jirto kayd la hubiyey; soo deji ka hor habka offline.",
            1,
        ));
    }
    let work = TempDirectory::new(&cache_root()?, "git-fetch")?;
    fs::create_dir(work.path.join("empty"))
        .map_err(|failure| manifest_error(io_reason(&failure), 1))?;
    let mut advertised = run(
        &work.path,
        &["ls-remote", "--", &source.url, "HEAD"],
        None,
        INDEX_LIMIT,
    )?;
    if advertised.is_empty() {
        advertised = run(
            &work.path,
            &["ls-remote", "--refs", "--", &source.url],
            None,
            INDEX_LIMIT,
        )?;
    }
    let head_oid = std::str::from_utf8(&advertised)
        .ok()
        .and_then(|text| text.split_whitespace().next())
        .filter(|text| valid_oid(text));
    let Some(head_oid) = head_oid else {
        return Err(manifest_error("Kaydka Git ma laha kaydin sax ah.", 1));
    };
    let repo = work.path.join("repo");
    fs::create_dir(&repo).map_err(|failure| manifest_error(io_reason(&failure), 1))?;
    let format = if head_oid.len() == 64 {
        "--object-format=sha256"
    } else {
        "--object-format=sha1"
    };
    run(
        &repo,
        &["init", "--bare", format, "--template=../empty"],
        None,
        INDEX_LIMIT,
    )?;
    let wanted = pinned
        .and_then(|pin| pin.commit.as_deref())
        .unwrap_or(&source.revision);
    run(
        &repo,
        &[
            "fetch",
            "--no-tags",
            "--no-recurse-submodules",
            "--depth=1",
            "--",
            &source.url,
            wanted,
        ],
        None,
        INDEX_LIMIT,
    )?;
    let commit = String::from_utf8(run(
        &repo,
        &["rev-parse", "--verify", "FETCH_HEAD^{commit}"],
        None,
        1024,
    )?)
    .map_err(|_| manifest_error("Aqoonsiga kaydinta Git ma saxna.", 1))?
    .trim()
    .to_string();
    if !valid_oid(&commit) || pinned.is_some_and(|pin| pin.commit.as_ref() != Some(&commit)) {
        return Err(manifest_error(
            "Kaydinta Git ma waafaqsana soplang.lock.",
            1,
        ));
    }
    let files = read_tree(&repo, &commit)?;
    let manifest = manifest_from_files(&files)?;
    if manifest.name != name {
        return Err(manifest_error(
            format!(
                "Xirmada Git waxay leedahay magaca '{}', laakiin waxaa loo yaqaan '{name}'.",
                manifest.name
            ),
            1,
        ));
    }
    let sumad = hash_files(&files);
    if pinned
        .is_some_and(|pin| pin.sumad.as_ref() != Some(&sumad) || pin.version != manifest.version)
    {
        return Err(manifest_error(
            "Xogta xirmada Git ma waafaqsana soplang.lock.",
            1,
        ));
    }
    let cached = store_tree(&sumad, &files)?;
    Ok((
        LockEntry {
            name: name.into(),
            version: manifest.version,
            path: identity,
            sumad: Some(sumad),
            commit: Some(commit),
            selectors: vec![source.revision.clone()],
        },
        cached,
    ))
}

fn read_tree(repo: &Path, commit: &str) -> Result<Vec<(String, Vec<u8>)>, SoplangError> {
    let index = run(
        repo,
        &["ls-tree", "-r", "-z", "-l", "--full-tree", commit],
        None,
        INDEX_LIMIT,
    )?;
    let mut entries = Vec::new();
    let mut total = 0usize;
    let mut input = Vec::new();
    for record in index
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let split = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| manifest_error("Liiska faylasha Git ma saxna.", 1))?;
        let header = std::str::from_utf8(&record[..split])
            .map_err(|_| manifest_error("Liiska Git ma aha UTF-8.", 1))?;
        let fields: Vec<_> = header.split_whitespace().collect();
        if fields.len() != 4
            || !matches!(fields[0], "100644" | "100755")
            || fields[1] != "blob"
            || !valid_oid(fields[2])
        {
            return Err(manifest_error("Xirmada Git waxay qaadan kartaa faylal caadi ah oo keliya; xidhiidho iyo submodules lama taageero.",1));
        }
        let path = std::str::from_utf8(&record[split + 1..])
            .map_err(|_| manifest_error("Jidka Git ma aha UTF-8.", 1))?;
        validate_archive_path(path)?;
        if path.len() > 4096
            || path
                .split('/')
                .any(|part| part.eq_ignore_ascii_case(".git"))
        {
            return Err(manifest_error("Jidka Git ma saxna.", 1));
        }
        let size = fields[3]
            .parse::<usize>()
            .map_err(|_| manifest_error("Cabbirka faylka Git ma saxna.", 1))?;
        total = total
            .checked_add(size)
            .ok_or_else(|| manifest_error("Xirmada Git aad ayay u weyn tahay.", 1))?;
        if total > MAX_PACKAGE || entries.len() >= MAX_FILES {
            return Err(manifest_error("Xirmada Git aad ayay u weyn tahay.", 1));
        }
        input.extend_from_slice(fields[2].as_bytes());
        input.push(b'\n');
        entries.push((path.to_string(), fields[2].to_string(), size));
    }
    let bytes = run(
        repo,
        &["cat-file", "--batch"],
        Some(input),
        MAX_PACKAGE + INDEX_LIMIT,
    )?;
    let mut remaining = bytes.as_slice();
    let mut files = Vec::new();
    for (path, oid, size) in entries {
        let end = remaining
            .iter()
            .position(|byte| *byte == b'\n')
            .ok_or_else(|| manifest_error("Xogta Git ma dhammaystirna.", 1))?;
        let header = std::str::from_utf8(&remaining[..end])
            .map_err(|_| manifest_error("Xogta Git ma saxna.", 1))?;
        if header != format!("{oid} blob {size}") {
            return Err(manifest_error(
                "Xogta Git ma waafaqsana liiska faylasha.",
                1,
            ));
        }
        remaining = &remaining[end + 1..];
        if remaining.len() <= size || remaining[size] != b'\n' {
            return Err(manifest_error("Faylka Git ma dhammaystirna.", 1));
        }
        files.push((path, remaining[..size].to_vec()));
        remaining = &remaining[size + 1..];
    }
    if !remaining.is_empty() {
        return Err(manifest_error(
            "Xogta Git waxay leedahay wax dheeraad ah.",
            1,
        ));
    }
    Ok(files)
}

fn details(message: &str, detail: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        1,
        1,
        ErrorMeta::default()
            .with_code(codes::E057_PACKAGE)
            .with_details(detail),
    )
}

fn capture(mut pipe: impl Read, limit: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    let mut overflow = false;
    let mut buffer = [0u8; 8192];
    loop {
        let n = pipe.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        let keep = n.min(limit.saturating_sub(bytes.len()));
        bytes.extend_from_slice(&buffer[..keep]);
        overflow |= keep < n;
    }
    Ok((bytes, overflow))
}

fn run(
    repo: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    limit: usize,
) -> Result<Vec<u8>, SoplangError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("GIT_ALLOW_PROTOCOL", "https:http:ssh:file")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_CEILING_DIRECTORIES", repo);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
        "GIT_SHALLOW_FILE",
    ] {
        command.env_remove(key);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().map_err(|failure| {
        details(
            "Git lama bilaabin; hubi in Git rakiban yahay.",
            io_reason(&failure),
        )
    })?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || capture(stdout, limit));
    let errors = std::thread::spawn(move || capture(stderr, 256 * 1024));
    let writer = input.map(|bytes| {
        let mut pipe = child.stdin.take().unwrap();
        std::thread::spawn(move || pipe.write_all(&bytes))
    });
    let timeout = std::env::var("SOPLANG_GIT_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (1..=300000).contains(value))
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_secs(60));
    let start = Instant::now();
    let mut timed_out = false;
    let mut exited = None;
    let status = loop {
        if exited.is_none() {
            match child.try_wait() {
                Ok(status) => exited = status,
                Err(failure) => {
                    kill_tree(&mut child);
                    let _ = child.wait();
                    break Err(failure);
                }
            }
        }
        let drained = reader.is_finished()
            && errors.is_finished()
            && writer.as_ref().is_none_or(|writer| writer.is_finished());
        if drained && exited.is_some() {
            break Ok(exited.unwrap());
        }
        if start.elapsed() >= timeout {
            timed_out = true;
            kill_tree(&mut child);
            break if let Some(status) = exited {
                Ok(status)
            } else {
                child.wait()
            };
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let output = reader
        .join()
        .map_err(|_| manifest_error("Akhriska Git wuu istaagay.", 1))?
        .map_err(|failure| details("Xogta Git lama akhrin.", io_reason(&failure)))?;
    let errors = errors
        .join()
        .map_err(|_| manifest_error("Akhriska Git wuu istaagay.", 1))?
        .map_err(|failure| details("Farriinta Git lama akhrin.", io_reason(&failure)))?;
    let written = writer.map(|writer| writer.join());
    if timed_out {
        return Err(manifest_error("Waqtigii Git wuu dhammaaday.", 1));
    }
    let status = status.map_err(|failure| details("Git ma dhammaan.", io_reason(&failure)))?;
    if !status.success() {
        return Err(details(
            "Git ma soo dejin karo xirmada ama tixraaca la bixiyey.",
            String::from_utf8_lossy(&errors.0).into_owned(),
        ));
    }
    if output.1 || errors.1 {
        return Err(manifest_error("Xogta Git way ka weyn tahay xadka.", 1));
    }
    if written.is_some_and(|result| !matches!(result, Ok(Ok(())))) {
        return Err(manifest_error("Doodaha Git lama gudbin.", 1));
    }
    Ok(output.0)
}

fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        // Every child was started in a fresh process group. Kill its transports
        // too so inherited pipe handles cannot keep the reader threads waiting.
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        if let Ok(pid) = i32::try_from(child.id()) {
            let _ = kill(-pid, 9);
        }
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repository_spellings_have_one_canonical_identity() {
        let short = parse("github.com/goobo/xisaab@v1.2.3").unwrap().unwrap();
        let full = parse("https://github.com/goobo/xisaab.git#v1.2.3")
            .unwrap()
            .unwrap();
        let table = GitSource::new("https://GITHUB.com/goobo/xisaab/", "v1.2.3").unwrap();
        assert_eq!(short.spec(), full.spec());
        assert_eq!(short.spec(), table.spec());
        assert_eq!(
            short.spec(),
            "git+https://github.com/goobo/xisaab.git#v1.2.3"
        );
        assert_eq!(
            parse("git@git.example:team/xisaab.git")
                .unwrap()
                .unwrap()
                .url,
            "ssh://git@git.example/team/xisaab.git"
        );
        assert_eq!(
            parse("git+https://example.com/xisaab#feature@topic")
                .unwrap()
                .unwrap()
                .revision,
            "feature@topic"
        );
        assert!(parse("https://example.com/xisaab.sopkg").unwrap().is_none());
        assert!(parse("../xisaab").unwrap().is_none());
    }
    #[test]
    fn git_sources_reject_option_refspec_and_transport_injection() {
        for remote in [
            "ext::sh -c command",
            "git://example.com/x",
            "https://example.com/x?token=secret",
            "https://u:p@example.com/x",
            "https://example.com/x#main",
        ] {
            assert_eq!(
                GitSource::new(remote, "HEAD").unwrap_err().code(),
                Some("E057"),
                "{remote}"
            );
        }
        for rev in [
            "",
            "-upload-pack=evil",
            "a:b",
            "a..b",
            "foo\\bar",
            "refs//main",
            "foo@{0}",
            "a b",
            "a\nb",
            "a.lock",
            "/main",
            "main/",
        ] {
            assert_eq!(
                GitSource::new("https://example.com/repo", rev)
                    .unwrap_err()
                    .code(),
                Some("E057"),
                "{rev:?}"
            );
        }
        assert!(valid_oid(&"a".repeat(40)) && valid_oid(&"a".repeat(64)));
        assert!(!valid_oid(&"a".repeat(41)) && !valid_oid(&"A".repeat(40)));
    }
}
