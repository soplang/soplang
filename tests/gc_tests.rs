use std::cell::RefCell;
use std::rc::{Rc, Weak};

use soplang::runtime::{self, RuntimeContext, SoplangValue};
use soplang::Value;

fn cycle() -> (SoplangValue, Weak<RefCell<Vec<Value>>>) {
    let value = runtime::soplang_list_new();
    runtime::soplang_list_push(value, value);
    let Value::List(list) = runtime::soplang_to_value(value).unwrap() else {
        unreachable!()
    };
    (value, Rc::downgrade(&list))
}

#[test]
fn explicit_roots_preserve_cycles_and_stale_handles_never_alias_reused_slots() {
    let context = RuntimeContext::new();
    let (value, weak) = context.with(cycle);
    let root = context.root(value).unwrap();
    context.collect();
    assert!(weak.upgrade().is_some());
    drop(root);
    assert_eq!(context.collect().after.lists, 0);
    assert!(weak.upgrade().is_none());
    context.with(|| {
        assert_eq!(
            runtime::soplang_to_value(value).unwrap_err().code(),
            Some("E035")
        );
        let fresh = runtime::soplang_list_new();
        assert_ne!(fresh.payload, value.payload);
        assert!(runtime::soplang_to_value(fresh).is_ok());
        assert!(runtime::soplang_to_value(value).is_err());
    });
}

#[test]
fn host_values_and_borrows_preserve_live_graphs_without_blocking_other_reclamation() {
    let context = RuntimeContext::new();
    let (value, weak) = context.with(cycle);
    let host = context.with(|| runtime::soplang_to_value(value).unwrap());
    let Value::List(list) = &host else {
        unreachable!()
    };
    let borrowed = list.borrow_mut();
    let (_, garbage) = context.with(cycle);
    let report = context.collect();
    assert_eq!(report.deferred, 1);
    assert!(garbage.upgrade().is_none());
    assert_eq!(borrowed.len(), 1);
    drop(borrowed);
    context.collect();
    assert!(weak.upgrade().is_some());
    drop(host);
    context.collect();
    assert!(weak.upgrade().is_none());
}

#[test]
fn collection_identity_is_stable_and_conversion_does_not_accumulate_aliases() {
    let context = RuntimeContext::new();
    context.with(|| {
        let list = runtime::soplang_list_new();
        let object = runtime::soplang_object_new();
        let list_value = runtime::soplang_to_value(list).unwrap();
        let object_value = runtime::soplang_to_value(object).unwrap();
        for _ in 0..10000 {
            assert_eq!(runtime::value_to_soplang(&list_value).payload, list.payload);
            assert_eq!(
                runtime::value_to_soplang(&object_value).payload,
                object.payload
            );
        }
        assert_eq!(context.memory_stats().lists, 1);
        assert_eq!(context.memory_stats().objects, 1);
    });
    assert_eq!(context.collect().after.values(), 0);
}

#[test]
fn strings_errors_and_generated_root_layouts_are_traced() {
    let context = RuntimeContext::new();
    context.with(|| {
        let text = runtime::soplang_str("kept".as_ptr(), 4);
        let tags = [text.tag as i64];
        let payloads = [text.payload];
        runtime::soplang_enter_roots(tags.as_ptr(), payloads.as_ptr(), 1, 1);
        context.collect();
        assert!(runtime::soplang_to_value(text).is_ok());
        runtime::soplang_leave_roots();
        context.collect();
        assert!(runtime::soplang_to_value(text).is_err());

        let text = runtime::soplang_str("interleaved".as_ptr(), 11);
        let slots = [text.tag as i64, text.payload];
        runtime::soplang_enter_roots(slots.as_ptr(), unsafe { slots.as_ptr().add(1) }, 1, 2);
        context.collect();
        assert!(runtime::soplang_to_value(text).is_ok());
        runtime::soplang_leave_roots();
        let error = runtime::value_to_soplang(
            &runtime::stdlib::builtin_khalad(vec![Value::Str("error".into())]).unwrap(),
        );
        let pin = context.root(error).unwrap();
        context.collect();
        assert!(runtime::soplang_to_value(error).is_ok());
        drop(pin);
        assert_eq!(context.collect().after.values(), 0);
    });
}

#[test]
fn host_closures_keep_captures_and_unreachable_closure_cycles_are_reclaimed() {
    extern "C" fn unused_entry() {}
    let context = RuntimeContext::new();
    let (closure, weak, captured) = context.with(|| {
        runtime::soplang_register_user_fn("f".as_ptr(), 1, unused_entry as *const u8, 1);
        let captured = runtime::soplang_list_new();
        let closure = runtime::soplang_make_closure("f".as_ptr(), 1, 1, &captured);
        runtime::soplang_list_push(captured, closure);
        let Value::Closure(data) = runtime::soplang_to_value(closure).unwrap() else {
            unreachable!()
        };
        (closure, Rc::downgrade(&data), captured)
    });
    let host = context.with(|| runtime::soplang_to_value(closure).unwrap());
    context.collect();
    context.with(|| assert!(runtime::soplang_to_value(captured).is_ok()));
    assert!(weak.upgrade().is_some());
    drop(host);
    let report = context.collect();
    assert_eq!(report.after.closures, 0);
    assert_eq!(report.after.lists, 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn context_teardown_breaks_unreachable_cycles() {
    let weak = {
        let context = RuntimeContext::new();
        context.with(cycle).1
    };
    assert!(weak.upgrade().is_none());
}

#[test]
fn repl_heap_remains_bounded_across_successes_and_failures() {
    let mut session = soplang::ReplSession::new(false).unwrap();
    session.eval("door result = 0", None).unwrap();
    for iteration in 0..40 {
        let result = session.eval("door garbage = [] garbage.geli(garbage) garbage = maran result = result + 1 tuur \"expected\"", None);
        assert!(result.is_err());
        let report = session.collect();
        assert_eq!(report.after.active_frames, 0, "iteration {iteration}");
        assert_eq!(report.after.lists, 0, "iteration {iteration}");
        assert!(
            report.after.values() < 8,
            "iteration {iteration}: {:?}",
            report.after
        );
    }
    assert_eq!(session.get_global("result"), Some(Value::Int(40)));
    assert!(session.memory_stats().collections >= 40);
}

#[test]
fn automatic_collection_bounds_heap_values_in_long_running_programs() {
    let mut session = soplang::ReplSession::new(false).unwrap();
    session
        .eval(include_str!("fixtures/garbage_collection.sop"), None)
        .unwrap();
    let stats = session.memory_stats();
    assert!(stats.collections > 10, "{stats:?}");
    assert!(stats.reclaimed > 20000, "{stats:?}");
    assert!(stats.values() < 2500, "{stats:?}");
    let report = session.collect();
    assert!(report.after.values() < 50, "{report:?}");
    assert_eq!(report.after.active_frames, 0);
}
