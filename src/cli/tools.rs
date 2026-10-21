//! `fmt`, `check`, `test`, `doc`, `debug`, `profile`, `run`, and `dhis`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::options::Options;
use super::testing::{materialize, split_tests};
use crate::data::parse_schema;
use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
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
    if args == ["--help"] || args == ["-h"] {
        if let Some(help) = command_help(command) {
            println!("{help}");
            return Ok(0);
        }
    }
    match command {
        "fmt" => format_file(&args),
        "xog" => super::data::dispatch(&args),
        "check" => check_file(&args),
        "test" => super::testing::run(&args),
        "doc" => document_file(&args),
        "debug" => super::debug::run(&args),
        "profile" => profile_file(&args),
        "run" => run_project(&args),
        "dhis" | "build" => build_project(&args),
        "add" | "remove" | "update" => package_command(command, &args),
        "lsp" if args.is_empty() => {
            super::lsp::serve()?;
            Ok(0)
        }
        _ => Err(tool_error(format!("Qalabka '{command}' lama aqoonsan."))),
    }
}

fn format_file(args: &[String]) -> Result<i32, SoplangError> {
    let options = Options::parse(args, &["--qor"], &[], false)?;
    let write = options.flag("--qor");
    let path = options.one(true, "soplang fmt FAYLKA [--qor]")?.unwrap();
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
    let options = Options::parse(args, &["--adag", "--strict", "--offline"], &[], false)?;
    let strict = options.flag("--adag") || options.flag("--strict");
    if options.flag("--offline") {
        std::env::set_var("SOPLANG_OFFLINE", "1");
    }
    let path = PathBuf::from(
        options
            .one(true, "soplang check FAYLKA [--adag] [--offline]")?
            .unwrap(),
    );
    let text = crate::analysis::overlay_loader(&Default::default(), &path).map_err(tool_error)?;
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
        &|imported| crate::analysis::overlay_loader(&Default::default(), imported),
        &|| false,
    );
    if let Some(error) = analysis.error {
        return Err(error);
    }
    let schema = path.parent().unwrap_or(Path::new(".")).join("schema.sop");
    if schema.is_file() {
        let text =
            crate::analysis::overlay_loader(&Default::default(), &schema).map_err(tool_error)?;
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
    let options = Options::parse(args, &["--gaar"], &["--qor"], false)?;
    let path = PathBuf::from(
        options
            .one(true, "soplang doc FAYLKA [--qor NATIIJO] [--gaar]")?
            .unwrap(),
    );
    let text = crate::analysis::overlay_loader(&Default::default(), &path).map_err(tool_error)?;
    let stmts = parse_source(&text, Some(&path))?;
    let output = super::document::render(&text, &path, &stmts, options.flag("--gaar"))?;
    if let Some(target) = options.value("--qor") {
        let target = Path::new(target);
        let target = if target.exists() || fs::symlink_metadata(target).is_ok() {
            fs::canonicalize(target)
                .map_err(|failure| crate::error::file_write_error(target, &failure))?
        } else {
            target.to_path_buf()
        };
        if fs::canonicalize(&path).ok().as_ref() == Some(&target) {
            return Err(tool_error("Dukumentigu ma beddeli karo faylka koodhka."));
        }
        let permissions = fs::metadata(&target).ok().map(|meta| meta.permissions());
        if permissions.as_ref().is_some_and(|mode| mode.readonly()) {
            return Err(tool_error("Faylka dukumentigu waa akhris-keliya."));
        }
        crate::runtime::replace_file_with_permissions(&target, output.as_bytes(), permissions)?;
    } else {
        print!("{output}");
    }
    Ok(0)
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
    let options = Options::parse(args, &["--adag", "--strict", "--offline"], &[], true)?;
    let dir = Path::new(
        options
            .one(false, "soplang run [GALKA] [--adag] [--offline] [-- DOODO]")?
            .unwrap_or("."),
    );
    if options.flag("--offline") {
        std::env::set_var("SOPLANG_OFFLINE", "1");
    }
    let entry = project_entry(dir)?;
    crate::runtime::set_program_arguments(options.trailing.clone());
    let text = fs::read_to_string(&entry)
        .map_err(|failure| crate::error::file_read_error(&entry, &failure))?;
    run_source(
        &text,
        Some(&entry),
        false,
        false,
        options.flag("--adag") || options.flag("--strict"),
    )?;
    Ok(0)
}

fn build_project(args: &[String]) -> Result<i32, SoplangError> {
    let options = Options::parse(
        args,
        &["--adag", "--strict", "--offline", "--quiet", "-q"],
        &["--qor", "--output", "-o", "--opt-level", "-O"],
        false,
    )?;
    let dir = Path::new(
        options
            .one(
                false,
                "soplang dhis [GALKA] [-o NATIIJO] [-O 0..3] [--adag] [--offline] [-q]",
            )?
            .unwrap_or("."),
    );
    let output_values: Vec<_> = ["--qor", "--output", "-o"]
        .into_iter()
        .filter_map(|key| options.value(key))
        .collect();
    let optimization: Vec<_> = ["--opt-level", "-O"]
        .into_iter()
        .filter_map(|key| options.value(key))
        .collect();
    if output_values.len() > 1 || optimization.len() > 1 {
        return Err(tool_error(
            "Jidka natiijada iyo heerka hagaajinta midkiiba hal mar sheeg.",
        ));
    }
    let opt = optimization
        .first()
        .map(|value| {
            value
                .parse::<u8>()
                .ok()
                .filter(|value| *value <= 3)
                .ok_or_else(|| tool_error("Heerka hagaajintu waa inuu u dhexaysaa 0 iyo 3."))
        })
        .transpose()?
        .unwrap_or(2);
    if options.flag("--offline") {
        std::env::set_var("SOPLANG_OFFLINE", "1");
    }
    let entry = project_entry(dir)?;
    let root = project_root(dir)?;
    let manifest = parse_manifest(
        &fs::read_to_string(root.join(MANIFEST_FILE)).map_err(|failure| {
            crate::error::file_read_error(&root.join(MANIFEST_FILE), &failure)
        })?,
    )?;
    let out = output_values.first().map(PathBuf::from).unwrap_or_else(|| {
        root.join("dhis")
            .join(format!("{}{}", manifest.name, std::env::consts::EXE_SUFFIX))
    });
    let source = fs::read_to_string(&entry)
        .map_err(|failure| crate::error::file_read_error(&entry, &failure))?;
    crate::build_source(
        &source,
        Some(&entry),
        &out,
        opt,
        options.flag("--adag") || options.flag("--strict"),
    )?;
    if !options.flag("--quiet") && !options.flag("-q") {
        println!("Barnaamijka waa la dhisay: {}", out.display());
    }
    Ok(0)
}

fn package_command(command: &str, args: &[String]) -> Result<i32, SoplangError> {
    let options = Options::parse(args, &["--offline"], &[], false)?;
    let required = match command {
        "add" => 2,
        "remove" => 1,
        _ => 0,
    };
    if options.positional.len() < required || options.positional.len() > required + 1 {
        return Err(tool_error(command_help(command).unwrap()));
    }
    if options.flag("--offline") {
        std::env::set_var("SOPLANG_OFFLINE", "1");
    }
    let root = project_root(Path::new(
        options
            .positional
            .get(required)
            .map(String::as_str)
            .unwrap_or("."),
    ))?;
    let message = match command {
        "add" => {
            crate::package::add_dependency(&root, &options.positional[0], &options.positional[1])?
        }
        "remove" => crate::package::remove_dependency(&root, &options.positional[0])?,
        _ => crate::package::update_dependencies(&root)?,
    };
    println!("{message}");
    Ok(0)
}

fn command_help(command: &str) -> Option<&'static str> {
    Some(match command {
        "fmt"=>"Adeegsiga: soplang fmt FAYLKA [--qor]\nHabee koodhka; --qor waxay si ammaan ah u beddeshaa faylka.",
        "check"=>"Adeegsiga: soplang check FAYLKA [--adag] [--offline]\nHubi koodhka iyo schema.sop adigoon fulin.",
        "doc"=>"Adeegsiga: soplang doc FAYLKA [--qor NATIIJO] [--gaar]\nSoo saar dukumenti Markdown ah oo leh noocyada, doodaha iyo faallooyinka ///. --gaar waxay ku dartaa magacyada gaarka ah.",
        "run"=>"Adeegsiga: soplang run [GALKA] [--adag] [--offline] [-- DOODO]\nFuli main.sop ama lib.sop ee gelidda xirmada.",
        "dhis"|"build"=>"Adeegsiga: soplang dhis [GALKA] [-o NATIIJO] [-O 0..3] [--adag] [--offline] [-q]\nDhis barnaamij LLVM ah; haddii aan jid la sheegin, qor dhis/MAGAC.",
        "profile"=>"Adeegsiga: soplang profile FAYLKA\nCabbir wicitaannada iyo wakhtiga hawlaha.",
        "add"=>"Adeegsiga: soplang add MAGAC IL [GALKA] [--offline]\nKu dar ku-tiirsi Git ah ama il kale oo la taageero, dabadeed cusbooneysii qufulka.",
        "remove"=>"Adeegsiga: soplang remove MAGAC [GALKA] [--offline]\nKa saar ku-tiirsiga, dabadeed cusbooneysii qufulka.",
        "update"=>"Adeegsiga: soplang update [GALKA] [--offline]\nDib u xalli ku-tiirsiyada oo cusbooneysii qufulka.",
        "lsp"=>"Adeegsiga: soplang lsp\nKu wada xidhiidh hab-maamuuska adeegaha luuqadda adigoo adeegsanaya gelinta iyo soo-saarka caadiga ah.",
        _=>return None,
    })
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
    let start = fs::canonicalize(start)
        .map_err(|failure| crate::error::file_read_error(start, &failure))?;
    let start = start.as_path();
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

pub(super) fn tool_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}
