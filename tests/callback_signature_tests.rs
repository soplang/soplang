use soplang::runtime::{self, RuntimeContext, Value};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static CALLS: AtomicUsize = AtomicUsize::new(0);
fn host_function(values: Vec<Value>) -> Result<Value, soplang::SoplangError> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(Value::Int(values.len() as i64))
}

#[test]
fn unknown_host_callbacks_are_never_probed_for_signature_information() {
    let context = RuntimeContext::new();
    context.with(|| {
        let callback = runtime::value_to_soplang(&Value::NativeFunction(host_function));
        RuntimeContext::new().with(|| {
            assert_eq!(
                runtime::soplang_to_value(callback).unwrap_err().code(),
                Some("E035")
            );
        });
        for _ in 0..100 {
            assert_eq!(
                runtime::value_to_soplang(&Value::NativeFunction(host_function)).payload,
                callback.payload
            );
        }
        assert_eq!(CALLS.load(Ordering::SeqCst), 0);
        for values in [vec![], vec![Value::Int(1), Value::Int(2)]] {
            let list = runtime::value_to_soplang(&Value::List(Rc::new(RefCell::new(values))));
            let result = runtime::soplang_call_method(list, "aaddin".as_ptr(), 6, &callback, 1);
            let error = runtime::take_pending_error();
            assert!(error.is_none(), "{error:?}");
            let Value::List(result) = runtime::soplang_to_value(result).unwrap() else {
                panic!("list")
            };
            assert!(result.borrow().iter().all(|value| *value == Value::Int(1)));
        }
    });
    assert_eq!(CALLS.load(Ordering::SeqCst), 2);
}
