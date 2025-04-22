#!/usr/bin/env python3
import sys
import os
import unittest

# Add the current directory to the Python path
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

# Import test modules
from tests.test_lexer import TestLexer
from tests.test_parser import TestParser
from tests.test_interpreter import TestInterpreter

if __name__ == "__main__":
    # Create test suite with all tests
    test_suite = unittest.TestSuite()
    
    # Add all test cases
    test_suite.addTest(unittest.makeSuite(TestLexer))
    test_suite.addTest(unittest.makeSuite(TestParser))
    test_suite.addTest(unittest.makeSuite(TestInterpreter))
    
    # Run tests with verbose output
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(test_suite)
    
    # Exit with appropriate status code
    sys.exit(not result.wasSuccessful()) 