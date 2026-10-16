# Debugging Soplang

This document covers debugging Soplang programs and inspecting the compiler. Programs run through the lexer, parser, semantic analysis, HIR, and the Cranelift JIT.

## Table of Contents

- [Debugging Soplang Programs](#debugging-soplang-programs)
  - [Understanding Error Messages](#understanding-error-messages)
  - [Adding Debug Output](#adding-debug-output)
  - [Step-by-Step Debugging](#step-by-step-debugging)
  - [Common Runtime Errors](#common-runtime-errors)

- [Debugging the compiler](#debugging-the-compiler)
  - [Parse tree and HIR](#parse-tree-and-hir)
  - [A single test](#a-single-test)
  - [Stepping through the Rust code](#stepping-through-the-rust-code)

## Debugging Soplang Programs

### Understanding Error Messages

Soplang error messages follow this structure:

```
Khalad [type]: [message] at line [line], position [position]
```

Where:
- `[type]` can be one of:
  - `lexer` - Error in tokenizing the source code
  - `parser` - Error in parsing the tokens into AST
  - `runtime` - Error during program execution
  - `type` - Type error (wrong type for an operation)
- `[message]` is a description of the error
- `[line]` and `[position]` indicate where the error occurred

#### Example Error Messages

1. **Lexical Error**
   ```
   Khalad lexer: Unexpected character: @ at line 5, position 3
   ```

2. **Parsing Error**
   ```
   Khalad parser: Expected ')', got '+' at line 7, position 15
   ```

3. **Runtime Error**
   ```
   Khalad runtime: Undefined variable: 'foo' at line 10
   ```

4. **Type Error**
   ```
   Khalad type: 'x' waa abn laakin qiimaheeda "hello" ma ahan abn
   ```

### Adding Debug Output

The most straightforward way to debug Soplang programs is to add debug output using the `qor()` function:

```
// Debug variable values
qor("Debug - x: " + x)
qor("Debug - y: " + y)

// Debug control flow
qor("Entering loop")
kuceli i 0 ilaa 5 {
    qor("  Loop iteration: " + i)
}
qor("Exiting loop")

// Debug conditional branches
haddii (condition) {
    qor("Condition is true")
} ugudambeyn {
    qor("Condition is false")
}
```

### Step-by-Step Debugging

For complex issues, use step-by-step execution:

1. Add a comment to split your program into logical sections:
   ```
   // ==== SECTION 1: Initialize variables ====
   door x = 10
   door y = 20

   // ==== SECTION 2: Calculate result ====
   door result = x * y + 5
   ```

2. Add debug points between sections:
   ```
   qor("Debug: After initialization, x=" + x + ", y=" + y)
   ```

3. Comment out subsequent sections to isolate problems:
   ```
   // ==== SECTION 1: Initialize variables ====
   door x = 10
   door y = 20
   qor("Debug: x=" + x + ", y=" + y)

   /* // Temporarily comment out
   // ==== SECTION 2: Calculate result ====
   door result = x * y + 5
   */
   ```

### Common Runtime Errors

#### 1. Type Errors

**Error**: `Khalad type: [variable] waa [type] laakin qiimaheeda [value] ma ahan [type]`

**Solution**: Check that you're using variables with the correct types and explicitly convert types when needed:
```
abn num = abn(userInput)  // Convert string to number
```

#### 2. Undefined Variables

**Error**: `Khalad runtime: Undefined variable: [name]`

**Solution**: Make sure the variable is declared before use and check for typos:
```
door myVar = 10  // Declare first
qor(myVar)       // Then use
```

#### 3. Invalid Operations

**Error**: `Khalad runtime: Cannot [operation] on [type]`

**Solution**: Ensure you're using the right operations for the data type:
```
// Wrong: door text = "Hello" * 5
// Right: door text = "Hello" + " World"
```

#### 4. Scope Issues

**Error**: Variable not accessible or has unexpected value

**Solution**: Remember that variables declared inside a block are only accessible within that block:
```
haddii (condition) {
    door x = 10  // Only visible inside this block
}
// qor(x)  // Error: x is not defined here
```

## Debugging the compiler

### Parse tree and HIR

`--ast` prints the parse tree and does not run the program. `--hir` prints the flat instructions for each function and for the top level, and also does not run.

```bash
cargo build --release
./target/release/soplang --ast examples/08_functions.sop
./target/release/soplang --hir examples/08_functions.sop
```

Use `--ast` when a program is rejected while parsing, or when a statement is bound to the wrong form. Use `--hir` when parsing succeeds and the value or control flow is wrong: look for the slot copies, calls, and returns of that function.

### A single test

```bash
cargo test --test interpreter_tests test_hello_world -- --nocapture
cargo test --test semantic_tests test_semantic_class_typed_var -- --nocapture
```

`--nocapture` shows `qor` output and compiler errors from that test.

### Stepping through the Rust code

Build the debug binary and run it under a debugger:

```bash
cargo build
rust-gdb --args target/debug/soplang examples/01_hello.sop
```

`rust-lldb` works the same way on macOS. Break in `run_source`, `HirLowering::lower`, or `CraneliftBackend::compile_function` depending on the stage you care about.

JIT time is measured with `cargo bench`. See [benchmarks/README.md](../../benchmarks/README.md).

## Conclusion

For a Soplang program, `qor` at the points you care about is still the fastest check. For the compiler, `--ast` and `--hir` show the two representations before native code, and `cargo test` locks the behavior.

See [TEST_METHODOLOGY.md](TEST_METHODOLOGY.md) for the suites and [EXAMPLES.md](EXAMPLES.md) for the example programs.
