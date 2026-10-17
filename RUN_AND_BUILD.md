# Running Soplang 3 and building executables

Soplang runs files through Cranelift JIT and builds standalone executables
through LLVM. See [installation](docs/installation.md) for release packages and
[build requirements](docs/INSTALLING_PACKAGES.md) for the native toolchain.

## Build the compiler from source

From the repository root, with Rust 1.94.0 and a system linker installed:

```sh
cargo build --release --locked
./target/release/soplang --version
```

On Windows the executable is `target/release/soplang.exe`. Native Soplang builds
also need clang and the platform linker/SDK.

## Run a program

```sh
./target/release/soplang examples/01_hello.sop
printf 'TestUser\n' | ./target/release/soplang examples/13_input.sop
./target/release/soplang -c 'qor("Salaan!")'
./target/release/soplang -c '1 + 2'
```

A single command expression prints its result. File execution prints explicit
`qor` calls. Errors include the original source file, line, column, and caret.
Use `--no-color` or `NO_COLOR=1` for plain diagnostics.

## REPL

```sh
./target/release/soplang
./target/release/soplang -i examples/01_hello.sop
```

The REPL keeps functions, classes, and globals between inputs and echoes single
expressions. It supports multiline input and history. Type `/caawi` or `/help`
for commands, and `/bixi` or `/exit` to leave. `-i` after a file opens the shell
when that program finishes.

## Build a standalone executable

```sh
./target/release/soplang --build examples/01_hello.sop
./barnaamij/01_hello

./target/release/soplang --build examples/27_inheritance.sop -o inheritance
./inheritance
```

The default output is `barnaamij/<source-stem>`; `-o` selects another path.
`--opt-level 0..3` selects native optimization, with 2 as the default. Add
`--quiet` to suppress the build progress and success message.

The build resolves imports, lowers HIR to textual LLVM IR, compiles it with
clang, and links the matching runtime with rustc. The output contains compiled
code and the runtime. It can run without Soplang or its source files, while
retaining the target platform's ordinary system-library requirements.

Installed `--build` uses the runtime libraries bundled with its package. Keep
those libraries with the executable, and use the exact Rust compiler recorded
in `lib/soplang/rustc-version.txt`.

## Check every example and diagnostic

```sh
cargo test --locked --test examples_tests
cargo test --locked --test diagnostic_tests
cargo test --locked --test llvm_ir_tests
```

The example tests supply stdin, check expected stdout, and exercise both JIT
and native executables. The diagnostic suite checks 63 intentional failures
and their source locations. See the [example catalog](examples/README.md).

For performance measurements, use the
[Python/v2/v3 benchmark suite](benchmarks/comparison/README.md). Its report
separates execution latency from LLVM build costs.
