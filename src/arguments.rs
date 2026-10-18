//! Somali rendering of Clap's structured parse failures.

use clap::error::{ContextKind, ContextValue, ErrorKind};

pub const USAGE: &str = "soplang [XULASHOOYINKA] [FAYLKA]";

pub fn format_argument_error(error: &clap::Error) -> String {
    let context = |kind| match error.get(kind) {
        Some(ContextValue::String(value)) => Some(value.as_str()),
        _ => None,
    };
    let argument = context(ContextKind::InvalidArg).unwrap_or("?");
    let value = context(ContextKind::InvalidValue).unwrap_or("?");
    let message = match error.kind() {
        ErrorKind::UnknownArgument => format!("Xulashada ama doodda '{argument}' lama aqoonsan."),
        ErrorKind::InvalidValue if value.is_empty() => {
            format!("Xulashada '{argument}' waxay u baahan tahay qiime.")
        }
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => {
            format!("Qiimaha '{value}' ee '{argument}' ma saxna. Hubi nooca iyo xadka qiimaha la oggol yahay.")
        }
        ErrorKind::ArgumentConflict => {
            format!("Xulashada '{argument}' waa la soo celiyey ama waxay ka hor imanaysaa xulasho kale.")
        }
        ErrorKind::MissingRequiredArgument => "Dood loo baahan yahay ayaa maqan.".to_owned(),
        ErrorKind::TooFewValues | ErrorKind::WrongNumberOfValues => {
            format!("Xulashada '{argument}' ma helin tiradii qiimayaasha loo baahnaa.")
        }
        ErrorKind::TooManyValues => {
            format!("Xulashada '{argument}' waxaa la siiyey qiimayaal ka badan intii loo baahnaa.")
        }
        ErrorKind::InvalidUtf8 => "Doodaha amarku ma aha qoraal UTF-8 ah oo sax ah.".to_owned(),
        ErrorKind::NoEquals => format!("Xulashada '{argument}' iyo qiimaheeda ku kala xidh '='."),
        _ => "Doodaha amarku ma saxna.".to_owned(),
    };
    let suggestion = context(ContextKind::SuggestedArg)
        .map(|name| format!("\nMa waxaad ula jeedday '{name}'?\n"))
        .unwrap_or_default();
    format!("Khalad: {message}\n{suggestion}\nAdeegsiga: {USAGE}\n\nAdeegso --help si aad u aragto caawimada.\n")
}
