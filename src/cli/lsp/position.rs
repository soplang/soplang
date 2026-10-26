//! LSP uses UTF-16 code units; compiler spans use UTF-8 bytes or scalar columns.

use serde_json::{json, Value};

pub(super) fn at(text: &str, offset: usize) -> (u64, u64) {
    let (mut line, mut character) = (0, 0);
    for (index, ch) in text.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u64;
        }
    }
    (line, character)
}

pub(super) fn offset(text: &str, line: u64, character: u64) -> Option<usize> {
    let mut start = 0;
    let mut lines = text.split_inclusive('\n');
    for _ in 0..line {
        start += lines.next()?.len();
    }
    let source = lines.next().unwrap_or("").trim_end_matches(['\r', '\n']);
    let mut units = 0;
    for (index, ch) in source.char_indices() {
        if units == character {
            return Some(start + index);
        }
        units += ch.len_utf16() as u64;
        if units > character {
            return None;
        }
    }
    Some(start + source.len())
}

pub(super) fn scalar_offset(text: &str, line: usize, column: usize) -> usize {
    let start = text
        .split_inclusive('\n')
        .take(line.saturating_sub(1))
        .map(str::len)
        .sum::<usize>();
    let tail = text.get(start..).unwrap_or("");
    start
        + tail
            .chars()
            .take_while(|ch| *ch != '\n' && *ch != '\r')
            .take(column.saturating_sub(1))
            .map(char::len_utf8)
            .sum::<usize>()
}

pub(super) fn range(text: &str, start: usize, end: usize) -> Value {
    let (line, character) = at(text, start);
    let (end_line, end_character) = at(text, end);
    json!({"start": {"line": line, "character": character}, "end": {"line": end_line, "character": end_character}})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplementary_characters_and_crlf_round_trip_without_splitting_surrogates() {
        let text = "a😀b\r\nCali\n";
        assert_eq!(at(text, "a😀".len()), (0, 3));
        assert_eq!(offset(text, 0, 3), Some("a😀".len()));
        assert_eq!(offset(text, 0, 2), None);
        assert_eq!(offset(text, 0, 99), Some("a😀b".len()));
        assert_eq!(offset(text, 1, 1), Some("a😀b\r\nC".len()));
        for (index, _) in text
            .char_indices()
            .filter(|(_, ch)| *ch != '\r' && *ch != '\n')
        {
            let (line, column) = at(text, index);
            assert_eq!(offset(text, line, column), Some(index));
        }
    }
}
