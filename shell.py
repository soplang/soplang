#!/usr/bin/env python3
import soplang
import os
import sys
import readline  # For command history

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


def print_welcome():
    """Print a welcome message with language information."""
    print(
        f"{COLORS['BOLD']}{COLORS['CYAN']}Soplang Interactive Shell{COLORS['RESET']}")
    print(
        f"{COLORS['YELLOW']}Version 1.0 - A Somali-inspired programming language{COLORS['RESET']}")
    print("Type 'exit()' or 'quit()' to exit, 'help()' for help.")
    print("=" * 60)


def print_help():
    """Print help information about Soplang."""
    print(f"\n{COLORS['BOLD']}Soplang Help{COLORS['RESET']}")
    print("=" * 60)
    print("Soplang is a programming language with Somali-based keywords.")
    print("\nBasic syntax:")
    print(
        f"  {COLORS['YELLOW']}keyd{COLORS['RESET']} x = 10                 # Variable declaration")
    print(f"  {COLORS['YELLOW']}haddii{COLORS['RESET']} x > 5 {COLORS['YELLOW']}markaas{COLORS['RESET']}      # If statement")
    print(f"  {COLORS['YELLOW']}ku_celi{COLORS['RESET']} i = 0 {COLORS['YELLOW']}ilaa{COLORS['RESET']} 10 {COLORS['YELLOW']}markaas{COLORS['RESET']}  # For loop")
    print(f"  {COLORS['YELLOW']}inta_ay{COLORS['RESET']} x > 0 {COLORS['YELLOW']}markaas{COLORS['RESET']}     # While loop")
    print(
        f"  {COLORS['YELLOW']}shaqo{COLORS['RESET']} add(a, b)             # Function definition")
    print(
        f"  {COLORS['YELLOW']}soo_celi{COLORS['RESET']} x                  # Return statement")
    print("\nShell commands:")
    print("  help()    - Display this help message")
    print("  clear()   - Clear the screen")
    print("  exit()    - Exit the shell")
    print("  run(file) - Run a Soplang file")
    print("=" * 60)


def clear_screen():
    """Clear the terminal screen."""
    os.system('cls' if os.name == 'nt' else 'clear')


def run_file(filename):
    """Run a Soplang file."""
    try:
        with open(filename, 'r') as f:
            text = f.read()
        result, error = soplang.run(filename, text)
        if error:
            print(f"{COLORS['RED']}{error.as_string()}{COLORS['RESET']}")
        elif result:
            if hasattr(result, 'elements') and len(result.elements) > 0:
                if len(result.elements) == 1:
                    print(repr(result.elements[0]))
                else:
                    print(repr(result))
            else:
                print(repr(result))
    except FileNotFoundError:
        print(
            f"{COLORS['RED']}Error: File '{filename}' not found.{COLORS['RESET']}")


def main():
    """Main function to run the Soplang shell."""
    print_welcome()

    # Enable command history
    histfile = os.path.join(os.path.expanduser("~"), ".soplang_history")
    try:
        readline.read_history_file(histfile)
        readline.set_history_length(1000)
    except FileNotFoundError:
        pass

    while True:
        try:
            # Handle multiline input
            text = ""
            line = input(f"{COLORS['GREEN']}soplang > {COLORS['RESET']}")

            # Check for shell commands
            if line.strip() == "help()":
                print_help()
                continue
            elif line.strip() == "clear()":
                clear_screen()
                continue
            elif line.strip() in ["exit()", "quit()"]:
                print("Goodbye!")
                break
            elif line.strip().startswith("run(") and line.strip().endswith(")"):
                filename = line.strip()[4:-1].strip("'\"")
                run_file(filename)
                continue

            # Handle multiline code
            text += line
            while line.strip().endswith(":") or (text.count("markaas") > text.count("dhamee")):
                try:
                    line = input(
                        f"{COLORS['YELLOW']}......... {COLORS['RESET']}")
                    text += "\n" + line
                except KeyboardInterrupt:
                    print("\nMultiline input cancelled.")
                    text = ""
                    break

            if text.strip() == "":
                continue

            # Save to history
            readline.add_history(text)

            # Run the code
            result, error = soplang.run('<stdin>', text)

            if error:
                print(f"{COLORS['RED']}{error.as_string()}{COLORS['RESET']}")
            elif result:
                if hasattr(result, 'elements') and len(result.elements) > 0:
                    if len(result.elements) == 1:
                        print(
                            f"{COLORS['CYAN']}{repr(result.elements[0])}{COLORS['RESET']}")
                    else:
                        print(
                            f"{COLORS['CYAN']}{repr(result)}{COLORS['RESET']}")
                else:
                    print(f"{COLORS['CYAN']}{repr(result)}{COLORS['RESET']}")

        except KeyboardInterrupt:
            print("\nUse exit() or quit() to exit")
        except EOFError:
            print("\nGoodbye!")
            break
        except Exception as e:
            print(f"{COLORS['RED']}Shell Error: {str(e)}{COLORS['RESET']}")

    # Save history
    try:
        readline.write_history_file(histfile)
    except:
        pass


if __name__ == "__main__":
    main()
