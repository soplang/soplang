use soplang::{analyze, frontend::parse_source, runtime, ReplSession, Value};

#[test]
fn nullable_and_union_syntax_preserves_logical_or_and_null_expressions() {
    for source in [
        "abn? n=maran",
        "maran n=maran",
        "maran|abn n=1",
        "madoor qoraal? n=maran",
        "hawl f(abn|qoraal a,qoraal? b=maran):abn|qoraal{celi a}",
        "hawl f():maran{celi}",
        "maran f() hawl f(){}",
        "qor(run||been)",
    ] {
        let stmts = parse_source(source, None).unwrap_or_else(|e| panic!("{source}: {e}"));
        analyze(&stmts).unwrap_or_else(|e| panic!("{source}: {e}"));
    }
    for source in [
        "abn?? n=maran",
        "abn|door n=1",
        "abn| n=1",
        "hawl f(a):?{}",
        "qor(1|2)",
        "abn? *x=1",
    ] {
        assert!(parse_source(source, None).is_err(), "{source}");
    }
    let wide = format!("{} value=1", vec!["abn"; 33].join("|"));
    assert!(parse_source(&wide, None).is_err());
}

#[test]
fn union_checks_validate_all_possible_source_types_and_unknown_classes() {
    for (source, code) in [
        ("abn|qoraal n=run", "E022"),
        ("abn n=maran", "E022"),
        ("hawl f(qoraal? value){} f()", "E022"),
        ("maran n=1", "E022"),
        ("hawl f(abn? a){} f('x')", "E022"),
        ("hawl f():abn|qoraal {celi run}", "E022"),
        ("hawl f():abn|qoraal {celi}", "E022"),
        ("abn|qoraal source=1 abn dest=source", "E022"),
        ("abn|Missing? n=maran", "E021"),
        ("hawl f(abn|Missing? a){}", "E021"),
    ] {
        let error = analyze(&parse_source(source, None).unwrap()).unwrap_err();
        assert_eq!(error.code(), Some(code), "{source}: {error}");
    }
    let source = "hawl f(qoraal? name=maran):qoraal?{celi name} abn|qoraal value=1";
    soplang::semantic::analyze_with_options(
        &parse_source(source, None).unwrap(),
        soplang::semantic::AnalyzeOptions {
            strict: true,
            repl: false,
        },
    )
    .unwrap();
}

#[test]
fn editor_and_ast_display_normalized_nullable_contracts() {
    use soplang::frontend::{Stmt, TypeAnnotation};
    let stmts = parse_source("abn|maran|abn value=1", None).unwrap();
    let Stmt::VarDecl { type_ann, .. } = stmts[0].unlocated() else {
        panic!("declaration")
    };
    assert_eq!(type_ann.to_string(), "abn?");
    assert_eq!(soplang::analysis::index::type_name(type_ann), "abn?");
    assert!(matches!(type_ann,TypeAnnotation::Union(types) if types.len()==2));
    assert!(stmts[0].to_string().contains("abn? value"));
}

#[test]
fn repl_contracts_retain_descriptors_and_restore_values_after_failed_checks() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval(
            "qoraal? name=maran hawl f(qoraal? a=maran):qoraal?{celi a}",
            None,
        )
        .unwrap();
    session.collect();
    session.eval("name=f(a:'Ayaan')", None).unwrap();
    assert_eq!(session.get_global("name"), Some(Value::Str("Ayaan".into())));
    session.eval("hawl wrong(){celi 1}", None).unwrap();
    assert_eq!(
        session.eval("name=wrong()", None).unwrap_err().code(),
        Some("E022")
    );
    assert_eq!(session.get_global("name"), Some(Value::Str("Ayaan".into())));
    session.collect();
    session.eval("name=f()", None).unwrap();
    assert_eq!(session.get_global("name"), Some(Value::Null));
}

#[test]
fn invalid_native_contracts_are_recoverable_and_foreign_handles_are_rejected() {
    let context = runtime::RuntimeContext::new();
    for bytes in [
        &b""[..],
        &b"abn|"[..],
        &b"unknown"[..],
        &b"@"[..],
        &b"\xff"[..],
        &vec![b'|'; 65][..],
    ] {
        let error = context
            .execute(|| {
                runtime::soplang_check_contract(
                    1,
                    1,
                    bytes.as_ptr(),
                    bytes.len(),
                    std::ptr::null(),
                    0,
                )
            })
            .unwrap_err();
        assert_eq!(error.code(), Some("E022"));
    }
    assert_eq!(
        context
            .execute(|| runtime::soplang_check_contract(
                1,
                1,
                std::ptr::null(),
                1,
                std::ptr::null(),
                0
            ))
            .unwrap_err()
            .code(),
        Some("E022")
    );
    assert_eq!(
        context
            .execute(|| runtime::soplang_check_contract(
                -1,
                1,
                b"abn".as_ptr(),
                3,
                std::ptr::null(),
                0
            ))
            .unwrap_err()
            .code(),
        Some("E022")
    );
    let foreign = runtime::RuntimeContext::new();
    let text = foreign.with(|| runtime::soplang_str(b"x".as_ptr(), 1));
    assert_eq!(
        context
            .execute(|| runtime::soplang_check_contract(
                text.tag as i64,
                text.payload,
                b"qoraal|maran".as_ptr(),
                12,
                std::ptr::null(),
                0
            ))
            .unwrap_err()
            .code(),
        Some("E035")
    );
    context
        .execute(|| runtime::soplang_check_contract(1, 1, b"abn".as_ptr(), 3, std::ptr::null(), 0))
        .unwrap();
}

#[test]
fn class_alternatives_reject_conflicting_metadata_borrows() {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    let context = runtime::RuntimeContext::new();
    let object = Rc::new(RefCell::new(HashMap::new()));
    let value = context.with(|| runtime::value_to_soplang(&Value::Object(Rc::clone(&object))));
    let borrowed = object.borrow_mut();
    let descriptor = b"@Base|maran";
    let error = context
        .execute(|| {
            runtime::soplang_check_contract(
                value.tag as i64,
                value.payload,
                descriptor.as_ptr(),
                descriptor.len(),
                std::ptr::null(),
                0,
            )
        })
        .unwrap_err();
    assert_eq!(error.code(), Some("E022"));
    assert!(error.message().contains("lama akhriyi karo"));
    drop(borrowed);
}
