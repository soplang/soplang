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
    assert!(
        out.status.success(),
        "program failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_shared_collection_contracts() {
    assert_eq!(
        run_program(include_str!("fixtures/shared_collections.sop")),
        "shared collections ok\n"
    );
}

#[test]
fn test_approved_vocabulary_and_constructor_compatibility() {
    assert_eq!(
        run_program(include_str!("fixtures/approved_vocabulary.sop")),
        "approved vocabulary ok\n"
    );
}

#[test]
fn test_collection_operations() {
    assert_eq!(
        run_program(include_str!("fixtures/collection_operations.sop")),
        "collection operations ok\n"
    );
}

#[test]
fn test_collection_consistency() {
    assert_eq!(
        run_program(include_str!("fixtures/collection_consistency.sop")),
        "collection consistency ok\n"
    );
}

#[test]
fn test_global_updates() {
    assert_eq!(
        run_program(include_str!("fixtures/global_updates.sop")),
        "global updates ok\n"
    );
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
fn test_repl_qor_multiple_arguments() {
    let mut session = soplang::ReplSession::new(false).expect("session");
    session
        .eval(r#"door magac1 = "Ayaan"; door magac2 = "Cali""#, None)
        .unwrap();
    session.eval("qor(magac1, magac2)", None).unwrap();
    session.eval("qor()", None).unwrap();
    session
        .eval("door output = qor(magac1, magac2, 3)", None)
        .unwrap();
    assert_eq!(global_string("output"), "Ayaan Cali 3");
}

#[test]
fn test_maran_literal() {
    let output = run_program("door value = maran qor(value) qor(nooc(value))");
    assert_eq!(output, "maran\nmaran\n");
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
fn test_truthiness_descending_ranges_and_runtime_types() {
    let output = run_program(
        r#"
        haddii ("") { qor("bad") } ugudambeyn { qor("empty") }
        haddii (teed()) { qor("bad") } ugudambeyn { qor("list") }
        haddii ({}) { qor("bad") } ugudambeyn { qor("object") }
        haddii (-0.0) { qor("bad") } ugudambeyn { qor("zero") }
        kuceli (i 3 ilaa 1 :: -1) { qor(i) }
        fasax { kuceli (i 1 ilaa 3 :: 0) {} } qabo (e) { qor("step") }

        hawl wrong() { celi "wrong" }
        abn number = 1
        fasax { number = wrong() } qabo (e) { qor("assignment") }

        hawl typed(abn value) { celi value }
        door callable = typed
        fasax { callable("wrong") } qabo (e) { qor("parameter") }
        "#,
    );
    assert_eq!(
        output,
        "empty\nlist\nobject\nzero\n3\n2\n1\nstep\nassignment\nparameter\n"
    );
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
fn test_nested_function_keeps_outer_local() {
    let output = run_program(
        r#"
        hawl plain() {
            hawl hi() { celi "hi" }
            celi hi
        }
        door p = plain()
        qor(p())

        hawl make() {
            door x = "kept"
            hawl inner() { celi x }
            celi inner
        }
        door f = make()
        qor(f())

        hawl snap() {
            door x = "first"
            hawl shot() { celi x }
            x = "second"
            celi shot
        }
        door s = snap()
        qor(s())

        door g = "global"
        hawl outer() {
            hawl readg() { celi g }
            celi readg
        }
        door h = outer()
        g = "changed"
        qor(h())

        hawl greet(name) {
            hawl inner() { celi name }
            celi inner
        }
        door greet_f = greet("Amina")
        qor(greet_f())

        hawl a() {
            door x = "deep"
            hawl b() {
                hawl c() { celi x }
                celi c
            }
            celi b
        }
        door mid = a()
        door deep = mid()
        qor(deep())

        hawl add(base) {
            hawl plus(n) { celi base + n }
            celi plus
        }
        door plus = add(10)
        qor(plus(5))

        qaab Sanduuq {
            hawl dhaw(magac) { nafta.magac = magac }
            hawl reader() {
                hawl inner() { celi nafta.magac }
                celi inner
            }
        }
        door box = cusub Sanduuq("Mia")
        door read = box.reader()
        qor(read())
    "#,
    );
    assert_eq!(output, "hi\nkept\nfirst\nchanged\nAmina\ndeep\n15\nMia\n");
}

#[test]
fn test_anonymous_functions_as_inline_callbacks() {
    let output = run_program(
        r#"
        door laban = hawl(x) { celi x * 2 }
        qor(laban(5))

        door nums = teed(1, 2, 3, 4)
        qor(nums.aaddin(hawl(x) { celi x * 2 }))
        qor(nums.shaandhee(hawl(x) { celi x % 2 == 0 }))

        hawl scale(factor) {
            celi nums.aaddin(hawl(x) { celi x * factor })
        }
        qor(scale(3))

        hawl make(base) {
            celi hawl(n) { celi hawl(extra) { celi base + n + extra } }
        }
        door first = make(10)
        door second = first(2)
        qor(second(3))
    "#,
    );
    assert_eq!(output, "10\n[2, 4, 6, 8]\n[2, 4]\n[3, 6, 9, 12]\n15\n");
}

#[test]
fn test_v3_language_and_standard_library() {
    let output = run_program(
        r#"
        door calls = 0;
        hawl touch() { calls = calls + 1; celi run; }
        door skipped_and = been && touch();
        door skipped_or = run || touch();
        qor(calls);

        qor(abn(" 42 "));
        qor(jajab("3.5"));
        fasax { baaxad(0, 3, 0); } qabo (e) { qor("range"); }

        door text = "éSoomaali";
        qor(text[0]);
        qor(text[-1]);
        qor(text[-20]);
        qor(text.dherer());
        qor("  hi  ".masax_bilow());
        qor("  hi  ".masax_dhamaad());
        qor("banana".tiri("an"));
        qor("ha".ku_celi(3));
        qor("banana".raadi_dambe("an"));

        door nums = teed(1, 2, 3);
        nums.geli(1, 9);
        qor(nums);
        qor(nums.ka_saar(-1));
        qor(nums.koowaad());
        qor(nums.ugu_dambeeya());
        qor(nums.isku_gee(hawl(total, item) { celi total + item; }, 0));
        qor(nums.dhammaan(hawl(item) { celi item > 0; }));
        qor(nums.qaar(hawl(item) { celi item == 9; }));
        qor(nums[-20]);
        fasax { nums[20] = 4; } qabo (e) { qor("index"); }

        door qof = { magac: "Ayaan" };
        qor(qof["magac"]);
        qof["da"] = 20;
        qor(qof.hel("da", 0));
        qof.dhig("dal", "Soomaaliya");
        qor(qof.ka_qaad("dal"));
        qor(qof.hel("maqan", "default"));

        qor(dhammeystir(3.6));
        qor(sugan(-5));
        qor(ugu_yar(8, 2, 5));
        qor(ugu_weyn(8, 2, 5));
        qor(awood(2, 3));
        qor(xidid(9));

        door identity = hawl(value) { celi value; };
        fasax { identity(); } qabo (e) { qor("arity"); }
        "#,
    );
    assert_eq!(
        output,
        concat!(
            "0\n42\n3.5\nrange\né\ni\nmaran\n9\nhi  \n  hi\n2\nhahaha\n3\n",
            "[1, 9, 2, 3]\n3\n1\n2\n12\nrun\nrun\nmaran\nindex\n",
            "Ayaan\n20\nSoomaaliya\ndefault\n4\n5\n2\n8\n8\n3\narity\n"
        )
    );
}

#[test]
fn test_standard_library_values_and_argument_counts() {
    let output = run_program(
        r#"
        door nums = teed(10, 2, 1.5)
        nums.habee()
        qor(nums)
        qor(dherer("éSoomaali"))
        qor("éSoomaali".jar(0, 2))
        qor("éSoomaali".raadi("Soo"))
        qor(nooc(daji(4.9)))

        fasax { nums.dherer(1) } qabo (e) { qor("list") }
        door obj = { magac: "Ayaan" }
        fasax { obj.fure(1) } qabo (e) { qor("object") }
        door text = "abc"
        fasax { text.bilow() } qabo (e) { qor("string") }
        fasax { dherer("a", "b") } qabo (e) { qor("builtin") }
        fasax {
            nums.aaddin(hawl(x) { celi x }, hawl(x) { celi x })
        } qabo (e) { qor("callback") }
    "#,
    );
    assert_eq!(
        output,
        "[1.5, 2, 10]\n9\néS\n1\nabn\nlist\nobject\nstring\nbuiltin\ncallback\n"
    );
}

#[test]
fn test_standard_library_method_catalog() {
    let output = run_program(
        r#"
        door values = teed(3, 1, 2)
        qor(values.dherer())
        qor(values.leeyahay(1))
        qor(values.muuji(2))
        qor(values.jar(0, 2))
        door copied = values.nuqul()
        copied.rog()
        qor(copied)
        copied.habee()
        qor(copied)
        qor(values.kudar(teed(4)))
        values.kudar(4)
        qor(values.kasaar())
        qor(values)
        copied.nadiifi()
        qor(copied)

        door record = { b: 2, a: 1 }
        qor(record.fure())
        qor(record.qiime())
        qor(record.lamaane())
        qor(record.leeyahay("a"))
        door merged = record.kudar({ c: 3 })
        qor(merged.c)
        door record_copy = record.nuqul()
        record_copy.a = 9
        qor(record.a)
        record.tir("a")
        qor(record.leeyahay("a"))
        record.nadiifi()
        qor(record.fure())

        door text = "  Soplang-Soomaali  "
        door clean = text.masax()
        qor(clean.qeybi("-"))
        qor(clean.leeyahay("Soomaali"))
        qor(clean.bilow("Sop"))
        qor(clean.dhamaad("aali"))
        qor(clean.beddel("o", "0"))
        qor(clean.beddel_dhammaan("o", "0"))
        qor("/".kudar(teed("a", "b", "c")))
        qor(clean.jar(0, 7))
        qor(clean.xarfaha_waaweyn())
        qor(clean.xarfaha_yaryar())
        qor(clean.raadi("Soo"))

        door random_number = xul(1, 2)
        qor(random_number >= 1 && random_number < 2)
        door random_item = xul(teed("a", "b"))
        qor(teed("a", "b").leeyahay(random_item))
    "#,
    );
    assert_eq!(
        output,
        concat!(
            "3\nrun\n2\n[3, 1]\n[2, 1, 3]\n[1, 2, 3]\n",
            "[3, 1, 2, 4]\n4\n[3, 1, 2]\n[]\n",
            "[a, b]\n[1, 2]\n[[a, 1], [b, 2]]\nrun\n3\n1\nbeen\n[]\n",
            "[Soplang, Soomaali]\nrun\nrun\nrun\nS0plang-Soomaali\n",
            "S0plang-S00maali\na/b/c\nSoplang\nSOPLANG-SOOMAALI\n",
            "soplang-soomaali\n8\nrun\nrun\n"
        )
    );
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
fn test_repl_global_updates_across_snippets() {
    let mut session = soplang::ReplSession::new(false).expect("session");
    session
        .eval(
            "door total = 0 hawl increment() { total = total + 1 } increment() door observed = total",
            None,
        )
        .unwrap();
    assert_eq!(global_int("total"), 1);
    assert_eq!(global_int("observed"), 1);
    session
        .eval("increment() total = total + 1 observed = total", None)
        .unwrap();
    assert_eq!(global_int("total"), 3);
    assert_eq!(global_int("observed"), 3);
    session
        .eval(
            "kuceli (index 1 ilaa 3) { observed = observed + index }",
            None,
        )
        .unwrap();
    assert_eq!(global_int("observed"), 9);
    assert_eq!(global_int("index"), 4);
    session
        .eval(
            "kuceli (index 1 ilaa 2) { observed = observed + index }",
            None,
        )
        .unwrap();
    assert_eq!(global_int("observed"), 12);
    assert_eq!(global_int("index"), 3);
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
