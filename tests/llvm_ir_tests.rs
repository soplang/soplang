//! LLVM IR linked to the runtime. The binary does not call `run_source`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use soplang::backend::llvm_ir::{emit_llvm_ir, link_llvm_binary, link_llvm_binary_with_opt};
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
    assert!(ir.contains("define i32 @main(i32 %argc, ptr %argv)"), "IR has no LLVM main");
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
    run_llvm_with_opt(name, source, 2)
}

fn run_llvm_with_opt(name: &str, source: &str, opt: u8) -> String {
    let stmts = soplang::frontend::parse_source(source, None).expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    assert!(!ir.contains("run_source"));
    let bin = temp_bin(name);
    link_llvm_binary_with_opt(&ir, &bin, opt).unwrap_or_else(|e| panic!("{e:?}"));
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
fn test_llvm_qor_multiple_arguments() {
    assert_eq!(
        run_llvm("qor_arguments", include_str!("../examples/29_output.sop")),
        include_str!("../examples/29_output.expected")
    );
}

#[test]
fn llvm_tasks_wait_for_compiled_callbacks() {
    assert_eq!(
        run_llvm(
            "tasks",
            r#"
            hawl labalaab(abn n) { celi n * 2 }
            hawl parent(abn n) { celi sug(shaqo(labalaab, n)) + 1 }
            qor(sug(shaqo(parent, 20)))
            "#,
        ),
        "41\n"
    );
}

#[test]
fn llvm_channels_exchange_values_between_tasks() {
    assert_eq!(
        run_llvm(
            "channels",
            r#"
            door channel = marin(0)
            hawl producer() { dir(channel, 42) xidh(channel) }
            door task = shaqo(producer)
            qor(qaad(channel).qiime)
            qor(qaad(channel).dhammaad)
            sug(task)
            "#,
        ),
        "42\nrun\n"
    );
}

#[test]
fn llvm_loop_polls_run_other_tasks_and_deliver_cancellation() {
    assert_eq!(
        run_llvm(
            "task_poll",
            r#"
            door ready = been
            hawl mark() { ready = run }
            door marker = shaqo(mark)
            abn attempts = 0
            intay (!ready && attempts < 100) { attempts = attempts + 1 }
            hubi(ready && attempts < 100)
            sug(marker)

            door cleaned = been
            hawl spin() {
                tijaabi {
                    abn i = 0
                    intay (i < 1000000) { i = i + 1 }
                } dhammee { cleaned = run }
            }
            hawl stop(task) { baaji(task) }
            door worker = shaqo(spin)
            door stopper = shaqo(stop, worker)
            door caught = been
            tijaabi { sug(worker) } qabo (e) { caught = run }
            hubi(caught && cleaned)
            sug(stopper)
            qor("task polls ok")
            "#,
        ),
        "task polls ok\n"
    );
}

#[test]
fn llvm_timer_tasks_complete_and_cancel() {
    assert_eq!(
        run_llvm(
            "timers",
            r#"
            hawl wait() { sug(hurdada(10)) celi 8 }
            qor(sug(shaqo(wait)))
            door timer = hurdada(10000)
            baaji(timer)
            tijaabi { sug(timer) } qabo (e) { qor("cancelled") }
            "#,
        ),
        "8\ncancelled\n"
    );
}

#[test]
fn llvm_task_deadlines_cancel_and_drain_with_cleanup() {
    assert_eq!(
        run_llvm(
            "task_deadline",
            r#"
            hubi(sug_waqti(shaqo(hawl() { celi 3 }), 1000) == 3)
            door cleaned = been
            hawl slow() {
                tijaabi { sug(hurdada(10000)) }
                dhammee { cleaned = run }
            }
            door task = shaqo(slow)
            door expired = been
            tijaabi { sug_waqti(task, 10) }
            qabo (e) { expired = e == "Waqtigii shaqada loo qabtay wuu dhammaaday." }
            hubi(expired && cleaned)
            qor("deadline ok")
            "#,
        ),
        "deadline ok\n"
    );
}

#[test]
fn llvm_async_file_tasks_round_trip() {
    let file = std::env::temp_dir().join(format!(
        "soplang_llvm_async_file_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
        sug(qor_fayl_shaqo("{path}", "one"))
        sug(kudar_fayl_shaqo("{path}", "-two"))
        hubi(sug(akhrifayl_shaqo("{path}")) == "one-two")
        sug(qor_baytyo_shaqo("{path}", baytyo([65])))
        sug(kudar_baytyo_shaqo("{path}", baytyo([66])))
        hubi(sug(akhrifayl_shaqo("{path}")) == "AB")
        qor("async files ok")
        "#
    );
    assert_eq!(run_llvm("async_file_tasks", &source), "async files ok\n");
    let _ = fs::remove_file(file);
}

#[test]
fn llvm_async_http_request_returns_the_standard_response() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let url = format!("http://{}/item", listener.local_addr().expect("address"));
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .expect("timeout");
        let mut request = [0u8; 4096];
        let size = stream.read(&mut request).expect("read request");
        assert!(String::from_utf8_lossy(&request[..size]).starts_with("GET "));
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\nConnection: close\r\n\r\nLLVM")
            .expect("response");
    });
    let source = format!(
        r#"
        door got = sug(http_codso_shaqo("GET", "{url}"))
        hubi(got.heer == 200 && got.qoraal == "LLVM")
        qor("async http ok")
        "#
    );
    assert_eq!(run_llvm("async_http", &source), "async http ok\n");
    server.join().expect("server");
}

#[test]
fn llvm_callable_annotations_preserve_values_and_validate_callbacks() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                "callable_types",
                include_str!("fixtures/callable_types.sop"),
                opt
            ),
            "callable types ok\n"
        );
    }
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

#[test]
fn test_llvm_error_boundaries_stop_failed_expressions() {
    assert_eq!(
        run_llvm(
            "error_boundaries",
            include_str!("fixtures/error_boundaries.sop")
        ),
        "error boundaries ok\n"
    );
}

#[test]
fn test_llvm_raising_and_guaranteed_cleanup() {
    assert_eq!(
        run_llvm("raise_cleanup", include_str!("fixtures/raise_cleanup.sop")),
        "raise and cleanup ok\n"
    );
}

#[test]
fn test_llvm_structured_errors_and_causes() {
    assert_eq!(
        run_llvm(
            "structured_errors",
            include_str!("fixtures/structured_errors.sop")
        ),
        "structured errors ok\n"
    );
}

#[test]
fn test_llvm_wide_calls_and_callable_expressions() {
    assert_eq!(
        run_llvm(
            "callable_expressions",
            include_str!("fixtures/callable_expressions.sop")
        ),
        "callable expressions ok\n"
    );
}

#[test]
fn test_llvm_argument_buffers_stay_bounded_in_loops() {
    let source = r#"
        hawl last(a,b,c,d,e,f,g,h,i,j,k,l) { celi l }
        abn index = 0
        intay (index < 100000) {
            last(1,2,3,4,5,6,7,8,9,10,11,12)
            index = index + 1
        }
        qor(index)
    "#;
    // Disable LLVM optimizations: fixed argument buffers must remain bounded
    // through code generation itself, including in debug native executables.
    assert_eq!(
        run_llvm_with_opt("bounded_call_buffers", source, 0),
        "100000\n"
    );
}

#[test]
fn test_llvm_garbage_collection_preserves_active_values() {
    assert_eq!(
        run_llvm(
            "garbage_collection",
            include_str!("fixtures/garbage_collection.sop")
        ),
        "garbage collection ok\n"
    );
}

#[test]
fn test_llvm_escaped_raw_and_multiline_literals() {
    assert_eq!(
        run_llvm(
            "string_literals",
            include_str!("fixtures/string_literals.sop")
        ),
        "string literals ok\n"
    );
}

#[test]
fn test_llvm_value_and_template_formatting() {
    assert_eq!(
        run_llvm("formatting", include_str!("fixtures/formatting.sop")),
        "formatting ok\n"
    );
}

#[test]
fn test_llvm_interpolated_strings() {
    assert_eq!(
        run_llvm("interpolation", include_str!("fixtures/interpolation.sop")),
        "interpolation ok\n"
    );
}

#[test]
fn test_llvm_numeric_boundaries() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("numeric_edges_{opt}"),
                include_str!("fixtures/numeric_edges.sop"),
                opt
            ),
            "numeric edges ok\n"
        );
    }
}

#[test]
fn test_llvm_collection_iteration_and_protocol() {
    assert_eq!(
        run_llvm("iteration", include_str!("fixtures/iteration.sop")),
        "iteration ok\n"
    );
}

#[test]
fn test_llvm_iteration_gc_and_reentrant_callbacks() {
    assert_eq!(
        run_llvm("iteration_gc", include_str!("fixtures/iteration_gc.sop")),
        "iteration gc ok\n"
    );
}

#[test]
fn test_llvm_default_arguments() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_defaults_{opt}"),
                include_str!("fixtures/default_arguments.sop"),
                opt
            ),
            "default arguments ok\n"
        );
    }
}

#[test]
fn test_llvm_variadic_arguments() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_variadic_{opt}"),
                include_str!("fixtures/variadic_arguments.sop"),
                opt
            ),
            "variadic arguments ok\n"
        );
    }
}

#[test]
fn test_llvm_callback_signatures() {
    assert_eq!(
        run_llvm(
            "llvm_callback_signatures",
            include_str!("fixtures/callback_signatures.sop")
        ),
        "callback signatures ok\n"
    );
}

#[test]
fn test_llvm_named_arguments() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_named_{opt}"),
                include_str!("fixtures/named_arguments.sop"),
                opt
            ),
            "named arguments ok\n"
        );
    }
}

#[test]
fn test_llvm_bytes() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_bytes_{opt}"),
                include_str!("fixtures/bytes.sop"),
                opt
            ),
            "bytes ok\n"
        );
    }
}

#[test]
fn test_llvm_union_types() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_union_{opt}"),
                include_str!("fixtures/union_types.sop"),
                opt
            ),
            "union types ok\n"
        );
    }
}

#[test]
fn test_llvm_typed_collections() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_typed_{opt}"),
                include_str!("fixtures/typed_collections.sop"),
                opt
            ),
            "typed collections ok\n"
        );
    }
}

#[test]
fn test_llvm_tuples() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_tuples_{opt}"),
                include_str!("fixtures/tuples.sop"),
                opt
            ),
            "tuples ok\n"
        );
    }
}

#[test]
fn test_llvm_sets() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("llvm_sets_{opt}"),
                include_str!("fixtures/sets.sop"),
                opt
            ),
            "sets ok\n"
        );
    }
}

#[test]
fn test_llvm_typed_sets() {
    for opt in [0, 2] {
        assert_eq!(run_llvm_with_opt(&format!("llvm_typed_sets_{opt}"), include_str!("fixtures/typed_sets.sop"), opt), "typed sets ok\n");
    }
}

#[test]
fn test_llvm_text_methods() {
    for opt in [0, 2] {
        assert_eq!(run_llvm_with_opt(&format!("llvm_text_methods_{opt}"), include_str!("fixtures/text_methods.sop"), opt), "text methods ok\n");
    }
}

#[test]
fn test_llvm_list_operations() {
    for opt in [0, 2] {
        assert_eq!(run_llvm_with_opt(&format!("llvm_list_operations_{opt}"), include_str!("fixtures/list_operations.sop"), opt), "list operations ok\n");
    }
}

#[test]
fn test_llvm_object_operations() {
    for opt in [0, 2] {
        assert_eq!(run_llvm_with_opt(&format!("llvm_object_operations_{opt}"), include_str!("fixtures/object_operations.sop"), opt), "object operations ok\n");
    }
}

#[test]
fn test_llvm_numeric_library() {
    for opt in [0, 2] {
        assert_eq!(run_llvm_with_opt(&format!("llvm_numeric_library_{opt}"), include_str!("fixtures/numeric_library.sop"), opt), "numeric library ok\n");
    }
}

#[test]
fn test_llvm_text_files() {
    let dir = std::env::temp_dir().join(format!(
        "soplang_llvm_text_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    let file = dir.join("llvm-note.txt");
    let path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
qor_fayl("{path}", "jit")
hubi(akhrifayl("{path}") == "jit", "read")
kudar_fayl("{path}", "-llvm")
hubi(akhrifayl("{path}") == "jit-llvm", "append")
qor("llvm text ok")
"#
    );
    for opt in [0, 2] {
        let _ = fs::remove_file(&file);
        assert_eq!(
            run_llvm_with_opt_in_dir(
                &format!("llvm_text_files_{opt}"),
                &source,
                opt,
                &dir,
            ),
            "llvm text ok\n"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_llvm_filesystem() {
    let dir = std::env::temp_dir().join(format!(
        "soplang_llvm_fs_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    let nested = dir.join("pkg");
    let file = nested.join("item.txt");
    let nested_path = nested.display().to_string().replace('\\', "/");
    let file_path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
abuur_gal("{nested_path}")
qor_fayl("{file_path}", "llvm-fs")
hubi(fayl_jiraa("{file_path}"), "exists")
door info = fayl_xog("{file_path}")
hubi(info.nooc == "fayl", "kind")
qor("llvm fs ok")
"#
    );
    for opt in [0, 2] {
        let _ = fs::remove_dir_all(&nested);
        assert_eq!(
            run_llvm_with_opt_in_dir(&format!("llvm_filesystem_{opt}"), &source, opt, &dir),
            "llvm fs ok\n"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_llvm_binary_files() {
    let dir = std::env::temp_dir().join(format!(
        "soplang_llvm_bin_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    let file = dir.join("payload.bin");
    let path = file.display().to_string().replace('\\', "/");
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door data = baytyo([9, 8, 7])
qor_baytyo("{path}", data)
hubi(dherer(akhrif_baytyo("{path}")) == 3, "size")
qor("llvm binary ok")
"#
    );
    for opt in [0, 2] {
        let _ = fs::remove_file(&file);
        assert_eq!(
            run_llvm_with_opt_in_dir(&format!("llvm_binary_{opt}"), &source, opt, &dir),
            "llvm binary ok\n"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_llvm_json() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door value = json_akhris(e"{\"a\":1}")
hubi(value.a == 1 && json_qor(value) == e"{\"a\":1}", "json")
qor("llvm json ok")
"#;
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(&format!("llvm_json_{opt}"), source, opt),
            "llvm json ok\n"
        );
    }
}

#[test]
fn test_llvm_csv() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door rows = csv_akhris("a,b")
hubi(rows[0][0] == "a" && csv_qor(rows) == "a,b", "csv")
qor("llvm csv ok")
"#;
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(&format!("llvm_csv_{opt}"), source, opt),
            "llvm csv ok\n"
        );
    }
}

#[test]
fn test_llvm_url() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(url_qor("a b") == "a%20b" && url_akhris("a%20b") == "a b", "url")
qor("llvm url ok")
"#;
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(&format!("llvm_url_{opt}"), source, opt),
            "llvm url ok\n"
        );
    }
}

#[test]
fn test_llvm_base64() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(b64_qor("Man") == "TWFu" && qoraal(b64_akhris("TWFu")) == "Man", "base64")
qor("llvm base64 ok")
"#;
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(&format!("llvm_base64_{opt}"), source, opt),
            "llvm base64 ok\n"
        );
    }
}

#[test]
fn test_llvm_regex() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(regex_raadi("a+", "baa") == "aa", "regex")
qor("llvm regex ok")
"#;
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(&format!("llvm_regex_{opt}"), source, opt),
            "llvm regex ok\n"
        );
    }
}

#[test]
fn test_llvm_time() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(waqti_qor(0) == "1970-01-01T00:00:00Z", "time")
qor("llvm time ok")
"#;
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(&format!("llvm_time_{opt}"), source, opt),
            "llvm time ok\n"
        );
    }
}

#[test]
fn test_llvm_program_arguments() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door args = amarro()
hubi(dherer(args) == 1 && args[0] == "llvm-arg", "args")
qor("llvm args ok")
"#;
    let stmts = soplang::frontend::parse_source(source, None).expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    let bin = temp_bin("llvm_program_arguments");
    link_llvm_binary_with_opt(&ir, &bin, 2).unwrap_or_else(|e| panic!("{e:?}"));
    let out = Command::new(&bin)
        .arg("llvm-arg")
        .output()
        .expect("run llvm binary");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = fs::remove_file(&bin);
    assert!(out.status.success(), "failed: {stderr}\nstdout: {stdout}");
    assert_eq!(stdout, "llvm args ok\n");
}

#[test]
fn test_llvm_child_process() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door ok = fuli("/bin/echo", ["LLVM"])
hubi(ok.koodh == 0 && ok.soosaar == e"LLVM\n", "child")
qor("llvm child ok")
"#;
    let stmts = soplang::frontend::parse_source(source, None).expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    let bin = temp_bin("llvm_child_process");
    link_llvm_binary_with_opt(&ir, &bin, 2).unwrap_or_else(|e| panic!("{e:?}"));
    let out = Command::new(&bin).output().expect("run llvm binary");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = fs::remove_file(&bin);
    assert!(out.status.success(), "failed: {stderr}\nstdout: {stdout}");
    assert_eq!(stdout, "llvm child ok\n");
}

fn run_llvm_with_opt_in_dir(name: &str, source: &str, opt: u8, dir: &Path) -> String {
    let stmts = soplang::frontend::parse_source(source, None).expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir, &sym).unwrap_or_else(|e| panic!("{e}"));
    let bin = temp_bin(name);
    link_llvm_binary_with_opt(&ir, &bin, opt).unwrap_or_else(|e| panic!("{e}"));
    let out = Command::new(&bin).current_dir(dir).output().expect("run");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = fs::remove_file(&bin);
    assert!(out.status.success(), "binary failed: {stderr}\nstdout: {stdout}");
    stdout
}

#[test]
fn test_llvm_json_class_identity() {
    for opt in [0, 2] {
        assert_eq!(
            run_llvm_with_opt(
                &format!("json_identity_{opt}"),
                include_str!("fixtures/json_class_identity.sop"),
                opt,
            ),
            "json identity ok\n"
        );
    }
}

#[test]
fn llvm_database_transport_options_preserve_errors_and_bundled_defaults() {
    let source = include_str!("fixtures/database_transport_options.sop");
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/database_transport_options.sop");
    for opt in [0, 2] {
        let binary = temp_bin(&format!("database_transport_{opt}"));
        soplang::build_source(source, Some(&path), &binary, opt, false).unwrap();
        let output = Command::new(&binary).output().unwrap();
        let _ = fs::remove_file(&binary);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"database transport ok\n");
    }
}
