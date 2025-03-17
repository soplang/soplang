#!/usr/bin/env python3
# ======================================================
# Soplang - The Somali Programming Language
# Main entry point for both shell and file execution
# ======================================================

from src.shell import SoplangShell
import sys


def main():
    """
    Main entry point for Soplang.

    This function handles starting the Soplang environment:
    - If run with a filename argument, it executes that file
    - If run without arguments, it launches the interactive shell

    Usage:
        python main.py                   # Start interactive shell
        python main.py filename.so       # Execute a Soplang file
    """
    shell = SoplangShell()

    # Check if a filename was passed as an argument
    if len(sys.argv) > 1:
        filename = sys.argv[1]
        shell.run_file(filename)
    else:
        # No filename provided, start interactive shell
        shell.run()


if __name__ == "__main__":
    main()
