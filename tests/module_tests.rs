//! Namespaced file imports (3.4 stage).

use std::fs;
use std::process::Command;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "soplang_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn namespaced_import_hides_private_names_and_keeps_flat_imports() {
    let dir = temp_dir("module");
    fs::write(
        dir.join("math.sop"),
        "soosaar wad, base\ndoor base = 10\nhawl wad(a, b) { celi a + b + base }\nhawl kale() { celi 1 }\nhawl _hidden() { celi 1 }\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "math.sop" sida xisaab
hawl hubi(v, name) { haddii (!v) { tuur name } }
door add = xisaab.wad
hubi(add(xisaab.base, 2) == 22, "call")
door failed = been
tijaabi { wad(1, 1) } qabo (e) { failed = run }
hubi(failed, "hidden function")
failed = been
tijaabi { xisaab._hidden() } qabo (e) { failed = run }
hubi(failed, "private")
failed = been
tijaabi { xisaab.kale() } qabo (e) { failed = run }
hubi(failed, "unlisted")
qor("module ok")
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "module ok\n");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn namespaced_import_builds_with_llvm() {
    let dir = temp_dir("module_llvm");
    fs::write(dir.join("math.sop"), "hawl wad(a, b) { celi a + b }\n").unwrap();
    let main = dir.join("main.sop");
    fs::write(
        &main,
        "keen \"math.sop\" sida xisaab\nqor(xisaab.wad(2, 3))\n",
    )
    .unwrap();
    let binary = dir.join("module-bin");
    let built = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("--build")
        .arg(&main)
        .arg("-o")
        .arg(&binary)
        .arg("-q")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = Command::new(&binary).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "5\n");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn private_and_nested_function_signatures_do_not_shadow_caller_builtins() {
    let dir = temp_dir("module_signatures");
    fs::write(
        dir.join("private.sop"),
        "soosaar value\nhawl hubi(a, b, c) {}\ndoor value = 42\n",
    )
    .unwrap();
    let source = r#"
keen "private.sop" sida m
hawl same(abn n): abn { celi n }
hawl outer() {
    hawl same(qoraal a, qoraal b): qoraal { celi a + b }
    celi same("a", "b")
}
door nested = hawl() { hawl hubi(a, b, c, d) {} celi 1 }
hubi(run)
hubi(same(7) == 7 && outer() == "ab" && nested() == 1)
qor(m.value)
"#;
    let main = dir.join("main.sop");
    fs::write(&main, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(&main)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"42\n");
    let binary = dir.join(format!("signatures{}", std::env::consts::EXE_SUFFIX));
    soplang::build_source(source, Some(&main), &binary, 2, false).unwrap();
    let output = Command::new(binary).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"42\n");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unknown_export_is_rejected() {
    let dir = temp_dir("module_export");
    fs::write(
        dir.join("math.sop"),
        "soosaar maqan\nhawl wad(a, b) { celi a + b }\n",
    )
    .unwrap();
    fs::write(dir.join("main.sop"), "keen \"math.sop\" sida xisaab\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("maqan"), "{stderr}");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn selective_import_binds_only_requested_names() {
    let dir = temp_dir("module_select");
    fs::write(
        dir.join("math.sop"),
        "soosaar wad, base\ndoor base = 10\nhawl wad(a, b) { celi a + b }\nhawl kale() { celi 9 }\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "math.sop" ka wad sida add, base
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(add(base, 2) == 12, "selected")
door failed = been
tijaabi { wad(1, 1) } qabo (e) { failed = run }
hubi(failed, "original name")
failed = been
tijaabi { kale() } qabo (e) { failed = run }
hubi(failed, "unimported")
qor("select ok")
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "select ok\n");
    let _ = fs::remove_dir_all(dir);
}

fn assert_backends(dir: &std::path::Path, expected: &str, strict: bool) {
    let main = dir.join("main.sop");
    let mut run = Command::new(env!("CARGO_BIN_EXE_soplang"));
    if strict {
        run.arg("--strict");
    }
    let output = run.arg(&main).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    for level in [0, 2] {
        let binary = dir.join(format!("module-{level}{}", std::env::consts::EXE_SUFFIX));
        let mut build = Command::new(env!("CARGO_BIN_EXE_soplang"));
        if strict {
            build.arg("--strict");
        }
        let output = build
            .arg("--build")
            .arg(&main)
            .arg("-o")
            .arg(&binary)
            .arg("--opt-level")
            .arg(level.to_string())
            .arg("-q")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = Command::new(binary).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn module_functions_keep_lexical_names_defaults_and_interpolation() {
    let dir = temp_dir("module_lexical");
    fs::write(
        dir.join("m.sop"),
        r#"
soosaar wad, shadow, defaultValue, formatted, make
hawl wad(abn x):abn { celi x+1 }
hawl shadow(wad) { celi wad(2) }
hawl defaultValue(x=wad(2)) { celi x }
hawl formatted() { celi f"{wad(2)}" }
qaab Local {
    hawl dhis(abn n=7) { nafta.n=n }
    hawl cod() { celi nafta.n }
}
hawl make(Local? value=maran) {
    haddii (value==maran) { value=cusub Local() }
    teed[Local] typed=[value]
    celi typed[0]
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "m.sop" sida m
qor(m.shadow(hawl(x){celi x+9}),m.defaultValue(),m.formatted(),m.make().cod())
"#,
    )
    .unwrap();
    assert_backends(&dir, "11 3 3 7\n", false);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn repeated_and_diamond_imports_share_initialization_and_exports() {
    let dir = temp_dir("module_cache");
    fs::write(
        dir.join("shared.sop"),
        "qor(\"shared\")\ndoor values=[]\nhawl add(n){values.geli(n)}\n",
    )
    .unwrap();
    fs::write(dir.join("left.sop"), "qor(\"left start\")\nkeen \"shared.sop\" sida shared\nqor(\"left end\")\nhawl add(n){shared.add(n)}\n").unwrap();
    fs::write(
        dir.join("right.sop"),
        "keen \"shared.sop\" sida shared\nhawl add(n){shared.add(n)}\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
qor("main")
keen "left.sop" sida left
keen "right.sop" sida right
keen "shared.sop" sida a
keen "./shared.sop" sida b
keen "shared.sop" ka values, add sida insert
left.add(1) right.add(2) insert(3)
a.values.geli(4)
qor(b.values==[1,2,3,4],values==[1,2,3,4])
"#,
    )
    .unwrap();
    assert_backends(&dir, "main\nleft start\nshared\nleft end\nrun run\n", false);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn namespaced_cycles_report_the_importing_file() {
    let dir = temp_dir("module_cycle");
    fs::write(dir.join("main.sop"), "keen \"other.sop\" sida other\n").unwrap();
    fs::write(dir.join("other.sop"), "keen \"main.sop\" sida main\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("wareeg") && stderr.contains("other.sop:1:"),
        "{stderr}"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn module_generated_initializers_work_in_strict_mode() {
    let dir = temp_dir("module_strict");
    fs::write(dir.join("m.sop"), "hawl wad(abn x):abn{celi x+1}\n").unwrap();
    fs::write(
        dir.join("main.sop"),
        "keen \"m.sop\" sida m\nkeen \"m.sop\" ka wad sida add\nqor(m.wad(2),add(4))\n",
    )
    .unwrap();
    assert_backends(&dir, "3 5\n", true);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn repl_import_cache_survives_collection_and_failed_analysis() {
    let dir = temp_dir("module_repl");
    let path = dir.join("main.sop");
    fs::write(&path, "").unwrap();
    fs::write(
        dir.join("m.sop"),
        "door values=[]\nhawl add(n){values.geli(n)}\n",
    )
    .unwrap();
    let mut session = soplang::ReplSession::new(false).unwrap();
    session
        .eval("keen \"m.sop\" sida first\nfirst.add(1)", Some(&path))
        .unwrap();
    session.collect();
    session
        .eval("keen \"m.sop\" sida second\nsecond.add(2)", Some(&path))
        .unwrap();
    session
        .eval(
            "haddii (first.values != [1,2] || second.values != [1,2]) { tuur \"cache reset\" }",
            Some(&path),
        )
        .unwrap();
    fs::write(dir.join("new.sop"), "door values=[3]\n").unwrap();
    assert!(session
        .eval(
            "keen \"new.sop\" sida pending\nabn bad=\"wrong\"",
            Some(&path)
        )
        .is_err());
    session.eval("keen \"new.sop\" sida valid\nhaddii (valid.values != [3]) { tuur \"lost initializer\" }", Some(&path)).unwrap();
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn repl_can_retry_an_initializer_after_a_runtime_error() {
    let dir = temp_dir("module_retry");
    let path = dir.join("main.sop");
    let ready = dir.join("ready").to_str().unwrap().replace('\\', "/");
    fs::write(&path, "").unwrap();
    fs::write(
        dir.join("m.sop"),
        format!("haddii (!fayl_jiraa(\"{ready}\")) {{ tuur \"not ready\" }}\ndoor values=[4]\n"),
    )
    .unwrap();
    let mut session = soplang::ReplSession::new(false).unwrap();
    assert!(session
        .eval("keen \"m.sop\" sida pending", Some(&path))
        .is_err());
    session.collect();
    fs::write(dir.join("ready"), "").unwrap();
    session
        .eval(
            "keen \"m.sop\" sida valid\nhaddii (valid.values != [4]) { tuur \"retry\" }",
            Some(&path),
        )
        .unwrap();
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn public_classes_construct_inherit_and_typecheck() {
    let dir = temp_dir("module_classes");
    fs::write(
        dir.join("people.sop"),
        r#"
soosaar Qof, Arday
qaab Qof {
    hawl dhis(qoraal magac): maran { nafta.magac = magac }
    hawl magac(): qoraal { celi nafta.magac }
}
qaab Arday dhaxal Qof {
    hawl dhis(qoraal magac): maran { waalid.dhis(magac) }
    hawl nooc(): qoraal { celi "arday" }
}
qaab Local {
    hawl dhis(): maran {}
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "people.sop" sida m
qaab Koox dhaxal m.Qof {
    hawl dhis(qoraal magac): maran { waalid.dhis(magac) }
    hawl nooc(): qoraal { celi "koox" }
}
m.Qof person = cusub m.Qof("Amina")
m.Qof student = cusub m.Arday("Cali")
m.Qof group = cusub Koox("Hodan")
qor(person.magac(), student.magac(), student.nooc(), group.magac(), group.nooc())
"#,
    )
    .unwrap();
    assert_backends(&dir, "Amina Cali arday Hodan koox\n", true);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn selective_class_imports_keep_one_identity() {
    let dir = temp_dir("module_class_select");
    fs::write(
        dir.join("people.sop"),
        "soosaar Qof\nqaab Qof {\n    hawl dhis(abn n): maran { nafta.n = n }\n    hawl n(): abn { celi nafta.n }\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        "keen \"people.sop\" ka Qof sida Student\nStudent s = cusub Student(4)\nqor(s.n())\n",
    )
    .unwrap();
    assert_backends(&dir, "4\n", true);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn class_aliases_are_lexical_and_reexports_keep_the_original_identity() {
    let dir = temp_dir("lexical_class_aliases");
    for (name, number) in [("one", 1), ("two", 2)] {
        fs::write(
            dir.join(format!("{name}.sop")),
            format!("soosaar C\nqaab C {{ hawl value(): abn {{ celi {number} }} }}\n"),
        )
        .unwrap();
    }
    for (module, leaf) in [("a", "one"), ("b", "two")] {
        fs::write(
            dir.join(format!("{module}.sop")),
            format!(
                r#"
keen "{leaf}.sop" sida m
keen "{leaf}.sop" ka C sida Local
soosaar Local, make, accept
hawl make(): m.C {{ celi cusub m.C() }}
hawl accept(teed[Local] values): Local {{ celi values[0] }}
"#
            ),
        )
        .unwrap();
    }
    fs::write(
        dir.join("main.sop"),
        r#"
keen "a.sop" sida a
keen "b.sop" sida b
a.Local first = cusub a.Local()
b.Local second = cusub b.Local()
qor(a.make().value(), b.make().value(), a.accept([first]).value(), b.accept([second]).value())
"#,
    )
    .unwrap();
    assert_backends(&dir, "1 2 1 2\n", true);

    let mut session = soplang::ReplSession::new(false).unwrap();
    session
        .eval("keen \"a.sop\" sida a", Some(&dir.join("main.sop")))
        .unwrap();
    session
        .eval("a.Local first = cusub a.Local()", None)
        .unwrap();
    session
        .eval("keen \"b.sop\" sida b", Some(&dir.join("main.sop")))
        .unwrap();
    session
        .eval("door answer = a.accept([first]).value()", None)
        .unwrap();
    assert_eq!(session.get_global("answer"), Some(soplang::Value::Int(1)));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn conflicting_class_imports_are_rejected_in_the_callers_scope() {
    let dir = temp_dir("class_alias_conflict");
    for name in ["one", "two"] {
        fs::write(
            dir.join(format!("{name}.sop")),
            "qaab C { hawl value() { celi 1 } }\n",
        )
        .unwrap();
    }
    let main = dir.join("main.sop");
    fs::write(
        &main,
        "keen \"one.sop\" ka C sida Local\nkeen \"two.sop\" ka C sida Local\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(main)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("E020"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn exported_methods_capture_module_values_defaults_and_private_functions() {
    let dir = temp_dir("class_environments");
    fs::write(
        dir.join("people.sop"),
        r#"
soosaar C
teed[abn] secret = [11]
hawl private(abn n): abn { celi n + secret[0] }
qaab C {
    hawl dhis(abn n = secret[0]): maran { nafta.n = n }
    hawl value(abn n = secret[0]): abn { celi private(n) + nafta.n }
}
"#,
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "people.sop" sida m
m.C first = cusub m.C()
m.C second = cusub m.C(n: 2)
qor(first.value(), second.value(n: 3))
"#,
    )
    .unwrap();
    assert_backends(&dir, "33 16\n", true);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn class_factories_keep_each_instances_lexical_methods_after_collection() {
    let dir = temp_dir("class_factory_capture");
    let source = r#"
hawl make(seed) {
    qaab Parent {
        hawl dhis(n = seed) { nafta.n = n }
        hawl value(n = seed) { celi seed + nafta.n + n }
    }
    qaab Child dhaxal Parent {
        hawl value(n = seed) { celi waalid.value(n: n) + seed }
    }
    celi cusub Child()
}
door first = make(1)
door second = make(10)
kuceli (i ku taxane(1500)) { door cycle = [] cycle.geli(cycle) }
qor(first.value(), second.value(n: 2), first.value(n: 3))
"#;
    fs::write(dir.join("main.sop"), source).unwrap();
    assert_backends(&dir, "4 32 6\n", false);
    let mut session = soplang::ReplSession::new(false).unwrap();
    session.eval(source, Some(&dir.join("main.sop"))).unwrap();
    session.collect();
    session
        .eval("door checked = first.value(n: 3)", None)
        .unwrap();
    assert_eq!(session.get_global("checked"), Some(soplang::Value::Int(6)));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn private_and_unknown_classes_are_not_exported() {
    let dir = temp_dir("module_class_private");
    fs::write(
        dir.join("people.sop"),
        "soosaar Qof\nqaab Qof { hawl dhis() {} }\nqaab Local { hawl dhis() {} }\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        "keen \"people.sop\" sida m\ndoor hidden = cusub m.Local()\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("lama soo saarin"), "{stderr}");
    fs::write(dir.join("main.sop"), "keen \"people.sop\" ka Local\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("lama soo saarin"), "{stderr}");
    fs::write(
        dir.join("people.sop"),
        "soosaar Maqan\nqaab Qof { hawl dhis() {} }\n",
    )
    .unwrap();
    fs::write(dir.join("main.sop"), "keen \"people.sop\" sida m\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("Maqan"), "{stderr}");
    fs::remove_dir_all(dir).unwrap();
}
