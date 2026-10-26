//! Small command option parser. Unknown, repeated and incomplete options fail.
use crate::error::SoplangError;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Options {
    pub positional: Vec<String>,
    pub trailing: Vec<String>,
    flags: HashSet<String>,
    values: HashMap<String, String>,
}
impl Options {
    pub fn parse(
        args: &[String],
        flags: &[&str],
        values: &[&str],
        trailing: bool,
    ) -> Result<Self, SoplangError> {
        let mut result = Self::default();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            if arg == "--" {
                if trailing {
                    result.trailing.extend(args.cloned());
                } else {
                    result.positional.extend(args.cloned());
                }
                break;
            }
            if flags.contains(&arg.as_str()) {
                if !result.flags.insert(arg.clone()) {
                    return Err(super::tools::tool_error(format!(
                        "Doorashada '{arg}' waa la soo celiyey."
                    )));
                }
            } else if values.contains(&arg.as_str()) {
                let value = args
                    .next()
                    .filter(|value| !value.starts_with('-'))
                    .ok_or_else(|| {
                        super::tools::tool_error(format!(
                            "Doorashada '{arg}' waxay u baahan tahay qiime."
                        ))
                    })?;
                if result.values.insert(arg.clone(), value.clone()).is_some() {
                    return Err(super::tools::tool_error(format!(
                        "Doorashada '{arg}' waa la soo celiyey."
                    )));
                }
            } else if arg.starts_with('-') {
                return Err(super::tools::tool_error(format!(
                    "Doorashada '{arg}' lama aqoonsan."
                )));
            } else {
                result.positional.push(arg.clone());
            }
        }
        Ok(result)
    }
    pub fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }
    pub fn value(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
    pub fn one(&self, required: bool, usage: &str) -> Result<Option<&str>, SoplangError> {
        if self.positional.len() > 1 || required && self.positional.is_empty() {
            return Err(super::tools::tool_error(format!("Adeegsiga: {usage}")));
        }
        Ok(self.positional.first().map(String::as_str))
    }
}
