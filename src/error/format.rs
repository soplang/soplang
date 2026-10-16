//! Format errors with source context, Unicode-aware carets, and hints.

use colored::Colorize;
use unicode_width::UnicodeWidthChar;

use super::SoplangError;

// Tabs use four-column stops in diagnostic output. Source columns themselves
// count Unicode scalar values, matching the lexer, and are one-based.
fn expand(text: &str, mut column: usize) -> (String, usize) {
    let start = column;
    let mut rendered = String::new();
    for ch in text.chars() {
        if ch == '\t' {
            let width = 4 - column % 4;
            rendered.push_str(&" ".repeat(width));
            column += width;
        } else {
            rendered.push(ch);
            column += ch.width().unwrap_or(0);
        }
    }
    (rendered, column - start)
}

/// Format an error with its original file and source line when available.
pub fn format_error_with_source(err: &SoplangError, source: Option<&str>) -> String {
    let (line, col) = err.location();
    let meta = err.meta();
    let mut out = format!("{}\n", err.to_string().red().bold());
    if let Some(file) = &meta.file {
        out.push_str(&format!("     --> {}:{}:{}\n", file.display(), line, col));
    }
    let content = meta.source_line.as_deref().or_else(|| {
        source.and_then(|s| {
            line.checked_sub(1)
                .and_then(|index| s.split('\n').nth(index))
        })
    });
    if let Some(content) = content.filter(|_| line > 0 && col > 0) {
        let content = content.trim_end_matches('\r');
        let prefix: String = content.chars().take(col - 1).collect();
        let (_, padding) = expand(&prefix, 0);
        let count = if meta.end_line == Some(line) {
            meta.end_col.unwrap_or(col).saturating_sub(col) + 1
        } else {
            1
        };
        let selected: String = content.chars().skip(col - 1).take(count).collect();
        let (_, width) = expand(&selected, padding);
        let (rendered, _) = expand(content, 0);
        let gutter = line.to_string().len().max(4);
        out.push_str(&format!("{line:>gutter$} │   {rendered}\n"));
        out.push_str(&format!(
            "{} │   {}{}\n",
            " ".repeat(gutter),
            " ".repeat(padding),
            "^".repeat(width.max(1)).red()
        ));
    }
    if let Some(hint) = meta.hint.as_deref() {
        out.push_str(&format!("{} {}\n", "     └─".dimmed(), hint.dimmed()));
    }
    out
}
