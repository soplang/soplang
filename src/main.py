from lexer import Lexer
from parser import Parser
from interpreter import Interpreter
import os
import sys


def run_soplang_file(filename):
    """Run a Soplang file through the lexer, parser, and interpreter"""

    print(f"\n🔹 Running Soplang file: {filename}")

    try:
        with open(filename, 'r') as file:
            code = file.read()

        # 1) Tokenize
        lexer = Lexer(code)
        tokens = lexer.tokenize()

        # 2) Parse
        parser = Parser(tokens)
        ast = parser.parse()

        # 3) Interpret
        inter = Interpreter()
        print("\n🔹 Program Output:")
        inter.interpret(ast)
        print("\n✅ Program finished successfully.")

    except FileNotFoundError:
        print(f"❌ Error: File '{filename}' not found.")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        print("Program execution failed.")


def print_usage():
    """Print usage information"""
    print("Usage: python src/main.py [example_file.so]")
    print("\nAvailable examples:")
    for file in os.listdir("examples"):
        if file.endswith(".so"):
            print(f"  - examples/{file}")


def main():
    """Main entry point"""
    print("\n--- Soplang Programming Language ---")

    if len(sys.argv) > 1:
        # Run the specified file
        filename = sys.argv[1]
        run_soplang_file(filename)
    else:
        # No file specified, show available examples
        print("\nWelcome to Soplang!")
        print("Please specify a Soplang file (.so) to run.")
        print_usage()


if __name__ == "__main__":
    main()
