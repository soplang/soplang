# Soplang Error Examples

This directory contains example files that demonstrate various types of errors in Soplang and how to handle them. These examples are designed to help you understand how Soplang reports errors and how to debug common issues.

## Files Overview

1. **01_lexer_errors.sop**: Demonstrates errors that occur during the lexical analysis phase (tokenization).
2. **02_parser_errors.sop**: Demonstrates errors that occur during the parsing phase (syntax analysis).
3. **03_type_errors.sop**: Demonstrates errors related to Soplang's type system.
4. **04_runtime_errors.sop**: Demonstrates errors that occur during program execution.
5. **05_control_flow_errors.sop**: Demonstrates errors related to control flow statements.
6. **06_error_handling.sop**: Demonstrates proper error handling techniques in Soplang.

## How to Use These Examples

Each file contains commented-out examples of different error types. To see a specific error:

1. Uncomment **only one** error example at a time
2. Run the file using the Soplang interpreter
3. Observe the error message
4. Re-comment the example before trying another one

```bash
# Example: Running a lexer error example
python main.py examples/errors/01_lexer_errors.sop
```

## Error Message Format

Soplang error messages follow this format:

```
Khalad [type]: [message] at line [line], position [position]
```

Where:
- `[type]` can be: `lexer`, `parser`, `runtime`, or `type`
- `[message]` is a description of the error in Somali
- `[line]` and `[position]` indicate where the error occurred

## Error Handling

The `06_error_handling.sop` file demonstrates how to use the `isku_day`/`qabo` (try/catch) mechanism to handle errors gracefully in your Soplang programs.

## Common Error Types

1. **Lexer Errors**: Problems with invalid characters or unterminated strings/comments
2. **Parser Errors**: Problems with syntax, like missing brackets or incomplete expressions
3. **Type Errors**: Problems with type mismatches or invalid operations for a type
4. **Runtime Errors**: Problems that occur during execution like division by zero
5. **Control Flow Errors**: Problems with the use of control flow statements outside their allowed context

## Additional Resources

For more information on debugging in Soplang, see the [Debugging Guide](../../docs/testing/DEBUGGING.md).
