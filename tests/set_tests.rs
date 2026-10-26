use runtime::{RuntimeContext, SetData, SoplangValue, TupleData};
use soplang::{analyze, frontend::parse_source, runtime, ReplSession, Value};
use std::cell::RefCell;
use std::rc::Rc;

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
fn sets_have_stable_owned_handles_and_trace_snapshot_tuple_keys() {
    let context = RuntimeContext::new();
    let foreign = RuntimeContext::new();
    let (value, weak) = context.with(|| {
        let tuple = TupleData::new(vec![Value::Str("owned".into())]);
        let weak = Rc::downgrade(&tuple);
        let set = Value::Set(SetData::new(vec![Value::Tuple(tuple)]).unwrap());
        let tagged = runtime::value_to_soplang(&set);
        assert_eq!(tagged.tag, runtime::TAG_SET);
        for _ in 0..1000 {
            assert_eq!(runtime::value_to_soplang(&set).payload, tagged.payload);
        }
        (tagged, weak)
    });
    let pin = context.root(value).unwrap();
    assert_eq!(context.memory_stats().sets, 1);
    assert_eq!(
        foreign
            .execute(|| method(value, "dherer", &[]))
            .unwrap_err()
            .code(),
        Some("E035")
    );
    let cursor = context.execute(|| method(value, "wareeg", &[])).unwrap();
    let cursor_pin = context.root(cursor).unwrap();
    context.collect();
    assert!(weak.upgrade().is_some());
    assert_eq!(context.memory_stats().tuples, 1);
    drop(pin);
    let report = context.collect();
    assert_eq!(report.after.sets, 0);
    assert!(weak.upgrade().is_some());
    assert_eq!(
        context
            .execute(|| method(value, "dherer", &[]))
            .unwrap_err()
            .code(),
        Some("E035")
    );
    drop(cursor_pin);
    assert_eq!(context.collect().after.values(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn set_keys_are_immutable_reflexive_and_preserve_numeric_kinds() {
    let set = SetData::new(vec![
        Value::Float(-0.0),
        Value::Float(0.0),
        Value::Int(0),
        Value::Bool(false),
        Value::Null,
        Value::Float(f64::INFINITY),
    ])
    .unwrap();
    assert_eq!(set.len(), 5);
    let other = SetData::new(set.values().into_iter().rev().collect()).unwrap();
    assert_eq!(Value::Set(set), Value::Set(other));
    for value in [
        Value::Float(f64::NAN),
        Value::List(Rc::new(RefCell::new(vec![]))),
        Value::Object(Rc::new(RefCell::new(Default::default()))),
        Value::Bytes(Rc::new(RefCell::new(vec![]))),
        Value::Set(SetData::new(vec![]).unwrap()),
        Value::Tuple(TupleData::new(vec![Value::Float(f64::NAN)])),
        Value::Function(runtime::value::FunctionId(1)),
        Value::NativeFunction(runtime::stdlib::builtin_teed),
        Value::Error(Rc::new(soplang::error::runtime_error("test", 0, 0))),
    ] {
        match SetData::new(vec![value]) {
            Err(error) => assert_eq!(error.code(), Some("E041")),
            Ok(_) => panic!("mutable or non-reflexive key was accepted"),
        }
    }
}

#[test]
fn set_annotations_remain_contextual_and_support_nested_unions() {
    for source in [
        "urur x=urur()",
        "hawl f(urur? x=maran):urur?{celi x}",
        "xirmo[urur] x=xirmo(urur())",
    ] {
        let ast = parse_source(source, None).unwrap();
        soplang::semantic::analyze_with_options(
            &ast,
            soplang::semantic::AnalyzeOptions {
                strict: true,
                repl: false,
            },
        )
        .unwrap();
    }
    for (source, code) in [("qaab urur{}", "E020"), ("urur x=[]", "E022")] {
        assert_eq!(
            analyze(&parse_source(source, None).unwrap())
                .unwrap_err()
                .code(),
            Some(code)
        );
    }
    let mut session = ReplSession::new(false).unwrap();
    session.eval("door urur=[1] urur[0]=2", None).unwrap();
    assert!(
        matches!(session.get_global("urur"), Some(Value::List(values)) if values.borrow()[0]==Value::Int(2))
    );
}

#[test]
fn overly_deep_tuple_keys_fail_without_exhausting_small_stacks() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let context = RuntimeContext::new();
            let mut value = Value::Null;
            for _ in 0..3000 {
                value = Value::Tuple(TupleData::new(vec![value]));
            }
            context.with(|| runtime::value_to_soplang(&value));
            assert!(SetData::new(vec![value.clone()]).is_err());
            drop(value);
            assert_eq!(context.collect().after.values(), 0);
        })
        .unwrap()
        .join()
        .unwrap();
}
