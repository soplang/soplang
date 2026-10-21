use runtime::{RuntimeContext, SoplangValue};
use soplang::{runtime, ReplSession, Value};
use std::cell::RefCell;
use std::rc::Rc;
fn call(name: &str, args: &[SoplangValue]) -> SoplangValue {
    let callee = runtime::soplang_get_builtin(name.as_ptr(), name.len());
    runtime::soplang_call(callee, args.as_ptr(), args.len() as i32)
}
fn draw(context: &RuntimeContext) -> u64 {
    context.execute(|| call("xul", &[])).unwrap().payload as u64
}
fn seed(context: &RuntimeContext, value: i64) {
    context
        .execute(|| call("abuur", &[runtime::soplang_int(value)]))
        .unwrap();
}

#[test]
fn random_streams_are_context_owned_and_nested_activation_restores_the_caller() {
    let first = RuntimeContext::new();
    let second = RuntimeContext::new();
    let reference = RuntimeContext::new();
    for context in [&first, &second, &reference] {
        seed(context, 0);
    }
    let expected = [draw(&reference), draw(&reference), draw(&reference)];
    assert_eq!(draw(&first), expected[0]);
    assert_eq!(draw(&second), expected[0]);
    first.with(|| {
        second.with(|| {
            call("xul", &[]);
        });
    });
    assert_eq!(draw(&first), expected[1]);
    first.collect();
    assert_eq!(draw(&first), expected[2]);
    assert_eq!(first.memory_stats().values(), 0);
}

#[test]
fn invalid_random_calls_and_conflicting_list_borrows_do_not_advance_the_stream() {
    let context = RuntimeContext::new();
    seed(&context, 17);
    let expected = draw(&context);
    seed(&context, 17);
    let values = Rc::new(RefCell::new(vec![Value::Int(1)]));
    let tagged = context.with(|| runtime::value_to_soplang(&Value::List(Rc::clone(&values))));
    let borrowed = values.borrow_mut();
    assert_eq!(
        context
            .execute(|| call("xul", &[tagged]))
            .unwrap_err()
            .code(),
        Some("E044")
    );
    drop(borrowed);
    assert_eq!(
        context
            .execute(|| call("xul", &[runtime::soplang_int(2), runtime::soplang_int(1)]))
            .unwrap_err()
            .code(),
        Some("E044")
    );
    assert_eq!(draw(&context), expected);
}

#[test]
fn numeric_functions_and_seeded_sequences_persist_across_repl_collection_and_shadowing() {
    let mut session = ReplSession::new(false).unwrap();
    session
        .eval("abuur(23) door first=xul() door saved=sin", None)
        .unwrap();
    let first = session.get_global("first").unwrap();
    session.collect();
    session.eval("abuur(23) door repeated=xul() door sin=hawl(x){celi x+10} door custom=sin(1) door original=saved(0)",None).unwrap();
    assert_eq!(session.get_global("repeated"), Some(first));
    assert_eq!(session.get_global("custom"), Some(Value::Int(11)));
    assert_eq!(session.get_global("original"), Some(Value::Float(0.0)));
    session.collect();
    session
        .eval(
            "door large=weyn(9223372036854775806,9223372036854775807)",
            None,
        )
        .unwrap();
    assert_eq!(session.get_global("large"), Some(Value::Int(i64::MAX)));
}
