//! `fmt`, `check`, `test`, `doc`, `debug`, `profile`, `run`, and `dhis`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::data::parse_schema;
use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::Stmt;
use crate::frontend::imports::resolve_imports;
use crate::frontend::parse_source;
use crate::package::{parse_manifest, MANIFEST_FILE};
use crate::{run_source, run_statements};

pub fn dispatch(command: &str) -> Result<i32, SoplangError> {
    let args: Vec<String> = std::env::args().skip(2).collect();
    match command {
        "fmt" => format_file(&args),
        "check" => check_file(&args),
        "test" => test_file(&args),
        "doc" => document_file(&args),
        "debug" => {
            let path = required_file(&args, "soplang debug FAYLKA")?;
            run_path(&path, false)?;
            Ok(0)
        }
        "profile" => profile_file(&args),
        "run" => run_project(&args),
        "dhis" => build_project(&args),
        _ => Err(tool_error(format!("Qalabka '{command}' lama aqoonsan."))),
    }
}

fn format_file(args: &[String]) -> Result<i32, SoplangError> {
    let write = flag(args, "--qor");
    let path =
        positional(args, &[]).ok_or_else(|| tool_error("Adeegsiga: soplang fmt FAYLKA [--qor]"))?;
    let target = fs::canonicalize(&path)
        .map_err(|error| crate::error::file_read_error(Path::new(&path), &error))?;
    let text = fs::read_to_string(&target)
        .map_err(|error| crate::error::file_read_error(Path::new(&path), &error))?;
    let formatted = format_source(&text, Path::new(&path))?;
    if write {
        let permissions = fs::metadata(&target)
            .map_err(|error| crate::error::file_write_error(&target, &error))?
            .permissions();
        if permissions.readonly() {
            return Err(tool_error(
                "Faylkan waa akhris-keliya; qaabkiisa lama qori karo.",
            ));
        }
        crate::runtime::replace_file_with_permissions(
            &target,
            formatted.as_bytes(),
            Some(permissions),
        )?;
    } else {
        print!("{formatted}");
        let _ = std::io::stdout().flush();
    }
    Ok(0)
}

pub fn format_source(text: &str, path: &Path) -> Result<String, SoplangError> {
    crate::frontend::formatter::format_source(text, path)
}

fn check_file(args: &[String]) -> Result<i32, SoplangError> {
    let strict = flag(args, "--adag");
    let path = positional(args, &[])
        .ok_or_else(|| tool_error("Adeegsiga: soplang check FAYLKA [--adag]"))?;
    let path = PathBuf::from(path);
    let text = fs::read_to_string(&path).map_err(|error| tool_error(error.to_string()))?;
    if path.file_name().and_then(|name| name.to_str()) == Some("schema.sop") {
        parse_schema(&text).map_err(tool_error)?;
        println!("schema.sop waa sax.");
        return Ok(0);
    }
    let analysis = crate::analysis::check_source(
        &text,
        &path,
        strict,
        crate::analysis::Limits::default(),
        &|imported| fs::read_to_string(imported).map_err(|error| error.to_string()),
        &|| false,
    );
    if let Some(error) = analysis.error {
        return Err(error);
    }
    let schema = path.parent().unwrap_or(Path::new(".")).join("schema.sop");
    if schema.is_file() {
        let text = fs::read_to_string(&schema).map_err(|error| tool_error(error.to_string()))?;
        parse_schema(&text).map_err(tool_error)?;
    }
    let stmts = analysis.statements;
    let (prelude, setups, cleanups, tests) = split_tests(&stmts);
    for (index, _) in tests.iter().enumerate() {
        let program = materialize(&prelude, &setups, &cleanups, &tests, index);
        crate::semantic::analyze_with_options(
            &program,
            crate::semantic::AnalyzeOptions {
                strict,
                repl: false,
            },
        )?;
    }
    println!("{} waa sax.", path.display());
    Ok(0)
}

fn test_file(args: &[String]) -> Result<i32, SoplangError> {
    let filter = option(args, "--shaandhe");
    let index = option(args, "--lambarka").and_then(|value| value.parse::<usize>().ok());
    let parallel = flag(args, "--wada");
    let cover = flag(args, "--dabool");
    let path = positional_skip(args, &["--shaandhe", "--lambarka"]).ok_or_else(|| {
        tool_error("Adeegsiga: soplang test FAYLKA [--shaandhe MAGAC] [--wada] [--dabool]")
    })?;
    let path = PathBuf::from(path);
    let text = fs::read_to_string(&path).map_err(|error| tool_error(error.to_string()))?;
    let stmts = resolve_imports(parse_source(&text, Some(&path))?, Some(&path))?;
    let (prelude, setups, cleanups, tests) = split_tests(&stmts);
    let mut selected: Vec<usize> = tests
        .iter()
        .enumerate()
        .filter(|(_, test)| {
            filter
                .as_ref()
                .is_none_or(|filter| test.name.contains(filter))
        })
        .map(|(index, _)| index)
        .collect();
    if let Some(index) = index {
        if index >= tests.len() {
            return Err(tool_error("Lambarka tijaabada ma jiro."));
        }
        selected = vec![index];
    }
    if selected.is_empty() {
        println!("Tijaabo lama helin.");
        return Ok(0);
    }
    if parallel && selected.len() > 1 && index.is_none() {
        return run_parallel(&path, &selected, cover);
    }
    if cover {
        std::env::set_var("SOPLANG_DABOOL", "1");
    }
    let total = selected.len();
    let mut failed = 0usize;
    let mut covered = Vec::new();
    for offset in selected {
        let program = materialize(&prelude, &setups, &cleanups, &tests, offset);
        if cover {
            crate::runtime::clear_coverage();
        }
        let result = run_statements(&program, false);
        if cover {
            covered.extend(crate::runtime::coverage_lines());
        }
        let test = &tests[offset];
        let passed = match (&result, test.expect_error) {
            (Ok(()), false) | (Err(_), true) => true,
            (Ok(()), true) => false,
            (Err(_), false) => false,
        };
        if passed {
            println!("waa: {}", test.name);
        } else {
            failed += 1;
            let detail = match result {
                Err(error) => error.message().to_string(),
                Ok(()) => "khalad ayaa la filayay.".into(),
            };
            println!("fashil: {}: {detail}", test.name);
        }
    }
    if cover {
        covered.sort_unstable();
        covered.dedup();
        for line in &covered {
            eprintln!("dabool {line}");
        }
        println!(
            "dabool: {}",
            covered
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("{} waa, {} fashil.", total - failed, failed);
    Ok(i32::from(failed > 0))
}

fn run_parallel(path: &Path, selected: &[usize], cover: bool) -> Result<i32, SoplangError> {
    let exe = std::env::current_exe().map_err(|error| tool_error(error.to_string()))?;
    let mut children = Vec::new();
    for index in selected {
        let mut command = Command::new(&exe);
        command
            .arg("test")
            .arg(path)
            .arg("--lambarka")
            .arg(index.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if cover {
            command.arg("--dabool").env("SOPLANG_DABOOL", "1");
        }
        children.push(
            command
                .spawn()
                .map_err(|error| tool_error(error.to_string()))?,
        );
    }
    let total = children.len();
    let mut failed = 0;
    let mut covered = Vec::new();
    for child in children {
        let output = child
            .wait_with_output()
            .map_err(|error| tool_error(error.to_string()))?;
        print!("{}", String::from_utf8_lossy(&output.stdout));
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            if let Some(number) = line.strip_prefix("dabool ") {
                if let Ok(number) = number.parse::<usize>() {
                    covered.push(number);
                }
            } else if !line.is_empty() {
                eprint!("{line}\n");
            }
        }
        if !output.status.success() {
            failed += 1;
        }
    }
    if cover {
        covered.sort_unstable();
        covered.dedup();
        println!(
            "dabool: {}",
            covered
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("{} waa, {} fashil.", total - failed, failed);
    Ok(i32::from(failed > 0))
}

fn document_file(args: &[String]) -> Result<i32, SoplangError> {
    let write = option(args, "--qor");
    let path = positional_skip(args, &["--qor"])
        .ok_or_else(|| tool_error("Adeegsiga: soplang doc FAYLKA [--qor FAYLKA]"))?;
    let path = PathBuf::from(path);
    let text = fs::read_to_string(&path).map_err(|error| tool_error(error.to_string()))?;
    let stmts = parse_source(&text, Some(&path))?;
    let mut out = format!("# {}\n\n", path.display());
    for stmt in &stmts {
        document_stmt(stmt, &mut out, 2);
    }
    if let Some(target) = write {
        fs::write(target, out).map_err(|error| tool_error(error.to_string()))?;
    } else {
        print!("{out}");
    }
    Ok(0)
}

fn document_stmt(stmt: &Stmt, out: &mut String, level: usize) {
    let heading = "#".repeat(level);
    match stmt.unlocated() {
        Stmt::FuncDef { name, params, .. } => {
            let names: Vec<&str> = params.iter().map(|param| param.name.as_str()).collect();
            out.push_str(&format!(
                "{heading} hawl {name}\n\nDoodo: {}\n\n",
                names.join(", ")
            ));
        }
        Stmt::ClassDef { name, parent, body } => {
            out.push_str(&format!("{heading} qaab {name}"));
            if let Some(parent) = parent {
                out.push_str(&format!(" dhaxal {parent}"));
            }
            out.push_str("\n\n");
            for stmt in body {
                document_stmt(stmt, out, level + 1);
            }
        }
        Stmt::Test {
            name, expect_error, ..
        } => {
            out.push_str(&format!(
                "{heading} tijaabo {name}{}\n\n",
                if *expect_error { " qalad" } else { "" }
            ));
        }
        Stmt::Export(names) => out.push_str(&format!("{heading} soosaar {}\n\n", names.join(", "))),
        Stmt::Located { stmt, .. } => document_stmt(stmt, out, level),
        _ => {}
    }
}

fn profile_file(args: &[String]) -> Result<i32, SoplangError> {
    let path = required_file(args, "soplang profile FAYLKA")?;
    crate::runtime::set_profile(true);
    let started = Instant::now();
    let result = run_path(&path, false);
    let elapsed = started.elapsed().as_secs_f64();
    for (name, count) in crate::runtime::profile_counts() {
        println!("{count}\t{name}");
    }
    println!("waqti\t{elapsed}");
    result?;
    Ok(0)
}

fn run_project(args: &[String]) -> Result<i32, SoplangError> {
    let dir = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let entry = project_entry(&dir)?;
    run_path(&entry, false)?;
    Ok(0)
}

fn build_project(args: &[String]) -> Result<i32, SoplangError> {
    let dir = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let entry = project_entry(&dir)?;
    let root = project_root(&dir)?;
    let manifest = parse_manifest(
        &fs::read_to_string(root.join(MANIFEST_FILE))
            .map_err(|error| tool_error(error.to_string()))?,
    )?;
    let out_dir = root.join("dhis");
    fs::create_dir_all(&out_dir).map_err(|error| tool_error(error.to_string()))?;
    let out = out_dir.join(&manifest.name);
    let source = fs::read_to_string(&entry).map_err(|error| tool_error(error.to_string()))?;
    crate::build_source(&source, Some(&entry), &out, 2, false)?;
    println!("{}", out.display());
    Ok(0)
}

fn run_path(path: &Path, strict: bool) -> Result<(), SoplangError> {
    let text = fs::read_to_string(path).map_err(|error| tool_error(error.to_string()))?;
    run_source(&text, Some(path), false, false, strict)
}

pub fn project_entry(start: &Path) -> Result<PathBuf, SoplangError> {
    let root = project_root(start)?;
    let manifest = parse_manifest(
        &fs::read_to_string(root.join(MANIFEST_FILE))
            .map_err(|error| tool_error(error.to_string()))?,
    )?;
    let source = root.join(manifest.source);
    for name in ["main.sop", "lib.sop"] {
        let path = source.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(tool_error(
        "Xirmadu ma leh main.sop ama lib.sop ee gelidda.",
    ))
}

fn project_root(start: &Path) -> Result<PathBuf, SoplangError> {
    let mut dir = if start.is_file() {
        start.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if dir.join(MANIFEST_FILE).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(tool_error("soplang.toml lama helin."))
}

#[derive(Clone)]
struct TestCase {
    name: String,
    expect_error: bool,
    body: Vec<Stmt>,
}

fn split_tests(stmts: &[Stmt]) -> (Vec<Stmt>, Vec<Vec<Stmt>>, Vec<Vec<Stmt>>, Vec<TestCase>) {
    let mut prelude = Vec::new();
    let mut setups = Vec::new();
    let mut cleanups = Vec::new();
    let mut tests = Vec::new();
    for stmt in stmts {
        match stmt.unlocated() {
            Stmt::Test {
                name,
                expect_error,
                body,
            } => tests.push(TestCase {
                name: name.clone(),
                expect_error: *expect_error,
                body: body.clone(),
            }),
            Stmt::Fixture { setup, body } => {
                if *setup {
                    setups.push(body.clone());
                } else {
                    cleanups.push(body.clone());
                }
            }
            _ => prelude.push(stmt.clone()),
        }
    }
    (prelude, setups, cleanups, tests)
}

fn materialize(
    prelude: &[Stmt],
    setups: &[Vec<Stmt>],
    cleanups: &[Vec<Stmt>],
    tests: &[TestCase],
    index: usize,
) -> Vec<Stmt> {
    let mut program = prelude.to_vec();
    let mut body = Vec::new();
    for setup in setups {
        body.extend(setup.clone());
    }
    body.extend(tests[index].body.clone());
    let cleanup: Vec<Stmt> = cleanups.iter().flatten().cloned().collect();
    if cleanup.is_empty() {
        program.extend(body);
    } else {
        program.push(Stmt::TryCatch {
            try_body: body,
            err_var: Some("_khalad_tijaabo".into()),
            err_details: None,
            catch_body: vec![Stmt::Throw(crate::frontend::Expr::Identifier(
                "_khalad_tijaabo".into(),
            ))],
            finally_body: cleanup,
        });
    }
    program
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1).cloned())
}

fn positional(args: &[String], flags: &[&str]) -> Option<String> {
    positional_skip(args, flags)
}

fn positional_skip(args: &[String], valued: &[&str]) -> Option<String> {
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if valued.contains(&arg.as_str()) {
            skip = true;
            continue;
        }
        if !arg.starts_with('-') {
            return Some(arg.clone());
        }
    }
    None
}

fn required_file(args: &[String], usage: &str) -> Result<PathBuf, SoplangError> {
    positional(args, &[])
        .map(PathBuf::from)
        .ok_or_else(|| tool_error(format!("Adeegsiga: {usage}")))
}

fn tool_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}
