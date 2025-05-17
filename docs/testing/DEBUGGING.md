# Debugging Soplang

This document provides guidance on debugging both the Soplang interpreter itself and Soplang programs. It covers various techniques, tools, and common issues you might encounter.

## Table of Contents

- [Debugging Soplang Programs](#debugging-soplang-programs)
  - [Understanding Error Messages](#understanding-error-messages)
  - [Adding Debug Output](#adding-debug-output)
  - [Step-by-Step Debugging](#step-by-step-debugging)
  - [Common Runtime Errors](#common-runtime-errors)

- [Debugging the Interpreter](#debugging-the-interpreter)
  - [Debugging the Lexer](#debugging-the-lexer)
  - [Debugging the Parser](#debugging-the-parser)
  - [Debugging the Interpreter](#debugging-the-interpreter-1)
  - [Using Logging](#using-logging)
  - [Advanced Debugging Techniques](#advanced-debugging-techniques)

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

## Debugging the Interpreter

### Debugging the Lexer

When tokens aren't being recognized properly:

1. **Enable lexer debug mode**:
   ```bash
   python main.py --debug-lexer examples/my_program.so
   ```

2. **Check token outputs**:
   ```
   Token(TokenType.DOOR, 'door')
   Token(TokenType.IDENTIFIER, 'x')
   Token(TokenType.EQUAL, '=')
   Token(TokenType.NUMBER, 5.0)
   ```

3. **Identify issues** like:
   - Missing tokens
   - Wrong token types
   - Incorrect token values

### Debugging the Parser

When the syntax tree isn't being constructed correctly:

1. **Enable parser debug mode**:
   ```bash
   python main.py --debug-parser examples/my_program.so
   ```

2. **Inspect the AST output**:
   ```
   ASTNode(PROGRAM, children=[
     ASTNode(VARIABLE_DECLARATION, value='x', children=[
       ASTNode(LITERAL, value=5.0)
     ])
   ])
   ```

3. **Look for issues** like:
   - Missing nodes
   - Incorrect node types
   - Wrong node relationships
   - Wrong values

### Debugging the Interpreter

When runtime behavior is unexpected:

1. **Enable interpreter debug mode**:
   ```bash
   python main.py --debug-interpreter examples/my_program.so
   ```

2. **Follow execution flow**:
   ```
   Executing: VARIABLE_DECLARATION (x)
   Evaluating: LITERAL (5.0)
   Variable x = 5.0
   ```

3. **Watch for issues** like:
   - Incorrect variable values
   - Unexpected control flow
   - Missing or extra executions

### Using Logging

For persistent debugging, use the logging system:

1. **Enable logging**:
   ```bash
   python main.py --log-level=DEBUG examples/my_program.so
   ```

2. **View different log levels**:
   - `ERROR`: Only serious errors
   - `WARNING`: Warnings and errors
   - `INFO`: General information
   - `DEBUG`: Detailed debugging information

3. **Check log file** at `soplang_debug.log`

### Advanced Debugging Techniques

#### 1. Python Debugger (pdb)

Add `import pdb; pdb.set_trace()` at a specific point in the interpreter code to start interactive debugging:

```python
def evaluate(self, node):
    if node.type == NodeType.BINARY_OPERATION and node.value == "+":
        import pdb; pdb.set_trace()  # Start debugging here
        # Rest of the code
```

Then use pdb commands:
- `n` - Next line
- `s` - Step into function
- `c` - Continue execution
- `p variable` - Print variable
- `q` - Quit debugging

#### 2. AST Visualization

To visualize the abstract syntax tree:

```bash
python -m tools.ast_visualizer examples/my_program.so
```

This generates a graphical representation of the AST to help understand program structure.

#### 3. Performance Profiling

To identify performance bottlenecks:

```bash
python -m cProfile -o profile.out main.py examples/my_program.so
python -m pstats profile.out
```

Then in the pstats interface:
```
sort cumtime
stats 10
```

## Conclusion

Effective debugging is essential for both developing the Soplang interpreter and writing Soplang programs. By understanding error messages, using debug output, and leveraging debugging tools, you can quickly identify and fix issues in your code.

Remember that the most powerful debugging technique is often the simplest: adding strategic debug output with `qor()` can help track program state and identify where problems occur.

For more information on Soplang's testing methodology, see [TEST_METHODOLOGY.md](TEST_METHODOLOGY.md), and for details on example programs, see [EXAMPLES.md](EXAMPLES.md).
