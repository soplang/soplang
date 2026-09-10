# Soplang Compiler Plan

> **Soplang is a compiled language.** The tree-walking interpreter lives in the separate [`soplang-interpreter`](https://github.com/soplang/soplang-interpreter) repo. This tree runs programs with **Cranelift**. `soplang --build` produces a standalone binary by embedding the program in a Cargo runner. A direct **LLVM** backend is still phase 5.

## Current status

Phases 1–4 and 6 are done. Phase 5 (LLVM IR via `inkwell`) has not started.

| Path | What it does today |
|------|--------------------|
| `soplang file.sop` | Lex → parse → `keen` → analyze → HIR → Cranelift JIT → run |
| REPL | One `ReplSession`: the same JIT module, class registry, and globals across lines |
| `soplang --build file.sop` | Inline `keen` from the source path, embed that text in a Cargo runner, JIT at process start |
| Static `abn` / `jajab` | Native `i64` / `f64` ops in the JIT. `door` stays on the tagged runtime |

Division and modulo by zero still call the runtime, including inside `fasax`.

---

## Table of Contents

1. [Current status](#current-status)
2. [Goals](#goals)
3. [Architecture Overview](#architecture-overview)
4. [Pipeline Stages](#pipeline-stages)
5. [Value Representation](#value-representation)
6. [Runtime Library](#runtime-library)
7. [Cranelift JIT Design](#cranelift-jit-design)
8. [LLVM AOT Design](#llvm-aot-design)
9. [Phase 1 — Semantic Analyzer](#phase-1--semantic-analyzer)
10. [Phase 2 — HIR (High-Level IR)](#phase-2--hir-high-level-ir)
11. [Phase 3 — Runtime Library](#phase-3--runtime-library)
12. [Phase 4 — Cranelift JIT Backend](#phase-4--cranelift-jit-backend)
13. [Phase 5 — LLVM AOT Backend](#phase-5--llvm-aot-backend)
14. [Phase 6 — Remove Interpreter, Final Wiring](#phase-6--remove-interpreter-final-wiring)
15. [Crate Layout](#crate-layout)
16. [Validation + Timeline](#validation--timeline)

---

## Goals

- **Compiled-only**: no tree-walking interpreter in the final product.
- **Two backends** sharing the same front-end (Lexer, Parser, AST, Semantic, HIR):
  - **Cranelift JIT**: `soplang file.sop` → HIR → native machine code in memory → run. Near-native speed, fast compile, ~5 MB dep.
  - **Build today:** `soplang --build file.sop` embeds the program (with `keen` inlined) in a Cargo runner. The binary JIT-compiles at startup.
  - **LLVM AOT (phase 5, not started):** HIR → LLVM IR → object file → standalone binary, aimed at stronger optimization of static `abn` and `jajab`.
- **Reuse**: Lexer, Parser, AST, `Value`, `SoplangError`, stdlib (wrapped by runtime for native backends).
- **Remove interpreter**: `src/interpreter.rs` deleted in Phase 6.

---

## Architecture Overview

```
  Source (.sop)
       │
       ▼
  ┌─────────────────────────────┐
  │  Lexer  →  Token stream     │  (existing)
  └─────────────────────────────┘
       │
       ▼
  ┌─────────────────────────────┐
  │  Parser  →  AST             │  (existing)
  └─────────────────────────────┘
       │
       ▼
  ┌─────────────────────────────────────────────────────┐
  │  Semantic Analyzer  (Phase 1)                        │
  │  • Name resolution & scope building                  │
  │  • Variable slot assignment (local #0, #1, …)        │
  │  • Closure analysis (which vars escape to heap?)     │
  │  • Static type checking (abn, jajab, qoraal, …)      │
  │  • Constant folding & propagation                    │
  └─────────────────────────────────────────────────────┘
       │  Annotated AST + Symbol Table
       ▼
  ┌─────────────────────────────────────────────────────┐
  │  HIR Lowering  (Phase 2)                             │
  │  Flat, linear IR.  No nested AST.  SSA-ready.        │
  │  Variables are slots, jumps are explicit labels.     │
  └─────────────────────────────────────────────────────┘
       │  HIR (HirModule)
       ▼
  ┌────────────────────────────────────────────────────────────────────────┐
  │  TWO BACKENDS  (same HIR → two targets)                                 │
  ├─────────────────────────────────┬──────────────────────────────────────┤
  │  Cranelift JIT                   │  LLVM AOT                            │
  │  HIR → Cranelift IR              │  HIR → LLVM IR                        │
  │  → native code in memory         │  → .o → link → standalone binary     │
  │  Used for: soplang file.sop      │  Used for: soplang build file.sop     │
  │  (Phase 4, done)                │  (Phase 5, not started)               │
  └─────────────────────────────────┴──────────────────────────────────────┘
```

`--build` does not take the LLVM column yet. It lowers nothing into LLVM IR. See [Current status](#current-status).

---

## Pipeline Stages

| Stage | Module | Input → Output | Phase |
|-------|--------|----------------|-------|
| Lexer | `lexer.rs` | Source → Tokens | existing |
| Parser | `parser.rs` | Tokens → AST | existing |
| Semantic Analyzer | `semantic.rs` | AST → Annotated AST + SymbolTable | 1 |
| HIR Lowering | `hir.rs` | Annotated AST → HIR | 2 |
| Runtime Library | `runtime.rs` | C-ABI functions called by both backends | 3 |
| Cranelift Backend | `backend/cranelift.rs` | HIR → native code (JIT) | 4 |
| LLVM Backend | `backend/llvm.rs` | HIR → LLVM IR → binary | 5 |

---

## Value Representation

Both Cranelift and LLVM use the same **tagged value** for dynamic types (`door`):

```
SoplangValue = { tag: u8, payload: i64 }   (C ABI: 16 bytes, passed as two i64 in practice)

Tag  Meaning    Payload
──── ─────────  ────────────────────────────────
 0   Null       0
 1   Int        i64 value directly
 2   Float      f64 bit-cast to i64
 3   Bool       0 or 1
 4   Str        pointer to heap String (as i64)
 5   List       pointer to Rc<Vec<Value>> (as i64)
 6   Object     pointer to Rc<HashMap> (as i64)
 7   Function   pointer to closure struct (as i64)
```

**Static types (`abn`, `jajab`) skip boxing in both backends:**
- `abn x = 42` → emit native `i64` in Cranelift/LLVM; no struct, no runtime call.
- `jajab y = 3.14` → emit native `f64` (double).  
This is where Soplang's static type system gives large optimization wins.

---

## Runtime Library

Both backends call a small Rust runtime via C ABI. New file: `src/runtime.rs`.

```rust
#[repr(C)]
pub struct SoplangValue { pub tag: u8, pub payload: i64 }

// Primitives
#[no_mangle] pub extern "C" fn soplang_int(n: i64) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_float(x: f64) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_str(ptr: *const u8, len: usize) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_bool(b: bool) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_null() -> SoplangValue { ... }

// Arithmetic (dynamic dispatch)
#[no_mangle] pub extern "C" fn soplang_add(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_sub(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_mul(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_div(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_mod(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_neg(a: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_not(a: SoplangValue) -> SoplangValue { ... }

// Comparison
#[no_mangle] pub extern "C" fn soplang_eq(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_ne(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_lt(a: SoplangValue, b: SoplangValue) -> SoplangValue { ... }
// ... le, gt, ge

// IO / stdlib
#[no_mangle] pub extern "C" fn soplang_qor(v: SoplangValue) { ... }
#[no_mangle] pub extern "C" fn soplang_gelin() -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_nooc(v: SoplangValue) -> SoplangValue { ... }

// Collections
#[no_mangle] pub extern "C" fn soplang_list_new() -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_list_push(list: SoplangValue, val: SoplangValue) { ... }
#[no_mangle] pub extern "C" fn soplang_object_new() -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_get_index(obj: SoplangValue, idx: SoplangValue) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_set_index(obj: SoplangValue, idx: SoplangValue, val: SoplangValue) { ... }
#[no_mangle] pub extern "C" fn soplang_get_prop(obj: SoplangValue, name: *const u8, len: usize) -> SoplangValue { ... }
#[no_mangle] pub extern "C" fn soplang_set_prop(obj: SoplangValue, name: *const u8, len: usize, val: SoplangValue) { ... }

// Calls
#[no_mangle] pub extern "C" fn soplang_call(callee: SoplangValue, args: *const SoplangValue, n: i32) -> SoplangValue { ... }
```

Each function converts `SoplangValue` to/from the existing `Value` enum and delegates to existing stdlib logic where applicable.

---

## Cranelift JIT Design

**Crates:** `cranelift-codegen`, `cranelift-jit`, `cranelift-frontend`, `cranelift-module`.

### Role

- **Invocation:** `soplang file.sop` (default “run” path).
- **Flow:** HIR → Cranelift IR → native machine code in process memory → call compiled function.
- **No standalone binary:** user always runs the `soplang` executable with a `.sop` file.

### Value in Cranelift

Pass `SoplangValue` as two `i64` (tag widened): params/returns `(i64, i64)`. For static-typed locals use plain `i64` or `f64` and only box at boundaries (e.g. call, return, `qor`).

### Structure

```rust
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_codegen::ir::{types, AbiParam, InstBuilder};

pub struct CraneliftBackend {
    module:   JITModule,
    ctx:      cranelift_codegen::Context,
    fn_ctx:   FunctionBuilderContext,
}

impl CraneliftBackend {
    pub fn new() -> Self { ... }
    pub fn compile_module(&mut self, hir: &HirModule) -> Result<(), SoplangError> { ... }
    pub fn get_main_fn(&self) -> *const u8 { ... }  // entry point to call
}
```

### Optimization

Cranelift applies its own passes (constant propagation, dead code elimination, register allocation). Static `abn` and `jajab` slots emit `iadd`, `fadd`, and the matching compare and remainder ops, and still write the value tag so `qor` and stores keep working. `door` slots stay on runtime calls. The REPL keeps one `CraneliftBackend` and compiles each line with `compile_incremental`.

---

## LLVM AOT Design

**Status:** not implemented. `inkwell` is not a dependency. `soplang --build` is the Cargo runner described in [Current status](#current-status). The design below is the later phase 5 target.

**Crate:** `inkwell` (LLVM 17+ bindings).

### Role

- **Invocation:** `soplang build file.sop` (or `soplang build file.sop -o mybin`).
- **Flow:** HIR → LLVM IR → optimize (O2) → emit .o → link with runtime → standalone binary.
- **Standalone binary:** user gets an executable that runs without `soplang`.

### Value type in LLVM IR

```llvm
%SoplangValue = type { i8, i64 }
declare %SoplangValue @soplang_add(%SoplangValue, %SoplangValue)
declare %SoplangValue @soplang_int(i64)
declare void @soplang_qor(%SoplangValue)
; ... etc
```

### Static type optimization

For `abn x = 42; abn y = x + 3` emit pure `i64` arithmetic; LLVM constant-folds and optimizes. For `door` use `%SoplangValue` and runtime calls.

### Structure

```rust
pub struct LlvmBackend<'ctx> {
    ctx:      &'ctx Context,
    module:   Module<'ctx>,
    builder:  Builder<'ctx>,
    value_ty: StructType<'ctx>,
}

impl<'ctx> LlvmBackend<'ctx> {
    pub fn compile_module(&mut self, hir: &HirModule) -> Result<(), SoplangError> { ... }
    pub fn emit_object_file(&self, path: &Path) -> Result<(), SoplangError> { ... }
    pub fn link_binary(&self, obj_path: &Path, out_path: &Path) -> Result<(), SoplangError> { ... }
}
```

---

## Phase 1 — Semantic Analyzer

**Goal:** Walk the AST, build symbol table, resolve names, assign variable slots, analyse closures, validate static types.

### New file: `src/semantic.rs`

```rust
pub struct SymbolTable {
    pub scopes:    Vec<Scope>,
    pub functions: Vec<FunctionMeta>,
    pub classes:   HashMap<String, ClassMeta>,
}

pub struct Scope {
    pub vars: HashMap<String, VarInfo>,
}

pub struct VarInfo {
    pub slot:       usize,
    pub type_ann:   TypeAnnotation,
    pub is_const:   bool,
    pub is_captured: bool,
}

pub struct FunctionMeta {
    pub name:        String,
    pub param_slots: Vec<usize>,
    pub local_count: usize,
    pub captures:    Vec<String>,
}

pub fn analyze(stmts: &[Stmt]) -> Result<SymbolTable, SoplangError> { ... }
```

**Tasks:** push/pop scopes, assign slots, mark captured vars, validate static types, record `FunctionMeta` per `hawl`.

### Deliverable

- All example files pass semantic analysis. Symbol table has correct slots and metadata.

---

## Phase 2 — HIR (High-Level IR)

**Goal:** Define flat, backend-agnostic IR. Lower annotated AST → HIR.

### New file: `src/hir.rs`

```rust
pub enum HirInstr {
    Const { dst: Slot, val: HirConst },
    Copy { dst: Slot, src: Slot },
    Load { dst: Slot, name: String },
    Store { name: String, src: Slot },
    BinOp { dst: Slot, op: BinOpKind, lhs: Slot, rhs: Slot, typed: bool },
    UnOp { dst: Slot, op: UnOpKind, src: Slot },
    BuildList { dst: Slot, items: Vec<Slot> },
    BuildObject { dst: Slot, pairs: Vec<(String, Slot)> },
    GetIndex { dst: Slot, obj: Slot, idx: Slot },
    SetIndex { obj: Slot, idx: Slot, val: Slot },
    GetProp { dst: Slot, obj: Slot, prop: String },
    SetProp { obj: Slot, prop: String, val: Slot },
    Label(LabelId),
    Jump(LabelId),
    JumpIf { cond: Slot, on_true: LabelId, on_false: LabelId },
    Call { dst: Slot, callee: Slot, args: Vec<Slot> },
    CallMethod { dst: Slot, obj: Slot, method: String, args: Vec<Slot> },
    Return { val: Slot },
    Break(LabelId),
    Continue(LabelId),
    TryBegin { catch: LabelId },
    TryEnd,
    BindError { dst: Slot },
}

pub enum HirConst { Int(i64), Float(f64), Str(String), Bool(bool), Null }
pub type Slot = usize;
pub type LabelId = usize;

pub struct HirFunction {
    pub name:        String,
    pub params:      Vec<Slot>,
    pub local_count: usize,
    pub body:        Vec<HirInstr>,
    pub is_static:   bool,
}

pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub top_level:  Vec<HirInstr>,
}
```

**Lowering:** `HirLowering::lower(sym, stmts) -> HirModule` with backpatching for jumps.

### Deliverable

- `--dump-hir` prints valid HIR for all examples. No panics in lowering.

---

## Phase 3 — Runtime Library

**Goal:** Implement `src/runtime.rs` with all `extern "C"` functions that Cranelift and LLVM IR will call.

### Steps

1. Define `SoplangValue` (repr(C)) and conversion to/from existing `Value`.
2. Implement each runtime function (arithmetic, comparison, qor, gelin, list/object get/set, call).
3. Reuse existing stdlib logic behind these wrappers.
4. Ensure ABI is stable (e.g. two i64 for Value on 64-bit).

### Deliverable

- Runtime compiles. Unit tests or small C/JIT callers can call runtime and get correct results.

---

## Phase 4 — Cranelift JIT Backend

**Status:** done.

**Goal:** Full “run” path: source → … → HIR → Cranelift → native code → execute. All language features work.

### Steps

1. **HIR → Cranelift IR:** For each `HirFunction` and top-level block, emit Cranelift blocks and instructions. Use runtime calls for dynamic ops; use native `i64`/`f64` for static-typed slots.
2. **Control flow:** Map `Label`/`Jump`/`JumpIf` to Cranelift blocks and branches.
3. **Calls:** Compile user functions to Cranelift functions; `Call` compiles to indirect or direct call with correct signature (SoplangValue = two i64).
4. **Closures:** Closure value = function pointer + env pointer; pass env into compiled function as extra arg.
5. **Classes/methods:** `cusub` and method dispatch via runtime helpers or compiled stubs that call runtime.
6. **Import:** Lex/parse/analyze/lower/compile imported file; merge “globals” into current JIT context or call into compiled module.
7. **Try/catch:** Unwind or setjmp/longjmp to catch block; store error in slot.
8. **REPL:** Each input is a small module; compile and run; keep globals in a persistent JIT context.

### Entry point

```rust
pub fn run_source(source: &str, path: Option<&Path>) -> Result<(), SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let ast    = Parser::new(tokens).parse()?;
    let sym    = semantic::analyze(&ast)?;
    let hir    = hir::HirLowering::lower(&sym, &ast);
    let mut jit = backend::cranelift::CraneliftBackend::new();
    jit.compile_module(&hir)?;
    jit.run_main()
}
```

### Deliverable

- All 43 examples produce correct output via `soplang file.sop`. Integration tests (`.expected`) pass.

---

## Phase 5 — LLVM AOT Backend

**Status:** not started. Standalone binaries today come from the Cargo runner (`src/backend/llvm.rs` is that runner, not LLVM IR).

**Goal:** `soplang --build file.sop` emits a real native binary from HIR, without JIT-compiling the source at startup.

### Steps

1. **HIR → LLVM IR:** Same HIR as Cranelift; emit LLVM IR via inkwell (runtime declarations, `%SoplangValue`, static vs dynamic as in Cranelift).
2. **Optimization:** Run O2 (or configurable) passes on the module.
3. **Emit .o:** Write object file; link with runtime (same `runtime.rs` compiled into a static lib or linked from the soplang binary).
4. **CLI:** `soplang build <file.sop> [-o output]` → compile → link → output binary.

### Deliverable

- `soplang build examples/hello.sop -o hello` produces `./hello` that runs and matches expected output.

---

## Phase 6 — Remove Interpreter, Final Wiring

**Status:** done for the interpreter. The run path is Cranelift. The build path is the Cargo runner until phase 5.

**Goal:** Interpreter removed. Single run path (Cranelift) and single build path (LLVM). Tests and docs updated.

### Steps

1. Delete `src/interpreter.rs` and any interpreter-only code.
2. Ensure `lib.rs` and `main.rs` use only semantic → HIR → Cranelift (run) or LLVM (build).
3. REPL/shell: compile each line with Cranelift and run in a persistent context.
4. Tests: all use `run_source` (Cranelift); no references to `Interpreter`.
5. Benchmarks: measure Cranelift run and LLVM-built binary; update RESULTS.md.
6. Docs: README and COMPILER_PLAN state “compiled language; Cranelift for run, LLVM for build”.

### Deliverable

- No interpreter in tree. `cargo test` green. `soplang file.sop` and `soplang build file.sop` documented and working.

---

## Crate Layout

```
src/
├── token.rs
├── lexer.rs
├── ast.rs
├── parser.rs
├── value.rs
├── scope.rs
├── error.rs
├── stdlib.rs
│
├── semantic.rs       Phase 1
├── hir.rs            Phase 2
├── runtime.rs        Phase 3 — C-ABI for both backends
│
├── backend/
│   ├── mod.rs
│   ├── cranelift.rs  Phase 4 — JIT (soplang file.sop)
│   └── llvm.rs       Phase 5 — AOT (soplang build file.sop)
│
├── shell.rs          REPL: compile + run via Cranelift
├── main.rs           CLI: run vs build
├── lib.rs            run_source (Cranelift), build_source (LLVM)
│
└── interpreter.rs    DELETED Phase 6
```

---

## Validation + Timeline

| Phase | Focus | Validation |
|-------|--------|------------|
| 1 | Semantic Analyzer | Done |
| 2 | HIR | Done |
| 3 | Runtime Library | Done |
| 4 | Cranelift JIT | Done. `soplang file.sop` and the REPL session |
| 5 | LLVM AOT | Not started. `--build` is the Cargo runner |
| 6 | Remove interpreter | Done. No interpreter in this repo |

### Backend summary

| Backend | Command | Output | Dep |
|---------|---------|--------|-----|
| **Cranelift JIT** | `soplang file.sop` | Run in process; no standalone binary | cranelift-* ~5 MB |
| **Cargo runner** | `soplang --build file.sop` | Standalone binary that JIT-compiles embedded source | cranelift (same as run) |
| **LLVM AOT** | later replacement for `--build` | Standalone binary from LLVM IR | inkwell + LLVM, not added yet |

Next work is **Phase 5** (real LLVM). Do not treat `src/backend/llvm.rs` as that backend.
