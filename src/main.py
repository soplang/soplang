"""
Soplang Core Runtime Module
===========================

This module contains the core functions for running Soplang code:
- Parsing and executing Soplang files
- Displaying usage information
- Main entry point for direct file execution
"""

from src.lexer import Lexer
from src.parser import Parser
from src.interpreter import Interpreter
import os
import sys


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
        with open(filename, 'r') as file:
            code = file.read()

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
        print(f"❌ Error: File '{filename}' not found.")
        return 1  # Error
    except Exception as e:
        print(f"\n❌ Error: {e}")
        print("Program execution failed.")
        return 1  # Error


def print_usage():
    """
    Display usage information and available example files

    Shows how to run Soplang files and lists all example files in the examples directory
    """
    print("\n📚 Usage Guide:")
    print("  python main.py [filename.so]     - Run a Soplang file")
    print("  python main.py                   - Start interactive shell")

    print("\n📂 Available examples:")
    examples_dir = "examples"
    try:
        examples = [f for f in os.listdir(examples_dir) if f.endswith('.so')]
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
        print("Please specify a Soplang file (.so) to run, or run without arguments for the interactive shell.")
        print_usage()
        return 0  # Success


if __name__ == "__main__":
    sys.exit(main())
