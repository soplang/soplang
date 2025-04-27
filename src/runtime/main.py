"""
Soplang Core Runtime Module
===========================

This module contains the core functions for running Soplang code:
- Parsing and executing Soplang files
- Displaying usage information
- Main entry point for direct file execution
"""

import os
import sys

from src.core.lexer import Lexer
from src.core.parser import Parser
from src.runtime.interpreter import Interpreter
from src.utils.errors import SoplangError


def run_soplang_file(filename):
    """
    Run a Soplang file through the lexer, parser, and interpreter

    This function handles the complete execution pipeline:
    1. Read the source file
    2. Tokenize the source code
    3. Parse tokens into an abstract syntax tree
    4. Interpret and execute the program

    Args:
        filename (str): Path to the Soplang file to execute

    Returns:
        int: Exit code (0 for success, 1 for error)
    """
    print(f"\n🔹 Running Soplang file: {filename}")

    try:
        with open(filename, "r") as file:
            code = file.read()

        # Ensure code ends with a newline to avoid parsing issues
        if not code.endswith("\n"):
            code += "\n"

        # 1) Tokenize the source code
        lexer = Lexer(code)
        tokens = lexer.tokenize()

        # 2) Parse tokens into an AST
        parser = Parser(tokens)
        ast = parser.parse()

        # 3) Interpret and execute the AST
        inter = Interpreter()
        print("\n🔹 Program Output:")
        inter.interpret(ast)
        print("\n✅ Program finished successfully.")
        return 0  # Success

    except FileNotFoundError:
        # File not found error in Somali
        print(f"❌ Khalad: Faylka '{filename}' ma helin.")
        return 1  # Error
    except SoplangError as e:
        # Display error message - already formatted in Somali
        print(f"\n❌ {e}")
        return 1  # Error
    except Exception as e:
        # Convert Python exceptions to Somali error messages
        from src.utils.errors import RuntimeError

        # Format different types of Python errors as Somali errors
        if "missing 1 required positional argument" in str(e):
            # Function missing argument
            func_name = str(e).split(".")[0]
            error = RuntimeError(
                "missing_argument", func_name=func_name, expected="1", provided="0"
            )
        elif "division by zero" in str(e):
            # Division by zero
            error = RuntimeError("division_by_zero")
        elif "list index out of range" in str(e):
            # List index out of range
            error = RuntimeError("index_out_of_range", index="?")
        else:
            # Generic error
            error = RuntimeError(f"Khalad: {str(e)}")

        print(f"\n❌ {error}")
        return 1  # Error


def print_usage():
    """
    Display usage information and available example files

    Shows how to run Soplang files and lists all example files in the examples directory
    """
    print("\n📚 Usage Guide:")
    print("  python main.py [filename.sop]     - Run a Soplang file")
    print("  python main.py                   - Start interactive shell")

    print("\n📂 Available examples:")
    examples_dir = "examples"
    try:
        examples = [
            f
            for f in os.listdir(examples_dir)
            if f.endswith(".sop") or f.endswith(".so")
        ]
        if examples:
            for example in sorted(examples):
                print(f"  • {examples_dir}/{example}")
        else:
            print("  • No example files found")
    except FileNotFoundError:
        print("  • Examples directory not found")


def main():
    """
    Main entry point for running Soplang files directly

    Handles command-line arguments and launches the appropriate mode:
    - With a filename argument: Execute that file
    - Without arguments: Show help information

    Returns:
        int: Exit code (0 for success, 1 for error)
    """
    print("\n=== Soplang: The Somali Programming Language ===")

    if len(sys.argv) > 1:
        # Run the specified file
        filename = sys.argv[1]
        return run_soplang_file(filename)
    else:
        # No file specified, show welcome and usage information
        print("\n👋 Welcome to Soplang!")
        print(
            "Please specify a Soplang file (.sop) to run, or run without arguments for the interactive shell."
        )
        print_usage()
        return 0  # Success


if __name__ == "__main__":
    sys.exit(main())
