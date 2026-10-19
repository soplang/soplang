use soplang::analysis::{check_source, index::Index, Limits};
use soplang::runtime::{self, RuntimeContext};
use soplang::{frontend::parse_source, ReplSession, Value};
use std::{cell::RefCell, path::Path, rc::Rc};

fn make(value: &Value) -> runtime::SoplangValue {
    runtime::soplang_iter_begin(runtime::value_to_soplang(value))
}
fn next(value: runtime::SoplangValue) -> Value {
    runtime::soplang_to_value(runtime::soplang_iter_next(value, 1)).unwrap()
}
fn yielded(step: Value) -> Value {
    let Value::Object(step) = step else {
        panic!("missing step")
    };
    let step = step.borrow();
    assert_eq!(step.get("dhammaad"), Some(&Value::Bool(false)));
    step.get("qiime").unwrap().clone()
}

#[test]
fn host_roots_preserve_iterator_snapshots_and_foreign_handles_are_rejected() {
    let owner = RuntimeContext::new();
    let foreign = RuntimeContext::new();
    let iterator = owner.with(|| {
        make(&Value::List(Rc::new(RefCell::new(vec![
            Value::Str("é".into()),
            Value::Int(2),
        ]))))
    });
    let pin = owner.root(iterator).unwrap();
    owner.collect();
    assert_eq!(owner.memory_stats().iterators, 1);
    foreign.with(|| {
        assert_eq!(
            runtime::soplang_to_value(iterator).unwrap_err().code(),
            Some("E035")
        )
    });
    owner.with(|| assert_eq!(yielded(next(iterator)), Value::Str("é".into())));
    drop(pin);
    owner.collect();
    assert_eq!(owner.memory_stats().iterators, 0);
    owner.with(|| {
        assert_eq!(
            runtime::soplang_to_value(iterator).unwrap_err().code(),
            Some("E035")
        )
    });
    let fresh = owner.with(|| make(&Value::Str("new".into())));
    assert_ne!(iterator.payload, fresh.payload);
}

#[test]
fn unreachable_iterator_collection_cycles_are_reclaimed() {
    let owner = RuntimeContext::new();
    let (list_weak, iterator_weak) = owner.with(|| {
        let list = Rc::new(RefCell::new(Vec::new()));
        list.borrow_mut().push(Value::List(Rc::clone(&list)));
        let iterator = make(&Value::List(Rc::clone(&list)));
        let Value::Iterator(data) = runtime::soplang_to_value(iterator).unwrap() else {
            panic!("iterator")
        };
        list.borrow_mut().push(Value::Iterator(Rc::clone(&data)));
        (Rc::downgrade(&list), Rc::downgrade(&data))
    });
    owner.collect();
    assert!(list_weak.upgrade().is_none());
    assert!(iterator_weak.upgrade().is_none());
    assert_eq!(owner.memory_stats().values(), 0);
}

#[test]
fn host_iterator_keeps_snapshot_values_until_consumption_and_context_drop() {
    let owner = RuntimeContext::new();
    let (host, list_weak) = owner.with(|| {
        let list = Rc::new(RefCell::new(vec![Value::Int(7)]));
        let source = Value::List(Rc::new(RefCell::new(vec![Value::List(Rc::clone(&list))])));
        let iterator = make(&source);
        (
            runtime::soplang_to_value(iterator).unwrap(),
            Rc::downgrade(&list),
        )
    });
    owner.collect();
    assert!(list_weak.upgrade().is_some());
    drop(owner);
    assert!(list_weak.upgrade().is_none());
    let foreign = RuntimeContext::new();
    foreign.with(|| {
        let sv = runtime::value_to_soplang(&host);
        assert_eq!(
            runtime::soplang_to_value(sv).unwrap_err().code(),
            Some("E035")
        );
    });
}

#[test]
fn foreach_headers_keep_ku_contextual_and_validate_bindings() {
    for source in [
        "door ku=1 kuceli (i ku ilaa 2) {}",
        "door ilaa=[] kuceli (i ku ilaa) {}",
        "door ilaa='x' kuceli (i ku ilaa + 'y') {}",
        "kuceli (ku ku [1,2]) {}",
        "kuceli (a,b,c ku [[1,2,3]]) {}",
    ] {
        parse_source(source, None).unwrap();
    }
    for source in [
        "kuceli (a,a ku [[1,2]]) {}",
        "kuceli (a,b [1]) {}",
        "kuceli (a ku) {}",
    ] {
        assert!(parse_source(source, None).is_err(), "{source}");
    }
    let source = "door ku=1\nkuceli (item ku [ku]) { qor(item,ku) }";
    let result = check_source(
        source,
        Path::new("/tmp/foreach.sop"),
        false,
        Limits::default(),
        &|_| unreachable!(),
        &|| false,
    );
    assert!(result.error.is_none(), "{:?}", result.error);
    let index = Index::build(&result.files);
    for (name, count) in [("ku", 3), ("item", 2)] {
        let symbol = index.symbols.iter().find(|s| s.name == name).unwrap();
        assert_eq!(
            index
                .occurrences
                .iter()
                .filter(|o| o.symbol == symbol.id)
                .count(),
            count
        );
    }
}

#[test]
fn isolated_repl_sessions_keep_independent_cursors_and_failed_loops_recover() {
    let mut first = ReplSession::new(false).unwrap();
    let mut second = ReplSession::new(false).unwrap();
    for session in [&mut first, &mut second] {
        session.eval("door cursor=wareeg([1,2])", None).unwrap();
    }
    first.eval("door one=xiga(cursor).qiime", None).unwrap();
    first.eval("door two=xiga(cursor).qiime", None).unwrap();
    second.eval("door one=xiga(cursor).qiime", None).unwrap();
    assert_eq!(first.get_global("two"), Some(Value::Int(2)));
    assert_eq!(second.get_global("one"), Some(Value::Int(1)));
    let error = first.eval("kuceli (a,b ku [[1]]) {}", None).unwrap_err();
    assert_eq!(error.code(), Some("E037"));
    first
        .eval(
            "door recovered=0 kuceli (v ku [3,4]) { recovered=recovered+v }",
            None,
        )
        .unwrap();
    assert_eq!(first.get_global("recovered"), Some(Value::Int(7)));
}

#[test]
fn protocol_recursion_is_contained_on_a_small_native_stack() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let mut session = ReplSession::new(false).unwrap();
            session
                .eval(
                    r#"
            door recursive = maran
            recursive = {wareeg: hawl() { celi wareeg(recursive) }}
            door caught = been
            tijaabi { wareeg(recursive) } qabo (e, xog) { caught = xog.summad == "E037" }
            door after = xiga(wareeg([8])).qiime
        "#,
                    None,
                )
                .unwrap();
            assert_eq!(session.get_global("caught"), Some(Value::Bool(true)));
            assert_eq!(session.get_global("after"), Some(Value::Int(8)));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn foreach_respects_constants_and_function_control_boundaries() {
    for (source, code) in [
        ("madoor fixed=1 kuceli (fixed ku [2]) {}", "E020"),
        (
            "kuceli (item ku [1]) { door callback=hawl() { jooji } }",
            "E022",
        ),
    ] {
        let ast = parse_source(source, None).unwrap();
        assert_eq!(soplang::analyze(&ast).unwrap_err().code(), Some(code));
    }
    let result = check_source(
        "abn total=0 kuceli (item ku [1,2]) { total=total+item }",
        Path::new("/tmp/foreach-strict.sop"),
        true,
        Limits::default(),
        &|_| unreachable!(),
        &|| false,
    );
    assert!(result.error.is_none(), "{:?}", result.error);
}

#[test]
fn temporary_iterators_do_not_accumulate_in_repl_contexts() {
    let mut session = ReplSession::new(false).unwrap();
    session.eval("door total=0", None).unwrap();
    for _ in 0..40 {
        session
            .eval(
                "kuceli (index, value ku tirso(taxane(25))) { total=total+value }",
                None,
            )
            .unwrap();
        session.collect();
        let stats = session.memory_stats();
        assert_eq!(stats.iterators, 0);
        assert!(stats.values() < 10, "{stats:?}");
    }
    assert_eq!(session.get_global("total"), Some(Value::Int(12000)));
}

#[test]
fn snapshots_report_conflicting_host_borrows_without_unwinding_the_abi() {
    let owner = RuntimeContext::new();
    let list = Rc::new(RefCell::new(vec![Value::Int(1)]));
    let borrowed = list.borrow_mut();
    let error = owner
        .execute(|| make(&Value::List(Rc::clone(&list))))
        .unwrap_err();
    assert_eq!(error.code(), Some("E037"));
    drop(borrowed);
    owner
        .execute(|| {
            let iterator = make(&Value::List(Rc::clone(&list)));
            assert_eq!(yielded(next(iterator)), Value::Int(1));
        })
        .unwrap();
}
