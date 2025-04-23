#!/usr/bin/env python3
import sys
import os
import unittest

# Add the project root directory to the Python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))

# Import test modules
from tests.test_lexer import TestLexer
from tests.test_parser import TestParser
from tests.test_interpreter import TestInterpreter

if __name__ == "__main__":
    # Create test loader
    loader = unittest.TestLoader()
    
    # Create test suite with all test classes
    test_suite = unittest.TestSuite()
    
    # Add all test cases using the loader
    test_suite.addTests(loader.loadTestsFromTestCase(TestLexer))
    test_suite.addTests(loader.loadTestsFromTestCase(TestParser))
    test_suite.addTests(loader.loadTestsFromTestCase(TestInterpreter))
    
    # Run tests with verbose output
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    # Exit with appropriate status code
    sys.exit(not result.wasSuccessful()) 