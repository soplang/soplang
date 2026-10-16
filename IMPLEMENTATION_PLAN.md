# Soplang Rust Implementation Plan

> Step-by-step execution plan for rewriting Soplang in Rust. Each phase is self-contained and produces a verifiable deliverable. (The legacy Python implementation has been moved to [soplang/soplang-interpreter](https://github.com/soplang/soplang-interpreter).)

**Status: Phases 1–7 complete.** The Rust implementation (lexer, parser, stdlib, REPL, CLI, tests, benchmarks, Cranelift JIT, AOT) is the runtime. **Compiler work** continues in **[COMPILER_PLAN.md](COMPILER_PLAN.md)**.

---

## Table of Contents

1. [Goals](#goals)
2. [Repository Layout](#repository-layout)
3. [Rust Crate Design](#rust-crate-design)
4. [Phase 1 — Project Bootstrap + Lexer](#phase-1--project-bootstrap--lexer)
5. [Phase 2 — AST + Parser](#phase-2--ast--parser)
6. [Phase 3 — Values + Environment + Core Interpreter](#phase-3--values--environment--core-interpreter)
7. [Phase 4 — Functions + Classes + Import + Try/Catch](#phase-4--functions--classes--import--trycatch)
8. [Phase 5 — Standard Library](#phase-5--standard-library)
9. [Phase 6 — REPL + CLI + Error Messages](#phase-6--repl--cli--error-messages)
10. [Phase 7 — Tests, Benchmarks, Final Wiring](#phase-7--tests-benchmarks-final-wiring)
11. [Crate Dependencies](#crate-dependencies)
12. [Validation Strategy](#validation-strategy)
13. [Timeline Summary](#timeline-summary)

---

## Goals

- Produce a **single native binary** (`soplang`) that runs `.sop` files and provides an interactive REPL
- Be **100% feature-compatible** — every `.sop` example must produce correct output
- Improve performance for compute-heavy programs
- Improve error messages with coloured, line-highlighted output
- Use the **project root** as the Rust crate; all Rust source lives in `src/` at root

---

## Repository Layout

The Rust implementation lives at **project root** with Rust source in `src/`. After the implementation is complete the layout will look like:

```
soplang/                 ← Rust project root (Cargo.toml here)
├── Cargo.toml           ← Rust crate manifest
├── src/                 ← Rust implementation (primary)
│   ├── main.rs
│   ├── token.rs
│   ├── lexer.rs
│   ├── ast.rs
│   ├── parser.rs
│   ├── value.rs
│   ├── env.rs
│   ├── interpreter.rs
│   ├── stdlib.rs
│   ├── error.rs
│   └── shell.rs
├── tests/               ← Rust integration tests
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   ├── interpreter_tests.rs
│   └── examples_tests.rs
├── examples/            ← shared .sop example files
├── docs/
├── ANALYSIS.md
├── IMPLEMENTATION_PLAN.md
└── README.md
```

---

## Rust Crate Design

```
src/
│
├── token.rs        TokenType enum + Token struct
├── lexer.rs        Lexer struct: source → Vec<Token>
│
├── ast.rs          Expr enum + Stmt enum + TypeAnnotation enum
├── parser.rs       Parser struct: Vec<Token> → Vec<Stmt>
│
├── value.rs        Value enum (runtime values)
├── env.rs          Environment struct (scoped variable store)
├── interpreter.rs  Interpreter struct: Vec<Stmt> → side effects
│
├── stdlib.rs       All built-in functions and methods
├── error.rs        SoplangError enum + Display (Somali messages)
│
├── shell.rs        Interactive REPL (rustyline)
└── main.rs         CLI entry point (clap)
```

---

## Phase 1 — Project Bootstrap + Lexer

**Goal:** A `cargo run -- file.sop` that tokenises a `.sop` file and prints the token stream.

### Steps

#### 1.1 — Scaffold Rust project

The repo root is the Rust crate. If there is no `Cargo.toml` yet:

```bash
# From repo root (soplang/)
cargo init --name soplang
```

This creates `Cargo.toml` and `src/main.rs` at the project root. All Rust code goes in `src/`.

#### 1.2 — `token.rs` — Token types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Door, Madoor, Hawl, Celi, Qor, Gelin,
    Haddii, HaddiiKale, Ugudambeyn,
    Dooro, Xaalad, Kuceli, Intay,
    Jooji, Soco, Fasax, Qabo,
    Keen, Qaab, Dhaxal, Cusub, Nafta,
    // Static type keywords
    Abn, Jajab, Qoraal, Bool, Teed, Walax,
    // Literals
    Identifier, Number, StringLit,
    True, False, Null,
    // Operators
    Plus, Minus, Star, Slash, Modulo,
    EqEq, NotEq, Greater, Less, GreaterEq, LessEq,
    And, Or, Not, Assign,
    // Structural
    Comma, Colon, Semicolon,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Dot, Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind:   TokenType,
    pub lexeme: String,     // raw text from source
    pub line:   usize,
    pub col:    usize,
}
```

#### 1.3 — `error.rs` — Error skeleton

```rust
#[derive(Debug)]
pub enum SoplangError {
    Lexer   { msg: String, line: usize, col: usize },
    Parser  { msg: String, line: usize, col: usize },
    Runtime { msg: String, line: usize, col: usize },
    Type    { msg: String, line: usize, col: usize },
    Import  { msg: String, line: usize, col: usize },
}

impl std::fmt::Display for SoplangError { ... }  // prints Somali messages
```

Use the `thiserror` crate to derive `Error`.

#### 1.4 — `lexer.rs` — Lexer

```rust
pub struct Lexer<'a> {
    source: &'a str,
    chars:  std::iter::Peekable<std::str::Chars<'a>>,
    line:   usize,
    col:    usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self { ... }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, SoplangError> { ... }
    fn next_token(&mut self) -> Result<Token, SoplangError> { ... }
    fn read_identifier(&mut self, first: char) -> Token { ... }
    fn read_number(&mut self, first: char) -> Token { ... }
    fn read_string(&mut self, quote: char) -> Result<Token, SoplangError> { ... }
    fn skip_line_comment(&mut self) { ... }
    fn skip_block_comment(&mut self) -> Result<(), SoplangError> { ... }
    fn peek(&mut self) -> Option<char> { ... }
    fn advance(&mut self) -> Option<char> { ... }
}
```

Keyword lookup — use a `phf_map!` (compile-time perfect hash) or `HashMap` populated once:

```rust
fn keyword(s: &str) -> Option<TokenType> {
    match s {
        "door"       => Some(TokenType::Door),
        "madoor"     => Some(TokenType::Madoor),
        "hawl"       => Some(TokenType::Hawl),
        // ... all keywords
        "run"        => Some(TokenType::True),
        "been"       => Some(TokenType::False),
        "maran"      => Some(TokenType::Null),
        _            => None,
    }
}
```

#### 1.5 — `main.rs` — CLI skeleton

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = std::fs::read_to_string(&args[1]).unwrap();
    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => tokens.iter().for_each(|t| println!("{:?}", t)),
        Err(e)     => eprintln!("{}", e),
    }
}
```

### Deliverable
`cargo run -- examples/hello.sop` prints all tokens with line/col information.

---

## Phase 2 — AST + Parser

**Goal:** Parse any `.sop` file into a typed, printable AST.

### Steps

#### 2.1 — `ast.rs` — Typed AST nodes

Replace the Python generic `ASTNode` with separate strongly-typed enums:

```rust
#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Abn, Jajab, Qoraal, Bool, Teed, Walax, Dynamic,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp  { op: String, left: Box<Expr>, right: Box<Expr> },
    UnaryOp   { op: String, expr: Box<Expr> },
    Call      { name: String, args: Vec<Expr> },
    MethodCall{ obj: Box<Expr>, method: String, args: Vec<Expr> },
    Index     { obj: Box<Expr>, idx: Box<Expr> },
    Property  { obj: Box<Expr>, prop: String },
    List      (Vec<Expr>),
    Object    (Vec<(String, Expr)>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64), Float(f64), Str(String), Bool(bool), Null,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl  { name: String, type_ann: TypeAnnotation, is_const: bool, value: Expr, line: usize },
    Assign   { target: Expr, value: Expr, line: usize },
    FuncDef  { name: String, params: Vec<Param>, body: Vec<Stmt> },
    ClassDef { name: String, parent: Option<String>, body: Vec<Stmt> },
    If       { cond: Expr, then_body: Vec<Stmt>, elseifs: Vec<(Expr,Vec<Stmt>)>, else_body: Option<Vec<Stmt>> },
    Switch   { expr: Expr, cases: Vec<(Expr,Vec<Stmt>)>, default: Option<Vec<Stmt>> },
    For      { var: String, start: Expr, end: Expr, step: Option<Expr>, body: Vec<Stmt> },
    While    { cond: Expr, body: Vec<Stmt> },
    Return   (Option<Expr>),
    Break,
    Continue,
    TryCatch { try_body: Vec<Stmt>, err_var: String, catch_body: Vec<Stmt> },
    Import   (String),
    Block    (Vec<Stmt>),
    Expr     (Expr),
}
```

#### 2.2 — `parser.rs` — Recursive descent parser

```rust
pub struct Parser {
    tokens:  Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { ... }
    pub fn parse(&mut self) -> Result<Vec<Stmt>, SoplangError> { ... }

    // Statement parsers
    fn parse_stmt(&mut self)             -> Result<Stmt, SoplangError> { ... }
    fn parse_var_decl(&mut self, ...)    -> Result<Stmt, SoplangError> { ... }
    fn parse_func_def(&mut self)         -> Result<Stmt, SoplangError> { ... }
    fn parse_class_def(&mut self)        -> Result<Stmt, SoplangError> { ... }
    fn parse_if(&mut self)               -> Result<Stmt, SoplangError> { ... }
    fn parse_switch(&mut self)           -> Result<Stmt, SoplangError> { ... }
    fn parse_for(&mut self)              -> Result<Stmt, SoplangError> { ... }
    fn parse_while(&mut self)            -> Result<Stmt, SoplangError> { ... }
    fn parse_try_catch(&mut self)        -> Result<Stmt, SoplangError> { ... }
    fn parse_import(&mut self)           -> Result<Stmt, SoplangError> { ... }

    // Expression parsers (precedence chain)
    fn parse_logical(&mut self)          -> Result<Expr, SoplangError> { ... }
    fn parse_comparison(&mut self)       -> Result<Expr, SoplangError> { ... }
    fn parse_additive(&mut self)         -> Result<Expr, SoplangError> { ... }
    fn parse_multiplicative(&mut self)   -> Result<Expr, SoplangError> { ... }
    fn parse_unary(&mut self)            -> Result<Expr, SoplangError> { ... }
    fn parse_postfix(&mut self)          -> Result<Expr, SoplangError> { ... }
    fn parse_primary(&mut self)          -> Result<Expr, SoplangError> { ... }

    // Helpers
    fn advance(&mut self) -> &Token { ... }
    fn peek(&self) -> &Token { ... }
    fn check(&self, kind: &TokenType) -> bool { ... }
    fn expect(&mut self, kind: TokenType) -> Result<&Token, SoplangError> { ... }
    fn at_end(&self) -> bool { ... }
}
```

#### 2.3 — AST pretty-printer (debug only)

Implement `Display` for `Expr` and `Stmt` to print a tree view for debugging.

### Deliverable
`cargo run -- examples/hello.sop --ast` prints the parsed AST. Parser handles all 43 example files without errors.

---

## Phase 3 — Values + Environment + Core Interpreter

**Goal:** Execute basic programs: variable declarations, arithmetic, comparisons, if/switch, for/while loops, print.

### Steps

#### 3.1 — `value.rs` — Runtime value type

```rust
use std::rc::Rc;
use std::cell::RefCell;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<IndexMap<String, Value>>>),
    Function(Rc<FunctionDef>),
    NativeFunction(fn(Vec<Value>) -> Result<Value, SoplangError>),
    Null,
}

#[derive(Debug)]
pub struct FunctionDef {
    pub name:   String,
    pub params: Vec<String>,
    pub body:   Vec<Stmt>,
    pub env:    Rc<RefCell<Env>>,  // captured env for closures (Phase 4)
}
```

Implement `PartialEq`, `Display`, and a `type_name() -> &'static str` method on `Value`.

#### 3.2 — `env.rs` — Scoped environment

```rust
pub struct Env {
    vars:   HashMap<String, Value>,
    types:  HashMap<String, TypeAnnotation>,
    consts: HashSet<String>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self { ... }
    pub fn new_child(parent: Rc<RefCell<Env>>) -> Self { ... }
    pub fn define(&mut self, name: &str, value: Value, type_ann: TypeAnnotation, is_const: bool) { ... }
    pub fn get(&self, name: &str) -> Option<Value> { ... }       // walks parent chain
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), SoplangError> { ... }
    pub fn get_type(&self, name: &str) -> Option<TypeAnnotation> { ... }
    pub fn is_const(&self, name: &str) -> bool { ... }
}
```

#### 3.3 — Control flow signal type

```rust
pub enum Signal {
    None,
    Break,
    Continue,
    Return(Value),
}
```

#### 3.4 — `interpreter.rs` — Core interpreter

```rust
pub struct Interpreter {
    globals: Rc<RefCell<Env>>,
    classes: HashMap<String, ClassDef>,
}

impl Interpreter {
    pub fn new() -> Self { ... }

    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), SoplangError> { ... }

    // Statement execution
    fn exec_stmt(&mut self, stmt: &Stmt, env: Rc<RefCell<Env>>) -> Result<Signal, SoplangError> { ... }
    fn exec_var_decl(&mut self, ...) -> Result<Signal, SoplangError> { ... }
    fn exec_assign(&mut self, ...) -> Result<Signal, SoplangError> { ... }
    fn exec_if(&mut self, ...) -> Result<Signal, SoplangError> { ... }
    fn exec_switch(&mut self, ...) -> Result<Signal, SoplangError> { ... }
    fn exec_for(&mut self, ...) -> Result<Signal, SoplangError> { ... }
    fn exec_while(&mut self, ...) -> Result<Signal, SoplangError> { ... }
    fn exec_block(&mut self, body: &[Stmt], env: Rc<RefCell<Env>>) -> Result<Signal, SoplangError> { ... }

    // Expression evaluation
    fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Env>>) -> Result<Value, SoplangError> { ... }
    fn eval_binary(&mut self, op: &str, l: Value, r: Value, line: usize) -> Result<Value, SoplangError> { ... }
    fn eval_unary(&mut self, op: &str, v: Value) -> Result<Value, SoplangError> { ... }

    // Type validation
    fn validate_type(&self, name: &str, val: &Value, ann: &TypeAnnotation, line: usize) -> Result<(), SoplangError> { ... }
}
```

**Binary operator implementation:**

| Op | `Int+Int` | `Float+Float` | `Str+Str` | `Int+Str` |
|----|-----------|--------------|-----------|-----------|
| `+` | `Int` | `Float` | concatenate | auto-convert int to string |
| `-` `*` `/` `%` | numeric | numeric | TypeError | TypeError |
| `==` `!=` `>` etc. | `Bool` | `Bool` | `Bool` (lexicographic) | TypeError |
| `&&` `\|\|` | short-circuit → `Bool` | | | |

### Deliverable
`cargo run -- examples/01_dynamic_typing.sop` through `examples/09_while_loops.sop` all produce correct output.

---

## Phase 4 — Functions + Classes + Import + Try/Catch

**Goal:** Full language support.

### Steps

#### 4.1 — User-defined functions

```rust
fn exec_func_def(&mut self, name: &str, params: &[Param], body: &[Stmt], env: Rc<RefCell<Env>>)
    -> Result<Signal, SoplangError>
{
    let func_def = FunctionDef { name, params, body, env: env.clone() };
    env.borrow_mut().define(name, Value::Function(Rc::new(func_def)), TypeAnnotation::Dynamic, false);
    Ok(Signal::None)
}

fn call_function(&mut self, func: &FunctionDef, args: Vec<Value>, line: usize)
    -> Result<Value, SoplangError>
{
    // Create child env from the function's captured env (true lexical scoping)
    let call_env = Rc::new(RefCell::new(Env::new_child(func.env.clone())));
    for (param, arg) in func.params.iter().zip(args.iter()) {
        call_env.borrow_mut().define(param, arg.clone(), TypeAnnotation::Dynamic, false);
    }
    match self.exec_block(&func.body, call_env)? {
        Signal::Return(v) => Ok(v),
        _                 => Ok(Value::Null),
    }
}
```

#### 4.2 — Class definitions and instantiation

```rust
struct ClassDef {
    name:    String,
    parent:  Option<String>,
    methods: HashMap<String, FunctionDef>,
}
```

`cusub ClassName(args)`:
1. Look up `ClassDef` in `self.classes`
2. Create `Value::Object(Rc::new(RefCell::new(IndexMap::from([("__class__", Value::Str(name))]))))`
3. If class has `dhaw` method, call it with `nafta = instance` as first arg
4. Return the instance

Method calls on instance:
1. Walk class chain (current → parent → ...) to find method
2. Bind `nafta` to instance in a new env
3. Execute method body

#### 4.3 — Import (`keen`)

```rust
fn exec_import(&mut self, filename: &str, current_file: &Path, env: Rc<RefCell<Env>>)
    -> Result<Signal, SoplangError>
{
    let path = current_file.parent().unwrap().join(filename);
    let source = std::fs::read_to_string(&path)
        .map_err(|_| SoplangError::Import { msg: format!("Faylka '{}' ma helin", filename), .. })?;
    let tokens  = Lexer::new(&source).tokenize()?;
    let stmts   = Parser::new(tokens).parse()?;
    // Execute in the same env (flat import, matching Python behaviour for now)
    self.exec_block(&stmts, env)?;
    Ok(Signal::None)
}
```

#### 4.4 — Try/catch (`fasax/qabo`)

```rust
fn exec_try_catch(&mut self, try_body: &[Stmt], err_var: &str, catch_body: &[Stmt], env: Rc<RefCell<Env>>)
    -> Result<Signal, SoplangError>
{
    match self.exec_block(try_body, env.clone()) {
        Ok(signal) => Ok(signal),
        Err(e) => {
            env.borrow_mut().define(err_var, Value::Str(e.to_string()), TypeAnnotation::Dynamic, false);
            self.exec_block(catch_body, env)
        }
    }
}
```

### Deliverable
All 43 example files run correctly. Functions, OOP, imports all working.

---

## Phase 5 — Standard Library

**Goal:** Every built-in function and method produces results byte-identical to the Python implementation.

### Steps

#### 5.1 — `stdlib.rs` structure

```rust
pub fn get_builtin_functions() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("qor".into(),    Value::NativeFunction(builtin_qor));
    m.insert("gelin".into(),  Value::NativeFunction(builtin_gelin));
    m.insert("nooc".into(),   Value::NativeFunction(builtin_nooc));
    // ... all 14 builtins
    m
}

fn builtin_qor(args: Vec<Value>) -> Result<Value, SoplangError> {
    let s = value_to_string(args.into_iter().next().unwrap_or(Value::Null));
    println!("{}", s);
    Ok(Value::Str(s))
}
```

#### 5.2 — `value_to_string()` — matches Python `qoraal()`

- `Bool(true)` → `"run"`, `Bool(false)` → `"been"`
- `Null` → `"maran"`
- `Int(n)` → `n.to_string()`
- `Float(f)` → if `f == f.floor()` → `"3"`, else `"3.14"` (match Python's float repr)
- `List(...)` → `"[item1, item2]"` (recursive)
- `Object(...)` → `"{'key': value}"` (recursive)

#### 5.3 — Method dispatch in interpreter

```rust
fn eval_method_call(&mut self, obj_expr: &Expr, method: &str, arg_exprs: &[Expr], env: Rc<RefCell<Env>>)
    -> Result<Value, SoplangError>
{
    let receiver = self.eval_expr(obj_expr, env.clone())?;
    let args: Vec<Value> = arg_exprs.iter()
        .map(|a| self.eval_expr(a, env.clone()))
        .collect::<Result<_,_>>()?;

    match &receiver {
        Value::List(lst)   => dispatch_list_method(method, lst.clone(), args),
        Value::Object(obj) => dispatch_object_method(method, obj.clone(), args),
        Value::Str(s)      => dispatch_string_method(method, s.clone(), args),
        _                  => Err(SoplangError::Runtime { msg: format!("..."), .. }),
    }
}
```

#### 5.4 — Implement all methods

Implement the full method tables from the analysis:

- **List (12):** `kasaar`, `dherer`, `kudar`, `leeyahay`, `nuqul`, `nadiifi`, `rog`, `habee`, `shaandhee`, `jar`, `aaddin`, `muuji`
- **Object (8):** `fure`, `leeyahay`, `tir`, `kudar`, `nuqul`, `nadiifi`, `qiime`, `lamaane`
- **String (12):** `qeybi`, `leeyahay`, `dhamaad`, `bilow`, `beddel`, `beddel_dhammaan`, `kudar`, `jar`, `xarafaha_weyn`, `xarfaha_yaryar`, `masax`, `raadi`

For `shaandhee` (filter) and `aaddin` (map), the argument is a `Value::Function` — call it through the interpreter's function-call machinery.

### Deliverable
`python psrc/check_examples.py` passes all tests. A new `cargo test -- --test-output immediate` runs examples and compares stdout.

---

## Phase 6 — REPL + CLI + Error Messages

**Goal:** Production-quality shell and CLI.

### Steps

#### 6.1 — `shell.rs` — Interactive REPL

```toml
# Cargo.toml
rustyline = "14"
```

```rust
use rustyline::{Editor, error::ReadlineError};

pub struct Shell {
    interpreter: Interpreter,
    editor:      Editor<()>,
}

impl Shell {
    pub fn run(&mut self) {
        loop {
            match self.editor.readline("soplang> ") {
                Ok(line) => {
                    self.editor.add_history_entry(&line);
                    self.execute(&line);
                }
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    fn execute(&mut self, source: &str) {
        let result = Lexer::new(source).tokenize()
            .and_then(|t| Parser::new(t).parse())
            .and_then(|s| self.interpreter.run_stmts(s));
        if let Err(e) = result {
            eprintln!("{}", e);
        }
    }
}
```

#### 6.2 — `main.rs` — Full CLI with `clap`

```toml
clap = { version = "4", features = ["derive"] }
```

```rust
#[derive(Parser)]
#[command(name = "soplang", about = "The Somali Programming Language")]
struct Cli {
    #[arg(short = 'v', long)] version:     bool,
    #[arg(short = 'c', long)] command:     Option<String>,
    #[arg(short = 'f', long)] file:        Option<PathBuf>,
    #[arg(short = 'e', long)] example:     Option<usize>,
    #[arg(short = 'i', long)] interactive: bool,
    filename: Option<PathBuf>,
}
```

Flags:
| Flag | Action |
|------|--------|
| `-v` / `--version` | print version and exit |
| `-c CODE` | execute code snippet, exit |
| `-f FILE` | execute file, exit |
| `-e N` | run example number N from `examples/` |
| `-i` | open interactive shell after file execution |
| `filename` | positional file argument |
| _(no args)_ | open interactive shell |

#### 6.3 — Coloured error output

```toml
colored = "2"
```

Error display format:

```
[Khalad runtime] sadar 7, goobta 12
  Doorsame aan la qeexin: 'x'

   7 │   qor(x)
         ^^^
```

Implement a `format_error_with_source(err, source)` function that:
1. Extracts the line from the source string
2. Adds a `^^^` pointer under the relevant column
3. Colours the error header red and the pointer red

### Deliverable
`./target/release/soplang` (or `cargo run --release`) behaves identically to `python -m psrc` for all flags and interactive use.

---

## Phase 7 — Tests, Benchmarks, Final Wiring

**Goal:** Verified, benchmarked Rust implementation becomes the primary binary.

### Steps

#### 7.1 — Unit tests

Create `tests/` at project root with:

```
lexer_tests.rs       — tokenise known inputs, assert token types and lexemes
parser_tests.rs      — parse snippets, assert AST structure
interpreter_tests.rs — run short programs, assert returned values
examples_tests.rs    — run each example file, assert stdout
```

Example test:

```rust
#[test]
fn test_hello_world() {
    let output = run_program(r#"qor("Salaan, Adduunka!")"#);
    assert_eq!(output, "Salaan, Adduunka!\n");
}

#[test]
fn test_for_loop() {
    let output = run_program(r#"
        kuceli (i 1 ilaa 4) {
            qor(i)
        }
    "#);
    assert_eq!(output, "1\n2\n3\n");
}
```

#### 7.2 — Integration tests against examples

```rust
#[test_each::file(glob = "examples/*.sop")]
fn test_example_file(path: &Path) {
    let expected = read_expected_output(path);  // from .expected file or Python run
    let actual   = run_file(path);
    assert_eq!(actual, expected);
}
```

Pre-generate expected output files by running `python -m psrc examples/XX.sop > examples/XX.expected` for all 43 examples.

#### 7.3 — Benchmarks

```toml
[dev-dependencies]
criterion = "0.5"
```

```rust
fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib_30", |b| b.iter(|| run_file("examples/bench_fib.sop")));
}
```

Compare `cargo bench` results vs `python -m psrc` using `hyperfine`.

#### 7.4 — Update root `Makefile`

```makefile
# Rust targets (project root is the Rust crate)
build:
	cargo build --release

run:
	./target/release/soplang $(FILE)

test-rust:
	cargo test

bench:
	cargo bench
```

#### 7.5 — Update root `README.md`

- Change primary "Running Soplang" section to use `cargo build --release` and `./target/release/soplang`
- Move Python instructions to "Legacy / Reference — psrc/"
- Add benchmark results section

### Deliverable
- All Rust unit + integration tests pass (`cargo test`)
- Rust binary produces byte-identical output for all 43 examples
- Benchmark shows measurable speedup over Python
- `make build` + `make run FILE=examples/hello.sop` works from repo root

---

## Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 1.x | Ergonomic `Error` derive for `SoplangError` |
| `indexmap` | 2.x | `IndexMap` for ordered object keys (preserves insertion order) |
| `rustyline` | 14.x | REPL: readline, history, tab completion |
| `clap` | 4.x | CLI argument parsing with derive macros |
| `colored` | 2.x | Coloured terminal output for errors |
| `criterion` | 0.5 | Benchmarks |

Optional / future:
| Crate | Purpose |
|-------|---------|
| `phf` | Compile-time perfect hash for keyword lookup |
| `unicode-segmentation` | Correct Unicode character counting in strings |

---

## Validation Strategy

After each phase, run the following checks before moving to the next phase:

```bash
# Run example files with Rust binary and compare to Python output
for f in examples/*.sop; do
    expected=$(python -m psrc "$f" 2>&1)
    actual=$(./target/release/soplang "$f" 2>&1)
    if [ "$expected" != "$actual" ]; then
        echo "MISMATCH: $f"
        diff <(echo "$expected") <(echo "$actual")
    fi
done
```

Phase-specific checkpoints:

| Phase | Check |
|-------|-------|
| 1 | Lexer outputs correct token stream for all examples |
| 2 | Parser produces no errors on all 43 example files |
| 3 | Examples 01–09 (variables, ops, control flow, loops) match Python |
| 4 | Examples 10–20 (functions, classes, import) match Python |
| 5 | Examples 20–43 (stdlib, string/list/object methods) match Python |
| 6 | CLI flags, REPL, and error messages match Python shell |
| 7 | All 43 examples pass; `cargo test` 100% green |

---

## Timeline Summary

| Phase | Module(s) | Key deliverable |
|-------|-----------|----------------|
| **1** | `token.rs`, `lexer.rs`, `error.rs` | Working lexer, token stream printout |
| **2** | `ast.rs`, `parser.rs` | Typed AST, all examples parse cleanly |
| **3** | `value.rs`, `env.rs`, `interpreter.rs` (core) | Variables, arithmetic, control flow, loops |
| **4** | `interpreter.rs` (functions/classes/import) | Functions, OOP, imports, try/catch |
| **5** | `stdlib.rs` | All 46 built-in functions and methods |
| **6** | `shell.rs`, `main.rs` | REPL, CLI flags, coloured errors |
| **7** | `tests/`, benchmarks, Makefile | 100% test coverage, benchmarks, primary binary |

When ready, say **"start Phase 1"** to begin scaffolding the Rust project and implementing the lexer.
