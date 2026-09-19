//! Integration tests: run short programs via binary, assert stdout.

use std::process::Command;

/// Run program source with -c and return stdout as string.
fn run_program(source: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg("-c")
        .arg(source)
        .output()
        .expect("run soplang");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_hello_world() {
    let output = run_program(r#"qor("Salaan, Adduunka!")"#);
    assert_eq!(output, "Salaan, Adduunka!\n");
}

#[test]
fn test_qor_number() {
    let output = run_program("qor(1+2)");
    assert_eq!(output, "3\n");
}

#[test]
fn test_for_loop() {
    let output = run_program(
        r#"
        kuceli (i 1 ilaa 4) {
            qor(i)
        }
    "#,
    );
    assert_eq!(output, "1\n2\n3\n4\n");
}

#[test]
fn test_nooc() {
    let output = run_program("qor(nooc(5))");
    assert_eq!(output, "abn\n");
}

#[test]
fn test_teed_dherer() {
    let output = run_program("qor(teed(1,2,3).dherer())");
    assert_eq!(output, "3\n");
}

#[test]
fn test_oop_class_example() {
    let output = run_program(
        r#"
        qaab Bisad {
            hawl dhaw(magac) { nafta.magac = magac }
            hawl magaca() { celi nafta.magac }
        }
        door b = cusub Bisad("Luna")
        qor(b.magaca())
    "#,
    );
    assert_eq!(output, "Luna\n");
}

#[test]
fn test_static_abn_jajab_ops() {
    let output = run_program(
        r#"
        abn a = 10
        abn b = 4
        qor(a + b)
        qor(a - b)
        qor(a * b)
        qor(a / b)
        qor(a % b)
        jajab f = 1.5
        qor(f + a)
        haddii (a > b) { qor("weyn") }
        abn n = 3
        qor(-n)
        hawl ku_dar(abn x, abn y) { celi x + y }
        qor(ku_dar(2, 5))
        door d = 1
        qor(d + "a")
        fasax {
            abn z = 1
            abn w = 0
            qor(z / w)
        } qabo (e) { qor("ok") }
    "#,
    );
    assert_eq!(output, "14\n6\n40\n2.5\n2\n11.5\nweyn\n-3\n7\n1a\nok\n");
}

#[test]
fn test_waalid_calls_parent_method() {
    let output = run_program(
        r#"
        qaab A {
            hawl dhaw() { qor("A") }
            hawl cod() { celi "A" }
        }
        qaab B dhaxal A {
            hawl dhaw() { qor("B") }
            hawl cod() { celi waalid.cod() + "B" }
        }
        qaab C dhaxal B {
            hawl cod() { celi waalid.cod() + "C" }
        }
        door b = cusub B()
        qor(b.cod())
        door c = cusub C()
        qor(c.cod())
    "#,
    );
    assert_eq!(output, "B\nAB\nB\nABC\n");
}

#[test]
fn test_class_typed_variable() {
    let output = run_program(
        r#"
        qaab Xayawaan {
            hawl dhaw(magac) { nafta.magac = magac }
            hawl magaca() { celi nafta.magac }
        }
        qaab Bisad dhaxal Xayawaan {
            hawl dhaw(magac) { nafta.magac = magac }
        }
        qaab Kale { hawl dhaw() {} }
        Bisad b = cusub Bisad("Mia")
        qor(b.magaca())
        Xayawaan x = b
        qor(x.magaca())
        door d = cusub Kale()
        fasax {
            Bisad bad = d
            qor("no")
        } qabo (e) { qor("ok") }
    "#,
    );
    assert_eq!(output, "Mia\nMia\nok\n");
}

#[test]
fn test_try_catch_runtime() {
    let output = run_program(
        r#"
        fasax { qor(1/0) } qabo (e) { qor("ok") }
    "#,
    );
    assert_eq!(output, "ok\n");
}

#[test]
fn test_repl_session_keeps_names() {
    let mut session = soplang::ReplSession::new(false).expect("session");
    session.eval(r#"door natiijo = """#, None).unwrap();
    session
        .eval(
            r#"
            qaab Bisad {
                hawl salaam() { natiijo = "Mia" }
            }
            "#,
            None,
        )
        .unwrap();
    session.eval(r#"door b = cusub Bisad()"#, None).unwrap();
    session.eval(r#"b.salaam()"#, None).unwrap();
    assert_eq!(global_string("natiijo"), "Mia");

    session.eval("door x = 1", None).unwrap();
    session.eval("x = x + 4", None).unwrap();
    assert_eq!(global_int("x"), 5);

    session
        .eval(r#"hawl foo() { natiijo = "fn" }"#, None)
        .unwrap();
    session.eval("foo()", None).unwrap();
    assert_eq!(global_string("natiijo"), "fn");
}

fn global_string(name: &str) -> String {
    let sv = soplang::runtime::soplang_get_builtin(name.as_ptr(), name.len());
    match soplang::runtime::soplang_to_value(sv).unwrap() {
        soplang::Value::Str(s) => s,
        other => panic!("expected string for {name}, got {other:?}"),
    }
}

fn global_int(name: &str) -> i64 {
    let sv = soplang::runtime::soplang_get_builtin(name.as_ptr(), name.len());
    match soplang::runtime::soplang_to_value(sv).unwrap() {
        soplang::Value::Int(n) => n,
        other => panic!("expected int for {name}, got {other:?}"),
    }
}

#[test]
fn test_import_via_file() {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = std::process::Command::new(exe)
        .arg("examples/16_import.sop")
        .output()
        .expect("run import example");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, "6\n");
}
