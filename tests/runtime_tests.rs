//! Phase 3 (COMPILER_PLAN): Runtime library tests.

use std::cell::RefCell;
use std::rc::Rc;

use soplang::runtime::{
    soplang_add, soplang_call, soplang_eq, soplang_float, soplang_get_builtin, soplang_get_index,
    soplang_get_prop, soplang_int, soplang_list_new, soplang_list_push, soplang_lt, soplang_mod,
    soplang_mul, soplang_ne, soplang_null, soplang_object_new, soplang_set_prop, soplang_str,
    soplang_sub, soplang_to_value, TAG_BOOL, TAG_FLOAT, TAG_INT, TAG_LIST,
};
use soplang::runtime::stdlib::{
    builtin_daji, builtin_dherer, builtin_kor, list_habee, string_jar, string_raadi,
};
use soplang::Value;

fn cstr(s: &str) -> (*const u8, usize) {
    (s.as_ptr(), s.len())
}

#[test]
fn test_runtime_primitives() {
    let n = soplang_int(42);
    assert_eq!(n.tag, TAG_INT);
    assert_eq!(n.payload, 42);

    let x = soplang_float(3.14);
    assert_eq!(x.tag, TAG_FLOAT);

    let null = soplang_null();
    assert_eq!(null.tag, 0);
}

#[test]
fn test_runtime_arithmetic() {
    let a = soplang_int(10);
    let b = soplang_int(3);
    let sum = soplang_add(a, b);
    let v = soplang_to_value(sum).unwrap();
    assert!(matches!(v, soplang::Value::Int(13)));

    let diff = soplang_sub(a, b);
    let v = soplang_to_value(diff).unwrap();
    assert!(matches!(v, soplang::Value::Int(7)));

    let prod = soplang_mul(a, b);
    let v = soplang_to_value(prod).unwrap();
    assert!(matches!(v, soplang::Value::Int(30)));

    let m = soplang_mod(a, b);
    let v = soplang_to_value(m).unwrap();
    assert!(matches!(v, soplang::Value::Int(1)));
}

#[test]
fn test_runtime_comparison() {
    let a = soplang_int(5);
    let b = soplang_int(10);
    let eq = soplang_eq(a, b);
    assert_eq!(eq.tag, TAG_BOOL);
    assert_eq!(eq.payload, 0); // false

    let ne = soplang_ne(a, b);
    assert_eq!(ne.payload, 1); // true

    let lt = soplang_lt(a, b);
    assert_eq!(lt.payload, 1); // true
}

#[test]
fn test_runtime_string_concat() {
    let s1 = soplang_str("hello".as_ptr(), 5);
    let s2 = soplang_str(" world".as_ptr(), 6);
    let cat = soplang_add(s1, s2);
    let v = soplang_to_value(cat).unwrap();
    if let soplang::Value::Str(s) = v {
        assert_eq!(s, "hello world");
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_runtime_list_ops() {
    let lst = soplang_list_new();
    assert_eq!(lst.tag, TAG_LIST);

    let one = soplang_int(1);
    let two = soplang_int(2);
    let _ = soplang_list_push(lst, one);
    let lst2 = soplang_list_push(lst, two);

    let idx0 = soplang_get_index(lst2, soplang_int(0));
    let v = soplang_to_value(idx0).unwrap();
    assert!(matches!(v, soplang::Value::Int(1)));

    let idx1 = soplang_get_index(lst2, soplang_int(1));
    let v = soplang_to_value(idx1).unwrap();
    assert!(matches!(v, soplang::Value::Int(2)));
}

#[test]
fn test_runtime_object_ops() {
    let obj = soplang_object_new();
    let (name_ptr, name_len) = cstr("x");
    let val = soplang_int(42);
    let _ = soplang_set_prop(obj, name_ptr, name_len, val);

    let got = soplang_get_prop(obj, name_ptr, name_len);
    let v = soplang_to_value(got).unwrap();
    assert!(matches!(v, soplang::Value::Int(42)));
}

#[test]
fn test_runtime_builtin_call() {
    let (name_ptr, name_len) = cstr("qor");
    let qor = soplang_get_builtin(name_ptr, name_len);

    let arg = soplang_str("test".as_ptr(), 4);
    let args = [arg];
    let result = soplang_call(qor, args.as_ptr(), 1);
    let v = soplang_to_value(result).unwrap();
    if let soplang::Value::Str(s) = v {
        assert_eq!(s, "test");
    } else {
        panic!("expected string from qor");
    }
}

#[test]
fn test_rounding_functions_return_integers() {
    assert_eq!(builtin_daji(vec![Value::Float(4.9)]).unwrap(), Value::Int(4));
    assert_eq!(builtin_daji(vec![Value::Float(-4.1)]).unwrap(), Value::Int(-5));
    assert_eq!(builtin_kor(vec![Value::Float(4.1)]).unwrap(), Value::Int(5));
    assert_eq!(builtin_kor(vec![Value::Float(-4.9)]).unwrap(), Value::Int(-4));
}

#[test]
fn test_string_operations_use_character_indices() {
    let text = Value::Str("éSoomaali".to_string());
    assert_eq!(builtin_dherer(vec![text]).unwrap(), Value::Int(9));
    assert_eq!(
        string_jar("éSoomaali".to_string(), &[Value::Int(0), Value::Int(2)]).unwrap(),
        Value::Str("éS".to_string())
    );
    assert_eq!(
        string_raadi("éSoomaali".to_string(), &[Value::Str("Soo".to_string())]).unwrap(),
        Value::Int(1)
    );
}

#[test]
fn test_list_sort_orders_numbers_by_value() {
    let list = Rc::new(RefCell::new(vec![
        Value::Int(10),
        Value::Float(1.5),
        Value::Int(2),
    ]));
    list_habee(Rc::clone(&list), &[]).unwrap();
    assert_eq!(
        *list.borrow(),
        vec![Value::Float(1.5), Value::Int(2), Value::Int(10)]
    );
}
