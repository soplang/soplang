"""
This file demonstrates how the error handling system in Soplang should be fixed
to display proper Somali error messages with line and position information.
"""

# The bug in the current system:
# 1. Lexer is creating errors with a string error message instead of using predefined error codes
# 2. This causes double prefixes "Khalad markii loo qaybinayay: Khalad markii loo qaybinayay:"
# 3. Line and position information isn't being properly passed or displayed

# Example of how it's currently implemented in lexer.py:
# raise LexerError(f"Unexpected character: {self.current_char}", self.position)
# Which means the error message gets wrapped in an additional "Khalad markii loo qaybinayay: "

# Fixed solution:
# 1. Ensure errors are raised with proper error codes matching ErrorMessageManager.LEXER_ERRORS
# 2. Ensure the line number is tracked and passed to errors
# 3. Ensure error messages are properly displayed to the user

# Example of fixed lexer.py error handling:
"""
def next_token(self):
    # ... (current code) ...

    # Current implementation (problematic):
    raise LexerError(f"Unexpected character: {self.current_char}", self.position)

    # Fixed implementation:
    raise LexerError("unexpected_char",
                    position=self.position,
                    line=self.line,  # Need to track line number in lexer
                    char=self.current_char)  # Pass the character as a parameter
"""

# Example of fixed LexerError implementation in errors.py:
"""
class LexerError(SoplangError):
    def __init__(self, error_code, position=None, line=None, **kwargs):
        kwargs.update({"position": position, "line": line})

        # Directly use error_code without additional wrapper text:
        self.message = ErrorMessageManager.get_lexer_error(error_code, **kwargs)

        # Only prepend the error type once:
        self.full_message = f"Khalad lexer: {self.message}"
        super().__init__(self.full_message)
"""

# Example of how errors should be displayed in runtime/main.py:
"""
def run_soplang_file(filename):
    # ... (current code) ...

    try:
        # ... (code execution) ...
    except SoplangError as e:
        # Get the message directly from the error object,
        # which should already include all necessary information
        print(f"\n❌ {e}")
        print("Program execution failed.")
        return 1  # Error
"""

# Summary of changes needed:
# 1. Update the lexer to track line numbers (not just position)
# 2. Ensure all error-raising sites use error codes instead of string messages
# 3. Fix the error classes to format messages correctly without double prefixes
# 4. Update the error display code to show the properly formatted messages

# Expected output for the "@" character error:
# ❌ Khalad lexer: Xaraf aan la filayn: @ ee sadar 12, goobta 18
# Program execution failed.
