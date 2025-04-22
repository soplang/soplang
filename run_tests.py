#!/usr/bin/env python3
import os
import sys
import subprocess

# Get the directory of this file
script_dir = os.path.dirname(os.path.abspath(__file__))

# Path to the actual test runner
test_runner = os.path.join(script_dir, "tests", "runners", "run_all_tests.py")

if __name__ == "__main__":
    # Forward all arguments to the actual test runner
    result = subprocess.run([test_runner] + sys.argv[1:])
    sys.exit(result.returncode) 