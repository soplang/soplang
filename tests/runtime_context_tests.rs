//! Runtime ownership, nested activation, and independent sessions on one thread.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use soplang::runtime::{self, RuntimeContext};
use soplang::{ReplSession, Value};

fn store(name: &str, value: Value) {
    let value = runtime::value_to_soplang(&value);
    runtime::soplang_store_global(name.as_ptr(), name.len(), value.tag as i64, value.payload);
}

fn load(name: &str) -> Value {
    let value = runtime::soplang_get_builtin(name.as_ptr(), name.len());
    runtime::soplang_to_value(value).unwrap()
}

#[test]
fn nested_contexts_restore_globals_and_pending_errors_after_a_panic() {
    let outer = RuntimeContext::new();
    let inner = RuntimeContext::new();
    outer.with(|| {
        store("name", Value::Str("outer".into()));
        runtime::soplang_try_begin();
        runtime::soplang_div(runtime::soplang_int(1), runtime::soplang_int(0));
        assert_eq!(runtime::soplang_has_pending_error(), 1);

        let result = catch_unwind(AssertUnwindSafe(|| {
            inner.with(|| {
                assert_eq!(runtime::soplang_has_pending_error(), 0);
                store("name", Value::Str("inner".into()));
                assert_eq!(load("name"), Value::Str("inner".into()));
                panic!("exercise Rust scope restoration");
            });
        }));
        assert!(result.is_err());
        assert_eq!(load("name"), Value::Str("outer".into()));
        assert_eq!(runtime::soplang_has_pending_error(), 1);
        let error = runtime::soplang_get_caught_error();
        assert_eq!(
            runtime::soplang_to_value(error).unwrap(),
            Value::Str("Tiro looma qaybin karo eber.".into())
        );
        runtime::soplang_try_end();
    });
    assert_eq!(inner.get_global("name"), Some(Value::Str("inner".into())));
}

#[test]
fn constants_belong_to_their_context() {
    let first = RuntimeContext::new();
    let second = RuntimeContext::new();
    first.with(|| {
        store("fixed", Value::Int(1));
        runtime::soplang_mark_const("fixed".as_ptr(), 5);
    });
    second.with(|| {
        runtime::soplang_try_begin();
        store("fixed", Value::Int(2));
        assert_eq!(runtime::soplang_has_pending_error(), 0);
        runtime::soplang_try_end();
    });
    first.with(|| {
        runtime::soplang_try_begin();
        store("fixed", Value::Int(3));
        assert_eq!(runtime::soplang_has_pending_error(), 1);
        runtime::soplang_try_end();
    });
    assert_eq!(first.get_global("fixed"), Some(Value::Int(1)));
    assert_eq!(second.get_global("fixed"), Some(Value::Int(2)));
}

#[test]
fn dropping_a_context_releases_its_acyclic_collections() {
    let context = RuntimeContext::new();
    let weak = context.with(|| {
        let list = runtime::soplang_list_new();
        runtime::soplang_list_push(list, runtime::soplang_int(7));
        runtime::store_global("items", list);
        let Value::List(items) = runtime::soplang_to_value(list).unwrap() else {
            panic!("expected list");
        };
        Rc::downgrade(&items)
    });
    assert!(weak.upgrade().is_some());
    drop(context);
    assert!(weak.upgrade().is_none());
}

#[test]
fn repl_sessions_keep_their_own_globals_classes_and_closures() {
    let mut first = ReplSession::new(false).unwrap();
    let mut second = ReplSession::new(false).unwrap();
    for (session, initial, captured) in [(&mut first, 1, 10), (&mut second, 100, 1000)] {
        session
            .eval(
                &format!(
                    r#"
                    door total = {initial}
                    door items = [total]
                    qaab Sanduuq {{ hawl hel() {{ celi total }} }}
                    door box = cusub Sanduuq()
                    hawl make(base) {{ celi hawl() {{ celi base + total }} }}
                    door read = make({captured})
                    "#
                ),
                None,
            )
            .unwrap();
    }
    first
        .eval("items.geli(2) door observed = read() + box.hel()", None)
        .unwrap();
    second
        .eval("door observed = read() + box.hel()", None)
        .unwrap();
    assert_eq!(first.get_global("observed"), Some(Value::Int(12)));
    assert_eq!(second.get_global("observed"), Some(Value::Int(1200)));
    assert_eq!(second.get_global("items").unwrap().to_string(), "[100]");

    drop(second);
    first
        .eval("total = 5 observed = read() + box.hel()", None)
        .unwrap();
    assert_eq!(first.get_global("observed"), Some(Value::Int(20)));
    assert_eq!(first.get_global("items").unwrap().to_string(), "[1, 2]");
    assert!(first.get_global("missing").is_none());
}

#[test]
fn independent_programs_preserve_the_embedding_context() {
    let caller = RuntimeContext::new();
    caller.with(|| {
        store("name", Value::Str("host".into()));
        runtime::soplang_try_begin();
        for _ in 0..3 {
            soplang::run_source(
                "madoor name = 1 hawl read() { celi name } door value = read()",
                None,
                false,
                false,
                false,
            )
            .unwrap();
            assert_eq!(load("name"), Value::Str("host".into()));
            assert_eq!(runtime::soplang_has_pending_error(), 0);
        }
        runtime::soplang_try_end();
    });
    assert!(caller.get_global("read").is_none());
    assert!(caller.get_global("value").is_none());
}

#[test]
fn foreign_and_malformed_handles_are_rejected_without_aliasing() {
    let owner = RuntimeContext::new();
    let foreign = RuntimeContext::new();
    let values = owner.with(|| {
        vec![
            runtime::soplang_str("owner".as_ptr(), 5),
            runtime::soplang_list_new(),
            runtime::soplang_object_new(),
            runtime::soplang_get_builtin("qor".as_ptr(), 3),
        ]
    });
    foreign.with(|| {
        let other = [
            runtime::soplang_str("foreign".as_ptr(), 7),
            runtime::soplang_list_new(),
            runtime::soplang_object_new(),
            runtime::soplang_get_builtin("qor".as_ptr(), 3),
        ];
        for (owned, local) in values.iter().zip(other) {
            assert_ne!(owned.payload, local.payload);
            assert!(runtime::soplang_to_value(local).is_ok());
            let error = runtime::soplang_to_value(*owned).unwrap_err();
            assert_eq!(error.code(), Some("E035"));
        }
        for tag in [
            runtime::TAG_STR,
            runtime::TAG_LIST,
            runtime::TAG_OBJECT,
            runtime::TAG_FUNC,
            runtime::TAG_CLOSURE,
        ] {
            for payload in [0, -1, i64::MIN, i64::MAX] {
                let value = runtime::SoplangValue {
                    tag,
                    _pad: [0; 7],
                    payload,
                };
                assert_eq!(
                    runtime::soplang_to_value(value).unwrap_err().code(),
                    Some("E035")
                );
            }
        }
    });
    drop(owner);
    foreign.with(|| {
        for value in values {
            assert_eq!(
                runtime::soplang_to_value(value).unwrap_err().code(),
                Some("E035")
            );
        }
    });
}

#[test]
fn stale_operands_and_callables_produce_catchable_runtime_errors() {
    let stale = RuntimeContext::new().with(|| {
        [
            runtime::soplang_str("expired".as_ptr(), 7),
            runtime::soplang_get_builtin("qor".as_ptr(), 3),
        ]
    });
    let receiver = RuntimeContext::new();
    receiver.with(|| {
        for value in stale {
            runtime::soplang_try_begin();
            let result = runtime::soplang_call(value, std::ptr::null(), 0);
            assert_eq!(result.tag, 8);
            assert_eq!(runtime::soplang_has_pending_error(), 1);
            let error = runtime::soplang_get_caught_error();
            assert_eq!(
                runtime::soplang_to_value(error).unwrap(),
                Value::Str("Qiimahan ma jiro ama wuxuu ka tirsan yahay fulin kale.".into())
            );
            runtime::soplang_try_end();
        }
        runtime::soplang_try_begin();
        let result = runtime::soplang_add(stale[0], runtime::soplang_int(1));
        assert_eq!(result.tag, 8);
        assert_eq!(runtime::soplang_has_pending_error(), 1);
        runtime::soplang_try_end();

        // An otherwise valid callable must reject foreign arguments too.
        runtime::soplang_try_begin();
        let callee = runtime::soplang_get_builtin("nooc".as_ptr(), 4);
        let result = runtime::soplang_call(callee, stale.as_ptr(), 1);
        assert_eq!(result.tag, 8);
        assert_eq!(runtime::soplang_has_pending_error(), 1);
        runtime::soplang_try_end();

        runtime::soplang_try_begin();
        let result = runtime::soplang_capture_get(i64::MAX, 0);
        assert_eq!(result.tag, 8);
        assert_eq!(runtime::soplang_has_pending_error(), 1);
        runtime::soplang_try_end();
    });
}
