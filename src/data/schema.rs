//! `schema.sop` tables for the Soplang data layer.

use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub tables: Vec<Table>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: ColumnType,
    pub optional: bool,
    pub key: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColumnType {
    Int,
    Text,
    Float,
    Bool,
}

impl Schema {
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|table| table.name == name)
    }

    /// Canonical schema source for validated migration snapshots.
    pub(super) fn source(&self) -> String {
        let mut text = String::new();
        for table in &self.tables {
            text.push_str(&format!("jadwal {} {{\n", table.name));
            for column in &table.columns {
                let ty = match column.ty {
                    ColumnType::Int => "abn",
                    ColumnType::Text => "qoraal",
                    ColumnType::Float => "jajab",
                    ColumnType::Bool => "bool",
                };
                text.push_str(&format!(
                    "    {} {}{}{}\n",
                    column.name,
                    ty,
                    if column.optional { "?" } else { "" },
                    if column.key { " furaha" } else { "" }
                ));
            }
            text.push_str("}\n");
        }
        text
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        for table in &self.tables {
            hasher.update(table.name.as_bytes());
            hasher.update([0]);
            for column in &table.columns {
                hasher.update(column.name.as_bytes());
                hasher.update([
                    column.ty as u8,
                    u8::from(column.optional),
                    u8::from(column.key),
                ]);
            }
        }
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

impl ColumnType {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "abn" => Some(Self::Int),
            "qoraal" => Some(Self::Text),
            "jajab" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            _ => None,
        }
    }
}

/// Parse schema text supplied at runtime, retaining the string-error API.
pub fn parse_schema(text: &str) -> Result<Schema, String> {
    parse_schema_at(text, None).map_err(|error| error.message().to_string())
}

/// CLI/editor diagnostics retain the actual schema file and token range.
pub fn parse_schema_file(
    text: &str,
    path: &std::path::Path,
) -> Result<Schema, crate::error::SoplangError> {
    parse_schema_at(text, Some(path))
}

struct SourceLine<'a> {
    number: usize,
    raw: &'a str,
    code: &'a str,
}
impl SourceLine<'_> {
    fn words(&self) -> Vec<(&str, usize)> {
        let mut cursor = 0;
        self.code
            .split_whitespace()
            .map(|word| {
                let start = cursor + self.code[cursor..].find(word).unwrap();
                cursor = start + word.len();
                (word, start)
            })
            .collect()
    }
    fn error(
        &self,
        path: Option<&std::path::Path>,
        message: impl Into<String>,
        start: usize,
        width: usize,
    ) -> crate::error::SoplangError {
        let offset = self.raw.find(self.code).unwrap_or(0) + start;
        let col = self.raw[..offset.min(self.raw.len())].chars().count() + 1;
        let mut meta = crate::error::ErrorMeta::default()
            .with_code(crate::error::codes::E059_DATA)
            .with_span(self.number, col + width.max(1));
        meta.file = path.map(std::path::Path::to_path_buf);
        meta.source_line = Some(self.raw.into());
        crate::error::runtime_error_ex(message, self.number, col, meta)
    }
    fn word_error(
        &self,
        path: Option<&std::path::Path>,
        message: impl Into<String>,
        index: usize,
    ) -> crate::error::SoplangError {
        let words = self.words();
        let (word, start) = words.get(index).copied().unwrap_or(("", self.code.len()));
        self.error(path, message, start, word.chars().count())
    }
}
fn parse_schema_at(
    text: &str,
    path: Option<&std::path::Path>,
) -> Result<Schema, crate::error::SoplangError> {
    let mut tables = Vec::new();
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(index, raw)| SourceLine {
            number: index + 1,
            raw,
            code: raw.split('#').next().unwrap_or("").trim(),
        })
        .filter(|line| !line.code.is_empty());
    while let Some(header) = lines.next() {
        let rest = header
            .code
            .strip_prefix("jadwal")
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .ok_or_else(|| header.word_error(path, "Jadwalka ku bilow erayga jadwal.", 0))?
            .trim();
        let opened = rest.ends_with('{');
        let name = rest.strip_suffix('{').unwrap_or(rest).trim();
        if !ident(name) {
            return Err(header.word_error(path, format!("Magaca jadwalka '{name}' ma saxna."), 1));
        }
        if name.starts_with("soplang_") {
            return Err(header.word_error(
                path,
                "Magacyada ku bilaabma soplang_ waxaa loo reebay nidaamka.",
                1,
            ));
        }
        if tables.iter().any(|table: &Table| table.name == name) {
            return Err(header.word_error(
                path,
                format!("Jadwalka '{name}' waa la soo celiyey."),
                1,
            ));
        }
        if !opened {
            let Some(open) = lines.next() else {
                return Err(header.word_error(path, "Jadwalka ma furmin.", 1));
            };
            if open.code != "{" {
                return Err(open.word_error(path, "Jadwalka waa inuu ku furmo {.", 0));
            }
        }
        let mut columns = Vec::new();
        loop {
            let Some(line) = lines.next() else {
                return Err(header.word_error(path, format!("Jadwalka '{name}' ma xidhmin."), 1));
            };
            if line.code == "}" {
                break;
            }
            let words = line.words();
            let (column_name, _) = words[0];
            if !ident(column_name) {
                return Err(line.word_error(
                    path,
                    format!("Magaca tiirka '{column_name}' ma saxna."),
                    0,
                ));
            }
            let Some((kind, _)) = words.get(1).copied() else {
                return Err(line.word_error(
                    path,
                    format!("Nooca tiirka '{column_name}' waa ka maqan yahay."),
                    1,
                ));
            };
            let optional = kind.ends_with('?');
            let ty =
                ColumnType::parse(kind.strip_suffix('?').unwrap_or(kind)).ok_or_else(|| {
                    line.word_error(path, format!("Nooca '{kind}' lama aqoonsan."), 1)
                })?;
            let mut key = false;
            for (index, (flag, _)) in words.iter().enumerate().skip(2) {
                if *flag != "furaha" {
                    return Err(line.word_error(
                        path,
                        format!("Calaamadda '{flag}' lama aqoonsan."),
                        index,
                    ));
                }
                if key || ty != ColumnType::Int || optional {
                    return Err(line.word_error(
                        path,
                        format!("Furaha '{column_name}' waa inuu noqdaa abn hal mar."),
                        index,
                    ));
                }
                key = true;
            }
            if columns
                .iter()
                .any(|column: &Column| column.name == column_name)
            {
                return Err(line.word_error(
                    path,
                    format!("Tiirka '{column_name}' waa la soo celiyey."),
                    0,
                ));
            }
            if key && columns.iter().any(|column| column.key) {
                return Err(line.word_error(
                    path,
                    format!("Jadwalka '{name}' wuxuu yeelan karaa hal furaha abn oo keliya."),
                    2,
                ));
            }
            columns.push(Column {
                name: column_name.into(),
                ty,
                optional,
                key,
            });
        }
        if !columns.iter().any(|column| column.key) {
            return Err(header.word_error(
                path,
                format!("Jadwalka '{name}' wuxuu u baahan yahay hal furaha abn."),
                1,
            ));
        }
        tables.push(Table {
            name: name.into(),
            columns,
        });
    }
    if tables.is_empty() {
        return Err(SourceLine {
            number: 1,
            raw: text.lines().next().unwrap_or(""),
            code: "",
        }
        .error(path, "schema.sop ma leh jadwal.", 0, 1));
    }
    Ok(Schema { tables })
}

pub fn format_schema(
    text: &str,
    path: &std::path::Path,
) -> Result<String, crate::error::SoplangError> {
    parse_schema_file(text, path)?;
    let mut out = String::new();
    let mut depth = 0;
    for line in text.lines() {
        let (code, comment) = line
            .split_once('#')
            .map(|(code, comment)| (code, Some(comment)))
            .unwrap_or((line, None));
        let code = code
            .replace('{', " { ")
            .replace('}', " } ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if code == "}" {
            depth = 0;
        }
        if !code.is_empty() || comment.is_some() {
            out.push_str(&"    ".repeat(depth));
            out.push_str(&code);
        }
        if let Some(comment) = comment {
            if !code.is_empty() {
                out.push(' ');
            }
            out.push('#');
            out.push_str(comment.trim_end());
        }
        out.push('\n');
        if code.ends_with('{') {
            depth = 1;
        }
    }
    parse_schema_file(&out, path)?;
    Ok(out)
}

pub fn ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_lowercase())
        && chars.all(|ch| ch == '_' || ch.is_ascii_lowercase() || ch.is_ascii_digit())
        && name.len() <= 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_table_and_rejects_two_keys() {
        let schema =
            parse_schema("jadwal isticmaale {\n  id abn furaha\n  magac qoraal\n}\n").unwrap();
        assert_eq!(schema.tables[0].columns.len(), 2);
        assert!(parse_schema("jadwal t {\n  a abn furaha\n  b abn furaha\n}\n").is_err());
    }
}
