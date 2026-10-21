//! Compile explicit source debugging instrumentation in either backend.
use std::path::PathBuf;

use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
use crate::frontend::{imports::resolve_imports, parse_source};
use crate::hir::{HirInstr, HirLowering};
use crate::runtime::{parse_breakpoint, DebugConfig};

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}

pub(super) fn run(args: &[String]) -> Result<i32, SoplangError> {
    if args == ["--help"] || args == ["-h"] {
        println!("Adeegsiga: soplang debug FAYL [--jebi [FAYL:]SAF] [--soco] [--adag] [--dhis NATIIJO]\nWaxay joogsanaysaa ka hor weedha koowaad; --soco waxay sugaysaa barta joogsiga.\n--dhis waxay dhistaa barnaamij LLVM ah oo leh isla baarista, mana fulinayso.\nAmarro: soco, tallaabo, xiga, bax, raad, jir N, deegaan, eeg MAGAC, jebi FAYL:SAF, tir FAYL:SAF, jooji.");
        return Ok(0);
    }
    let mut path = None;
    let mut breaks = Vec::new();
    let mut config = DebugConfig::default();
    let mut strict = false;
    let mut build = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--jebi" | "--dhis" => {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with('-'))
                    .ok_or_else(|| error(format!("{arg} waxay u baahan tahay qiime.")))?;
                if arg == "--jebi" {
                    if breaks.len() == 128 {
                        return Err(error(
                            "Ugu badnaan 128 barood oo joogsi ah ayaa la bixin karaa.",
                        ));
                    }
                    breaks.push(value);
                } else if build.replace(PathBuf::from(value)).is_some() {
                    return Err(error("Hal jid oo dhisme ah ayaa la bixin karaa."));
                }
            }
            "--soco" => config.continue_at_start = true,
            "--adag" => strict = true,
            _ if arg.starts_with('-') => {
                return Err(error(format!("Doorashada '{arg}' lama aqoonsan.")))
            }
            _ if path.is_none() => path = Some(PathBuf::from(arg)),
            _ => return Err(error("Hal fayl oo keliya ayaa la baari karaa.")),
        }
    }
    let path = path.ok_or_else(|| {
        error(
            "Adeegsiga: soplang debug FAYL [--jebi [FAYL:]SAF] [--soco] [--adag] [--dhis NATIIJO]",
        )
    })?;
    for target in breaks {
        config
            .breakpoints
            .insert(parse_breakpoint(target, Some(&path))?);
    }
    crate::package::load_beside_source(&path)?;
    let source = std::fs::read_to_string(&path)
        .map_err(|failure| crate::error::file_read_error(&path, &failure))?;
    let statements = resolve_imports(parse_source(&source, Some(&path))?, Some(&path))?;
    let symbols = crate::semantic::analyze_with_options(
        &statements,
        crate::semantic::AnalyzeOptions {
            strict,
            repl: false,
        },
    )?;
    let hir = HirLowering::lower_with_debug(&symbols, &statements, config.clone());
    let mut locations = std::collections::BTreeSet::new();
    for instruction in hir
        .top_level
        .iter()
        .chain(hir.functions.iter().flat_map(|function| &function.body))
    {
        if let HirInstr::DebugPoint(metadata) = instruction {
            let location: crate::runtime::debugger::Location =
                serde_json::from_str(metadata).expect("generated debug location");
            let file = std::fs::canonicalize(&location.file).unwrap_or(location.file);
            locations.insert((file, location.line));
        }
    }
    for target in &config.breakpoints {
        if !locations.contains(target) {
            return Err(error(format!(
                "Barta joogsiga {}:{} ma leh weedh la fulin karo.",
                target.0.display(),
                target.1
            )));
        }
    }
    if let Some(output) = build {
        crate::backend::llvm::LlvmBackend::new().build_executable(&hir, &symbols, &output, 0)?;
    } else {
        let mut backend = crate::backend::cranelift::CraneliftBackend::new()?;
        backend.compile_module(&hir, &symbols)?;
        backend.run_main()?;
    }
    Ok(0)
}
