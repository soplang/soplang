//! Generate typed clients without opening a database or executing source.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(crate::error::codes::E059_DATA),
    )
}

pub(super) fn dispatch(args: &[String]) -> Result<i32, SoplangError> {
    if args == ["--help"]
        || args == ["-h"]
        || args == ["abuur", "--help"]
        || args == ["abuur", "-h"]
    {
        println!("Adeegsiga: soplang xog abuur [SCHEMA] [--qor MACMIILKA | --hubi MACMIILKA]\nSCHEMA haddii aan la sheegin waa schema.sop. Xog kaydsan lama beddelayo.\n--qor waxay abuureysaa ama cusbooneysiineysaa macmiil la dhalinay; fayl kale ma beddelayso.\n--hubi waxay xaqiijineysaa in macmiilku waafaqsan yahay schema, iyada oo aan waxba qorin.");
        return Ok(0);
    }
    if args.first().map(String::as_str) != Some("abuur") {
        return Err(error(
            "Adeegsiga: soplang xog abuur [SCHEMA] [--qor MACMIILKA | --hubi MACMIILKA]",
        ));
    }
    let mut schema = None;
    let mut output = None;
    let mut check = None;
    let mut args = args[1..].iter();
    while let Some(arg) = args.next() {
        if matches!(arg.as_str(), "--qor" | "--hubi") {
            if output.is_some() || check.is_some() {
                return Err(error(
                    "Hal doorasho oo --qor ama --hubi ah ayaa la bixin karaa.",
                ));
            }
            let path = Some(PathBuf::from(
                args.next()
                    .filter(|path| !path.starts_with('-'))
                    .ok_or_else(|| error(format!("{arg} waxay u baahan tahay jidka macmiilka.")))?,
            ));
            if arg == "--hubi" {
                check = path;
            } else {
                output = path;
            }
        } else if arg.starts_with('-') {
            return Err(error(format!("Doorashada '{arg}' lama aqoonsan.")));
        } else if schema.replace(PathBuf::from(arg)).is_some() {
            return Err(error("Hal schema oo keliya ayaa la bixin karaa."));
        }
    }
    let schema_path = schema.unwrap_or_else(|| "schema.sop".into());
    let source =
        crate::analysis::overlay_loader(&Default::default(), &schema_path).map_err(error)?;
    let schema = crate::data::parse_schema_file(&source, &schema_path)?;
    let generated = crate::data::generate_client(&schema).map_err(error)?;
    if let Some(path) = check {
        let existing =
            crate::analysis::overlay_loader(&Default::default(), &path).map_err(error)?;
        if existing != generated {
            return Err(error(
                "Macmiilka xogtu ma waafaqsana schema; dib u abuur ka hor dhismaha.",
            ));
        }
        println!("Macmiilka xogtu wuxuu waafaqsan yahay schema.");
    } else if let Some(path) = output {
        write_client(&path, &generated)?;
        println!("Macmiilka xogta waa la qoray: {}", path.display());
    } else {
        use std::io::Write;
        std::io::stdout()
            .lock()
            .write_all(generated.as_bytes())
            .map_err(|failure| {
                crate::error::io_error("Macmiilka xogta lama qori karin.", &failure)
            })?;
    }
    Ok(0)
}

fn write_client(path: &Path, source: &str) -> Result<(), SoplangError> {
    let (target, permissions) = if path.exists() || fs::symlink_metadata(path).is_ok() {
        let target = fs::canonicalize(path)
            .map_err(|failure| crate::error::file_write_error(path, &failure))?;
        let existing =
            crate::analysis::overlay_loader(&Default::default(), &target).map_err(error)?;
        if !existing.starts_with(crate::data::GENERATED_HEADER) {
            return Err(error(
                "Faylku ma aha macmiil la dhalinay; dooro jid kale si qoraalkii hore loo ilaaliyo.",
            ));
        }
        let permissions = fs::metadata(&target)
            .map_err(|failure| crate::error::file_write_error(&target, &failure))?
            .permissions();
        if permissions.readonly() {
            return Err(error(
                "Macmiilkani waa akhris-keliya; lama cusbooneysiin karo.",
            ));
        }
        (target, Some(permissions))
    } else {
        (path.to_path_buf(), None)
    };
    crate::runtime::replace_file_with_permissions(&target, source.as_bytes(), permissions)
}
