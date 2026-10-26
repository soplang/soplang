use soplang::analysis::{check_source, index::Index, Limits};
use soplang::frontend::{
    token::FormatPart,
    trivia::{lossless_tokens, protected_ranges},
};
use soplang::{Lexer, TokenType};
use std::path::Path;

#[test]
fn fields_preserve_original_unicode_offsets_and_format_spans() {
    let source = "door magac = 'é'\nqor(f\"é={magac} {1+2:04d}\")";
    let tokens = Lexer::new(source).tokenize().unwrap();
    let token = tokens
        .iter()
        .find(|t| t.kind == TokenType::FormattedString)
        .unwrap();
    assert_eq!(&source[token.start..token.end], "f\"é={magac} {1+2:04d}\"");
    let fields: Vec<_> = token
        .format_parts
        .iter()
        .filter_map(|part| match part {
            FormatPart::Field {
                tokens,
                format,
                format_token,
            } => Some((tokens, format, format_token)),
            _ => None,
        })
        .collect();
    assert_eq!(fields.len(), 2);
    let name = &fields[0].0[0];
    assert_eq!((name.line, name.col), (2, 10));
    assert_eq!(&source[name.start..name.end], "magac");
    assert_eq!(fields[1].1, "04d");
    let spec = fields[1].2.as_ref().unwrap();
    assert_eq!(&source[spec.start..spec.end], "04d");
    assert_eq!((spec.line, spec.col), (2, 22));
}

#[test]
fn malformed_fields_have_localized_errors_at_the_field() {
    for (source, code, col) in [
        (r#"f"{}""#, "E005", 3),
        (r#"f"{ :d}""#, "E005", 3),
        (r#"f"{x""#, "E005", 3),
        (r#"f"{x"#, "E005", 3),
        (r#"f"}""#, "E005", 3),
        (r#"f"{1:{2}}""#, "E005", 6),
        (r#"f"\q""#, "E004", 3),
        (r#"f"{e'\q'}""#, "E004", 6),
        (r#"f"unclosed"#, "E002", 1),
    ] {
        let error = Lexer::new(source).tokenize().unwrap_err();
        assert_eq!(error.code(), Some(code), "{source}: {error}");
        assert_eq!(error.location(), (1, col), "{source}: {error}");
    }
    let mut source = "1".to_owned();
    for _ in 0..65 {
        source = format!("f\"{{{source}}}\"");
    }
    assert_eq!(
        Lexer::new(&source).tokenize().unwrap_err().code(),
        Some("E005")
    );
}

#[test]
fn editor_ranges_cover_nested_literals_and_incomplete_input() {
    let literal = r#"f"outer {f"nested {"inner"}"} {'// not a comment'}""#;
    let source = format!("qor({literal}) // comment\n");
    let ranges = protected_ranges(&source);
    assert_eq!(&source[ranges[0].clone()], literal);
    assert_eq!(&source[ranges[1].clone()], "// comment");
    let pieces = lossless_tokens(&source).unwrap();
    let mut cursor = 0;
    for piece in pieces {
        assert_eq!(piece.range.start, cursor);
        cursor = piece.range.end;
    }
    assert_eq!(cursor, source.len());
    let unfinished = "f\"value {object[\"key\"]";
    assert_eq!(protected_ranges(unfinished), vec![0..unfinished.len()]);
}

#[test]
fn editor_index_resolves_interpolated_references_and_lambda_parameters() {
    let source = "abn number=3\nqoraal text=f\"{number}:{(hawl(abn p): abn { celi p+number })(2)}\"\nqor(text)";
    let analysis = check_source(
        source,
        Path::new("/tmp/interpolation-index.sop"),
        true,
        Limits::default(),
        &|_| unreachable!(),
        &|| false,
    );
    assert!(analysis.error.is_none(), "{:?}", analysis.error);
    let index = Index::build(&analysis.files);
    for (name, count) in [("number", 3), ("p", 2), ("text", 2)] {
        let symbol = index
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .unwrap();
        let occurrences: Vec<_> = index
            .occurrences
            .iter()
            .filter(|o| o.symbol == symbol.id)
            .collect();
        assert_eq!(occurrences.len(), count, "{name}: {occurrences:?}");
        for occurrence in occurrences {
            assert_eq!(
                &source[occurrence.location.start..occurrence.location.end],
                name
            );
        }
    }
}
