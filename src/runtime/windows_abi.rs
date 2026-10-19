//! x86_64 System V bridges for Windows generated code.
//!
//! Cranelift uses two integer registers for SoplangValue. Windows C passes this
//! aggregate indirectly, so generated code calls these bridges. The ordinary C
//! runtime API retains its platform ABI.

use super::abi::SoplangValue;
use std::ffi::c_int;

macro_rules! bridge {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) $(-> $result:ty)?) => {
        #[export_name = concat!("sysv_", stringify!($name))]
        pub extern "sysv64" fn $name($($arg: $ty),*) $(-> $result)? {
            super::abi::$name($($arg),*)
        }
    };
}

bridge!(
    soplang_set_location(
        file_ptr: *const u8,
        file_len: usize,
        line: usize,
        col: usize,
        end_col: usize,
        text_ptr: *const u8,
        text_len: usize,
    )
);
bridge!(soplang_try_begin() -> c_int);
bridge!(soplang_try_end());
bridge!(soplang_has_pending_error() -> c_int);
bridge!(soplang_finish_execution() -> c_int);
bridge!(soplang_raise(value: SoplangValue) -> SoplangValue);
bridge!(soplang_suspend_error());
bridge!(soplang_restore_error());
bridge!(soplang_discard_error());
bridge!(soplang_get_caught_error() -> SoplangValue);
bridge!(soplang_get_caught_error_value() -> SoplangValue);
bridge!(soplang_enter_function(name: *const u8, len: usize));
bridge!(soplang_leave_function());
bridge!(soplang_int(n: i64) -> SoplangValue);
bridge!(soplang_float(x: f64) -> SoplangValue);
bridge!(soplang_str(ptr: *const u8, len: usize) -> SoplangValue);
bridge!(soplang_bool(b: bool) -> SoplangValue);
bridge!(soplang_null() -> SoplangValue);
bridge!(soplang_add(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_sub(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_mul(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_div(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_mod(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_neg(a: SoplangValue) -> SoplangValue);
bridge!(soplang_not(a: SoplangValue) -> SoplangValue);
bridge!(soplang_truthy(value: SoplangValue) -> SoplangValue);
bridge!(
    soplang_range_continue(
        current: SoplangValue,
        end: SoplangValue,
        step: SoplangValue,
    ) -> SoplangValue
);
bridge!(soplang_eq(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_ne(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_lt(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_le(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_gt(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_ge(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_qor(v: SoplangValue));
bridge!(soplang_gelin() -> SoplangValue);
bridge!(soplang_nooc(v: SoplangValue) -> SoplangValue);
bridge!(soplang_list_new() -> SoplangValue);
bridge!(soplang_list_push(list: SoplangValue, val: SoplangValue) -> SoplangValue);
bridge!(soplang_object_new() -> SoplangValue);
bridge!(soplang_get_index(obj: SoplangValue, idx: SoplangValue) -> SoplangValue);
bridge!(
    soplang_set_index(
        obj: SoplangValue,
        idx: SoplangValue,
        val: SoplangValue,
    ) -> SoplangValue
);
bridge!(soplang_get_prop(obj: SoplangValue, name: *const u8, len: usize) -> SoplangValue);
bridge!(
    soplang_set_prop(
        obj: SoplangValue,
        name: *const u8,
        len: usize,
        val: SoplangValue,
    ) -> SoplangValue
);
bridge!(soplang_and(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(soplang_or(a: SoplangValue, b: SoplangValue) -> SoplangValue);
bridge!(
    soplang_new_instance(
        class_ptr: *const u8,
        class_len: usize,
        args_ptr: *const SoplangValue,
        n: i32,
    ) -> SoplangValue
);
bridge!(
    soplang_register_class(
        name_ptr: *const u8,
        name_len: usize,
        parent_ptr: *const u8,
        parent_len: usize,
        method_ptr: *const u8,
        method_len: usize,
        fn_payload: i64,
    )
);
bridge!(
    soplang_call_method(
        obj: SoplangValue,
        method_ptr: *const u8,
        method_len: usize,
        args_ptr: *const SoplangValue,
        n: i32,
    ) -> SoplangValue
);
bridge!(soplang_store_global(name: *const u8, len: usize, tag: i64, payload: i64));
bridge!(soplang_mark_const(name: *const u8, len: usize));
bridge!(
    soplang_check_class(
        tag: i64,
        payload: i64,
        class_ptr: *const u8,
        class_len: usize,
        name_ptr: *const u8,
        name_len: usize,
    )
);
bridge!(
    soplang_check_type(
        tag: i64,
        payload: i64,
        expected_type: i64,
        name_ptr: *const u8,
        name_len: usize,
    )
);
bridge!(soplang_get_builtin(name: *const u8, len: usize) -> SoplangValue);
bridge!(
    soplang_register_user_fn(
        name: *const u8,
        name_len: usize,
        ptr: *const u8,
        n_params: i64,
    ) -> i64
);
bridge!(
    soplang_make_closure(
        name: *const u8,
        name_len: usize,
        n: i64,
        args: *const SoplangValue,
    ) -> SoplangValue
);
bridge!(soplang_capture_get(env_ptr: i64, index: i64) -> SoplangValue);
bridge!(soplang_call(callee: SoplangValue, args: *const SoplangValue, n: i32,) -> SoplangValue);
