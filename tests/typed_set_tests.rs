use runtime::{RuntimeContext, SetData, SoplangValue};
use soplang::{analyze, frontend::parse_source, runtime, ReplSession, Value};
use std::cell::RefCell;
use std::rc::Rc;

fn contract(value: SoplangValue, text: &str) -> SoplangValue {
    runtime::soplang_check_contract(
        value.tag as i64,
        value.payload,
        text.as_ptr(),
        text.len(),
        b"value".as_ptr(),
        5,
    )
}
fn set(values: Vec<Value>) -> SoplangValue {
    runtime::value_to_soplang(&Value::Set(SetData::new(values).unwrap()))
}
fn method(value: SoplangValue, name: &str, args: &[SoplangValue]) -> SoplangValue {
    runtime::soplang_call_method(
        value,
        name.as_ptr(),
        name.len(),
        args.as_ptr(),
        args.len() as i32,
    )
}

#[test]
fn typed_set_syntax_checks_key_types_invariance_and_contextual_names() {
    for source in [
        "urur[] x=urur()",
        "urur[abn,qoraal] x=urur()",
        "urur[abn,] x=urur()",
        "hawl f(urur[abn] *items){}",
        "hawl f(urur[abn] **items){}",
    ] {
        assert!(parse_source(source, None).is_err(), "{source}");
    }
    let deep = format!(
        "urur[{}abn{}] x=urur()",
        "xirmo[".repeat(40),
        "]".repeat(40)
    );
    assert!(parse_source(&deep, None).is_err());
    for (source, code) in [
        ("urur[Missing] x=urur()", "E021"),
        ("urur[xirmo[Missing?]] x=urur()", "E021"),
        ("urur[teed] x=urur()", "E022"),
        ("urur[walax] x=urur()", "E022"),
        ("urur[baytyo] x=urur()", "E022"),
        ("urur[urur] x=urur()", "E022"),
        ("urur[xirmo[teed]] x=urur()", "E022"),
        ("qaab Person{} urur[Person] x=urur()", "E022"),
        ("urur[abn] x=[]", "E022"),
        ("urur[abn] x=urur() urur[jajab] y=x", "E022"),
        ("urur[xirmo[abn]] x=urur() urur[xirmo[qoraal]] y=x", "E022"),
    ] {
        let error = analyze(&parse_source(source, None).unwrap()).unwrap_err();
        assert_eq!(error.code(), Some(code), "{source}: {error}");
    }
    let source="urur[abn|qoraal] x=urur() urur[qoraal|abn] y=x urur[xirmo[abn|qoraal]] rows=urur() urur[xirmo[qoraal|abn]] same=rows urur[teed?] nullable=urur(maran) urur[xirmo[]] empty=urur(xirmo()) hawl f(urur[abn]? x=maran):urur[abn]?{celi x}";
    let ast = parse_source(source, None).unwrap();
    soplang::semantic::analyze_with_options(
        &ast,
        soplang::semantic::AnalyzeOptions {
            strict: true,
            repl: false,
        },
    )
    .unwrap();
    assert!(ast[0].to_string().contains("urur[abn | qoraal]"));
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("door urur=[1] urur[0]=2 door out=0 urur[0] out=3", None)
        .unwrap();
    assert!(
        matches!(session.get_global("urur"), Some(Value::List(values)) if values.borrow()[0]==Value::Int(2))
    );
}

#[test]
fn native_set_mutations_enforce_weak_deduplicated_contracts() {
    let context = RuntimeContext::new();
    let value = context.with(|| set(vec![Value::Int(1)]));
    let pin = context.root(value).unwrap();
    for _ in 0..1000 {
        assert_eq!(
            context
                .execute(|| contract(value, "urur[abn]"))
                .unwrap()
                .payload,
            value.payload
        );
    }
    assert_eq!(context.memory_stats().type_contracts, 1);
    let text = context.with(|| runtime::soplang_str(b"bad".as_ptr(), 3));
    assert_eq!(
        context
            .execute(|| method(value, "geli", &[text]))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    context
        .execute(|| method(value, "geli", &[runtime::soplang_int(2)]))
        .unwrap();
    let actual = context.with(|| runtime::soplang_to_value(value).unwrap());
    assert!(matches!(&actual,Value::Set(data) if data.values()==[Value::Int(1),Value::Int(2)]));
    assert_eq!(
        context
            .execute(|| method(value, "saar", &[runtime::soplang_int(1)]))
            .unwrap()
            .payload,
        1
    );
    context.execute(|| method(value, "nadiifi", &[])).unwrap();
    context.collect();
    assert_eq!(context.memory_stats().type_contracts, 1);
    drop(actual);
    drop(pin);
    assert_eq!(context.collect().after.type_contracts, 0);
    assert_eq!(context.memory_stats().sets, 0);
}

#[test]
fn failed_set_checks_and_invalid_native_descriptors_install_no_guards() {
    let context = RuntimeContext::new();
    let value = context.with(|| set(vec![Value::Str("x".into())]));
    for descriptor in [
        "urur[abn]",
        "urur[]",
        "urur[abn,qoraal]",
        "urur[teed]",
        "urur[xirmo[walax]]",
        "urur[@Missing]",
        "urur[abn]trailing",
    ] {
        assert_eq!(
            context
                .execute(|| contract(value, descriptor))
                .unwrap_err()
                .code(),
            Some("E022"),
            "{descriptor}"
        );
    }
    assert_eq!(context.memory_stats().type_contracts, 0);
    context
        .execute(|| method(value, "geli", &[runtime::soplang_int(1)]))
        .unwrap();
    let foreign = RuntimeContext::new();
    assert_eq!(
        foreign
            .execute(|| contract(value, "urur[abn]"))
            .unwrap_err()
            .code(),
        Some("E035")
    );
    context.collect();
    assert_eq!(
        context
            .execute(|| contract(value, "urur[abn]"))
            .unwrap_err()
            .code(),
        Some("E035")
    );
}

#[test]
fn set_dependencies_survive_repl_calls_and_detach_when_parent_dies() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval(
            "teed[urur[abn]] parent=[urur(1)] door child=parent[0]",
            None,
        )
        .unwrap();
    session.collect();
    assert_eq!(
        session.eval("child.geli('bad')", None).unwrap_err().code(),
        Some("E039")
    );
    session.eval("parent=[urur(2)]", None).unwrap();
    session.collect();
    session.eval("child.geli('detached')", None).unwrap();
    assert_eq!(
        session
            .eval("parent[0].geli('bad')", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
    session
        .eval(
            "hawl f(urur[abn] value):urur[abn]{celi value} door returned=f(urur(3))",
            None,
        )
        .unwrap();
    session.collect();
    assert_eq!(
        session
            .eval("returned.geli('bad')", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
}

#[test]
fn cyclic_parent_guards_release_sets_tuple_keys_and_metadata() {
    let context = RuntimeContext::new();
    let (root, weak_parent, weak_set, weak_tuple) = context.with(|| {
        let tuple = runtime::TupleData::new(vec![Value::Int(1)]);
        let set = SetData::new(vec![Value::Tuple(Rc::clone(&tuple))]).unwrap();
        let parent = Rc::new(RefCell::new(vec![Value::Set(Rc::clone(&set))]));
        parent.borrow_mut().push(Value::List(Rc::clone(&parent)));
        (
            runtime::value_to_soplang(&Value::List(Rc::clone(&parent))),
            Rc::downgrade(&parent),
            Rc::downgrade(&set),
            Rc::downgrade(&tuple),
        )
    });
    let pin = context.root(root).unwrap();
    context
        .execute(|| contract(root, "teed[teed|urur[xirmo[abn]]]"))
        .unwrap();
    context.collect();
    assert!(weak_set.upgrade().is_some() && weak_tuple.upgrade().is_some());
    assert_eq!(context.memory_stats().type_contracts, 1);
    drop(pin);
    let report = context.collect();
    assert_eq!(report.after.values(), 0);
    assert_eq!(report.after.type_contracts, 0);
    assert!(
        weak_parent.upgrade().is_none()
            && weak_set.upgrade().is_none()
            && weak_tuple.upgrade().is_none()
    );
}
