//! Phase 3 (COMPILER_PLAN): Runtime library tests.

use std::cell::RefCell;
use std::rc::Rc;

use soplang::runtime::stdlib::{
    builtin_abn, builtin_baaxad, builtin_daji, builtin_dhammeystir, builtin_dherer, builtin_jajab,
    builtin_kor, builtin_sugan, builtin_ugu_weyn, builtin_ugu_yar, list_habee, object_hel,
    string_dherer, string_jar, string_raadi,
};
use soplang::runtime::value::value_to_string;
use soplang::runtime::{
    soplang_add, soplang_call, soplang_eq, soplang_float, soplang_get_builtin, soplang_get_index,
    soplang_get_prop, soplang_int, soplang_list_new, soplang_list_push, soplang_lt, soplang_mod,
    soplang_mul, soplang_ne, soplang_null, soplang_object_new, soplang_set_index, soplang_set_prop,
    soplang_str, soplang_sub, soplang_to_value, TAG_BOOL, TAG_FLOAT, TAG_INT, TAG_LIST,
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
fn test_recursive_collections_format_and_compare_safely() {
    let left = Rc::new(RefCell::new(Vec::new()));
    left.borrow_mut().push(Value::List(Rc::clone(&left)));
    let right = Rc::new(RefCell::new(Vec::new()));
    right.borrow_mut().push(Value::List(Rc::clone(&right)));

    let left = Value::List(left);
    let right = Value::List(right);
    assert_eq!(value_to_string(&left), "[[...]]");
    assert_eq!(left, right);

    let object = Rc::new(RefCell::new(std::collections::HashMap::new()));
    object
        .borrow_mut()
        .insert("self".to_string(), Value::Object(Rc::clone(&object)));
    assert_eq!(value_to_string(&Value::Object(object)), "{self: {...}}");
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
    assert_eq!(
        builtin_daji(vec![Value::Float(4.9)]).unwrap(),
        Value::Int(4)
    );
    assert_eq!(
        builtin_daji(vec![Value::Float(-4.1)]).unwrap(),
        Value::Int(-5)
    );
    assert_eq!(builtin_kor(vec![Value::Float(4.1)]).unwrap(), Value::Int(5));
    assert_eq!(
        builtin_kor(vec![Value::Float(-4.9)]).unwrap(),
        Value::Int(-4)
    );
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

#[test]
fn test_v3_numeric_conversions_and_ranges() {
    assert_eq!(
        builtin_abn(vec![Value::Str(" 42 ".to_string())]).unwrap(),
        Value::Int(42)
    );
    assert_eq!(
        builtin_jajab(vec![Value::Str(" 3.5 ".to_string())]).unwrap(),
        Value::Float(3.5)
    );
    assert!(builtin_abn(vec![Value::Str("3.5".to_string())]).is_err());
    assert!(builtin_baaxad(vec![Value::Int(0), Value::Int(5), Value::Int(0)]).is_err());
    assert!(builtin_baaxad(vec![Value::Float(3.0)]).is_err());
    assert_eq!(
        builtin_dhammeystir(vec![Value::Float(3.6)]).unwrap(),
        Value::Int(4)
    );
    assert_eq!(builtin_sugan(vec![Value::Int(-4)]).unwrap(), Value::Int(4));
    assert_eq!(
        builtin_ugu_yar(vec![Value::Int(7), Value::Int(2)]).unwrap(),
        Value::Int(2)
    );
    assert_eq!(
        builtin_ugu_weyn(vec![Value::Float(1.5), Value::Int(2)]).unwrap(),
        Value::Float(2.0)
    );
}

#[test]
fn test_v3_unicode_negative_and_object_indexes() {
    let text = soplang_str("éS".as_ptr(), "éS".len());
    assert_eq!(
        soplang_to_value(soplang_get_index(text, soplang_int(-1))).unwrap(),
        Value::Str("S".to_string())
    );
    assert_eq!(
        soplang_to_value(soplang_get_index(text, soplang_int(-3))).unwrap(),
        Value::Null
    );

    let list = soplang_list_new();
    soplang_list_push(list, soplang_int(7));
    assert_eq!(
        soplang_to_value(soplang_get_index(list, soplang_int(-2))).unwrap(),
        Value::Null
    );

    let object = soplang_object_new();
    let key = soplang_str("magac".as_ptr(), 5);
    soplang_set_index(object, key, soplang_str("Ayaan".as_ptr(), 5));
    assert_eq!(
        soplang_to_value(soplang_get_index(object, key)).unwrap(),
        Value::Str("Ayaan".to_string())
    );
}

#[test]
fn test_v3_object_and_string_helpers() {
    let object = Rc::new(RefCell::new(std::collections::HashMap::new()));
    assert_eq!(
        object_hel(
            Rc::clone(&object),
            &[Value::Str("missing".to_string()), Value::Int(9)]
        )
        .unwrap(),
        Value::Int(9)
    );
    assert_eq!(string_dherer("éS".to_string(), &[]).unwrap(), Value::Int(2));
}
