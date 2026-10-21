//! `fmt`, `check`, `test`, `doc`, `debug`, `profile`, `run`, and `dhis`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::testing::{materialize, split_tests};
use crate::data::parse_schema;
use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::Stmt;
use crate::frontend::imports::resolve_imports;
use crate::frontend::parse_source;
use crate::package::{parse_manifest, MANIFEST_FILE};
use crate::run_source;

pub fn dispatch(command: &str) -> Result<i32, SoplangError> {
    let args: Vec<String> = std::env::args_os()
        .skip(2)
        .map(|arg| {
            arg.into_string()
                .map_err(|_| tool_error("Doodaha qalabku waa inay noqdaan qoraal UTF-8 ah."))
        })
        .collect::<Result<_, _>>()?;
    match command {
        "fmt" => format_file(&args),
        "xog" => super::data::dispatch(&args),
        "check" => check_file(&args),
        "test" => super::testing::run(&args),
        "doc" => document_file(&args),
        "debug" => super::debug::run(&args),
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
    if args.len() != 1 || args[0].starts_with('-') {
        return Err(tool_error("Adeegsiga: soplang profile FAYLKA"));
    }
    let path = PathBuf::from(&args[0]);
    crate::package::load_beside_source(&path)?;
    let text =
        fs::read_to_string(&path).map_err(|error| crate::error::file_read_error(&path, &error))?;
    let stmts = resolve_imports(parse_source(&text, Some(&path))?, Some(&path))?;
    let outcome = crate::run_statements_report(
        &stmts,
        false,
        crate::ExecutionOptions {
            profile: true,
            coverage: false,
        },
    )?;
    println!("wicitaanno\tguud_ilbiriqsi\tgaar_ilbiriqsi\thawl");
    for (name, row) in &outcome.report.profile {
        println!(
            "{}\t{:.9}\t{:.9}\t{}",
            row.calls,
            row.inclusive.as_secs_f64(),
            row.exclusive.as_secs_f64(),
            name
        );
    }
    println!("ururin\t{:.9}", outcome.report.compile_time.as_secs_f64());
    println!("waqti\t{:.9}", outcome.report.execution_time.as_secs_f64());
    outcome.error.map_or(Ok(0), Err)
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

fn tool_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}
