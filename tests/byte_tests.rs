use std::cell::RefCell;
use std::rc::Rc;

use runtime::{RuntimeContext, SoplangValue};
use soplang::{analyze, frontend::parse_source, runtime, ReplSession, Value};

fn tagged(bytes: &[u8]) -> SoplangValue {
    runtime::value_to_soplang(&Value::Bytes(Rc::new(RefCell::new(bytes.to_vec()))))
}

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
fn byte_handles_preserve_identity_and_reject_foreign_and_collected_values() {
    let owner = RuntimeContext::new();
    let foreign = RuntimeContext::new();
    let value = owner.with(|| tagged(&[0, 255]));
    assert_eq!(value.tag, runtime::TAG_BYTES);
    let root = owner.root(value).unwrap();
    owner.with(|| {
        let bytes = runtime::soplang_to_value(value).unwrap();
        for _ in 0..1000 {
            assert_eq!(runtime::value_to_soplang(&bytes).payload, value.payload);
        }
        assert_eq!(owner.memory_stats().bytes, 1);
    });
    foreign.with(|| {
        assert_eq!(
            runtime::soplang_to_value(value).unwrap_err().code(),
            Some("E035")
        )
    });
    owner.collect();
    owner.with(|| assert!(runtime::soplang_to_value(value).is_ok()));
    drop(root);
    assert_eq!(owner.collect().after.bytes, 0);
    owner.with(|| {
        assert_eq!(
            runtime::soplang_to_value(value).unwrap_err().code(),
            Some("E035")
        );
        assert_ne!(tagged(&[7]).payload, value.payload);
    });
}

#[test]
fn host_nested_bytes_are_traced_and_outlive_their_context_as_plain_data() {
    let context = RuntimeContext::new();
    let data = Rc::new(RefCell::new(vec![1, 2, 3]));
    let weak = Rc::downgrade(&data);
    let nested = Value::List(Rc::new(RefCell::new(vec![Value::Bytes(Rc::clone(&data))])));
    let handle = context.with(|| runtime::value_to_soplang(&nested));
    drop(data);
    assert_eq!(context.collect().after.bytes, 1);
    drop(nested);
    assert_eq!(context.collect().after.bytes, 0);
    assert!(weak.upgrade().is_none());
    context.with(|| assert!(runtime::soplang_to_value(handle).is_err()));
    let host = context.with(|| runtime::soplang_to_value(tagged(&[4])).unwrap());
    drop(context);
    let Value::Bytes(bytes) = host else {
        panic!("bytes")
    };
    assert_eq!(*bytes.borrow(), [4]);
}

#[test]
fn conflicting_host_borrows_return_errors_without_panicking_across_the_abi() {
    let context = RuntimeContext::new();
    let value = context.with(|| tagged(&[65]));
    let Value::Bytes(data) = context.with(|| runtime::soplang_to_value(value).unwrap()) else {
        panic!("bytes")
    };
    let read = data.borrow();
    let number = runtime::value_to_soplang(&Value::Int(66));
    assert_eq!(
        context
            .execute(|| method(value, "geli", &[number]))
            .unwrap_err()
            .code(),
        Some("E038")
    );
    drop(read);
    let write = data.borrow_mut();
    for name in ["qoraal", "dherer", "nuqul", "wareeg"] {
        assert_eq!(
            context
                .execute(|| method(value, name, &[]))
                .unwrap_err()
                .code(),
            Some("E038")
        );
    }
    context.collect();
    assert_eq!(*write, [65]);
    drop(write);
    context
        .execute(|| method(value, "geli", &[number]))
        .unwrap();
    assert_eq!(*data.borrow(), [65, 66]);
}

#[test]
fn byte_annotations_check_variables_parameters_returns_and_editor_types() {
    for source in [
        "baytyo data=[1]",
        "hawl f(baytyo data){} f([1])",
        "hawl f():baytyo {celi 'x'}",
    ] {
        let error = analyze(&parse_source(source, None).unwrap()).unwrap_err();
        assert_eq!(error.code(), Some("E022"), "{source}: {error}");
    }
    let error = analyze(&parse_source("qaab baytyo {}", None).unwrap()).unwrap_err();
    assert_eq!(error.code(), Some("E020"));
    let stmts=parse_source("baytyo data=baytyo() madoor baytyo fixed=baytyo([1]) hawl f(baytyo a=baytyo()):baytyo{celi a}", None).unwrap();
    soplang::semantic::analyze_with_options(
        &stmts,
        soplang::semantic::AnalyzeOptions {
            strict: true,
            repl: false,
        },
    )
    .unwrap();
    assert_eq!(
        soplang::analysis::index::type_name(&soplang::frontend::ast::TypeAnnotation::Bytes),
        "baytyo"
    );
}

#[test]
fn byte_values_survive_repl_collection_and_recover_from_utf8_failures() {
    let mut session = ReplSession::new(false).unwrap();
    session.eval("baytyo data=baytyo('é')", None).unwrap();
    session.collect();
    session.eval("data[0]=255", None).unwrap();
    let error = session.eval("qoraal(data)", None).unwrap_err();
    assert_eq!(error.code(), Some("E038"));
    session
        .eval("data[0]=195 door result=qoraal(data)", None)
        .unwrap();
    assert_eq!(session.get_global("result"), Some(Value::Str("é".into())));
}

#[test]
fn unreachable_collection_cycles_release_their_byte_storage() {
    let context = RuntimeContext::new();
    let (handle, weak) = context.with(|| {
        let bytes = Rc::new(RefCell::new(vec![7]));
        let weak = Rc::downgrade(&bytes);
        let list = Value::List(Rc::new(RefCell::new(vec![Value::Bytes(bytes)])));
        let Value::List(items) = &list else {
            unreachable!()
        };
        items.borrow_mut().push(list.clone());
        (runtime::value_to_soplang(&list), weak)
    });
    let root = context.root(handle).unwrap();
    assert_eq!(context.collect().after.bytes, 1);
    assert!(weak.upgrade().is_some());
    drop(root);
    assert_eq!(context.collect().after.values(), 0);
    assert!(weak.upgrade().is_none());
}
