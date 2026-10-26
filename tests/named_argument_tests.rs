use soplang::{analyze, frontend::parse_source, runtime, ReplSession, Value};

#[test]
fn named_calls_and_keyword_collectors_reject_invalid_syntax() {
    for source in [
        "hawl f(**options=[]){}",
        "hawl f(teed **options){}",
        "hawl f(**options,a){}",
        "hawl f(**a,**b){}",
        "hawl f(*a,*b){}",
        "f(a:1,2)",
        "f(a:1,a:2)",
    ] {
        assert!(parse_source(source, None).is_err(), "{source}");
    }
    for source in [
        "hawl f(*a,**b){}",
        "hawl f(teed *a,walax **b){}",
        "hawl f(walax **options){}",
        "hawl f(a=1,**b){}",
    ] {
        analyze(&parse_source(source, None).unwrap()).unwrap();
    }
}

#[test]
fn known_named_calls_are_validated_before_execution() {
    for source in [
        "hawl f(a){} f(b:1)",
        "hawl f(a,b){} f(a:1)",
        "hawl f(a){} f(1,a:2)",
        "hawl f(abn a){} f(a:'x')",
        "hawl f(**options){} f(1)",
        "qaab A {hawl dhis(abn a){}} door a=cusub A(a:'x')",
    ] {
        let error = analyze(&parse_source(source, None).unwrap()).unwrap_err();
        assert_eq!(error.code(), Some("E022"), "{source}: {error}");
    }
    // A parameter shadows a same-named global function signature.
    let source = "hawl f(a,b){} hawl invoke(f=hawl(x=1){celi x}){celi f(x:2)}";
    analyze(&parse_source(source, None).unwrap()).unwrap();
}

#[test]
fn strict_keyword_collectors_are_objects_and_named_values_keep_their_types() {
    let stmts = parse_source(
        "hawl f(abn a=1,walax **options):walax{celi options} f(a:2,key:'value')",
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
fn named_function_metadata_survives_repl_snippets_and_binding_failures() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("hawl f(a=1,b=a+1,**options){celi [a,b,options]}", None)
        .unwrap();
    session.eval("door result=f(b:9,key:3)", None).unwrap();
    let Some(Value::List(result)) = session.get_global("result") else {
        panic!("list")
    };
    assert_eq!(result.borrow()[0], Value::Int(1));
    assert_eq!(result.borrow()[1], Value::Int(9));
    assert!(session.eval("f(1,a:2)", None).is_err());
    session.collect();
    session.eval("result=f(a:7)", None).unwrap();
    let Some(Value::List(result)) = session.get_global("result") else {
        panic!("list")
    };
    assert_eq!(result.borrow()[1], Value::Int(8));
}

#[test]
fn malformed_named_call_descriptors_fail_without_entering_compiled_code() {
    let context = runtime::RuntimeContext::new();
    for (args, n, name_len, labels_len) in [
        (std::ptr::null(), 1, 0, 0),
        (std::ptr::null(), 0, 1, 0),
        (std::ptr::null(), -1, 0, 0),
    ] {
        let result = context.execute(|| {
            runtime::soplang_call_named(
                runtime::SoplangValue::null(),
                0,
                std::ptr::null(),
                name_len,
                args,
                n,
                std::ptr::null(),
                labels_len,
            )
        });
        assert_eq!(result.unwrap_err().code(), Some("E022"));
    }
}
