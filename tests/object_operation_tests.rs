use runtime::{RuntimeContext, SoplangValue};
use soplang::{runtime, ReplSession, Value};
use std::cell::RefCell;
use std::collections::HashMap;
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
fn contract(value: SoplangValue, text: &str) -> SoplangValue {
    runtime::soplang_check_contract(
        value.tag as i64,
        value.payload,
        text.as_ptr(),
        text.len(),
        std::ptr::null(),
        0,
    )
}
fn text(value: &str) -> SoplangValue {
    runtime::soplang_str(value.as_ptr(), value.len())
}
fn identity(args: Vec<Value>) -> Result<Value, soplang::error::SoplangError> {
    Ok(args[1].clone())
}

#[test]
fn native_object_method_borrows_are_recoverable_and_existing_inserts_do_not_write() {
    let context = RuntimeContext::new();
    let object = Rc::new(RefCell::new(HashMap::from([("a".into(), Value::Int(1))])));
    let (value, a, b, callback) = context.with(|| {
        (
            runtime::value_to_soplang(&Value::Object(Rc::clone(&object))),
            text("a"),
            text("b"),
            runtime::value_to_soplang(&Value::NativeFunction(identity)),
        )
    });
    let borrowed = object.borrow_mut();
    for (name, args) in [
        ("geli", vec![a, runtime::soplang_int(2)]),
        ("aaddin", vec![callback]),
        ("hel", vec![a]),
    ] {
        assert_eq!(
            context
                .execute(|| method(value, name, &args))
                .unwrap_err()
                .code(),
            Some("E043")
        );
    }
    drop(borrowed);
    let borrowed = object.borrow();
    assert_eq!(
        context
            .execute(|| method(value, "geli", &[a, runtime::soplang_int(9)]))
            .unwrap()
            .payload,
        value.payload
    );
    assert_eq!(
        context
            .execute(|| method(value, "geli", &[b, runtime::soplang_int(9)]))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    assert_eq!(borrowed.get("a"), Some(&Value::Int(1)));
    drop(borrowed);
    let other = Rc::new(RefCell::new(HashMap::new()));
    let other_value = context.with(|| runtime::value_to_soplang(&Value::Object(Rc::clone(&other))));
    let borrowed = other.borrow_mut();
    assert_eq!(
        context
            .execute(|| method(value, "ballaadhi", &[other_value]))
            .unwrap_err()
            .code(),
        Some("E043")
    );
    drop(borrowed);
    context
        .execute(|| method(value, "aaddin", &[callback]))
        .unwrap();
}

#[test]
fn object_mutations_validate_parent_contracts_and_release_dead_dependencies() {
    let context = RuntimeContext::new();
    let child = Rc::new(RefCell::new(HashMap::from([("a".into(), Value::Int(1))])));
    let (value, parent, b, bad, other) = context.with(|| {
        let value = runtime::value_to_soplang(&Value::Object(Rc::clone(&child)));
        let parent =
            runtime::value_to_soplang(&Value::Tuple(runtime::TupleData::new(vec![Value::Object(
                Rc::clone(&child),
            )])));
        let other =
            runtime::value_to_soplang(&Value::Object(Rc::new(RefCell::new(HashMap::from([
                ("b".into(), Value::Int(2)),
                ("c".into(), Value::Str("bad".into())),
            ])))));
        (value, parent, text("b"), text("bad"), other)
    });
    let pin = context.root(parent).unwrap();
    context
        .execute(|| contract(parent, "xirmo[walax[abn]]"))
        .unwrap();
    assert_eq!(
        context
            .execute(|| method(value, "geli", &[b, bad]))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    assert_eq!(
        context
            .execute(|| method(value, "ballaadhi", &[other]))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    assert_eq!(
        *child.borrow(),
        HashMap::from([("a".into(), Value::Int(1))])
    );
    drop(pin);
    context.collect();
    assert_eq!(context.memory_stats().type_contracts, 0);
    let (b, bad) = context.with(|| (text("b"), text("bad")));
    context
        .execute(|| method(value, "geli", &[b, bad]))
        .unwrap();
    assert_eq!(child.borrow().get("b"), Some(&Value::Str("bad".into())));
}

#[test]
fn deeply_nested_paths_are_iterative_and_conflicting_path_borrows_are_errors() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let context = RuntimeContext::new();
            let root = Rc::new(RefCell::new(HashMap::from([(
                "value".into(),
                Value::Int(7),
            )])));
            root.borrow_mut()
                .insert("self".into(), Value::Object(Rc::clone(&root)));
            context.with(|| runtime::value_to_soplang(&Value::Object(Rc::clone(&root))));
            let mut segments = vec![Value::Str("self".into()); 20000];
            segments.push(Value::Str("value".into()));
            let path = Value::Tuple(runtime::TupleData::new(segments));
            assert_eq!(
                context
                    .with(|| runtime::stdlib::object_hel(Rc::clone(&root), &[path]))
                    .unwrap(),
                Value::Int(7)
            );
            let path = Rc::new(RefCell::new(vec![Value::Str("value".into())]));
            let borrowed = path.borrow_mut();
            assert_eq!(
                context
                    .with(|| runtime::stdlib::object_hel(
                        Rc::clone(&root),
                        &[Value::List(Rc::clone(&path))]
                    ))
                    .unwrap_err()
                    .code(),
                Some("E043")
            );
            drop(borrowed);
            drop(root);
            assert_eq!(context.collect().after.values(), 0);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn typed_object_callbacks_and_nested_aliases_work_across_repl_collection() {
    let mut session = ReplSession::new(false).unwrap();
    session.eval("walax[abn] values={a:1,b:2} hawl make(){door factor=3 celi hawl(qoraal key,abn value):abn{celi value*factor}} door mapper=make()",None).unwrap();
    session.collect();
    session
        .eval(
            "door result=values.aaddin(mapper) values.ballaadhi({c:3})",
            None,
        )
        .unwrap();
    assert!(
        matches!(session.get_global("result"),Some(Value::Object(result)) if *result.borrow()==HashMap::from([("a".into(),Value::Int(3)),("b".into(),Value::Int(6))]))
    );
    session.collect();
    assert_eq!(
        session
            .eval("values.geli('d','bad')", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
    session
        .eval(
            "teed[walax[abn]] parent=[values] door nested={parent:parent}.hel(['parent',0])",
            None,
        )
        .unwrap();
    assert_eq!(
        session
            .eval("nested.ballaadhi({a:'bad'})", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
}
