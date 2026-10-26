//! Safe embedding must not depend on the caller retaining compiler input.
use soplang::{
    backend::cranelift::CraneliftBackend,
    frontend::parse_source,
    hir::{HirConst, HirInstr},
    semantic::analyze,
    HirLowering, HirModule, SymbolTable, Value,
};
use std::path::Path;

fn module(source: &str) -> (HirModule, SymbolTable) {
    let ast = parse_source(source, Some(Path::new("embedding.sop"))).unwrap();
    let sym = analyze(&ast).unwrap();
    (HirLowering::lower(&sym, &ast), sym)
}

fn overwrite(text: &mut String) {
    // Same byte length and capacity: even the old borrowed pointers remain
    // readable, so failure demonstrates changed input without inducing UB.
    text.replace_range(.., &"x".repeat(text.len()));
}

fn change_body(body: &mut [HirInstr]) {
    for instruction in body {
        match instruction {
            HirInstr::Const {
                val: HirConst::Str(text),
                ..
            } => overwrite(text),
            HirInstr::FormatValue { format, .. } => overwrite(format),
            HirInstr::SetLocation(span) => overwrite(&mut span.source_line),
            _ => (),
        }
    }
}

#[test]
fn generated_code_owns_literal_format_and_diagnostic_bytes() {
    let source = r#"
        hawl render(number) { celi f"value={number:04d}" }
        hawl fail() { tuur "problem" }
        door rendered = render(10)
        door details = maran
        tijaabi { fail() } qabo (e, xog) { details = xog }
    "#;
    let (mut hir, sym) = module(source);
    let mut backend = CraneliftBackend::new().unwrap();
    backend.compile_module(&hir, &sym).unwrap();
    change_body(&mut hir.top_level);
    for function in &mut hir.functions {
        overwrite(&mut function.name);
        change_body(&mut function.body);
    }
    backend.run_main().unwrap();
    assert_eq!(
        backend.get_global("rendered"),
        Some(Value::Str("value=0010".into()))
    );
    let Some(Value::Error(error)) = backend.get_global("details") else {
        panic!("missing error")
    };
    assert_eq!(error.message(), "problem");
    let span = error.meta();
    assert_eq!(span.trace[0].function, "fail");
    assert_eq!(
        span.source_line.as_deref(),
        Some("        hawl fail() { tuur \"problem\" }")
    );
}

#[test]
fn compiled_modules_survive_input_drop_and_incremental_recompilation() {
    let mut backend = CraneliftBackend::new().unwrap();
    {
        let (hir, sym) = module(r#"hawl earlier() { celi "retained" }"#);
        backend.compile_module(&hir, &sym).unwrap();
    }
    for index in 0..20 {
        let (hir, sym) = module(&format!("door scratch{index} = '{index}'"));
        backend.compile_incremental(&hir, &sym).unwrap();
        backend.run_main().unwrap();
    }
    {
        let (hir, sym) = module("door result = earlier()");
        backend.compile_incremental(&hir, &sym).unwrap();
    }
    backend.run_main().unwrap();
    assert_eq!(
        backend.get_global("result"),
        Some(Value::Str("retained".into()))
    );
}
