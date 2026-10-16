//! LLVM IR linked to the runtime. The binary does not call `run_source`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use soplang::backend::llvm_ir::{emit_llvm_ir, link_llvm_binary};
use soplang::{analyze, HirLowering, Lexer, Parser};

fn temp_bin(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut p = std::env::temp_dir();
    p.push(format!("soplang_{name}_{}_{stamp}", std::process::id()));
    p.with_extension(std::env::consts::EXE_EXTENSION)
}

fn runtime_add_call() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "call x86_64_sysvcc %SoplangValue @sysv_soplang_add"
    } else {
        "call %SoplangValue @soplang_add"
    }
}

#[test]
fn test_llvm_ir_prints_without_run_source() {
    let source = r#"
        hawl add(a, b) {
            celi a + b
        }
        abn n = 0
        intay (n < 3) {
            haddii (n == 1) {
                qor(add(n, 10))
            } ugudambeyn {
                qor(n)
            }
            n = n + 1
        }
    "#;
    let tokens = Lexer::new(source).tokenize().expect("lex");
    let stmts = Parser::new(tokens).parse().expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    assert!(ir.contains("define i32 @main()"), "IR has no LLVM main");
    assert!(
        ir.contains(runtime_add_call()),
        "arithmetic does not call the runtime"
    );
    assert!(
        !ir.contains("run_source"),
        "LLVM binary must not go through run_source"
    );

    let bin = temp_bin("llvm_loop");
    link_llvm_binary(&ir, &bin).unwrap_or_else(|e| panic!("{e}"));
    let out = Command::new(&bin).output().expect("run llvm binary");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = fs::remove_file(&bin);
    assert!(
        out.status.success(),
        "binary failed: {stderr}\nstdout: {stdout}"
    );
    assert_eq!(stdout, "0\n11\n2\n");
}

fn run_llvm(name: &str, source: &str) -> String {
    let tokens = Lexer::new(source).tokenize().expect("lex");
    let stmts = Parser::new(tokens).parse().expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    assert!(!ir.contains("run_source"));
    let bin = temp_bin(name);
    link_llvm_binary(&ir, &bin).unwrap_or_else(|e| panic!("{e}"));
    let out = Command::new(&bin).output().expect("run llvm binary");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = fs::remove_file(&bin);
    assert!(
        out.status.success(),
        "{name} failed: {stderr}\nstdout: {stdout}"
    );
    stdout
}

#[test]
fn test_llvm_shared_collection_contracts() {
    assert_eq!(
        run_llvm(
            "shared_collections",
            include_str!("fixtures/shared_collections.sop")
        ),
        "shared collections ok\n"
    );
}

#[test]
fn test_llvm_approved_vocabulary_and_constructor_compatibility() {
    assert_eq!(
        run_llvm(
            "approved_vocabulary",
            include_str!("fixtures/approved_vocabulary.sop")
        ),
        "approved vocabulary ok\n"
    );
}

#[test]
fn test_llvm_collection_operations() {
    assert_eq!(
        run_llvm(
            "collection_operations",
            include_str!("fixtures/collection_operations.sop")
        ),
        "collection operations ok\n"
    );
}

#[test]
fn test_llvm_collection_consistency() {
    assert_eq!(
        run_llvm(
            "collection_consistency",
            include_str!("fixtures/collection_consistency.sop")
        ),
        "collection consistency ok\n"
    );
}

#[test]
fn test_llvm_global_updates() {
    assert_eq!(
        run_llvm(
            "global_updates",
            include_str!("fixtures/global_updates.sop")
        ),
        "global updates ok\n"
    );
}

#[test]
fn test_llvm_ir_lists_objects_try_classes_and_closures() {
    assert_eq!(
        run_llvm(
            "list",
            r#"
            door t = teed(10, 20)
            t[0] = 7
            qor(t[0])
            qor(t.dherer())
            "#,
        ),
        "7\n2\n"
    );
    assert_eq!(
        run_llvm(
            "object",
            r#"
            walax w = { magac: "Ali", da: 25 }
            qor(w.magac)
            w.da = 26
            qor(w.da)
            "#,
        ),
        "Ali\n26\n"
    );
    assert_eq!(
        run_llvm("try", r#"fasax { qor(1/0) } qabo (e) { qor("ok") }"#,),
        "ok\n"
    );
    assert_eq!(
        run_llvm(
            "waalid",
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
        ),
        "B\nAB\nB\nABC\n"
    );
    assert_eq!(
        run_llvm(
            "class",
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
        ),
        "Mia\nMia\nok\n"
    );
    assert_eq!(
        run_llvm(
            "closure",
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
        ),
        "hi\nkept\nfirst\nchanged\nAmina\ndeep\n15\nMia\n"
    );
}

#[test]
fn test_llvm_ir_anonymous_functions_as_inline_callbacks() {
    assert_eq!(
        run_llvm(
            "anonymous_functions",
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
        ),
        "10\n[2, 4, 6, 8]\n[2, 4]\n[3, 6, 9, 12]\n15\n"
    );
}

#[test]
fn test_llvm_ir_v3_runtime_and_short_circuit() {
    assert_eq!(
        run_llvm(
            "v3_runtime",
            r#"
            door calls = 0;
            hawl touch() { calls = calls + 1; celi run; }
            door skipped_and = been && touch();
            door skipped_or = run || touch();
            qor(calls);

            qor(abn("42"));
            door text = "éSoomaali";
            qor(text[-1]);
            qor(text.dherer());

            door nums = teed(1, 2, 3);
            nums.geli(1, 9);
            qor(nums.isku_gee(hawl(total, item) { celi total + item; }, 0));
            qor(nums.qaar(hawl(item) { celi item == 9; }));

            door qof = {};
            qof["magac"] = "Ayaan";
            qor(qof.hel("magac"));
            qor(dhammeystir(3.6));

            door identity = hawl(value) { celi value; };
            fasax { identity(); } qabo (e) { qor("arity"); }
            "#,
        ),
        "0\n42\ni\n9\n15\nrun\nAyaan\n4\narity\n"
    );
}

#[test]
fn test_llvm_ir_standard_library_values_and_argument_counts() {
    assert_eq!(
        run_llvm(
            "standard_library",
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
        ),
        "[1.5, 2, 10]\n9\néS\n1\nabn\nlist\nobject\nstring\nbuiltin\ncallback\n"
    );
}

#[test]
fn test_llvm_ir_native_numeric_ops_and_zero_fallback() {
    let source = r#"
        abn a = 10
        abn b = 4
        qor(a + b)
        qor(a - b)
        qor(a * b)
        qor(a / b)
        qor(a % b)
        jajab f = 5.5
        qor(f + a)
        qor(f % 2.0)
        qor(a > b)
        qor(-a)
        qor(-f)
        hawl ku_dar(abn x, abn y) { celi x + y }
        qor(ku_dar(2, 5))
        door d = 1
        qor(d + "a")
        fasax {
            abn z = 0
            qor(a / z)
        } qabo (e) { qor("div zero") }
        fasax {
            abn mz = 0
            qor(a % mz)
        } qabo (e) { qor("mod zero") }
        fasax {
            jajab fz = 0.0
            qor(f / fz)
        } qabo (e) { qor("float zero") }
    "#;
    let tokens = Lexer::new(source).tokenize().expect("lex");
    let stmts = Parser::new(tokens).parse().expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    for instruction in [
        " = add i64 ",
        " = sub i64 ",
        " = mul i64 ",
        " = srem i64 ",
        " = fadd double ",
        " = fdiv double ",
        " = frem double ",
        " = icmp sgt i64 ",
        " = fneg double ",
        "Nzero",
        runtime_add_call(),
    ] {
        assert!(
            ir.contains(instruction),
            "missing native or fallback IR: {instruction}"
        );
    }
    let bin = temp_bin("llvm_numeric");
    link_llvm_binary(&ir, &bin).unwrap_or_else(|e| panic!("{e}"));
    let out = Command::new(&bin).output().expect("run llvm binary");
    let _ = fs::remove_file(&bin);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8(out.stdout).expect("utf8"),
        "14\n6\n40\n2.5\n2\n15.5\n1.5\nrun\n-10\n-5.5\n7\n1a\ndiv zero\nmod zero\nfloat zero\n"
    );
}

#[test]
fn test_llvm_ir_truthiness_ranges_and_runtime_types() {
    assert_eq!(
        run_llvm(
            "truthiness_ranges_types",
            r#"
            haddii ("") { qor("bad") } ugudambeyn { qor("empty string") }
            haddii (teed()) { qor("bad") } ugudambeyn { qor("empty list") }
            haddii ({}) { qor("bad") } ugudambeyn { qor("empty object") }
            haddii (-0.0) { qor("bad") } ugudambeyn { qor("negative zero") }
            haddii ("x") { qor("text") }

            kuceli (i 3 ilaa 1 :: -1) { qor(i) }
            fasax {
                kuceli (i 1 ilaa 3 :: 0) { qor("bad") }
            } qabo (e) { qor("zero step") }

            hawl dynamic_value() { celi "wrong" }
            abn number = 1
            fasax { number = dynamic_value() } qabo (e) { qor("assignment") }

            hawl typed_return() : abn {
                door value = { result: "wrong" }
                celi value.hel("result")
            }
            fasax { typed_return() } qabo (e) { qor("return") }

            hawl typed_parameter(abn value) { celi value }
            door callable = typed_parameter
            fasax { callable("wrong") } qabo (e) { qor("parameter") }
            "#,
        ),
        concat!(
            "empty string\nempty list\nempty object\nnegative zero\ntext\n",
            "3\n2\n1\nzero step\nassignment\nreturn\nparameter\n"
        )
    );
}

#[test]
fn test_llvm_runtime_names_inside_strings_are_preserved() {
    assert_eq!(
        run_llvm("runtime_symbol_string", r#"qor(" = call @soplang_add")"#),
        " = call @soplang_add\n"
    );
}
