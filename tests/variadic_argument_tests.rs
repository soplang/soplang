use soplang::{analyze, frontend::parse_source, ReplSession, Value};

#[test]
fn rest_parameter_syntax_rejects_ambiguous_or_invalid_declarations() {
    for source in [
        "hawl f(*items, a) {}",
        "hawl f(*items, *other) {}",
        "hawl f(*items=[]) {}",
        "hawl f(abn *items) {}",
        "hawl f(a,*a) {}",
        "hawl f(a=items,*items) {}",
    ] {
        assert!(
            parse_source(source, None)
                .and_then(|stmts| analyze(&stmts))
                .is_err(),
            "{source}"
        );
    }
    for source in [
        "hawl f(*items,) {}",
        "hawl f(teed *items):teed {celi items}",
        "hawl f(a=1,*items) {}",
    ] {
        let stmts = parse_source(source, None).unwrap();
        analyze(&stmts).unwrap();
    }
}

#[test]
fn strict_variadic_functions_have_a_list_parameter() {
    let stmts = parse_source(
        "hawl f(abn a=1,*items):teed {celi items} f() f(1,2,'x')",
        None,
    )
    .unwrap();
    soplang::semantic::analyze_with_options(
        &stmts,
        soplang::semantic::AnalyzeOptions {
            strict: true,
            repl: false,
        },
    )
    .unwrap();
}

#[test]
fn variadic_functions_remain_callable_in_later_repl_snippets() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("hawl f(a=1,*items){celi items}", None)
        .unwrap();
    session.eval("door result=f()", None).unwrap();
    let Some(Value::List(items)) = session.get_global("result") else {
        panic!("list")
    };
    assert!(items.borrow().is_empty());
    session.eval("result=f(9,2,3)", None).unwrap();
    let Some(Value::List(items)) = session.get_global("result") else {
        panic!("list")
    };
    assert_eq!(*items.borrow(), vec![Value::Int(2), Value::Int(3)]);
}
