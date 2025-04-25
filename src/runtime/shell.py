#!/usr/bin/env python3
"""
Soplang Interactive Shell
A proper REPL (Read-Eval-Print Loop) environment for the Soplang programming language.
"""

import argparse
import atexit
import cmd
import glob
import os
import platform
import re
import sys
import traceback
from pathlib import Path

from colorama import Fore, Style, init

# Import readline conditionally - use it on Unix/Mac, but not on Windows
if platform.system() != "Windows":
    import readline
else:
    # On Windows, try to use prompt_toolkit as an alternative
    try:
        from prompt_toolkit import PromptSession
        from prompt_toolkit.history import FileHistory

        USE_PROMPT_TOOLKIT = True
    except ImportError:
        USE_PROMPT_TOOLKIT = False

from src.core.lexer import Lexer, Token
from src.core.parser import Parser
from src.core.tokens import TokenType
from src.runtime.interpreter import Interpreter
from src.utils.errors import (
    ImportError,
    LexerError,
    ParserError,
    RuntimeError,
    SoplangError,
)


class SoplangShell:
    def __init__(self):
        self.interpreter = Interpreter()
        self.history_file = os.path.expanduser("~/.soplang_history")
        self.setup_history()
        self.multiline_input = []
        self.commands = {
            "help": self.show_help,
            "exit": self.exit_shell,
            "quit": self.exit_shell,
            "clear": self.clear_screen,
            "load": self.load_file,
            "run": self.run_file,
            "examples": self.list_examples,
            "example": self.run_example,
            "reset": self.reset_interpreter,
            "vars": self.show_variables,
            "multiline": self.toggle_multiline,
        }
        self.in_multiline_mode = False
        self.last_examples_list = []

        # Initialize colorama for proper Windows console color support
        init(autoreset=True)

        # Initialize prompt_toolkit if we're using it
        if platform.system() == "Windows" and USE_PROMPT_TOOLKIT:
            self.prompt_session = PromptSession(history=FileHistory(self.history_file))

    def setup_history(self):
        """Set up command history persistence"""
        # Create history file if it doesn't exist
        history_path = Path(self.history_file)
        if not history_path.exists():
            history_path.touch()

        # Only use readline on non-Windows platforms
        if platform.system() != "Windows":
            # Read history file
            try:
                readline.read_history_file(self.history_file)
            except FileNotFoundError:
                pass

            # Set maximum number of items in history
            readline.set_history_length(1000)

            # Save history on exit
            atexit.register(readline.write_history_file, self.history_file)

    def run(self):
        """Start the interactive shell"""
        self.print_welcome()

        while True:
            try:
                # Determine the appropriate prompt text
                if self.in_multiline_mode:
                    if platform.system() == "Windows" and not USE_PROMPT_TOOLKIT:
                        # Simple prompt for Windows without prompt_toolkit
                        prompt_text = "... "
                    else:
                        # Colorized prompt for other platforms or with prompt_toolkit
                        prompt_text = "\033[1;33m... \033[0m"
                else:
                    if platform.system() == "Windows" and not USE_PROMPT_TOOLKIT:
                        # Simple prompt for Windows without prompt_toolkit
                        prompt_text = "\nsoplang> "
                    else:
                        # Colorized prompt for other platforms or with prompt_toolkit
                        prompt_text = "\n\033[1;36msoplang>\033[0m "

                # Get user input with the appropriate input method
                if platform.system() == "Windows" and USE_PROMPT_TOOLKIT:
                    # Use prompt_toolkit on Windows
                    user_input = self.prompt_session.prompt(prompt_text)
                else:
                    # Use standard input on other platforms
                    user_input = input(prompt_text)

                # Skip empty lines
                if not user_input.strip():
                    continue

                if self.in_multiline_mode:
                    # Check for end of multiline input
                    if user_input.strip() == ":end":
                        # Process the entire multiline input
                        full_code = "\n".join(self.multiline_input)
                        self.execute_code(full_code)
                        self.multiline_input = []
                        self.in_multiline_mode = False
                        continue

                    # Add to multiline buffer
                    self.multiline_input.append(user_input)
                    continue

                # Check for multiline mode activation
                if user_input.strip() == ":multiline":
                    self.toggle_multiline("")
                    continue

                # Process commands (starting with colon)
                if user_input.startswith(":"):
                    command = user_input[1:].strip()  # Remove the colon
                    self.process_command(command)
                    continue

                # Process Soplang code
                self.execute_code(user_input)

            except KeyboardInterrupt:
                print("\nUse :exit or :quit to exit the shell, or press Ctrl+D")
                if self.in_multiline_mode:
                    print("Exiting multiline mode")
                    self.multiline_input = []
                    self.in_multiline_mode = False
            except EOFError:
                print("\nExiting Soplang shell...")
                break

    def execute_code(self, code):
        """Execute Soplang code snippet using special shell handling"""
        original_code = code
        code = code.strip()

        # Skip empty lines and comments
        if not code or code.startswith("//"):
            return

        # Format the code first for better handling
        code = self._format_code(code)

        try:
            # DIRECT EXECUTION OF COMMON PATTERNS
            # This bypasses the normal parser for better interactive experience

            # Case 1: Print statements (qor)
            if code.startswith("qor"):
                # Extract the content to print (remove the trailing semicolon if present)
                code_without_semicolon = code[:-1] if code.endswith(";") else code
                match = re.search(
                    r'qor\s*\(\s*["\'](.*?)[\'"]\s*\)', code_without_semicolon
                )
                if match:
                    # Direct execution of print
                    print(match.group(1))
                    return

                # Handle print with variable or expression
                match = re.search(r"qor\s*\(\s*(.*?)\s*\)", code_without_semicolon)
                if match:
                    expr = match.group(1)

                    # Handle simple arithmetic with + operator
                    if "+" in expr and not any(c in expr for c in "\"'"):
                        try:
                            # Check if we're dealing with numeric variables
                            parts = expr.split("+")
                            total = 0
                            for part in parts:
                                part = part.strip()
                                if part in self.interpreter.variables:
                                    # Get the variable value
                                    value = self.interpreter.variables[part]
                                    if isinstance(value, (int, float)):
                                        total += value
                                    else:
                                        # Not numeric, switch to string concatenation
                                        raise ValueError("Non-numeric value")
                                else:
                                    # Try parsing as a number
                                    try:
                                        total += float(part)
                                    except ValueError:
                                        # Not a number, switch to string concatenation
                                        raise ValueError("Not a number")

                            # If we got here, all parts were numeric
                            print(total)
                            return
                        except ValueError:
                            # Fall back to string concatenation
                            pass

                    # Handle string concatenation with + operator
                    if "+" in expr:
                        parts = expr.split("+")
                        result = ""
                        for part in parts:
                            part = part.strip()
                            # Check if it's a variable
                            if part in self.interpreter.variables:
                                value = self.interpreter.variables[part]
                                result += str(value)
                            # Check if it's a string literal
                            elif (part.startswith('"') and part.endswith('"')) or (
                                part.startswith("'") and part.endswith("'")
                            ):
                                result += part[1:-1]
                            # Check if it's a function call like qoraal()
                            elif part.startswith("qoraal(") and part.endswith(")"):
                                var_name = part[len("qoraal("): -1].strip()
                                if var_name in self.interpreter.variables:
                                    value = str(self.interpreter.variables[var_name])
                                    result += value
                            else:
                                # Try as a raw value
                                result += part

                        print(result)
                        return

                    # Handle direct variable access
                    if expr in self.interpreter.variables:
                        print(self.interpreter.variables[expr])
                        return

                    # Try evaluating as a simple numeric expression
                    try:
                        # Replace variable names with their values
                        eval_expr = expr
                        for var_name, var_value in self.interpreter.variables.items():
                            if (
                                isinstance(var_value, (int, float)) and
                                var_name in eval_expr
                            ):
                                eval_expr = eval_expr.replace(var_name, str(var_value))

                        # Evaluate the expression if it looks safe
                        if all(c in "0123456789+-*/() " for c in eval_expr):
                            result = eval(eval_expr)
                            print(result)
                            return
                    except:
                        # If evaluation fails, continue with other methods
                        pass

            # Case 2: Variable declaration with door (dynamic typing)
            if code.startswith("door "):
                # Remove semicolon for regex processing if present
                code_without_semicolon = code[:-1] if code.endswith(";") else code
                # Match pattern: door name = value
                match = re.search(r"door\s+(\w+)\s*=\s*(.+)", code_without_semicolon)
                if match:
                    var_name = match.group(1)
                    var_value = match.group(2).strip()

                    # Handle string values
                    if (var_value.startswith('"') and var_value.endswith('"')) or (
                        var_value.startswith("'") and var_value.endswith("'")
                    ):
                        try:
                            # Extract the actual string value
                            string_value = var_value[1:-1]
                            # Assign it directly to the interpreter's variables
                            self.interpreter.variables[var_name] = string_value
                            print(f'\033[32m=> {var_name} = "{string_value}"\033[0m')
                            return
                        except Exception as e:
                            # Create a proper Soplang error for any issues
                            from src.utils.errors import RuntimeError

                            error = RuntimeError(f"Khalad fulinta: {str(e)}")
                            print(f"\033[31m{error.message}\033[0m")
                            return

                    # Handle numeric values
                    try:
                        if "." in var_value:
                            self.interpreter.variables[var_name] = float(var_value)
                        else:
                            self.interpreter.variables[var_name] = int(var_value)
                        print(
                            f"\033[32m=> {var_name} = {self.interpreter.variables[var_name]}\033[0m"
                        )
                        return
                    except ValueError:
                        # Check if it's a variable reference
                        if var_value in self.interpreter.variables:
                            self.interpreter.variables[var_name] = (
                                self.interpreter.variables[var_value]
                            )
                            print(
                                f"\033[32m=> {var_name} = {self.interpreter.variables[var_name]}\033[0m"
                            )
                            return

                        # Try evaluating as a simple numeric expression
                        try:
                            # Replace variable names with their values
                            eval_expr = var_value
                            for vname, vvalue in self.interpreter.variables.items():
                                if (
                                    isinstance(vvalue, (int, float)) and
                                    vname in eval_expr
                                ):
                                    eval_expr = eval_expr.replace(vname, str(vvalue))

                            # Evaluate the expression if it looks safe
                            if all(c in "0123456789+-*/() " for c in eval_expr):
                                result = eval(eval_expr)
                                self.interpreter.variables[var_name] = result
                                print(f"\033[32m=> {var_name} = {result}\033[0m")
                                return
                        except:
                            # If evaluation fails, continue with other methods
                            pass

            # Case 3: Static typing (tiro, qoraal)
            if code.startswith("tiro ") or code.startswith("qoraal "):
                # Remove semicolon for regex processing if present
                code_without_semicolon = code[:-1] if code.endswith(";") else code
                type_name = code_without_semicolon.split(" ")[0]  # tiro or qoraal
                # Match pattern: type name = value
                match = re.search(
                    rf"{type_name}\s+(\w+)\s*=\s*(.+)", code_without_semicolon
                )
                if match:
                    var_name = match.group(1)
                    var_value = match.group(2).strip()

                    if type_name == "tiro":
                        try:
                            # Try direct number conversion
                            self.interpreter.variables[var_name] = int(var_value)
                            self.interpreter.variable_types[var_name] = "tiro"
                            print(
                                f"\033[32m=> {var_name} = {self.interpreter.variables[var_name]} (tiro)\033[0m"
                            )
                            return
                        except ValueError:
                            # Try numeric expression
                            try:
                                # Replace variable names with their values
                                eval_expr = var_value
                                for vname, vvalue in self.interpreter.variables.items():
                                    if (
                                        isinstance(vvalue, (int, float)) and
                                        vname in eval_expr
                                    ):
                                        eval_expr = eval_expr.replace(
                                            vname, str(vvalue)
                                        )

                                # Evaluate the expression if it looks safe
                                if all(c in "0123456789+-*/() " for c in eval_expr):
                                    result = int(eval(eval_expr))
                                    self.interpreter.variables[var_name] = result
                                    self.interpreter.variable_types[var_name] = "tiro"
                                    print(
                                        f"\033[32m=> {var_name} = {result} (tiro)\033[0m"
                                    )
                                    return
                            except:
                                pass

                            # Use the exact error format from errors.py
                            from src.utils.errors import TypeError

                            error = TypeError(
                                "type_mismatch",
                                var_name=var_name,
                                expected_type="tiro",
                                value=var_value,
                            )
                            print(f"\033[31m{error.message}\033[0m")
                            return

                    elif type_name == "qoraal":
                        if (var_value.startswith('"') and var_value.endswith('"')) or (
                            var_value.startswith("'") and var_value.endswith("'")
                        ):
                            self.interpreter.variables[var_name] = var_value[1:-1]
                            self.interpreter.variable_types[var_name] = "qoraal"
                            print(
                                f'\033[32m=> {var_name} = "{self.interpreter.variables[var_name]}" (qoraal)\033[0m'
                            )
                            return
                        else:
                            # Use the exact error format from errors.py
                            from src.utils.errors import TypeError

                            error = TypeError(
                                "type_mismatch",
                                var_name=var_name,
                                expected_type="qoraal",
                                value=var_value,
                            )
                            print(f"\033[31m{error.message}\033[0m")
                            return

            # Case 4: Direct expression evaluation (calculator mode)
            if all(c in "0123456789+-*/() " for c in code.rstrip(";")):
                # Remove semicolon for eval if present
                code_without_semicolon = code[:-1] if code.endswith(";") else code
                try:
                    result = eval(code_without_semicolon)
                    print(f"\033[32m=> {result}\033[0m")
                    return
                except:
                    pass

            # If we got here, fall back to using the main.py to handle the input
            print("\033[33m// Using main interpreter...\033[0m")

            # Create a temporary file for the code
            import tempfile

            with tempfile.NamedTemporaryFile(suffix=".so", delete=False) as f:
                temp_filename = f.name
                f.write(original_code.encode("utf-8"))

            try:
                # Run the code with main.py
                main_path = os.path.join(
                    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                    "main.py",
                )

                # Check if main.py exists before trying to run it
                if not os.path.exists(main_path):
                    # Use Soplang's ParserError
                    from src.utils.errors import ParserError

                    # Create a proper Soplang parser error
                    error = ParserError(
                        "invalid_syntax", detail="Qoraalka syntax-kiisa waa khalad"
                    )
                    print(f"\033[31m{error.message}\033[0m")
                    return

                # Use subprocess instead of os.system to capture output and errors
                import subprocess

                try:
                    # Use a modified Python PATH so main.py can find its imports
                    env = os.environ.copy()
                    env["PYTHONPATH"] = os.path.dirname(
                        os.path.dirname(os.path.abspath(__file__))
                    )

                    # Run the command and capture output
                    process = subprocess.run(
                        ["python", main_path, temp_filename],
                        env=env,
                        capture_output=True,
                        text=True,
                        check=False,
                    )

                    # Print stdout if available
                    if process.stdout:
                        print(process.stdout)

                    # Handle error gracefully using Soplang's error system
                    if process.returncode != 0:
                        from src.utils.errors import (
                            ParserError,
                            RuntimeError,
                            SoplangError,
                            TypeError,
                        )

                        stderr = process.stderr
                        # Try to extract the actual Soplang error message if it exists
                        if "Khalad" in stderr:
                            # Find the error message which starts with "Khalad"
                            for line in stderr.split("\n"):
                                if line.strip().startswith("Khalad") or "❌" in line:
                                    print(f"\033[31m{line.strip()}\033[0m")
                                    return

                        # If we couldn't find a Soplang error, create an appropriate one using the ErrorMessageManager
                        if "ModuleNotFoundError" in stderr or "ImportError" in stderr:
                            from src.utils.errors import (
                                ErrorMessageManager,
                                ImportError,
                            )

                            error_msg = ErrorMessageManager.get_import_error(
                                "import_error",
                                filename=temp_filename,
                                error="Module not found",
                            )
                            print(f"\033[31m{error_msg}\033[0m")
                        elif "FileNotFoundError" in stderr:
                            from src.utils.errors import (
                                ErrorMessageManager,
                                ImportError,
                            )

                            error_msg = ErrorMessageManager.get_import_error(
                                "file_not_found", module=temp_filename
                            )
                            print(f"\033[31m{error_msg}\033[0m")
                        elif "SyntaxError" in stderr or "unexpected token" in stderr:
                            from src.utils.errors import (
                                ErrorMessageManager,
                                ParserError,
                            )

                            error_msg = ErrorMessageManager.get_parser_error(
                                "invalid_syntax",
                                detail="Khalad syntax ah ama qeexis khalad ah",
                            )
                            print(f"\033[31m{error_msg}\033[0m")
                        elif "TypeError" in stderr:
                            from src.utils.errors import ErrorMessageManager, TypeError

                            error_msg = ErrorMessageManager.get_type_error(
                                "invalid_operand", operator="?", type_name="?"
                            )
                            print(f"\033[31m{error_msg}\033[0m")
                        elif "NameError" in stderr:
                            # Try to extract the variable name from the NameError message
                            from src.utils.errors import (
                                ErrorMessageManager,
                                RuntimeError,
                            )

                            var_match = re.search(
                                r"name '([^']+)' is not defined", stderr
                            )
                            var_name = var_match.group(1) if var_match else "?"
                            error_msg = ErrorMessageManager.get_runtime_error(
                                "undefined_variable", name=var_name
                            )
                            print(f"\033[31m{error_msg}\033[0m")
                        else:
                            from src.utils.errors import (
                                ErrorMessageManager,
                                RuntimeError,
                            )

                            error_msg = ErrorMessageManager.get_runtime_error(
                                "Khalad fulinta: Qalad aan la aqoonsan"
                            )
                            print(f"\033[31m{error_msg}\033[0m")

                except subprocess.SubprocessError:
                    from src.utils.errors import ErrorMessageManager, RuntimeError

                    error_msg = ErrorMessageManager.get_runtime_error(
                        "Khalad fulinta: Ma awoodin inaan koodka fuliyo"
                    )
                    print(f"\033[31m{error_msg}\033[0m")

            finally:
                # Clean up the temporary file
                try:
                    os.unlink(temp_filename)
                except:
                    pass

        except Exception as e:
            # Convert Python exception to Soplang error using ErrorMessageManager
            from src.utils.errors import ErrorMessageManager, RuntimeError, SoplangError

            # If it's already a SoplangError, use its message
            if isinstance(e, SoplangError):
                print(f"\033[31m{e.message}\033[0m")
            else:
                # Try to determine the type of error
                error_msg = str(e)
                if "name" in error_msg and "is not defined" in error_msg:
                    # Extract the variable name from error like "'x' is not defined"
                    match = re.search(r"'([^']+)'", error_msg)
                    var_name = match.group(1) if match else "?"
                    error_msg = ErrorMessageManager.get_runtime_error(
                        "undefined_variable", name=var_name
                    )
                    print(f"\033[31m{error_msg}\033[0m")
                elif "division by zero" in error_msg:
                    error_msg = ErrorMessageManager.get_runtime_error(
                        "division_by_zero"
                    )
                    print(f"\033[31m{error_msg}\033[0m")
                elif "index" in error_msg and "out of range" in error_msg:
                    error_msg = ErrorMessageManager.get_runtime_error(
                        "index_out_of_range", index="?"
                    )
                    print(f"\033[31m{error_msg}\033[0m")
                elif (
                    "local variable" in error_msg and
                    "not associated with a value" in error_msg
                ):
                    # This indicates an issue with module scope/imports
                    error_msg = ErrorMessageManager.get_runtime_error(
                        "Khalad fulinta: Modul qeexis ayaa khaladan"
                    )
                    print(f"\033[31m{error_msg}\033[0m")
                else:
                    # Remove internal implementation details - sanitize the error message
                    # Replace Python-specific parts with Soplang-friendly errors
                    sanitized_error = error_msg

                    # Replace Python module paths
                    sanitized_error = re.sub(
                        r'File ".*?", line \d+, in .*?\n', "", sanitized_error
                    )

                    # Replace Python class names
                    sanitized_error = re.sub(r"<class \'.*?\'>", "", sanitized_error)

                    # Replace memory addresses
                    sanitized_error = re.sub(r"at 0x[0-9a-f]+", "", sanitized_error)

                    # Use a simpler error message if it's excessively technical
                    if len(sanitized_error) > 100 or "Traceback" in sanitized_error:
                        sanitized_error = "Khalad fulinta oo aan la aqoonsan"

                    # Generic runtime error with sanitized message using ErrorMessageManager
                    error_msg = ErrorMessageManager.get_runtime_error(
                        f"Khalad fulinta: {sanitized_error}"
                    )
                    print(f"\033[31m{error_msg}\033[0m")

            # Provide helpful hint for using alternatives
            print(
                "\033[33m// Hint: Try :multiline mode for complex code or :run to run a file\033[0m"
            )

    def _format_code(self, code, silent=True):
        """Format code for better interactive use"""
        code = code.strip()

        # Handle common Soplang syntax issues in interactive mode
        if code.startswith("qor"):
            # Fix qor with missing parentheses
            if "(" not in code and ('"' in code or "'" in code):
                match = re.search(r'qor\s*["\'](.+?)["\']', code)
                if match:
                    string_content = match.group(1)
                    code = f'qor("{string_content}")'

        # Fix variable declarations with bad spacing
        if any(code.startswith(prefix) for prefix in ["door ", "tiro ", "qoraal "]):
            if "=" in code and " = " not in code:
                code = re.sub(r"=", " = ", code, 1)

        # We no longer automatically add semicolons since we properly check for them
        # and show Soplang errors if they're missing

        if not silent:
            print(f"\033[90m// Processing: {code}\033[0m")
        return code

    def process_command(self, command):
        """Process shell commands"""
        cmd_parts = command.split(maxsplit=1)
        cmd = cmd_parts[0].lower() if cmd_parts else ""
        args = cmd_parts[1] if len(cmd_parts) > 1 else ""

        if cmd in self.commands:
            try:
                self.commands[cmd](args)
            except Exception as e:
                print(f"\033[31mError executing command: {str(e)}\033[0m")
                traceback.print_exc()
        else:
            print(
                f"\033[31mUnknown command: {cmd}. Type :help for available commands.\033[0m"
            )

    def show_help(self, args):
        """Show help information"""
        print("\n\033[1mSoplang Interactive Shell Commands:\033[0m")
        print("  :help             - Show this help message")
        print("  :exit, :quit      - Exit the shell")
        print("  :clear            - Clear the screen")
        print("  :load [filename]  - Load and display a Soplang file")
        print("  :run [filename]   - Run a Soplang file with shell interpreter")
        print("  :examples         - List available example programs")
        print("  :example [number] - Run an example by number")
        print("  :reset            - Reset the interpreter (clear all variables)")
        print("  :vars             - Show all defined variables")
        print("  :multiline        - Toggle multiline input mode (end with :end)")
        print("\n\033[1mInteractive Mode:\033[0m")
        print("  - Enter Soplang code directly for immediate execution")
        print("  - Multi-line input is supported with :multiline")
        print("  - Use Up/Down arrows to navigate command history")
        print("  - Simple expressions are automatically evaluated")
        print("\n\033[1mKeyboard Shortcuts:\033[0m")
        print("  - Ctrl+C          - Interrupt current operation")
        print("  - Ctrl+D          - Exit the shell")
        return True

    def exit_shell(self, args):
        """Exit the shell"""
        print("Exiting Soplang shell...")
        sys.exit(0)

    def clear_screen(self, args):
        """Clear the terminal screen"""
        os.system("cls" if os.name == "nt" else "clear")

    def load_file(self, filename):
        """Load and display a Soplang file"""
        if not filename:
            print("\033[31mFilename required. Usage: :load filename\033[0m")
            return

        try:
            with open(filename, "r") as file:
                content = file.read()

            print(f"\n\033[1mFile: {filename}\033[0m")
            print("=" * 40)
            print(content)
            print("=" * 40)
        except FileNotFoundError:
            print(f"\033[31mFile not found: {filename}\033[0m")
        except Exception as e:
            print(f"\033[31mError loading file: {e}\033[0m")

    def run_file(self, filename):
        """Run a Soplang file"""
        if not filename:
            print("\033[31mFilename required. Usage: :run filename\033[0m")
            return

        try:
            # Use the run_soplang_file function from src/runtime/main.py
            from src.runtime.main import run_soplang_file

            print(f"\n\033[1mRunning: {filename}\033[0m")
            print("=" * 40)

            # Call the function that properly tokenizes, parses, and interprets the file
            run_soplang_file(filename)

            print("=" * 40)
        except FileNotFoundError:
            print(f"\033[31mFile not found: {filename}\033[0m")
        except Exception as e:
            print(f"\033[31mError running file: {e}\033[0m")

    def list_examples(self, args):
        """List available example programs"""
        examples_dir = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "examples"
        )

        if not os.path.exists(examples_dir):
            print("\033[31mExamples directory not found\033[0m")
            return

        examples = [f for f in os.listdir(examples_dir) if f.endswith(".so")]

        if not examples:
            print("\033[31mNo example programs found\033[0m")
            return

        print("\n\033[1mAvailable Example Programs:\033[0m")

        # Sort examples and save the list for use with :example command
        sorted_examples = sorted(examples)
        self.last_examples_list = sorted_examples

        for i, example in enumerate(sorted_examples, 1):
            print(f"  {i}. {example}")
        print("\nRun an example with: :example [number] or :run examples/filename.so")

    def run_example(self, args):
        """Run an example by number"""
        if not args:
            print("\033[31mExample number required. Usage: :example [number]\033[0m")
            return

        try:
            example_number = int(args.strip())

            # Make sure we have examples loaded
            if not self.last_examples_list:
                self.list_examples("")

            if not self.last_examples_list:
                return

            if example_number < 1 or example_number > len(self.last_examples_list):
                print(
                    f"\033[31mInvalid example number. Choose between 1 and {len(self.last_examples_list)}\033[0m"
                )
                return

            # Get the example file name
            example_file = self.last_examples_list[example_number - 1]
            example_path = os.path.join(
                os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                "examples",
                example_file,
            )

            # Run the example
            self.run_file(example_path)

        except ValueError:
            print("\033[31mInvalid example number. Usage: :example [number]\033[0m")

    def print_welcome(self):
        """Print welcome message"""
        # Use colorama constants for better Windows compatibility
        print("")
        print(f"{Fore.BLUE}{Style.BRIGHT}" + "=" * 50 + f"{Style.RESET_ALL}")
        print(f"{Fore.BLUE}{Style.BRIGHT}          Soplang - The Somali Programming Language{Style.RESET_ALL}")
        print(f"{Fore.BLUE}{Style.BRIGHT}" + "=" * 50 + f"{Style.RESET_ALL}")
        print(f"Type Soplang code to execute it")
        print(f"Type {Style.BRIGHT}:help{Style.RESET_ALL} for a list of commands")
        print(
            f"Type {Style.BRIGHT}:exit{Style.RESET_ALL} to quit or press {Style.BRIGHT}Ctrl+D{Style.RESET_ALL}")
        print(f"{Fore.BLUE}{Style.BRIGHT}" + "=" * 50 + f"{Style.RESET_ALL}")
        print("")

    def toggle_multiline(self, args):
        """Toggle multiline input mode"""
        self.in_multiline_mode = not self.in_multiline_mode
        self.multiline_input = []

        if self.in_multiline_mode:
            print(
                "\n\033[1mEntering multiline mode.\033[0m Enter code across multiple lines."
            )
            print("Type \033[1m:end\033[0m on a line by itself to execute.")
        else:
            print("\n\033[1mExiting multiline mode.\033[0m")

    def reset_interpreter(self, args):
        """Reset the interpreter to clear all variables and state"""
        self.interpreter = Interpreter()
        print("\n\033[1mInterpreter reset.\033[0m All variables and state cleared.")

    def show_variables(self, args):
        """Show all defined variables in the current interpreter"""
        print("\n\033[1mDefined Variables:\033[0m")
        if not hasattr(self.interpreter, "variables") or not self.interpreter.variables:
            print("  No variables defined.")
            return

        for name, value in self.interpreter.variables.items():
            var_type = self.interpreter.variable_types.get(name, "dynamic")
            print(f"  {name} = {value} ({var_type})")


def main():
    """Main entry point for the Soplang shell"""
    # Setup command line argument parser
    parser = argparse.ArgumentParser(description="Soplang Interactive Shell")
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
        print("\n=== Soplang ===")
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
