use soplang::analysis::{check_source, index::Index, Limits};
use soplang::frontend::parse_source;
use soplang::{analyze, ReplSession, Value};
use std::path::Path;

#[test]
fn invalid_parameter_order_duplicates_and_dependencies_are_rejected() {
    for source in [
        "hawl f(a=1,b) {}",
        "hawl f(a,a=1) {}",
        "hawl f(a=a) {}",
        "hawl f(a=b,b=2) {}",
        "hawl f(a=b(),b=hawl(){celi 2}) {}",
        "hawl f(a=hawl(){celi b},b=2) {}",
        "hawl f(abn a='x') {}",
        "hawl f(a=hawl(){jooji}) {}",
    ] {
        let result = parse_source(source, None).and_then(|stmts| analyze(&stmts).map(|_| ()));
        assert!(result.is_err(), "accepted {source}");
    }
    // A parameter of a nested function shadows the outer unavailable name.
    let stmts = parse_source("hawl f(a=hawl(b=2){celi b},b=3){celi a()}", None).unwrap();
    analyze(&stmts).unwrap();
}

#[test]
fn strict_mode_checks_default_and_supplied_parameter_types() {
    for (source, valid) in [
        ("hawl f(abn a=2): abn {celi a} f()", true),
        ("hawl f(jajab a=2): jajab {celi a} f(3)", true),
        ("hawl f(abn a='x'): abn {celi a}", false),
        ("hawl f(a=2): abn {celi a}", false),
        ("hawl f(abn a=2): abn {celi a} f('x')", false),
    ] {
        let stmts = parse_source(source, None).unwrap();
        let result = soplang::semantic::analyze_with_options(
            &stmts,
            soplang::semantic::AnalyzeOptions {
                strict: true,
                repl: false,
            },
        );
        assert_eq!(result.is_ok(), valid, "{source}: {result:?}");
    }
}

#[test]
fn repl_defaults_survive_snippets_and_failed_calls() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("door count=1 hawl f(value=count){celi value}", None)
        .unwrap();
    session.eval("count=9 door result=f()", None).unwrap();
    assert_eq!(session.get_global("result"), Some(Value::Int(9)));
    assert!(session.eval("f(1,2)", None).is_err());
    session.eval("result=f(4)", None).unwrap();
    assert_eq!(session.get_global("result"), Some(Value::Int(4)));
    session.collect();
    session.eval("result=f()", None).unwrap();
    assert_eq!(session.get_global("result"), Some(Value::Int(9)));
}

#[test]
fn editor_indexes_parameter_defaults_and_nested_function_scopes() {
    let source = "hawl f(a={n:1}, cb=hawl(b=2){celi b}, c=a.n){celi cb()+c}\ndoor result=f()";
    let result = check_source(
        source,
        Path::new("/tmp/defaults.sop"),
        false,
        Limits::default(),
        &|_| unreachable!(),
        &|| false,
    );
    assert!(result.error.is_none(), "{:?}", result.error);
    let index = Index::build(&result.files);
    for (name, count) in [("a", 2), ("cb", 2), ("b", 2), ("c", 2)] {
        let symbol = index
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .unwrap();
        assert!(symbol.parameter);
        assert_eq!(
            index
                .occurrences
                .iter()
                .filter(|occurrence| occurrence.symbol == symbol.id)
                .count(),
            count,
            "{name}"
        );
    }
    let f = index
        .symbols
        .iter()
        .find(|symbol| symbol.name == "f")
        .unwrap();
    assert!(f.parameters.as_ref().unwrap()[0].contains(" = "));
}
