#!/usr/bin/env python3
"""
Soplang File Runner
------------------
This script runs Soplang (.spl) files directly.
Usage: python3 soplang_run.py <filename.spl>
"""

import soplang
import sys
import os

# ANSI color codes for terminal output
COLORS = {
    'RESET': '\033[0m',
    'BOLD': '\033[1m',
    'RED': '\033[91m',
    'GREEN': '\033[92m',
    'YELLOW': '\033[93m',
    'BLUE': '\033[94m',
    'MAGENTA': '\033[95m',
    'CYAN': '\033[96m',
}


def print_usage():
    """Print usage information."""
    print(f"{COLORS['BOLD']}Soplang File Runner{COLORS['RESET']}")
    print("Usage: python3 soplang_run.py <filename.spl>")
    print("Options:")
    print("  -h, --help    Show this help message")
    print("  -v, --verbose Show detailed output")


def run_file(filename, verbose=False):
    """Run a Soplang file with error handling."""
    if verbose:
        print(f"{COLORS['YELLOW']}Running file: {filename}{COLORS['RESET']}")

    try:
        with open(filename, 'r') as f:
            text = f.read()

        if verbose:
            print(
                f"{COLORS['YELLOW']}File loaded, executing...{COLORS['RESET']}")

        result, error = soplang.run(filename, text)

        if error:
            print(f"{COLORS['RED']}Error:{COLORS['RESET']}")
            print(f"{COLORS['RED']}{error.as_string()}{COLORS['RESET']}")
            return False
        elif result:
            if verbose:
                print(
                    f"{COLORS['GREEN']}Execution completed successfully.{COLORS['RESET']}")

            # Check if it's a return value or a list of results
            if hasattr(result, 'elements') and len(result.elements) > 0:
                if len(result.elements) == 1:
                    print(
                        f"{COLORS['CYAN']}{repr(result.elements[0])}{COLORS['RESET']}")
                else:
                    print(f"{COLORS['CYAN']}{repr(result)}{COLORS['RESET']}")
            else:
                print(f"{COLORS['CYAN']}{repr(result)}{COLORS['RESET']}")
            return True
        else:
            if verbose:
                print(
                    f"{COLORS['GREEN']}Execution completed with no result.{COLORS['RESET']}")
            return True

    except FileNotFoundError:
        print(
            f"{COLORS['RED']}Error: File '{filename}' not found.{COLORS['RESET']}")
        return False
    except Exception as e:
        print(f"{COLORS['RED']}Error: {str(e)}{COLORS['RESET']}")
        return False


def main():
    """Main function to parse arguments and run files."""
    if len(sys.argv) < 2 or sys.argv[1] in ['-h', '--help']:
        print_usage()
        return

    verbose = False
    if '-v' in sys.argv or '--verbose' in sys.argv:
        verbose = True
        # Remove verbose flags from arguments
        sys.argv = [arg for arg in sys.argv if arg not in ['-v', '--verbose']]

    filename = sys.argv[1]

    # Add .spl extension if not provided
    if not filename.endswith('.spl'):
        filename += '.spl'

    success = run_file(filename, verbose)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
