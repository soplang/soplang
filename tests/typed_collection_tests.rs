use runtime::{RuntimeContext, SoplangValue};
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
fn list(values: Vec<Value>) -> SoplangValue {
    runtime::value_to_soplang(&Value::List(Rc::new(RefCell::new(values))))
}

#[test]
fn typed_collection_syntax_checks_keys_arities_nesting_and_invariance() {
    for source in [
        "teed[] x=[]",
        "teed[abn,qoraal] x=[]",
        "walax[abn,abn] x={}",
        "abn[abn] x=1",
        "hawl f(teed[abn]? *x){}",
    ] {
        assert!(parse_source(source, None).is_err(), "{source}");
    }
    let deep = format!("{}abn{} x=[]", "teed[".repeat(40), "]".repeat(40));
    assert!(parse_source(&deep, None).is_err());
    for (source, code) in [
        ("teed[Missing] x=[]", "E021"),
        ("walax[qoraal,teed[Missing?]] x={}", "E021"),
        ("teed[abn] x='bad'", "E022"),
        ("teed[abn] x=[] teed[jajab] y=x", "E022"),
        ("walax[abn] x={} walax[qoraal] y=x", "E022"),
    ] {
        let error = analyze(&parse_source(source, None).unwrap()).unwrap_err();
        assert_eq!(error.code(), Some(code), "{source}: {error}");
    }
    let source="teed[abn|qoraal] x=[] teed[qoraal|abn] y=x teed[teed[abn|qoraal]|teed[qoraal|abn]] nested=[] teed[teed[qoraal|abn]] normalized=nested hawl gather(teed[abn] *items,walax[abn] **options):teed[abn]{celi items}";
    let stmts = parse_source(source, None).unwrap();
    soplang::semantic::analyze_with_options(
        &stmts,
        soplang::semantic::AnalyzeOptions {
            strict: true,
            repl: false,
        },
    )
    .unwrap();
    assert!(stmts[0].to_string().contains("teed[abn | qoraal]"));
}

#[test]
fn guards_are_weak_deduplicated_and_collected_with_their_owners() {
    let context = RuntimeContext::new();
    let value = context.with(|| list(vec![Value::Int(1)]));
    let pin = context.root(value).unwrap();
    for _ in 0..1000 {
        let checked = context.execute(|| contract(value, "teed[abn]")).unwrap();
        assert_eq!(checked.payload, value.payload);
    }
    assert_eq!(context.memory_stats().type_contracts, 1);
    context.collect();
    assert_eq!(context.memory_stats().type_contracts, 1);
    drop(pin);
    let report = context.collect();
    assert_eq!(report.after.type_contracts, 0);
    assert_eq!(report.after.values(), 0);
    context.with(|| {
        assert_eq!(
            runtime::soplang_to_value(value).unwrap_err().code(),
            Some("E035")
        )
    });
}

#[test]
fn native_mutations_share_guards_and_unrelated_host_borrows_do_not_block_writes() {
    let context = RuntimeContext::new();
    let value = context.with(|| list(vec![Value::Int(1)]));
    context.execute(|| contract(value, "teed[abn]")).unwrap();
    let host = context.with(|| runtime::soplang_to_value(value).unwrap());
    let Value::List(data) = host else {
        panic!("list")
    };
    let borrowed = data.borrow_mut();
    let other = context.with(|| list(vec![]));
    context
        .execute(|| runtime::soplang_list_push(other, runtime::soplang_int(2)))
        .unwrap();
    assert_eq!(*borrowed, [Value::Int(1)]);
    drop(borrowed);
    let error = context
        .execute(|| {
            let text = runtime::soplang_str(b"bad".as_ptr(), 3);
            runtime::soplang_list_push(value, text)
        })
        .unwrap_err();
    assert_eq!(error.code(), Some("E039"));
    assert_eq!(*data.borrow(), [Value::Int(1)]);
    let borrowed = data.borrow();
    assert_eq!(
        context
            .execute(|| runtime::soplang_list_push(value, runtime::soplang_int(2)))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    drop(borrowed);
    context
        .with(|| runtime::stdlib::list_geli(Rc::clone(&data), &[Value::Int(3)]))
        .unwrap();
    assert_eq!(*data.borrow(), [Value::Int(1), Value::Int(3)]);
}

#[test]
fn failed_contract_checks_do_not_restrict_previously_untyped_storage() {
    let context = RuntimeContext::new();
    let value = context.with(|| list(vec![Value::Str("bad".into())]));
    assert_eq!(
        context
            .execute(|| contract(value, "teed[abn]"))
            .unwrap_err()
            .code(),
        Some("E022")
    );
    assert_eq!(context.memory_stats().type_contracts, 0);
    context
        .execute(|| runtime::soplang_list_push(value, runtime::soplang_int(2)))
        .unwrap();
    for descriptor in [
        "teed[]",
        "teed[abn,qoraal]",
        "walax[abn,abn]",
        "teed[abn]trailing",
    ] {
        assert_eq!(
            context
                .execute(|| contract(value, descriptor))
                .unwrap_err()
                .code(),
            Some("E022")
        );
    }
}

#[test]
fn contracts_survive_repl_calls_and_nested_alias_replacement() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("teed[teed[abn]] rows=[[1]] door child=rows[0]", None)
        .unwrap();
    session.collect();
    assert_eq!(
        session.eval("child.geli('bad')", None).unwrap_err().code(),
        Some("E039")
    );
    session
        .eval("rows[0]=[2] child.geli('detached')", None)
        .unwrap();
    session.collect();
    assert_eq!(
        session
            .eval("rows[0].geli('bad')", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
    session
        .eval("hawl f(teed[abn] v):teed[abn]{celi v}", None)
        .unwrap();
    session.collect();
    session.eval("door result=f([3])", None).unwrap();
    assert_eq!(
        session.eval("result[0]='bad'", None).unwrap_err().code(),
        Some("E039")
    );
}

#[test]
fn cyclic_guarded_graphs_do_not_pin_collections_after_collection_or_context_drop() {
    let context = RuntimeContext::new();
    let (value, weak) = context.with(|| {
        let value = list(vec![]);
        runtime::soplang_list_push(value, value);
        let Value::List(data) = runtime::soplang_to_value(value).unwrap() else {
            panic!("list")
        };
        (value, Rc::downgrade(&data))
    });
    context.execute(|| contract(value, "teed[teed]")).unwrap();
    let pin = context.root(value).unwrap();
    context.collect();
    assert!(weak.upgrade().is_some());
    drop(pin);
    assert_eq!(context.collect().after.type_contracts, 0);
    assert!(weak.upgrade().is_none());
    let value = context.with(|| list(vec![]));
    context.execute(|| contract(value, "teed[abn]")).unwrap();
    drop(context);
}
