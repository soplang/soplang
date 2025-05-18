import os
import subprocess
import sys

# Files that are expected to have errors as part of their test
EXPECTED_ERROR_FILES = [
    "22_constant_reassignment_test.sop",
    "24_constant_type_error_test.sop",
    "25_constant_type_error.sop",
]


def run_example(filepath, filename):
    """Run a Soplang example file and return success/failure with error message"""
    try:
        result = subprocess.run(
            ["python", "main.py", filepath],
            capture_output=True,
            text=True,
            timeout=5,  # 5 second timeout
        )

        # For files that are supposed to demonstrate errors, a returncode of 0
        # and the presence of specific error messages is a success
        if filename in EXPECTED_ERROR_FILES:
            # These files are expected to fail with specific errors
            if result.returncode != 0 or "Khalad" in result.stdout:
                return True, None

        # For normal files
        if result.returncode == 0 and "Khalad" not in result.stdout:
            return True, None
        else:
            return False, result.stdout or result.stderr
    except Exception as e:
        return False, str(e)


def main():
    """Check all example files in the examples directory"""
    examples_dir = "examples"
    files = [f for f in os.listdir(examples_dir) if f.endswith(".sop")]
    files.sort()  # Sort files for consistent output

    print(f"Testing {len(files)} example files...\n")

    successful = []
    failed = []

    for filename in files:
        filepath = os.path.join(examples_dir, filename)
        print(f"Testing {filename}...", end=" ")
        success, error = run_example(filepath, filename)

        if success:
            print("✅ Success")
            successful.append(filename)
        else:
            print("❌ Failed")
            failed.append((filename, error))

    print(f"\nSummary: {len(successful)} successful, {len(failed)} failed")

    if failed:
        print("\nFailed files:")
        for filename, error in failed:
            print(f"\n{filename}:")
            error_lines = error.split("\n")[:5]  # Show first 5 lines of error
            for line in error_lines:
                print(f"  {line}")
            if len(error.split("\n")) > 5:
                print("  ...")

    return len(failed)


if __name__ == "__main__":
    sys.exit(main())
