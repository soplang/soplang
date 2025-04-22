#!/usr/bin/env python3
import sys
import os

# Add the current directory to the Python path
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

# Import the test module
from tests.test_interpreter import TestInterpreter
import unittest

if __name__ == "__main__":
    unittest.main() 