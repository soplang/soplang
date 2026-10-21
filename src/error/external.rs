//! Contextual Somali explanations, with original external errors kept for diagnosis.

use std::io;
use std::path::Path;

use super::{runtime_error_ex, ErrorMeta, SoplangError};

pub fn external_error(message: impl Into<String>, details: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default().with_details(details))
}

/// Explain the error kind rather than attempting to translate its Display text.
pub fn io_reason(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::NotFound => "Faylka ama galka lama helin.",
        io::ErrorKind::PermissionDenied => "Falkan lama oggola.",
        io::ErrorKind::AlreadyExists => "Fayl ama gal ayaa hore ugu jira meeshaas.",
        io::ErrorKind::InvalidInput => "Xogta la bixiyey ma saxna.",
        io::ErrorKind::InvalidData => "Xogta la akhriyey ma saxna.",
        io::ErrorKind::IsADirectory => "Meesha la tilmaamay waa gal, mana aha fayl.",
        io::ErrorKind::NotADirectory => "Qayb ka mid ah jidka la tilmaamay ma aha gal.",
        io::ErrorKind::StorageFull => "Meesha wax lagu kaydiyo way buuxdaa.",
        io::ErrorKind::ReadOnlyFilesystem => {
            "Meesha wax lagu kaydiyo waxa keliya oo laga oggol yahay akhris."
        }
        io::ErrorKind::BrokenPipe => "Xidhiidhkii wax lagu qorayey wuu xidhmay.",
        io::ErrorKind::Interrupted => "Hawshii waa la hakiyey.",
        io::ErrorKind::TimedOut => "Waqtigii hawsha loo qabtay wuu dhammaaday.",
        io::ErrorKind::WriteZero => "Wax xog ah lama qori karin.",
        io::ErrorKind::UnexpectedEof => "Xogtii ayaa dhammaatay ka hor intii la filayey.",
        io::ErrorKind::OutOfMemory => "Xusuusta kombiyuutarka kuma filna hawshan.",
        io::ErrorKind::Unsupported => "Nidaamkani ma taageero hawshan.",
        _ => "Sababta khaladka lama aqoonsan.",
    }
}

fn with_os_code(mut message: String, error: &io::Error) -> String {
    if let Some(code) = error.raw_os_error() {
        message.push_str(&format!(" (Koodhka nidaamka: {code})"));
    }
    message
}

pub fn io_error(message: impl Into<String>, error: &io::Error) -> SoplangError {
    let message = format!("{} {}", message.into(), io_reason(error));
    external_error(with_os_code(message, error), error.to_string())
}

pub fn file_read_error(path: &Path, error: &io::Error) -> SoplangError {
    let path = path.display();
    let message = match error.kind() {
        io::ErrorKind::NotFound => format!("Faylka '{path}' lama helin."),
        io::ErrorKind::PermissionDenied => format!("Lama oggola in la akhriyo faylka '{path}'."),
        io::ErrorKind::IsADirectory => format!("'{path}' waa gal; waxaa loo baahan yahay fayl."),
        io::ErrorKind::InvalidData => format!("Faylka '{path}' ma aha qoraal UTF-8 ah oo sax ah."),
        _ => format!("Faylka '{path}' lama akhrin karin. {}", io_reason(error)),
    };
    external_error(message, error.to_string())
}

pub fn path_io_error(path: &Path, error: &io::Error, action: &str) -> SoplangError {
    let path = path.display();
    let message = format!("{action} '{path}' ma guuleysan. {}", io_reason(error));
    external_error(with_os_code(message, error), error.to_string())
}

pub fn file_write_error(path: &Path, error: &io::Error) -> SoplangError {
    let path = path.display();
    let message = match error.kind() {
        io::ErrorKind::NotFound => format!("Faylka '{path}' lama qori karin. {}", io_reason(error)),
        io::ErrorKind::PermissionDenied => format!("Lama oggola in la qoro faylka '{path}'."),
        io::ErrorKind::IsADirectory => format!("'{path}' waa gal; waxaa loo baahan yahay fayl."),
        _ => format!("Faylka '{path}' lama qori karin. {}", io_reason(error)),
    };
    external_error(with_os_code(message, error), error.to_string())
}

pub fn input_error(error: &io::Error) -> SoplangError {
    if error.kind() == io::ErrorKind::InvalidData {
        external_error(
            "Xogta la geliyey ma aha qoraal UTF-8 ah oo sax ah.",
            error.to_string(),
        )
    } else {
        io_error("Xogta la gelinayo lama akhrin karin.", error)
    }
}

pub fn tool_start_error(tool: &str, error: &io::Error) -> SoplangError {
    let hint = if error.kind() == io::ErrorKind::NotFound {
        format!(" Rakib {tool}, dabadeed hubi in laga heli karo PATH.")
    } else {
        String::new()
    };
    io_error(format!("{tool} lama bilaabi karin.{hint}"), error)
}
