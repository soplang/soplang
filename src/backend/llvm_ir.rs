//! LLVM IR that calls the existing runtime.
//!
//! Constants, arithmetic, branches, loops, functions, lists, objects, classes,
//! `waalid`, `fasax`, closures, and native `abn`/`jajab` ops.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{runtime_error, SoplangError};
use crate::hir::{BinOpKind, HirConst, HirFunction, HirInstr, HirModule, SlotNum, UnOpKind};
use crate::semantic::SymbolTable;

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
declare %SoplangValue @soplang_list_new()
declare %SoplangValue @soplang_list_push(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_object_new()
declare %SoplangValue @soplang_get_index(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_set_index(%SoplangValue, %SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_get_prop(%SoplangValue, ptr, i64)
declare %SoplangValue @soplang_set_prop(%SoplangValue, ptr, i64, %SoplangValue)
declare %SoplangValue @soplang_call_method(%SoplangValue, ptr, i64, ptr, i32)
declare %SoplangValue @soplang_new_instance(ptr, i64, ptr, i32)
declare void @soplang_register_class(ptr, i64, ptr, i64, ptr, i64, i64)
declare i64 @soplang_register_user_fn(ptr, i64, ptr, i64)
declare i32 @soplang_try_begin()
declare void @soplang_try_end()
declare i32 @soplang_has_pending_error()
declare %SoplangValue @soplang_get_caught_error()
declare void @soplang_check_class(i64, i64, ptr, i64, ptr, i64)
declare void @soplang_mark_const(ptr, i64)
declare %SoplangValue @soplang_make_closure(ptr, i64, i64, ptr)
declare %SoplangValue @soplang_capture_get(i64, i64)
"#;

struct FnInfo {
    llvm_name: String,
    /// ABI values, including the leading closure environment when present.
    n_params: usize,
    has_env: bool,
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
    kinds: Vec<SlotNum>,
    seeded: Vec<bool>,
    slot_fn: HashMap<usize, usize>,
    is_main: bool,
    cont: usize,
}

pub fn emit_llvm_ir(hir: &HirModule, sym: &SymbolTable) -> Result<String, SoplangError> {
    let mut fns = Vec::new();
    let mut by_name = HashMap::new();
    for (i, f) in hir.functions.iter().enumerate() {
        let has_env = !f.capture_slots.is_empty();
        by_name.insert(f.name.clone(), i);
        fns.push(FnInfo {
            llvm_name: format!("sfn{i}"),
            n_params: f.params.len() + usize::from(has_env),
            has_env,
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
    emit_main(&mut m, &fns, &by_name, hir, sym)?;

    Ok(format!("{DECLS}\n{}\n{}", m.strings, m.body))
}

/// Compile `ir` and link it with the Soplang runtime.
/// `main` is the LLVM function. The Rust entry is `#![no_main]` so nothing calls `run_source`.
pub fn link_llvm_binary(ir: &str, out: &Path) -> Result<(), SoplangError> {
    link_llvm_binary_with_opt(ir, out, 2)
}

/// Compile and link LLVM IR with the requested optimization level (0..3).
pub fn link_llvm_binary_with_opt(ir: &str, out: &Path, opt_level: u8) -> Result<(), SoplangError> {
    let (rlib, deps) = soplang_rlib()?;
    let dir = std::env::temp_dir();
    let tag = out
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("soplang_llvm");
    let ll = dir.join(format!("{tag}.ll"));
    let obj = dir.join(format!("{tag}.o"));
    let wrapper = dir.join(format!("{tag}.rs"));
    fs::write(&ll, ir).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
    fs::write(&wrapper, LINK_WRAPPER).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| runtime_error(e.to_string(), 0, 0))?;
        }
    }
    let clang = Command::new("clang")
        .arg(format!("-O{}", opt_level.min(3)))
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
        .arg("-C")
        .arg(format!("opt-level={}", opt_level.min(3)))
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
static KEEP: [Keep; 43] = [
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
    Keep(soplang::runtime::soplang_list_new as *const ()),
    Keep(soplang::runtime::soplang_list_push as *const ()),
    Keep(soplang::runtime::soplang_object_new as *const ()),
    Keep(soplang::runtime::soplang_get_index as *const ()),
    Keep(soplang::runtime::soplang_set_index as *const ()),
    Keep(soplang::runtime::soplang_get_prop as *const ()),
    Keep(soplang::runtime::soplang_set_prop as *const ()),
    Keep(soplang::runtime::soplang_call_method as *const ()),
    Keep(soplang::runtime::soplang_new_instance as *const ()),
    Keep(soplang::runtime::soplang_register_class as *const ()),
    Keep(soplang::runtime::soplang_register_user_fn as *const ()),
    Keep(soplang::runtime::soplang_try_begin as *const ()),
    Keep(soplang::runtime::soplang_try_end as *const ()),
    Keep(soplang::runtime::soplang_has_pending_error as *const ()),
    Keep(soplang::runtime::soplang_get_caught_error as *const ()),
    Keep(soplang::runtime::soplang_check_class as *const ()),
    Keep(soplang::runtime::soplang_mark_const as *const ()),
    Keep(soplang::runtime::soplang_make_closure as *const ()),
    Keep(soplang::runtime::soplang_capture_get as *const ()),
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
    let has_env = info.has_env;
    let llvm_name = info.llvm_name.clone();
    let n_params = info.n_params;
    let cap = f
        .capture_slots
        .iter()
        .copied()
        .max()
        .map(|s| s + 1)
        .unwrap_or(0);
    let nslots = slots_needed(
        &f.body,
        f.local_count
            .max(f.params.iter().copied().max().map(|s| s + 1).unwrap_or(0))
            .max(cap),
    );
    let mut args = String::new();
    for i in 0..n_params {
        if i > 0 {
            args.push_str(", ");
        }
        args.push_str(&format!("%SoplangValue %a{i}"));
    }
    m.body.push_str(&format!(
        "define %SoplangValue @{}({args}) {{\nentry:\n",
        llvm_name
    ));
    let mut emit = FnEmit {
        m,
        fns,
        by_name,
        tmp: 0,
        nslots,
        kinds: numeric_kinds(&f.local_nums, nslots),
        seeded: numeric_seeds(&f.local_nums, nslots),
        slot_fn: HashMap::new(),
        is_main: false,
        cont: 0,
    };
    emit.alloc_slots();
    let user_base = if has_env {
        let env_pay = emit.fresh();
        emit.line(format!("  {env_pay} = extractvalue %SoplangValue %a0, 1"));
        for (i, &slot) in f.capture_slots.iter().enumerate() {
            let sv = emit.call_sv("soplang_capture_get", &format!("i64 {env_pay}, i64 {i}"));
            emit.unpack_store(&sv, slot);
        }
        1
    } else {
        0
    };
    for (i, &slot) in f.params.iter().enumerate() {
        emit.unpack_store(&format!("%a{}", user_base + i), slot);
    }
    emit.body(&f.body)?;
    emit.m.body.push_str("}\n\n");
    Ok(())
}

fn emit_main(
    m: &mut ModuleOut,
    fns: &[FnInfo],
    by_name: &HashMap<String, usize>,
    hir: &HirModule,
    sym: &SymbolTable,
) -> Result<(), SoplangError> {
    let nslots = slots_needed(&hir.top_level, 0);
    m.body.push_str("define i32 @main() {\nentry:\n");
    let mut emit = FnEmit {
        m,
        fns,
        by_name,
        tmp: 0,
        nslots,
        kinds: numeric_kinds(&hir.top_nums, nslots),
        seeded: numeric_seeds(&hir.top_nums, nslots),
        slot_fn: HashMap::new(),
        is_main: true,
        cont: 0,
    };
    emit.alloc_slots();
    emit.register_runtime(&hir.functions, sym);
    emit.body(&hir.top_level)?;
    emit.m.body.push_str("}\n");
    Ok(())
}

fn numeric_kinds(seed: &[SlotNum], nslots: usize) -> Vec<SlotNum> {
    (0..nslots)
        .map(|i| seed.get(i).copied().unwrap_or(SlotNum::Dyn))
        .collect()
}

fn numeric_seeds(seed: &[SlotNum], nslots: usize) -> Vec<bool> {
    (0..nslots)
        .map(|i| seed.get(i).copied().unwrap_or(SlotNum::Dyn) != SlotNum::Dyn)
        .collect()
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

    fn unpack(&mut self, sv: &str) -> (String, String) {
        let t8 = self.fresh();
        let tag = self.fresh();
        let pay = self.fresh();
        self.line(format!("  {t8} = extractvalue %SoplangValue {sv}, 0"));
        self.line(format!("  {tag} = zext i8 {t8} to i64"));
        self.line(format!("  {pay} = extractvalue %SoplangValue {sv}, 1"));
        (tag, pay)
    }

    fn unpack_store(&mut self, sv: &str, slot: usize) {
        let (tag, pay) = self.unpack(sv);
        self.store_slot(slot, &tag, &pay);
    }

    fn value_buf(&mut self, slots: &[usize]) -> String {
        if slots.is_empty() {
            return "null".to_string();
        }
        let n = slots.len();
        let buf = self.fresh();
        self.line(format!("  {buf} = alloca [{n} x %SoplangValue], align 8"));
        for (i, &slot) in slots.iter().enumerate() {
            let (t, p) = self.load_slot(slot);
            let sv = self.pack(&t, &p);
            let ep = self.fresh();
            self.line(format!(
                "  {ep} = getelementptr inbounds [{n} x %SoplangValue], ptr {buf}, i64 0, i64 {i}"
            ));
            self.line(format!("  store %SoplangValue {sv}, ptr {ep}, align 8"));
        }
        buf
    }

    fn register_runtime(&mut self, functions: &[HirFunction], sym: &SymbolTable) {
        for f in functions {
            let idx = self.by_name[&f.name];
            let info = &self.fns[idx];
            let llvm_name = info.llvm_name.clone();
            let n_params = info.n_params;
            let (p, n) = self.str_ptr(&f.name);
            self.line(format!(
                "  call i64 @soplang_register_user_fn(ptr {p}, i64 {n}, ptr @{llvm_name}, i64 {n_params})"
            ));
        }
        for (name, meta) in &sym.classes {
            for (method, mangled) in &meta.methods {
                let (np, nl) = self.str_ptr(mangled);
                let sv = self.call_sv("soplang_get_builtin", &format!("ptr {np}, i64 {nl}"));
                let (_, pay) = self.unpack(&sv);
                let (cp, cl) = self.str_ptr(name);
                let (mp, ml) = self.str_ptr(method);
                let (pp, pl) = match &meta.parent {
                    Some(parent) => {
                        let (p, n) = self.str_ptr(parent);
                        (p, n.to_string())
                    }
                    None => ("null".to_string(), "0".to_string()),
                };
                self.line(format!(
                    "  call void @soplang_register_class(ptr {cp}, i64 {cl}, ptr {pp}, i64 {pl}, ptr {mp}, i64 {ml}, i64 {pay})"
                ));
            }
        }
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
                self.set_kind(
                    *dst,
                    match val {
                        HirConst::Int(_) => SlotNum::Int,
                        HirConst::Float(_) => SlotNum::Float,
                        _ => SlotNum::Dyn,
                    },
                );
                Ok(false)
            }
            HirInstr::Copy { dst, src } => {
                let (t, p) = self.load_slot(*src);
                self.store_slot(*dst, &t, &p);
                self.set_kind(*dst, self.kind(*src));
                if let Some(idx) = self.slot_fn.get(src).copied() {
                    self.slot_fn.insert(*dst, idx);
                }
                Ok(false)
            }
            HirInstr::Load { dst, name } => {
                if let Some(idx) = self.fn_index(name) {
                    if !self.fns[idx].has_env {
                        self.slot_fn.insert(*dst, idx);
                    }
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
                dst,
                op,
                lhs,
                rhs,
                typed,
            } => {
                let lk = self.kind(*lhs);
                let rk = self.kind(*rhs);
                let native = if *typed && lk != SlotNum::Dyn && rk != SlotNum::Dyn {
                    self.native_binop(*op, *lhs, *rhs, lk, rk)
                } else {
                    None
                };
                let (tag, pay, kind) = if let Some(result) = native {
                    result
                } else {
                    let sv = self.binop(*op, *lhs, *rhs);
                    let (tag, pay) = self.unpack(&sv);
                    (tag, pay, SlotNum::Dyn)
                };
                self.store_slot(*dst, &tag, &pay);
                self.set_kind(*dst, kind);
                Ok(false)
            }
            HirInstr::UnOp { dst, op, src } => {
                let kind = self.kind(*src);
                let native = if *op == UnOpKind::Neg && kind != SlotNum::Dyn {
                    self.native_neg(*src, kind)
                } else {
                    None
                };
                let (tag, pay, result_kind) = if let Some(result) = native {
                    result
                } else {
                    let func = match op {
                        UnOpKind::Neg => "soplang_neg",
                        UnOpKind::Not => "soplang_not",
                    };
                    let sv = self.unary(func, *src);
                    let (tag, pay) = self.unpack(&sv);
                    (tag, pay, SlotNum::Dyn)
                };
                self.store_slot(*dst, &tag, &pay);
                self.set_kind(*dst, result_kind);
                Ok(false)
            }
            HirInstr::Label(id) => {
                self.line(format!("L{id}:"));
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
            HirInstr::BuildList { dst, items } => {
                let mut sv = self.call_sv("soplang_list_new", "");
                for &item in items {
                    let (lt, lp) = self.unpack(&sv);
                    let list = self.pack(&lt, &lp);
                    let (it, ip) = self.load_slot(item);
                    let iv = self.pack(&it, &ip);
                    sv = self.call_sv(
                        "soplang_list_push",
                        &format!("%SoplangValue {list}, %SoplangValue {iv}"),
                    );
                }
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::BuildObject { dst, pairs } => {
                let mut sv = self.call_sv("soplang_object_new", "");
                for (key, slot) in pairs {
                    let (ot, op) = self.unpack(&sv);
                    let obj = self.pack(&ot, &op);
                    let (kp, kl) = self.str_ptr(key);
                    let (vt, vp) = self.load_slot(*slot);
                    let val = self.pack(&vt, &vp);
                    sv = self.call_sv(
                        "soplang_set_prop",
                        &format!("%SoplangValue {obj}, ptr {kp}, i64 {kl}, %SoplangValue {val}"),
                    );
                }
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::GetIndex { dst, obj, idx } => {
                let sv = self.call_two("soplang_get_index", *obj, *idx);
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::SetIndex { obj, idx, val } => {
                let _ = self.call_three("soplang_set_index", *obj, *idx, *val);
                Ok(false)
            }
            HirInstr::GetProp { dst, obj, prop } => {
                let objv = self.pack_slot(*obj);
                let (pp, pl) = self.str_ptr(prop);
                let sv = self.call_sv(
                    "soplang_get_prop",
                    &format!("%SoplangValue {objv}, ptr {pp}, i64 {pl}"),
                );
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::SetProp { obj, prop, val } => {
                let objv = self.pack_slot(*obj);
                let valv = self.pack_slot(*val);
                let (pp, pl) = self.str_ptr(prop);
                let _ = self.call_sv(
                    "soplang_set_prop",
                    &format!("%SoplangValue {objv}, ptr {pp}, i64 {pl}, %SoplangValue {valv}"),
                );
                Ok(false)
            }
            HirInstr::CallMethod {
                dst,
                obj,
                method,
                args,
            } => {
                let objv = self.pack_slot(*obj);
                let (mp, ml) = self.str_ptr(method);
                let buf = self.value_buf(args);
                let n = args.len();
                let sv = self.call_sv(
                    "soplang_call_method",
                    &format!("%SoplangValue {objv}, ptr {mp}, i64 {ml}, ptr {buf}, i32 {n}"),
                );
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::NewInstance {
                dst,
                class_name,
                args,
            } => {
                let (cp, cl) = self.str_ptr(class_name);
                let buf = self.value_buf(args);
                let n = args.len();
                let sv = self.call_sv(
                    "soplang_new_instance",
                    &format!("ptr {cp}, i64 {cl}, ptr {buf}, i32 {n}"),
                );
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::CheckClass {
                src,
                class_name,
                var_name,
            } => {
                let (t, p) = self.load_slot(*src);
                let (cp, cl) = self.str_ptr(class_name);
                let (np, nl) = self.str_ptr(var_name);
                self.line(format!(
                    "  call void @soplang_check_class(i64 {t}, i64 {p}, ptr {cp}, i64 {cl}, ptr {np}, i64 {nl})"
                ));
                Ok(false)
            }
            HirInstr::MarkConst { name } => {
                let (np, nl) = self.str_ptr(name);
                self.line(format!(
                    "  call void @soplang_mark_const(ptr {np}, i64 {nl})"
                ));
                Ok(false)
            }
            HirInstr::TryBegin { .. } => {
                let v = self.fresh();
                self.line(format!("  {v} = call i32 @soplang_try_begin()"));
                Ok(false)
            }
            HirInstr::TryEnd => {
                self.line("  call void @soplang_try_end()");
                Ok(false)
            }
            HirInstr::CheckThrow { catch } => {
                let pending = self.fresh();
                let cmp = self.fresh();
                let cont = self.cont;
                self.cont += 1;
                self.line(format!(
                    "  {pending} = call i32 @soplang_has_pending_error()"
                ));
                self.line(format!("  {cmp} = icmp ne i32 {pending}, 0"));
                self.line(format!("  br i1 {cmp}, label %L{catch}, label %K{cont}"));
                self.line(format!("K{cont}:"));
                Ok(false)
            }
            HirInstr::BindError { dst } => {
                let sv = self.call_sv("soplang_get_caught_error", "");
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
            HirInstr::MakeClosure {
                dst,
                fn_name,
                captures,
            } => {
                let (np, nl) = self.str_ptr(fn_name);
                let buf = self.value_buf(captures);
                let n = captures.len();
                let sv = self.call_sv(
                    "soplang_make_closure",
                    &format!("ptr {np}, i64 {nl}, i64 {n}, ptr {buf}"),
                );
                self.unpack_store(&sv, *dst);
                Ok(false)
            }
        }
    }

    fn pack_slot(&mut self, slot: usize) -> String {
        let (t, p) = self.load_slot(slot);
        self.pack(&t, &p)
    }

    fn call_two(&mut self, func: &str, a: usize, b: usize) -> String {
        let av = self.pack_slot(a);
        let bv = self.pack_slot(b);
        self.call_sv(func, &format!("%SoplangValue {av}, %SoplangValue {bv}"))
    }

    fn call_three(&mut self, func: &str, a: usize, b: usize, c: usize) -> String {
        let av = self.pack_slot(a);
        let bv = self.pack_slot(b);
        let cv = self.pack_slot(c);
        self.call_sv(
            func,
            &format!("%SoplangValue {av}, %SoplangValue {bv}, %SoplangValue {cv}"),
        )
    }

    fn kind(&self, slot: usize) -> SlotNum {
        self.kinds.get(slot).copied().unwrap_or(SlotNum::Dyn)
    }

    fn set_kind(&mut self, slot: usize, kind: SlotNum) {
        if !self.seeded[slot] {
            self.kinds[slot] = kind;
        }
    }

    fn as_float(&mut self, payload: &str, kind: SlotNum) -> String {
        let value = self.fresh();
        if kind == SlotNum::Float {
            self.line(format!("  {value} = bitcast i64 {payload} to double"));
        } else {
            self.line(format!("  {value} = sitofp i64 {payload} to double"));
        }
        value
    }

    fn native_neg(&mut self, src: usize, kind: SlotNum) -> Option<(String, String, SlotNum)> {
        let (_, payload) = self.load_slot(src);
        let result = self.fresh();
        match kind {
            SlotNum::Int => {
                self.line(format!("  {result} = sub i64 0, {payload}"));
                Some(("1".into(), result, SlotNum::Int))
            }
            SlotNum::Float => {
                let value = self.as_float(&payload, kind);
                self.line(format!("  {result} = fneg double {value}"));
                let bits = self.fresh();
                self.line(format!("  {bits} = bitcast double {result} to i64"));
                Some(("2".into(), bits, SlotNum::Float))
            }
            SlotNum::Dyn => None,
        }
    }

    fn native_binop(
        &mut self,
        op: BinOpKind,
        lhs: usize,
        rhs: usize,
        lk: SlotNum,
        rk: SlotNum,
    ) -> Option<(String, String, SlotNum)> {
        if matches!(op, BinOpKind::And | BinOpKind::Or) {
            return None;
        }
        let (_, lp) = self.load_slot(lhs);
        let (_, rp) = self.load_slot(rhs);
        let use_float = op == BinOpKind::Div || lk == SlotNum::Float || rk == SlotNum::Float;
        if matches!(op, BinOpKind::Div | BinOpKind::Mod) {
            let zero = self.fresh();
            if rk == SlotNum::Int {
                self.line(format!("  {zero} = icmp eq i64 {rp}, 0"));
            } else {
                let divisor = self.as_float(&rp, rk);
                self.line(format!(
                    "  {zero} = fcmp oeq double {divisor}, 0.000000e+00"
                ));
            }
            let id = self.cont;
            self.cont += 1;
            let zero_label = format!("Nzero{id}");
            let ok_label = format!("Nok{id}");
            let merge_label = format!("Nmerge{id}");
            self.line(format!(
                "  br i1 {zero}, label %{zero_label}, label %{ok_label}"
            ));
            self.line(format!("{zero_label}:"));
            let sv = self.binop(op, lhs, rhs);
            let (zt, zp) = self.unpack(&sv);
            self.line(format!("  br label %{merge_label}"));
            self.line(format!("{ok_label}:"));
            let (nt, np, kind) = self.native_numeric(op, &lp, &rp, lk, rk, use_float);
            self.line(format!("  br label %{merge_label}"));
            self.line(format!("{merge_label}:"));
            let tag = self.fresh();
            let pay = self.fresh();
            self.line(format!(
                "  {tag} = phi i64 [{zt}, %{zero_label}], [{nt}, %{ok_label}]"
            ));
            self.line(format!(
                "  {pay} = phi i64 [{zp}, %{zero_label}], [{np}, %{ok_label}]"
            ));
            return Some((tag, pay, kind));
        }
        Some(self.native_numeric(op, &lp, &rp, lk, rk, use_float))
    }

    fn native_numeric(
        &mut self,
        op: BinOpKind,
        lp: &str,
        rp: &str,
        lk: SlotNum,
        rk: SlotNum,
        use_float: bool,
    ) -> (String, String, SlotNum) {
        if !use_float {
            let instruction = match op {
                BinOpKind::Add => "add",
                BinOpKind::Sub => "sub",
                BinOpKind::Mul => "mul",
                BinOpKind::Mod => "srem",
                BinOpKind::Eq => "icmp eq",
                BinOpKind::Ne => "icmp ne",
                BinOpKind::Lt => "icmp slt",
                BinOpKind::Le => "icmp sle",
                BinOpKind::Gt => "icmp sgt",
                BinOpKind::Ge => "icmp sge",
                _ => unreachable!(),
            };
            let value = self.fresh();
            self.line(format!("  {value} = {instruction} i64 {lp}, {rp}"));
            if matches!(
                op,
                BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Mod
            ) {
                return ("1".into(), value, SlotNum::Int);
            }
            let pay = self.fresh();
            self.line(format!("  {pay} = zext i1 {value} to i64"));
            return ("3".into(), pay, SlotNum::Dyn);
        }
        let l = self.as_float(lp, lk);
        let r = self.as_float(rp, rk);
        let instruction = match op {
            BinOpKind::Add => "fadd",
            BinOpKind::Sub => "fsub",
            BinOpKind::Mul => "fmul",
            BinOpKind::Div => "fdiv",
            BinOpKind::Mod => "frem",
            BinOpKind::Eq => "fcmp oeq",
            BinOpKind::Ne => "fcmp une",
            BinOpKind::Lt => "fcmp olt",
            BinOpKind::Le => "fcmp ole",
            BinOpKind::Gt => "fcmp ogt",
            BinOpKind::Ge => "fcmp oge",
            _ => unreachable!(),
        };
        let value = self.fresh();
        self.line(format!("  {value} = {instruction} double {l}, {r}"));
        if matches!(
            op,
            BinOpKind::Eq
                | BinOpKind::Ne
                | BinOpKind::Lt
                | BinOpKind::Le
                | BinOpKind::Gt
                | BinOpKind::Ge
        ) {
            let pay = self.fresh();
            self.line(format!("  {pay} = zext i1 {value} to i64"));
            ("3".into(), pay, SlotNum::Dyn)
        } else {
            let pay = self.fresh();
            self.line(format!("  {pay} = bitcast double {value} to i64"));
            ("2".into(), pay, SlotNum::Float)
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
