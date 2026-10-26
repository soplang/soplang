use runtime::{RuntimeContext, SoplangValue, TupleData};
use soplang::{analyze, frontend::parse_source, runtime, ReplSession, Value};
use std::cell::RefCell;
use std::rc::Rc;

fn tuple(values: Vec<Value>) -> SoplangValue {
    runtime::value_to_soplang(&Value::Tuple(TupleData::new(values)))
}
fn check(value: SoplangValue, desc: &str) -> SoplangValue {
    runtime::soplang_check_contract(
        value.tag as i64,
        value.payload,
        desc.as_ptr(),
        desc.len(),
        std::ptr::null(),
        0,
    )
}

#[test]
fn tuple_annotations_validate_shapes_and_preserve_contextual_identifiers() {
    for source in [
        "xirmo[] x=xirmo()",
        "madoor xirmo[abn] x=xirmo(1)",
        "hawl f(xirmo[abn|qoraal]? x=maran):xirmo[abn|qoraal]?{celi x}",
    ] {
        let stmts = parse_source(source, None).unwrap();
        soplang::semantic::analyze_with_options(
            &stmts,
            soplang::semantic::AnalyzeOptions {
                strict: true,
                repl: false,
            },
        )
        .unwrap();
    }
    for source in ["xirmo[abn,] x=xirmo()", "hawl f(xirmo[abn] *args){}"] {
        assert!(parse_source(source, None).is_err(), "{source}");
    }
    for (source, code) in [
        ("xirmo[Missing] x=xirmo()", "E021"),
        ("xirmo[abn] x=xirmo(1) xirmo[qoraal] y=x", "E022"),
        ("xirmo x=[]", "E022"),
        ("qaab xirmo{}", "E020"),
    ] {
        let error = analyze(&parse_source(source, None).unwrap()).unwrap_err();
        assert_eq!(error.code(), Some(code), "{source}: {error}");
    }
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("door xirmo=[1] xirmo[0]=2 door out=0 xirmo[0] out=3", None)
        .unwrap();
    assert!(
        matches!(session.get_global("xirmo"),Some(Value::List(values)) if values.borrow()[0]==Value::Int(2))
    );
}

#[test]
fn tuple_handles_are_context_owned_stable_and_rejected_after_collection() {
    let owner = RuntimeContext::new();
    let foreign = RuntimeContext::new();
    let value = owner.with(|| tuple(vec![Value::Int(1)]));
    assert_eq!(value.tag, runtime::TAG_TUPLE);
    let pin = owner.root(value).unwrap();
    owner.with(|| {
        let host = runtime::soplang_to_value(value).unwrap();
        for _ in 0..1000 {
            assert_eq!(runtime::value_to_soplang(&host).payload, value.payload);
        }
    });
    assert_eq!(owner.memory_stats().tuples, 1);
    assert_eq!(
        foreign
            .execute(|| runtime::soplang_get_index(value, runtime::soplang_int(0)))
            .unwrap_err()
            .code(),
        Some("E035")
    );
    owner.collect();
    drop(pin);
    assert_eq!(owner.collect().after.tuples, 0);
    owner.with(|| {
        assert_eq!(
            runtime::soplang_to_value(value).unwrap_err().code(),
            Some("E035")
        )
    });
}

#[test]
fn tuple_cycles_and_weak_nested_contracts_are_reclaimed() {
    let context = RuntimeContext::new();
    let (value, weak_tuple, weak_list) = context.with(|| {
        let list = Rc::new(RefCell::new(Vec::new()));
        let data = TupleData::new(vec![Value::List(Rc::clone(&list))]);
        list.borrow_mut().push(Value::Tuple(Rc::clone(&data)));
        (
            runtime::value_to_soplang(&Value::Tuple(Rc::clone(&data))),
            Rc::downgrade(&data),
            Rc::downgrade(&list),
        )
    });
    let pin = context.root(value).unwrap();
    context
        .execute(|| check(value, "xirmo[teed[xirmo]]"))
        .unwrap();
    context.collect();
    assert!(weak_tuple.upgrade().is_some() && weak_list.upgrade().is_some());
    assert_eq!(context.memory_stats().type_contracts, 1);
    drop(pin);
    let report = context.collect();
    assert_eq!(report.after.values(), 0);
    assert_eq!(report.after.type_contracts, 0);
    assert!(weak_tuple.upgrade().is_none() && weak_list.upgrade().is_none());
}

#[test]
fn tuples_retain_nested_contracts_across_repl_and_release_dead_parent_guards() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval(
            "xirmo[teed[abn]] holder=xirmo([1]) door child=holder[0]",
            None,
        )
        .unwrap();
    session.collect();
    assert_eq!(
        session.eval("child.geli('bad')", None).unwrap_err().code(),
        Some("E039")
    );
    session.eval("holder=xirmo([2])", None).unwrap();
    session.collect();
    session.eval("child.geli('detached')", None).unwrap();
    assert_eq!(
        session
            .eval("holder[0].geli('bad')", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
}

#[test]
fn deep_tuple_equality_display_and_collection_do_not_use_unbounded_call_stacks() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let context = RuntimeContext::new();
            let make = || {
                let mut value = Value::Null;
                for _ in 0..12000 {
                    value = Value::Tuple(TupleData::new(vec![value]));
                }
                context.with(|| runtime::value_to_soplang(&value))
            };
            let a = make();
            let b = make();
            let left = context.with(|| runtime::soplang_to_value(a).unwrap());
            let right = context.with(|| runtime::soplang_to_value(b).unwrap());
            assert_eq!(left, right);
            assert!(runtime::value_to_string(&left).contains("..."));
            drop(left);
            drop(right);
            assert_eq!(context.collect().after.values(), 0);
        })
        .unwrap()
        .join()
        .unwrap();
}
