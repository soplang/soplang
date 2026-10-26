use soplang::{
    analysis::{check_source, index::Index, overlay_loader, Limits},
    frontend::trivia::lossless_tokens,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
#[test]
fn editor_analysis_uses_unsaved_imports_without_execution() {
    let mut overlays = HashMap::new();
    overlays.insert(
        PathBuf::from("/tmp/editor-library.sop"),
        "qor(gelin())\nhawl add(abn a, abn b): abn { celi a+b }".to_owned(),
    );
    let result = check_source(
        "keen \"editor-library.sop\"\nqor(add(1,2))",
        Path::new("/tmp/editor-main.sop"),
        false,
        Limits::default(),
        &|p| overlay_loader(&overlays, p),
        &|| false,
    );
    assert!(result.error.is_none(), "{:?}", result.error);
    let index = Index::build(&result.files);
    let symbol = index.symbols.iter().find(|s| s.name == "add").unwrap();
    assert!(symbol.location.path.ends_with("editor-library.sop"));
    assert_eq!(
        index
            .occurrences
            .iter()
            .filter(|o| o.symbol == symbol.id)
            .count(),
        2
    );
}
#[test]
fn editor_analysis_preserves_file_redeclaration_and_strict_rules() {
    for (source, strict, code) in [
        ("door x=1 door x=2", false, "E020"),
        ("door x=1", true, "E022"),
    ] {
        let result = check_source(
            source,
            Path::new("/tmp/rules.sop"),
            strict,
            Limits::default(),
            &|_| unreachable!(),
            &|| false,
        );
        assert_eq!(result.error.unwrap().code(), Some(code));
    }
}
#[test]
fn lossless_source_pieces_cover_every_byte() {
    let source =
        "// heading\r\nqoraal x = 'a+b https://host/😀' /* comment */\nqor(x == 'C:\\work')\n";
    let pieces = lossless_tokens(source).unwrap();
    let mut cursor = 0;
    let mut restored = String::new();
    for piece in pieces {
        assert_eq!(piece.range.start, cursor);
        restored.push_str(&source[piece.range.clone()]);
        cursor = piece.range.end;
    }
    assert_eq!(cursor, source.len());
    assert_eq!(restored, source);
}
#[test]
fn rename_simulation_rejects_reverse_capture() {
    let result=check_source("abn other=1 hawl outer(abn value): abn { door f=hawl(): abn { celi other } celi value+f() }",Path::new("/tmp/capture.sop"),false,Limits::default(),&|_|unreachable!(),&||false);
    assert!(result.error.is_none());
    let index = Index::build(&result.files);
    let symbol = index.symbols.iter().find(|s| s.name == "value").unwrap();
    assert!(!index.rename_preserves_bindings(&symbol.id, "other"));
    assert!(index.rename_preserves_bindings(&symbol.id, "renamed"));
}

#[test]
fn analysis_resolves_bundled_and_unsaved_namespaced_classes() {
    let root =
        std::env::temp_dir().join(format!("soplang_analysis_overlay_{}", std::process::id()));
    let main = root.join("main.sop");
    let library = root.join("people.sop");
    let overlays = HashMap::from([(library.clone(),
        "soosaar Qof\nabn private=7\nqaab Qof { hawl value(): abn { celi private } }\nqor(gelin())\n".to_string())]);
    let result = check_source(
        "keen \"people.sop\" sida m\nm.Qof person=cusub m.Qof()\nqor(person.value())\n",
        &main,
        true,
        Limits::default(),
        &|path| overlay_loader(&overlays, path),
        &|| false,
    );
    assert!(result.error.is_none(), "{:?}", result.error);
    assert_eq!(result.files[0].imports[0].1, library);
    let result = check_source(
        "keen \"web\" sida w\ndoor app=cusub w.Web()",
        &main,
        false,
        Limits::default(),
        &|path| overlay_loader(&overlays, path),
        &|| false,
    );
    assert!(result.error.is_none(), "{:?}", result.error);
    assert!(result
        .files
        .iter()
        .any(|file| file.path.ends_with("web.sop")));
}

#[test]
fn editor_import_expansion_obeys_cancellation_and_depth_budgets() {
    let root = std::env::temp_dir().join(format!("soplang_analysis_limits_{}", std::process::id()));
    let mut overlays = HashMap::new();
    for i in 0..20 {
        overlays.insert(
            root.join(format!("m{i}.sop")),
            format!("keen \"m{}.sop\" sida next\n", i + 1),
        );
    }
    let source = "keen \"m0.sop\" sida m";
    let result = check_source(
        source,
        &root.join("main.sop"),
        false,
        Limits {
            max_depth: 3,
            ..Limits::default()
        },
        &|path| overlay_loader(&overlays, path),
        &|| false,
    );
    assert_eq!(result.error.unwrap().code(), Some("EDITOR_LIMIT"));
    assert!(result.files.len() <= 4);
    let result = check_source(
        source,
        &root.join("main.sop"),
        false,
        Limits::default(),
        &|_| unreachable!("cancelled requests do not read imports"),
        &|| true,
    );
    assert_eq!(result.error.unwrap().code(), Some("EDITOR_LIMIT"));
}

#[test]
fn filesystem_source_reads_are_bounded_and_unsaved_overlays_take_precedence() {
    let path =
        std::env::temp_dir().join(format!("soplang_editor_large_{}.sop", std::process::id()));
    std::fs::File::create(&path)
        .unwrap()
        .set_len(Limits::default().max_file_bytes as u64 + 1)
        .unwrap();
    assert!(overlay_loader(&HashMap::new(), &path)
        .unwrap_err()
        .contains("xadka"));
    let overlays = HashMap::from([(path.clone(), "door x = 1".into())]);
    assert_eq!(overlay_loader(&overlays, &path).unwrap(), "door x = 1");
    std::fs::remove_file(path).unwrap();
    #[cfg(unix)]
    {
        let dir = std::env::temp_dir().join(format!("soplang_editor_alias_{}", std::process::id()));
        let alias = dir.with_extension("link");
        std::fs::create_dir(&dir).unwrap();
        std::os::unix::fs::symlink(&dir, &alias).unwrap();
        let source = dir.join("large.sop");
        std::fs::File::create(&source)
            .unwrap()
            .set_len(Limits::default().max_file_bytes as u64 + 1)
            .unwrap();
        let requested = alias.join("large.sop");
        let overlays = HashMap::from([(requested.clone(), "door x = 2".into())]);
        assert_eq!(overlay_loader(&overlays, &requested).unwrap(), "door x = 2");
        std::fs::remove_file(alias).unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    assert!(overlay_loader(&HashMap::new(), Path::new("/dev/zero"))
        .unwrap_err()
        .contains("fayl caadi ah"));
}

#[test]
fn unfinished_function_scopes_retain_parameters_locals_and_imports() {
    let path = Path::new("/tmp/editor-unfinished.sop");
    for tail in [
        "local.",
        "qor(local",
        "door pending =",
        "qoraal broken = \"unfinished",
        "/* unfinished comment",
    ] {
        let source=format!("keen \"types.sop\" sida m\nhawl edit(abn count) {{\n m.Qof local = cusub m.Qof()\n {tail}");
        let mut overlays = HashMap::new();
        overlays.insert(
            PathBuf::from("/tmp/types.sop"),
            "qaab Qof { hawl magac(): qoraal { celi \"Amina\" } }".into(),
        );
        let analysis = check_source(
            &source,
            path,
            false,
            Limits::default(),
            &|p| overlay_loader(&overlays, p),
            &|| false,
        );
        assert!(
            analysis.error.is_some(),
            "incomplete source must retain its diagnostic"
        );
        assert!(soplang::frontend::parse_source(&source, Some(path)).is_err());
        let index = Index::build(&analysis.files);
        let names: Vec<_> = index
            .visible(path, source.len())
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect();
        assert!(
            names.contains(&"count") && names.contains(&"local"),
            "{tail}: {names:?}"
        );
        if tail == "local." {
            assert!(
                index
                    .members_at(path, source.len())
                    .unwrap()
                    .iter()
                    .any(|symbol| symbol.name == "magac"),
                "{index:?}"
            );
        }
        assert!(!index
            .symbols
            .iter()
            .any(|symbol| symbol.name.starts_with("__soplang_incomplete")));
    }
}

#[test]
fn lexical_errors_do_not_bypass_editor_token_or_nesting_budgets() {
    let path = Path::new("/tmp/editor-budgets.sop");
    for source in [
        format!("{}'unfinished", "qor(1)\n".repeat(35_000)),
        format!("{}'unfinished", "(".repeat(129)),
        format!("{}'unfinished", "!".repeat(513)),
    ] {
        let analysis = check_source(
            &source,
            path,
            false,
            Limits::default(),
            &|_| unreachable!(),
            &|| false,
        );
        assert!(analysis.files.is_empty());
        assert!(analysis.error.unwrap().message().contains("xadka"));
    }
}
