//! LLVM IR for a subset of HIR. Calls the existing runtime.
//!
//! Covered here: constants, arithmetic, branches, loops, and functions.
//! `soplang --build` is still the Cargo runner in `llvm.rs`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{runtime_error, SoplangError};
use crate::hir::{BinOpKind, HirConst, HirFunction, HirInstr, HirModule, UnOpKind};

const DECLS: &str = r#"%SoplangValue = type { i8, i64 }

declare %SoplangValue @soplang_int(i64)
declare %SoplangValue @soplang_float(double)
declare %SoplangValue @soplang_str(ptr, i64)
declare %SoplangValue @soplang_bool(i8)
declare %SoplangValue @soplang_null()
declare %SoplangValue @soplang_add(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_sub(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_mul(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_div(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_mod(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_neg(%SoplangValue)
declare %SoplangValue @soplang_not(%SoplangValue)
declare %SoplangValue @soplang_eq(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_ne(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_lt(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_le(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_gt(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_ge(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_and(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_or(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_get_builtin(ptr, i64)
declare void @soplang_store_global(ptr, i64, i64, i64)
declare %SoplangValue @soplang_call(%SoplangValue, ptr, i32)
declare void @soplang_check_type(i64, i64, i64, ptr, i64)
"#;

struct FnInfo {
    llvm_name: String,
    n_params: usize,
}

struct ModuleOut {
    strings: String,
    body: String,
    next_str: usize,
    str_ids: HashMap<String, String>,
}

impl ModuleOut {
    fn intern(&mut self, s: &str) -> String {
        if let Some(name) = self.str_ids.get(s) {
            return name.clone();
        }
        let name = format!("@s{}", self.next_str);
        self.next_str += 1;
        let bytes = llvm_c_bytes(s);
        let n = s.len();
        self.strings.push_str(&format!(
            "{name} = private unnamed_addr constant [{n} x i8] c\"{bytes}\"\n"
        ));
        self.str_ids.insert(s.to_string(), name.clone());
        name
    }
}

struct FnEmit<'a> {
    m: &'a mut ModuleOut,
    fns: &'a [FnInfo],
    by_name: &'a HashMap<String, usize>,
    tmp: usize,
    nslots: usize,
    slot_fn: HashMap<usize, usize>,
    is_main: bool,
}

pub fn emit_llvm_ir(hir: &HirModule) -> Result<String, SoplangError> {
    let mut fns = Vec::new();
    let mut by_name = HashMap::new();
    for (i, f) in hir.functions.iter().enumerate() {
        if !f.capture_slots.is_empty() {
            return Err(runtime_error("LLVM IR does not support closures yet", 0, 0));
        }
        by_name.insert(f.name.clone(), i);
        fns.push(FnInfo {
            llvm_name: format!("sfn{i}"),
            n_params: f.params.len(),
        });
    }

    let mut m = ModuleOut {
        strings: String::new(),
        body: String::new(),
        next_str: 0,
        str_ids: HashMap::new(),
    };
    for f in hir.functions.iter() {
        emit_function(&mut m, &fns, &by_name, f)?;
    }
    emit_main(&mut m, &fns, &by_name, &hir.top_level)?;

    Ok(format!("{DECLS}\n{}\n{}", m.strings, m.body))
}

/// Compile `ir` and link it with the Soplang runtime.
/// `main` is the LLVM function. The Rust entry is `#![no_main]` so nothing calls `run_source`.
pub fn link_llvm_binary(ir: &str, out: &Path) -> Result<(), SoplangError> {
    let (rlib, deps) = soplang_rlib()?;
    let dir = std::env::temp_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let ll = dir.join(format!("soplang_llvm_{stamp}.ll"));
    let obj = dir.join(format!("soplang_llvm_{stamp}.o"));
    let wrapper = dir.join(format!("soplang_llvm_{stamp}.rs"));
    fs::write(&ll, ir).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
    fs::write(&wrapper, LINK_WRAPPER).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
        }
    }
    let clang = Command::new("clang")
        .arg("-c")
        .arg(&ll)
        .arg("-o")
        .arg(&obj)
        .output()
        .map_err(|e| runtime_error(format!("clang did not start: {e}"), 0, 0))?;
    if !clang.status.success() {
        let stderr = String::from_utf8_lossy(&clang.stderr);
        let _ = fs::remove_file(&ll);
        return Err(runtime_error(
            format!("clang failed to compile LLVM IR\n{}", stderr.trim_end()),
            0,
            0,
        ));
    }
    let rustc = Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("--crate-name")
        .arg("soplang_llvm_bin")
        .arg("--crate-type")
        .arg("bin")
        .arg("--extern")
        .arg(format!("soplang={}", rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("-o")
        .arg(out)
        .arg(&wrapper)
        .arg("-C")
        .arg(format!("link-arg={}", obj.display()))
        .output()
        .map_err(|e| runtime_error(format!("rustc did not start: {e}"), 0, 0))?;
    let _ = fs::remove_file(&ll);
    let _ = fs::remove_file(&obj);
    let _ = fs::remove_file(&wrapper);
    if !rustc.status.success() {
        let stderr = String::from_utf8_lossy(&rustc.stderr);
        return Err(runtime_error(
            format!("rustc failed to link LLVM IR\n{}", stderr.trim_end()),
            0,
            0,
        ));
    }
    Ok(())
}

const LINK_WRAPPER: &str = r#"
#![no_main]
extern crate soplang;

struct Keep(*const ());
unsafe impl Sync for Keep {}

#[used]
static KEEP: [Keep; 24] = [
    Keep(soplang::runtime::soplang_int as *const ()),
    Keep(soplang::runtime::soplang_float as *const ()),
    Keep(soplang::runtime::soplang_str as *const ()),
    Keep(soplang::runtime::soplang_bool as *const ()),
    Keep(soplang::runtime::soplang_null as *const ()),
    Keep(soplang::runtime::soplang_add as *const ()),
    Keep(soplang::runtime::soplang_sub as *const ()),
    Keep(soplang::runtime::soplang_mul as *const ()),
    Keep(soplang::runtime::soplang_div as *const ()),
    Keep(soplang::runtime::soplang_mod as *const ()),
    Keep(soplang::runtime::soplang_neg as *const ()),
    Keep(soplang::runtime::soplang_not as *const ()),
    Keep(soplang::runtime::soplang_eq as *const ()),
    Keep(soplang::runtime::soplang_ne as *const ()),
    Keep(soplang::runtime::soplang_lt as *const ()),
    Keep(soplang::runtime::soplang_le as *const ()),
    Keep(soplang::runtime::soplang_gt as *const ()),
    Keep(soplang::runtime::soplang_ge as *const ()),
    Keep(soplang::runtime::soplang_and as *const ()),
    Keep(soplang::runtime::soplang_or as *const ()),
    Keep(soplang::runtime::soplang_get_builtin as *const ()),
    Keep(soplang::runtime::soplang_store_global as *const ()),
    Keep(soplang::runtime::soplang_call as *const ()),
    Keep(soplang::runtime::soplang_check_type as *const ()),
];
"#;

fn soplang_rlib() -> Result<(PathBuf, PathBuf), SoplangError> {
    let base = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let deps = base.join(profile).join("deps");
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    let entries = fs::read_dir(&deps)
        .map_err(|e| runtime_error(format!("cannot read {}: {e}", deps.display()), 0, 0))?;
    for ent in entries {
        let ent = ent.map_err(|e| runtime_error(e.to_string(), 0, 0))?;
        let path = ent.path();
        let file = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !(file.starts_with("libsoplang-") && file.ends_with(".rlib")) {
            continue;
        }
        let modified = path
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        if best.as_ref().map(|(t, _)| modified >= *t).unwrap_or(true) {
            best = Some((modified, path));
        }
    }
    match best {
        Some((_, p)) => Ok((p, deps)),
        None => Err(runtime_error(
            format!("soplang rlib not found in {}", deps.display()),
            0,
            0,
        )),
    }
}

fn emit_function(
    m: &mut ModuleOut,
    fns: &[FnInfo],
    by_name: &HashMap<String, usize>,
    f: &HirFunction,
) -> Result<(), SoplangError> {
    let info = &fns[by_name[&f.name]];
    let nslots = slots_needed(
        &f.body,
        f.local_count
            .max(f.params.iter().copied().max().map(|s| s + 1).unwrap_or(0)),
    );
    let mut args = String::new();
    for i in 0..info.n_params {
        if i > 0 {
            args.push_str(", ");
        }
        args.push_str(&format!("%SoplangValue %a{i}"));
    }
    m.body.push_str(&format!(
        "define %SoplangValue @{}({args}) {{\nentry:\n",
        info.llvm_name
    ));
    let mut emit = FnEmit {
        m,
        fns,
        by_name,
        tmp: 0,
        nslots,
        slot_fn: HashMap::new(),
        is_main: false,
    };
    emit.alloc_slots();
    for (i, &slot) in f.params.iter().enumerate() {
        emit.unpack_store(&format!("%a{i}"), slot);
    }
    emit.body(&f.body)?;
    emit.m.body.push_str("}\n\n");
    Ok(())
}

fn emit_main(
    m: &mut ModuleOut,
    fns: &[FnInfo],
    by_name: &HashMap<String, usize>,
    body: &[HirInstr],
) -> Result<(), SoplangError> {
    let nslots = slots_needed(body, 0);
    m.body.push_str("define i32 @main() {\nentry:\n");
    let mut emit = FnEmit {
        m,
        fns,
        by_name,
        tmp: 0,
        nslots,
        slot_fn: HashMap::new(),
        is_main: true,
    };
    emit.alloc_slots();
    emit.body(body)?;
    emit.m.body.push_str("}\n");
    Ok(())
}

impl<'a> FnEmit<'a> {
    fn line(&mut self, s: impl AsRef<str>) {
        self.m.body.push_str(s.as_ref());
        self.m.body.push('\n');
    }

    fn fresh(&mut self) -> String {
        let n = self.tmp;
        self.tmp += 1;
        format!("%v{n}")
    }

    fn alloc_slots(&mut self) {
        let n = self.nslots;
        self.line(format!("  %tags = alloca [{n} x i64], align 8"));
        self.line(format!("  %pays = alloca [{n} x i64], align 8"));
        for i in 0..n {
            let t = self.gep("tags", i);
            let p = self.gep("pays", i);
            self.line(format!("  store i64 0, ptr {t}, align 8"));
            self.line(format!("  store i64 0, ptr {p}, align 8"));
        }
    }

    fn gep(&mut self, which: &str, slot: usize) -> String {
        let p = self.fresh();
        let n = self.nslots;
        self.line(format!(
            "  {p} = getelementptr inbounds [{n} x i64], ptr %{which}, i64 0, i64 {slot}"
        ));
        p
    }

    fn store_slot(&mut self, slot: usize, tag: &str, pay: &str) {
        let t = self.gep("tags", slot);
        let p = self.gep("pays", slot);
        self.line(format!("  store i64 {tag}, ptr {t}, align 8"));
        self.line(format!("  store i64 {pay}, ptr {p}, align 8"));
    }

    fn load_slot(&mut self, slot: usize) -> (String, String) {
        let tptr = self.gep("tags", slot);
        let pptr = self.gep("pays", slot);
        let tag = self.fresh();
        let pay = self.fresh();
        self.line(format!("  {tag} = load i64, ptr {tptr}, align 8"));
        self.line(format!("  {pay} = load i64, ptr {pptr}, align 8"));
        (tag, pay)
    }

    fn pack(&mut self, tag: &str, pay: &str) -> String {
        let b = self.fresh();
        let s1 = self.fresh();
        let s2 = self.fresh();
        self.line(format!("  {b} = trunc i64 {tag} to i8"));
        self.line(format!(
            "  {s1} = insertvalue %SoplangValue poison, i8 {b}, 0"
        ));
        self.line(format!(
            "  {s2} = insertvalue %SoplangValue {s1}, i64 {pay}, 1"
        ));
        s2
    }

    fn unpack_store(&mut self, sv: &str, slot: usize) {
        let t8 = self.fresh();
        let tag = self.fresh();
        let pay = self.fresh();
        self.line(format!("  {t8} = extractvalue %SoplangValue {sv}, 0"));
        self.line(format!("  {tag} = zext i8 {t8} to i64"));
        self.line(format!("  {pay} = extractvalue %SoplangValue {sv}, 1"));
        self.store_slot(slot, &tag, &pay);
    }

    fn call_sv(&mut self, func: &str, args: &str) -> String {
        let v = self.fresh();
        self.line(format!("  {v} = call %SoplangValue @{func}({args})"));
        v
    }

    fn str_ptr(&mut self, s: &str) -> (String, usize) {
        let g = self.m.intern(s);
        let p = self.fresh();
        let n = s.len();
        self.line(format!(
            "  {p} = getelementptr inbounds [{n} x i8], ptr {g}, i64 0, i64 0"
        ));
        (p, n)
    }

    fn body(&mut self, body: &[HirInstr]) -> Result<(), SoplangError> {
        let mut terminated = false;
        for instr in body {
            if let HirInstr::Label(id) = instr {
                if !terminated {
                    self.line(format!("  br label %L{id}"));
                }
                self.line(format!("L{id}:"));
                terminated = false;
                continue;
            }
            if terminated {
                continue;
            }
            terminated = self.instr(instr)?;
        }
        if !terminated {
            if self.is_main {
                self.line("  ret i32 0");
            } else {
                self.line("  ret %SoplangValue zeroinitializer");
            }
        }
        Ok(())
    }

    fn instr(&mut self, instr: &HirInstr) -> Result<bool, SoplangError> {
        match instr {
            HirInstr::Const { dst, val } => {
                let sv = match val {
                    HirConst::Int(n) => self.call_sv("soplang_int", &format!("i64 {n}")),
                    HirConst::Float(x) => {
                        let bits = self.fresh();
                        let fp = self.fresh();
                        self.line(format!("  {bits} = add i64 {}, 0", x.to_bits() as i64));
                        self.line(format!("  {fp} = bitcast i64 {bits} to double"));
                        self.call_sv("soplang_float", &format!("double {fp}"))
                    }
                    HirConst::Str(s) => {
                        let (p, n) = self.str_ptr(s);
                        self.call_sv("soplang_str", &format!("ptr {p}, i64 {n}"))
                    }
                    HirConst::Bool(b) => {
                        let n = if *b { 1 } else { 0 };
                        self.call_sv("soplang_bool", &format!("i8 {n}"))
                    }
                    HirConst::Null => self.call_sv("soplang_null", ""),
                };
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::Copy { dst, src } => {
                let (t, p) = self.load_slot(*src);
                self.store_slot(*dst, &t, &p);
                if let Some(idx) = self.slot_fn.get(src).copied() {
                    self.slot_fn.insert(*dst, idx);
                }
                Ok(false)
            }
            HirInstr::Load { dst, name } => {
                if let Some(idx) = self.fn_index(name) {
                    self.slot_fn.insert(*dst, idx);
                }
                let (p, n) = self.str_ptr(name);
                let sv = self.call_sv("soplang_get_builtin", &format!("ptr {p}, i64 {n}"));
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::Store { name, src } => {
                let (p, n) = self.str_ptr(name);
                let (t, pay) = self.load_slot(*src);
                self.line(format!(
                    "  call void @soplang_store_global(ptr {p}, i64 {n}, i64 {t}, i64 {pay})"
                ));
                Ok(false)
            }
            HirInstr::BinOp {
                dst, op, lhs, rhs, ..
            } => {
                let sv = self.binop(*op, *lhs, *rhs);
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::UnOp { dst, op, src } => {
                let func = match op {
                    UnOpKind::Neg => "soplang_neg",
                    UnOpKind::Not => "soplang_not",
                };
                let sv = self.unary(func, *src);
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::Jump(id) | HirInstr::Break(id) | HirInstr::Continue(id) => {
                self.line(format!("  br label %L{id}"));
                Ok(true)
            }
            HirInstr::JumpIf {
                cond,
                on_true,
                on_false,
            } => {
                let (_, pay) = self.load_slot(*cond);
                let c = self.fresh();
                self.line(format!("  {c} = icmp ne i64 {pay}, 0"));
                self.line(format!(
                    "  br i1 {c}, label %L{on_true}, label %L{on_false}"
                ));
                Ok(true)
            }
            HirInstr::Call { dst, callee, args } => {
                if let Some(&idx) = self.slot_fn.get(callee) {
                    let llvm_name = self.fns[idx].llvm_name.clone();
                    let n_params = self.fns[idx].n_params;
                    let mut parts = Vec::new();
                    for i in 0..n_params {
                        if i < args.len() {
                            let (t, p) = self.load_slot(args[i]);
                            parts.push(self.pack(&t, &p));
                        } else {
                            parts.push("zeroinitializer".to_string());
                        }
                    }
                    let arg_s = parts
                        .iter()
                        .map(|p| {
                            if p == "zeroinitializer" {
                                "%SoplangValue zeroinitializer".to_string()
                            } else {
                                format!("%SoplangValue {p}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let sv = self.call_sv(&llvm_name, &arg_s);
                    self.unpack_store(&sv, *dst);
                } else {
                    let (ct, cp) = self.load_slot(*callee);
                    let csv = self.pack(&ct, &cp);
                    let sv = self.call_packed(&csv, args);
                    self.unpack_store(&sv, *dst);
                }
                Ok(false)
            }
            HirInstr::Return { val } => {
                if self.is_main {
                    self.line("  ret i32 0");
                } else {
                    let (t, p) = self.load_slot(*val);
                    let sv = self.pack(&t, &p);
                    self.line(format!("  ret %SoplangValue {sv}"));
                }
                Ok(true)
            }
            HirInstr::Pop { .. } => Ok(false),
            HirInstr::CheckType {
                src,
                type_tag,
                name,
            } => {
                let (t, p) = self.load_slot(*src);
                let (np, nlen) = self.str_ptr(name);
                self.line(format!(
                    "  call void @soplang_check_type(i64 {t}, i64 {p}, i64 {type_tag}, ptr {np}, i64 {nlen})"
                ));
                Ok(false)
            }
            other => Err(runtime_error(
                format!("LLVM IR does not support this instruction yet: {other}"),
                0,
                0,
            )),
        }
    }

    fn binop(&mut self, op: BinOpKind, lhs: usize, rhs: usize) -> String {
        let func = match op {
            BinOpKind::Add => "soplang_add",
            BinOpKind::Sub => "soplang_sub",
            BinOpKind::Mul => "soplang_mul",
            BinOpKind::Div => "soplang_div",
            BinOpKind::Mod => "soplang_mod",
            BinOpKind::Eq => "soplang_eq",
            BinOpKind::Ne => "soplang_ne",
            BinOpKind::Lt => "soplang_lt",
            BinOpKind::Le => "soplang_le",
            BinOpKind::Gt => "soplang_gt",
            BinOpKind::Ge => "soplang_ge",
            BinOpKind::And => "soplang_and",
            BinOpKind::Or => "soplang_or",
        };
        let (lt, lp) = self.load_slot(lhs);
        let (rt, rp) = self.load_slot(rhs);
        let a = self.pack(&lt, &lp);
        let b = self.pack(&rt, &rp);
        self.call_sv(func, &format!("%SoplangValue {a}, %SoplangValue {b}"))
    }

    fn unary(&mut self, func: &str, src: usize) -> String {
        let (t, p) = self.load_slot(src);
        let a = self.pack(&t, &p);
        self.call_sv(func, &format!("%SoplangValue {a}"))
    }

    fn call_packed(&mut self, callee: &str, args: &[usize]) -> String {
        let n = args.len();
        if n == 0 {
            return self.call_sv(
                "soplang_call",
                &format!("%SoplangValue {callee}, ptr null, i32 0"),
            );
        }
        let buf = self.fresh();
        self.line(format!("  {buf} = alloca [{n} x %SoplangValue], align 8"));
        for (i, &slot) in args.iter().enumerate() {
            let (t, p) = self.load_slot(slot);
            let sv = self.pack(&t, &p);
            let ep = self.fresh();
            self.line(format!(
                "  {ep} = getelementptr inbounds [{n} x %SoplangValue], ptr {buf}, i64 0, i64 {i}"
            ));
            self.line(format!("  store %SoplangValue {sv}, ptr {ep}, align 8"));
        }
        self.call_sv(
            "soplang_call",
            &format!("%SoplangValue {callee}, ptr {buf}, i32 {n}"),
        )
    }

    fn fn_index(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }
}

fn slots_needed(body: &[HirInstr], extra: usize) -> usize {
    slot_hi(body).max(extra).max(1)
}

fn slot_hi(body: &[HirInstr]) -> usize {
    let mut mx = 0usize;
    for instr in body {
        match instr {
            HirInstr::Const { dst, .. }
            | HirInstr::Load { dst, .. }
            | HirInstr::Pop { dst }
            | HirInstr::BindError { dst } => mx = mx.max(*dst + 1),
            HirInstr::Copy { dst, src } => mx = mx.max(*dst + 1).max(*src + 1),
            HirInstr::Store { src, .. } => mx = mx.max(*src + 1),
            HirInstr::BinOp { dst, lhs, rhs, .. } => {
                mx = mx.max(*dst + 1).max(*lhs + 1).max(*rhs + 1)
            }
            HirInstr::UnOp { dst, src, .. } => mx = mx.max(*dst + 1).max(*src + 1),
            HirInstr::BuildList { dst, items } => {
                mx = mx.max(*dst + 1);
                for s in items {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::BuildObject { dst, pairs } => {
                mx = mx.max(*dst + 1);
                for (_, s) in pairs {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::GetIndex { dst, obj, idx } => {
                mx = mx.max(*dst + 1).max(*obj + 1).max(*idx + 1)
            }
            HirInstr::SetIndex { obj, idx, val } => {
                mx = mx.max(*obj + 1).max(*idx + 1).max(*val + 1)
            }
            HirInstr::GetProp { dst, obj, .. } => mx = mx.max(*dst + 1).max(*obj + 1),
            HirInstr::SetProp { obj, val, .. } => mx = mx.max(*obj + 1).max(*val + 1),
            HirInstr::Call { dst, callee, args } => {
                mx = mx.max(*dst + 1).max(*callee + 1);
                for s in args {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::CallMethod { dst, obj, args, .. } => {
                mx = mx.max(*dst + 1).max(*obj + 1);
                for s in args {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::Return { val } => mx = mx.max(*val + 1),
            HirInstr::JumpIf { cond, .. } => mx = mx.max(*cond + 1),
            HirInstr::CheckType { src, .. } | HirInstr::CheckClass { src, .. } => {
                mx = mx.max(*src + 1)
            }
            HirInstr::NewInstance { dst, args, .. } => {
                mx = mx.max(*dst + 1);
                for s in args {
                    mx = mx.max(*s + 1);
                }
            }
            HirInstr::MakeClosure { dst, captures, .. } => {
                mx = mx.max(*dst + 1);
                for s in captures {
                    mx = mx.max(*s + 1);
                }
            }
            _ => {}
        }
    }
    mx
}

fn llvm_c_bytes(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'\"' => out.push_str("\\22"),
            32..=126 => out.push(b as char),
            _ => out.push_str(&format!("\\{b:02X}")),
        }
    }
    out
}
