//! Isolated native tests, bounded process workers, and source-aware coverage.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::error::{file_read_error, file_write_error, io_error, runtime_error, SoplangError};
use crate::frontend::{ast::Stmt, imports::resolve_imports, parse_source, Expr};
use crate::runtime::CoverageReport;

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error(message, 0, 0)
}

#[derive(Default)]
struct Options {
    path: Option<PathBuf>,
    filter: Option<String>,
    index: Option<usize>,
    workers: Option<usize>,
    cover: bool,
    report: Option<PathBuf>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, SoplangError> {
        let mut options = Self::default();
        let mut args = args.iter();
        let mut positional = false;
        while let Some(arg) = args.next() {
            if !positional && arg == "--" {
                positional = true;
                continue;
            }
            match arg.as_str() {
                "--dabool" if !positional => options.cover = true,
                "--wada" if !positional => {
                    options.workers = Some(
                        std::thread::available_parallelism()
                            .map(usize::from)
                            .unwrap_or(1)
                            .min(8),
                    )
                }
                "--shaandhe" | "--lambarka" | "--shaqaale" | "--warbixin" if !positional => {
                    let value = args
                        .next()
                        .filter(|value| !value.starts_with("--"))
                        .ok_or_else(|| error(format!("'{arg}' waxay u baahan tahay qiime.")))?;
                    match arg.as_str() {
                        "--shaandhe" => options.filter = Some(value.clone()),
                        "--warbixin" => options.report = Some(value.into()),
                        "--lambarka" => {
                            options.index = Some(value.parse().map_err(|_| {
                                error(
                                    "Lambarka tijaabadu waa inuu noqdaa tiro dhan oo aan tabnayn.",
                                )
                            })?)
                        }
                        _ => {
                            let workers = value
                                .parse::<usize>()
                                .ok()
                                .filter(|n| (1..=32).contains(n))
                                .ok_or_else(|| {
                                    error("Tirada shaqaaluhu waa inay u dhexaysaa 1 iyo 32.")
                                })?;
                            options.workers = Some(workers);
                        }
                    }
                }
                _ if !positional && arg.starts_with('-') => {
                    return Err(error(format!("Doorashada '{arg}' lama aqoonsan.")))
                }
                _ => {
                    if options.path.replace(arg.into()).is_some() {
                        return Err(error(
                            "Hal fayl ama gal oo tijaabooyin ah ayaa la bixin karaa.",
                        ));
                    }
                }
            }
        }
        if options.index.is_some() && options.filter.is_some() {
            return Err(error("--lambarka iyo --shaandhe lama wada adeegsan karo."));
        }
        Ok(options)
    }
}

#[derive(Clone)]
pub(super) struct TestCase {
    name: String,
    expect_error: bool,
    expected_code: Option<String>,
    body: Vec<Stmt>,
}

type TestParts = (Vec<Stmt>, Vec<Vec<Stmt>>, Vec<Vec<Stmt>>, Vec<TestCase>);

pub(super) fn split_tests(stmts: &[Stmt]) -> TestParts {
    let (mut prelude, mut setups, mut cleanups, mut tests) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for stmt in stmts {
        match stmt.unlocated() {
            Stmt::Test {
                name,
                expect_error,
                expected_code,
                body,
            } => tests.push(TestCase {
                name: name.clone(),
                expect_error: *expect_error,
                expected_code: expected_code.clone(),
                body: body.clone(),
            }),
            Stmt::Fixture { setup: true, body } => setups.push(body.clone()),
            Stmt::Fixture { setup: false, body } => cleanups.push(body.clone()),
            _ => prelude.push(stmt.clone()),
        }
    }
    (prelude, setups, cleanups, tests)
}

pub(super) fn materialize(
    prelude: &[Stmt],
    setups: &[Vec<Stmt>],
    cleanups: &[Vec<Stmt>],
    tests: &[TestCase],
    index: usize,
) -> Vec<Stmt> {
    let mut program = prelude.to_vec();
    let mut body: Vec<_> = setups.iter().flatten().cloned().collect();
    // Only the body is intercepted. Prelude/setup/cleanup and compilation errors
    // remain ordinary failures, regardless of a test's expected-error marker.
    body.push(Stmt::TryCatch {
        try_body: tests[index].body.clone(),
        err_var: Some("#test_message".into()),
        err_details: Some("#test_error".into()),
        catch_body: vec![Stmt::Expr(Expr::Call {
            name: "#test_failure".into(),
            args: vec![Expr::Identifier("#test_error".into())],
        })],
        finally_body: Vec::new(),
    });
    let cleanup: Vec<_> = cleanups.iter().flatten().cloned().collect();
    if cleanup.is_empty() {
        program.extend(body);
    } else {
        program.push(Stmt::TryCatch {
            try_body: body,
            err_var: None,
            err_details: None,
            catch_body: Vec::new(),
            finally_body: cleanup,
        });
    }
    program
}

struct Suite {
    path: PathBuf,
    parts: TestParts,
}

#[derive(Default)]
struct Summary {
    passed: usize,
    failed: usize,
    coverage: CoverageReport,
}

pub(super) fn run(args: &[String]) -> Result<i32, SoplangError> {
    if args == ["--help"] || args == ["-h"] {
        println!("Adeegsiga: soplang test [FAYLKA|GALKA] [--shaandhe MAGAC] [--wada | --shaqaale 1..32] [--dabool]\nGalalka waxaa laga raadiyaa *_test.sop, *.test.sop iyo faylasha .sop ee galka tests.\n--lambarka N: hal tijaabo, laga tiriyo eber, oo ku jirta hal fayl.");
        return Ok(0);
    }
    let options = Options::parse(args)?;
    let path = options.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let metadata = fs::metadata(&path).map_err(|failure| file_read_error(&path, &failure))?;
    if options.index.is_some() && !metadata.is_file() {
        return Err(error("--lambarka waxay u baahan tahay hal fayl."));
    }
    let mut files = Vec::new();
    if metadata.is_file() {
        files.push(fs::canonicalize(&path).map_err(|failure| file_read_error(&path, &failure))?);
    } else {
        discover(&path, false, 0, &mut 0, &mut files)?;
        files.sort();
    }
    let mut suites = Vec::new();
    let mut jobs = Vec::new();
    for path in files {
        crate::package::load_beside_source(&path)?;
        let text = fs::read_to_string(&path).map_err(|failure| file_read_error(&path, &failure))?;
        let stmts = resolve_imports(parse_source(&text, Some(&path))?, Some(&path))?;
        let parts = split_tests(&stmts);
        if options.index.is_some_and(|index| index >= parts.3.len()) {
            return Err(error("Lambarka tijaabada ma jiro."));
        }
        let suite = suites.len();
        for (index, test) in parts.3.iter().enumerate() {
            if options.index.is_none_or(|chosen| chosen == index)
                && options
                    .filter
                    .as_ref()
                    .is_none_or(|filter| test.name.contains(filter))
            {
                jobs.push((suite, index));
            }
        }
        suites.push(Suite { path, parts });
    }
    let summary = if options.workers.is_some_and(|n| n > 1) && jobs.len() > 1 {
        parallel(&suites, &jobs, options.workers.unwrap(), options.cover)?
    } else {
        let mut summary = Summary::default();
        for (suite, index) in jobs {
            let (prelude, setups, cleanups, tests) = &suites[suite].parts;
            let test = &tests[index];
            let program = materialize(prelude, setups, cleanups, tests, index);
            let result = crate::run_statements_report(
                &program,
                false,
                crate::ExecutionOptions {
                    coverage: options.cover,
                    profile: false,
                },
            );
            let failure = match result {
                Err(failure) => Some(failure),
                Ok(outcome) => {
                    summary.coverage.merge(&outcome.report.coverage);
                    match (outcome.error, outcome.report.body_error, test.expect_error) {
                        (Some(failure), _, _) => Some(failure),
                        (None, None, false) => None,
                        (None, Some(failure), true)
                            if test
                                .expected_code
                                .as_deref()
                                .is_none_or(|code| failure.code() == Some(code)) =>
                        {
                            None
                        }
                        (None, Some(failure), true) => Some(error(format!(
                            "Summadda '{}' ayaa la filayay; '{}' ayaa timid: {}",
                            test.expected_code.as_deref().unwrap(),
                            failure.code().unwrap_or("-"),
                            failure.message()
                        ))),
                        (None, None, true) => Some(error(
                            "Khalad ayaa la filayay, laakiin hawshu way dhammaatay.",
                        )),
                        (None, Some(failure), false) => Some(failure),
                    }
                }
            };
            if let Some(failure) = failure {
                summary.failed += 1;
                println!("fashil: {}: {}", test.name, failure.message());
                eprintln!("{}", crate::format_error_with_source(&failure, None));
            } else {
                summary.passed += 1;
                println!("waa: {}", test.name);
            }
        }
        summary
    };
    if let Some(path) = options.report {
        write_report(&path, &summary)?;
    } else {
        if options.cover {
            print_coverage(&summary.coverage);
        }
        if summary.passed + summary.failed == 0 {
            println!("Tijaabo lama helin.");
        }
        println!("{} waa, {} fashil.", summary.passed, summary.failed);
    }
    Ok(i32::from(summary.failed > 0))
}

fn discover(
    dir: &Path,
    in_tests: bool,
    depth: usize,
    visited: &mut usize,
    files: &mut Vec<PathBuf>,
) -> Result<(), SoplangError> {
    if depth > 32 || files.len() > 4096 {
        return Err(error(
            "Raadinta tijaabooyinku waxay dhaaftay xadka galalka ama faylasha.",
        ));
    }
    let in_tests = in_tests || dir.file_name().is_some_and(|name| name == "tests");
    for entry in fs::read_dir(dir).map_err(|failure| file_read_error(dir, &failure))? {
        *visited += 1;
        if *visited > 100_000 {
            return Err(error(
                "Raadinta tijaabooyinku waxay dhaaftay xadka faylasha.",
            ));
        }
        let entry = entry.map_err(|failure| io_error("Galka lama akhrin karin.", &failure))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.')
            || matches!(name.as_ref(), "target" | "node_modules" | "dhis" | "vendor")
        {
            continue;
        }
        let kind = entry
            .file_type()
            .map_err(|failure| file_read_error(&entry.path(), &failure))?;
        // Never follow directory links or revisit a tree through a file link.
        if kind.is_dir() {
            discover(&entry.path(), in_tests, depth + 1, visited, files)?;
        } else if kind.is_file()
            && (name.ends_with("_test.sop")
                || name.ends_with(".test.sop")
                || (in_tests && name.ends_with(".sop")))
        {
            files.push(
                fs::canonicalize(entry.path())
                    .map_err(|failure| file_read_error(&entry.path(), &failure))?,
            );
        }
    }
    Ok(())
}

fn print_coverage(coverage: &CoverageReport) {
    println!("dabool:");
    for (path, executable) in &coverage.executable {
        let empty = BTreeSet::new();
        let hit = coverage.hit.get(path).unwrap_or(&empty);
        let missed = executable
            .difference(hit)
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let reached = executable.intersection(hit).count();
        let percent = if executable.is_empty() {
            100.0
        } else {
            100.0 * reached as f64 / executable.len() as f64
        };
        println!(
            "{}: {}/{} ({:.1}%) maqan: {}",
            path.display(),
            reached,
            executable.len(),
            percent,
            if missed.is_empty() { "-" } else { &missed }
        );
    }
}

fn write_report(path: &Path, summary: &Summary) -> Result<(), SoplangError> {
    let rows: Vec<_> = summary.coverage.executable.iter().map(|(file, executable)| serde_json::json!({
        "file": file, "executable": executable, "hit": summary.coverage.hit.get(file).cloned().unwrap_or_default()
    })).collect();
    let report =
        serde_json::json!({ "passed": summary.passed, "failed": summary.failed, "coverage": rows })
            .to_string();
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|failure| file_write_error(path, &failure))?;
    file.write_all(report.as_bytes())
        .map_err(|failure| file_write_error(path, &failure))
}

fn read_report(path: &Path) -> Result<Summary, SoplangError> {
    let file = fs::File::open(path).map_err(|failure| file_read_error(path, &failure))?;
    let value: serde_json::Value = serde_json::from_reader(file).map_err(|failure| {
        crate::error::external_error("Warbixinta tijaabadu ma saxna.", failure.to_string())
    })?;
    let number = |value: &serde_json::Value| {
        value
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| error("Warbixinta tijaabadu ma saxna."))
    };
    let lines = |value: &serde_json::Value| -> Result<BTreeSet<usize>, SoplangError> {
        value
            .as_array()
            .ok_or_else(|| error("Warbixinta daboolku ma saxna."))?
            .iter()
            .map(&number)
            .collect()
    };
    let mut coverage = CoverageReport::default();
    for row in value["coverage"]
        .as_array()
        .ok_or_else(|| error("Warbixinta daboolku ma saxna."))?
    {
        let file = row["file"]
            .as_str()
            .ok_or_else(|| error("Jidka faylka daboolku ma saxna."))?;
        coverage
            .executable
            .insert(file.into(), lines(&row["executable"])?);
        coverage.hit.insert(file.into(), lines(&row["hit"])?);
    }
    Ok(Summary {
        passed: number(&value["passed"])?,
        failed: number(&value["failed"])?,
        coverage,
    })
}

struct Reports(PathBuf);

impl Reports {
    fn new() -> Result<Self, SoplangError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "soplang-tests-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(failure) if failure.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(failure) => return Err(file_write_error(&path, &failure)),
            }
        }
    }
}

impl Drop for Reports {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn parallel(
    suites: &[Suite],
    jobs: &[(usize, usize)],
    workers: usize,
    cover: bool,
) -> Result<Summary, SoplangError> {
    let exe = std::env::current_exe()
        .map_err(|failure| io_error("Soplang lama heli karin.", &failure))?;
    let reports = Reports::new()?;
    let next = AtomicUsize::new(0);
    // Each worker owns at most one child and drains both pipes. A launch error
    // cannot orphan previously started children; scoped workers finish first.
    let mut outputs = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..workers.min(jobs.len()) {
            handles.push(scope.spawn(|| {
                let mut outputs = Vec::new();
                loop {
                    let job = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(suite, index)) = jobs.get(job) else {
                        break;
                    };
                    let report = reports.0.join(format!("{job}.json"));
                    let mut command = Command::new(&exe);
                    command
                        .arg("test")
                        .arg(&suites[suite].path)
                        .arg("--lambarka")
                        .arg(index.to_string())
                        .arg("--warbixin")
                        .arg(&report);
                    if cover {
                        command.arg("--dabool");
                    }
                    outputs.push((job, report, command.output()));
                }
                outputs
            }));
        }
        let mut outputs = Vec::new();
        for handle in handles {
            outputs.extend(
                handle
                    .join()
                    .map_err(|_| error("Shaqaale tijaabo ayaa si lama filaan ah u joogsaday."))?,
            );
        }
        Ok::<_, SoplangError>(outputs)
    })?;
    outputs.sort_by_key(|(job, ..)| *job);
    let mut summary = Summary::default();
    for (_, report, output) in outputs {
        let output =
            output.map_err(|failure| io_error("Hawsha tijaabada lama bilaabi karin.", &failure))?;
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        if report.is_file() {
            let child = read_report(&report)?;
            if child.passed + child.failed != 1 || (child.failed == 0) != output.status.success() {
                return Err(error(
                    "Natiijada tijaabadu iyo warbixinteedu isma waafaqaan.",
                ));
            }
            summary.passed += child.passed;
            summary.failed += child.failed;
            summary.coverage.merge(&child.coverage);
        } else {
            summary.failed += 1;
        }
    }
    Ok(summary)
}
