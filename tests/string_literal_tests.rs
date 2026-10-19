use soplang::frontend::trivia::{lossless_tokens, protected_ranges};
use soplang::{Lexer, TokenType};

#[test]
fn explicit_escapes_decode_without_changing_legacy_strings() {
    let source = r#""C:\temp\new" r"C:\temp\new" e"C:\\temp\n\t\r\0\"\'\u{e9}\u{1F600}""#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, r"C:\temp\new");
    assert_eq!(tokens[1].lexeme, r"C:\temp\new");
    assert_eq!(tokens[2].lexeme, "C:\\temp\n\t\r\0\"'é😀");
    assert_eq!(&source[tokens[1].start..tokens[1].end], r#"r"C:\temp\new""#);
    assert_eq!(
        &source[tokens[2].start..tokens[2].end],
        r#"e"C:\\temp\n\t\r\0\"\'\u{e9}\u{1F600}""#
    );
}

#[test]
fn triple_quotes_preserve_content_and_track_following_locations() {
    let source = "r\"\"\"first ' and \"\nsecond\\n\"\"\"\nqor(1)";
    let tokens = Lexer::new(source).tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenType::String);
    assert_eq!(tokens[0].lexeme, "first ' and \"\nsecond\\n");
    assert_eq!((tokens[1].line, tokens[1].col), (3, 1));
    assert_eq!(
        &source[tokens[0].start..tokens[0].end],
        "r\"\"\"first ' and \"\nsecond\\n\"\"\""
    );
    let tokens = Lexer::new("e'''a\\n\\u{e9}''' ").tokenize().unwrap();
    assert_eq!(tokens[0].lexeme, "a\né");
}

#[test]
fn escaped_line_continuations_accept_lf_and_crlf() {
    for ending in ["\n", "\r\n"] {
        let source = format!("e\"a\\{ending}b\"");
        assert_eq!(Lexer::new(&source).tokenize().unwrap()[0].lexeme, "ab");
    }
}

#[test]
fn invalid_escapes_point_to_the_backslash_and_unclosed_literals_to_the_opening() {
    for source in [
        r#"e"\q""#,
        r#"e"\u{}""#,
        r#"e"\u{D800}""#,
        r#"e"\u{110000}""#,
        r#"e"\u1234""#,
        r#"e"\u{1234567}""#,
    ] {
        let error = Lexer::new(source).tokenize().unwrap_err();
        assert_eq!(error.code(), Some("E004"), "{source}");
        assert_eq!(error.location(), (1, 3), "{source}");
    }
    for source in ["r\"\"\"unfinished", "e'''unfinished", "\"\"\"unfinished"] {
        let error = Lexer::new(source).tokenize().unwrap_err();
        assert_eq!(error.code(), Some("E002"));
        assert_eq!(error.location(), (1, 1));
    }
}

#[test]
fn editor_ranges_cover_original_literal_bytes_including_prefixes_and_escapes() {
    let source = "door x = e\"é\\\"// protected\\n\"; /* comment */\nqor(r'''a\n\"b\"''')";
    let pieces = lossless_tokens(source).unwrap();
    let reconstructed: String = pieces
        .iter()
        .map(|piece| &source[piece.range.clone()])
        .collect();
    assert_eq!(reconstructed, source);
    let protected: Vec<_> = protected_ranges(source)
        .into_iter()
        .map(|range| &source[range])
        .collect();
    assert_eq!(
        protected,
        [
            "e\"é\\\"// protected\\n\"",
            "/* comment */",
            "r'''a\n\"b\"'''"
        ]
    );
    let unfinished = "e\"unfinished\\\"";
    assert_eq!(protected_ranges(unfinished)[0], 0..unfinished.len());
}
