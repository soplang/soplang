#!/usr/bin/env python3
# ======================================================
# Soplang - The Somali Programming Language
# Main entry point for both shell and file execution
# ======================================================

import argparse
import os
import sys

from src.core.lexer import Lexer
from src.core.parser import Parser
from src.runtime.interpreter import Interpreter
from src.runtime.shell import SoplangShell


def main():
    """
    Main entry point for Soplang.

    This function handles starting the Soplang environment:
    - If run with a filename argument, it executes that file
    - If run with flags, it processes the appropriate action
    - If run without arguments, it launches the interactive shell

    Usage:
        python main.py                   # Start interactive shell
        python main.py filename.sop      # Execute a Soplang file
        python main.py -e 1              # Run example number 1
        python main.py -c 'qor("Hello")' # Execute code snippet
        python main.py -v                # Display version information
    """
    # Setup command line argument parser
    parser = argparse.ArgumentParser(description="Soplang Programming Language")
    parser.add_argument(
        "-v",
        "--version",
        action="store_true",
        help="Display Soplang version information",
    )
    parser.add_argument("-f", "--file", metavar="FILE", help="Execute a Soplang file")
    parser.add_argument(
        "-e", "--example", metavar="N", type=int, help="Run example program number N"
    )
    parser.add_argument(
        "-i",
        "--interactive",
        action="store_true",
        help="Start interactive shell after executing file",
    )
    parser.add_argument(
        "-c", "--command", metavar="CODE", help="Execute Soplang code snippet"
    )
    parser.add_argument("filename", nargs="?", help="Soplang file to execute")

    # Parse arguments
    args = parser.parse_args()

    # Create shell instance
    shell = SoplangShell()

    # Display version information if requested
    if args.version:
        print("\n=== Soplang: The Somali Programming Language ===")
        print("Version: 0.1.0")
        print("Website: https://www.soplang.org/")
        print("License: MIT")
        return 0

    # Execute code snippet if provided
    if args.command:
        print("\n=== Executing Soplang code snippet ===")
        shell.execute_code(args.command)
        return 0

    # Run example if requested
    if args.example is not None:
        # Load example list
        shell.list_examples("")
        if not shell.last_examples_list:
            return 1

        if args.example < 1 or args.example > len(shell.last_examples_list):
            print(
                f"\033[31mInvalid example number. Choose between 1 and {len(shell.last_examples_list)}\033[0m"
            )
            return 1

        # Run the example
        example_file = shell.last_examples_list[args.example - 1]
        example_path = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "examples",
            example_file,
        )
        shell.run_file(example_path)

        # Start interactive shell afterward if requested
        if args.interactive:
            shell.run()

        return 0

    # Handle file if provided (either through --file or positional argument)
    filename = args.file or args.filename
    if filename:
        print(f"\n\033[1mRunning file: {filename}\033[0m")
        shell.run_file(filename)

        # Start interactive shell afterward if requested
        if args.interactive:
            shell.run()

        return 0

    # No specific command given, start interactive shell
    shell.run()
    return 0


if __name__ == "__main__":
    sys.exit(main())
