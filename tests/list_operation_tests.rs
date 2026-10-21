use runtime::{RuntimeContext, SoplangValue};
use soplang::{runtime, ReplSession, Value};
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
fn native_extension_and_removal_preserve_shared_contracts_and_host_borrows() {
    let context = RuntimeContext::new();
    let host = Rc::new(RefCell::new(vec![Value::Int(1)]));
    let value = context.with(|| runtime::value_to_soplang(&Value::List(Rc::clone(&host))));
    context
        .execute(|| {
            runtime::soplang_check_contract(
                value.tag as i64,
                value.payload,
                b"teed[abn]".as_ptr(),
                9,
                std::ptr::null(),
                0,
            )
        })
        .unwrap();
    let bad = context.with(|| {
        runtime::value_to_soplang(&Value::Tuple(runtime::TupleData::new(vec![
            Value::Int(2),
            Value::Str("bad".into()),
        ])))
    });
    assert_eq!(
        context
            .execute(|| method(value, "ballaadhi", &[bad]))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    assert_eq!(*host.borrow(), [Value::Int(1)]);
    let borrowed = host.borrow();
    assert_eq!(
        context
            .with(|| runtime::stdlib::list_saar(
                Rc::clone(&host),
                &[Value::Int(1), Value::Str("qiime".into())]
            ))
            .unwrap_err()
            .code(),
        Some("E039")
    );
    drop(borrowed);
    context
        .with(|| {
            runtime::stdlib::list_saar(
                Rc::clone(&host),
                &[Value::Int(1), Value::Str("qiime".into())],
            )
        })
        .unwrap();
    assert!(host.borrow().is_empty());
}

#[test]
fn adapter_cursors_retain_snapshot_members_after_the_source_is_collected() {
    let context = RuntimeContext::new();
    let (source, weak) = context.with(|| {
        let tuple = runtime::TupleData::new(vec![Value::Int(1)]);
        let weak = Rc::downgrade(&tuple);
        let source = Value::List(Rc::new(RefCell::new(vec![Value::Tuple(tuple)])));
        (runtime::value_to_soplang(&source), weak)
    });
    let cursor = context.execute(|| method(source, "tirso", &[])).unwrap();
    let pin = context.root(cursor).unwrap();
    context.collect();
    assert!(weak.upgrade().is_some());
    assert_eq!(context.memory_stats().lists, 0);
    drop(pin);
    assert_eq!(context.collect().after.values(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn sorting_callbacks_and_typed_extensions_work_across_repl_collections() {
    let mut session = ReplSession::new(false).unwrap();
    session.eval("teed[abn] values=[3,1,2] hawl make(){door factor=-1 celi hawl(x){celi x*factor}} door key=make()",None).unwrap();
    session.collect();
    session
        .eval("values.habee(key) values.ballaadhi(xirmo(4,5))", None)
        .unwrap();
    assert!(
        matches!(session.get_global("values"),Some(Value::List(values)) if *values.borrow()==[Value::Int(3),Value::Int(2),Value::Int(1),Value::Int(4),Value::Int(5)])
    );
    session.collect();
    assert_eq!(
        session
            .eval("values.ballaadhi([6,'bad'])", None)
            .unwrap_err()
            .code(),
        Some("E039")
    );
    session.eval("values.saar(4,'qiime')", None).unwrap();
}
