//! Create a project without replacing existing source or manifest files.

use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{codes, io_reason, runtime_error_ex, ErrorMeta, SoplangError};
use clap::Parser;

pub const INIT_USAGE: &str = "soplang init [GALKA] [--lib] [--name MAGAC]";

#[derive(Parser)]
#[command(
    name = "soplang init",
    override_usage = INIT_USAGE,
    about = "Abuur xirmo Soplang ah oo leh fayl bilow ah.",
    disable_help_flag = true,
    help_template = "{about}\n\nAdeegsiga: {usage}\n\nGalka:\n{positionals}\n\nXulashooyinka:\n{options}"
)]
pub struct InitOptions {
    #[arg(
        default_value = ".",
        hide_default_value = true,
        value_name = "GALKA",
        help = "Galka xirmada; haddii aan la sheegin, isticmaal galka hadda."
    )]
    pub directory: PathBuf,
    #[arg(
        long = "lib",
        help = "Abuur maktabad leh lib.sop halkii laga abuuri lahaa main.sop."
    )]
    pub library: bool,
    #[arg(
        long,
        value_name = "MAGAC",
        help = "Magaca xirmada; haddii aan la sheegin, ka soo qaad magaca galka."
    )]
    pub name: Option<String>,
    #[arg(short = 'h', long, action = clap::ArgAction::Help, help = "Muuji caawimada.")]
    help: Option<bool>,
}

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E057_PACKAGE),
    )
}

pub fn initialize(options: &InitOptions) -> Result<String, SoplangError> {
    let directory = std::path::absolute(&options.directory).map_err(|failure| {
        error(format!(
            "Jidka xirmada lama aqoonin. {}",
            io_reason(&failure)
        ))
    })?;
    let name = options
        .name
        .clone()
        .unwrap_or_else(|| default_name(&directory));
    if !crate::package::is_package_name(&name) {
        return Err(error("Magaca xirmadu waa inuu ka bilaabmaa xaraf yar, kana koobnaadaa xarfo yaryar, tirooyin, '-' ama '_', ugu badnaan 64 xaraf."));
    }
    let starter = if options.library {
        "lib.sop"
    } else {
        "main.sop"
    };
    let manifest = directory.join(crate::package::MANIFEST_FILE);
    let source = directory.join(starter);
    for path in [&manifest, &source] {
        if fs::symlink_metadata(path).is_ok() {
            return Err(error(format!(
                "Faylka '{}' hore ayuu u jiraa; lama beddelin.",
                path.display()
            )));
        }
    }
    let existed = directory.exists();
    fs::create_dir_all(&directory).map_err(|failure| {
        error(format!(
            "Galka xirmada lama abuurin. {}",
            io_reason(&failure)
        ))
    })?;
    let manifest_text = format!(
        "magac = \"{name}\"\nnusxa = \"0.1.0\"\nsoplang = \"{}\"\n",
        env!("CARGO_PKG_VERSION")
    );
    let source_text = if options.library {
        "soosaar salaan\n\nhawl salaan(): qoraal {\n    celi \"Salaan, adduun!\"\n}\n"
    } else {
        "qor(\"Salaan, adduun!\")\n"
    };
    let mut created = Vec::new();
    let result = (|| {
        for (path, text) in [(&source, source_text), (&manifest, manifest_text.as_str())] {
            let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
            created.push(path);
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
        }
        Ok::<_, std::io::Error>(())
    })();
    if let Err(failure) = result {
        for path in created {
            let _ = fs::remove_file(path);
        }
        if !existed {
            let _ = fs::remove_dir(&directory);
        }
        return Err(error(format!(
            "Xirmada lama abuurin. {}",
            io_reason(&failure)
        )));
    }
    Ok(format!(
        "Xirmada '{name}' waa la abuuray: {}\nFaylka bilowga: {starter}",
        directory.display()
    ))
}

fn default_name(directory: &Path) -> String {
    let raw = directory
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("app");
    let mut name = String::new();
    for ch in raw.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
        } else if !name.ends_with('-') {
            name.push('-');
        }
    }
    let mut name = name.trim_matches('-').to_string();
    if name.is_empty() {
        return "app".into();
    }
    if !name.starts_with(|ch: char| ch.is_ascii_lowercase()) {
        name.insert_str(0, "app-");
    }
    name.truncate(64);
    name
}
