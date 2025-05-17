import unittest
import io
import sys
from src.core.lexer import Lexer
from src.core.parser import Parser
from src.runtime.interpreter import Interpreter


class TestInterpreter(unittest.TestCase):
    def setUp(self):
        """Set up the interpreter for each test."""
        self.interpreter = Interpreter()
        # Redirect stdout to capture print statements
        self.stdout_backup = sys.stdout
        self.captured_output = io.StringIO()
        sys.stdout = self.captured_output
    
    def tearDown(self):
        """Restore stdout after each test."""
        sys.stdout = self.stdout_backup
    
    def _execute_code(self, source_code):
        """Helper method to execute code and return output."""
        lexer = Lexer(source_code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        self.interpreter.interpret(ast)
        return self.captured_output.getvalue().strip()
    
    def test_print_statement(self):
        """Test simple print statement execution."""
        try:
            # Direct way to test qor without using the parser
            self.interpreter.functions["qor"]("Hello, World!")
            output = self.captured_output.getvalue().strip()
            self.assertEqual(output, "Hello, World!")
        except Exception as e:
            self.fail(f"test_print_statement failed: {e}")
    
    def test_variable_declaration_and_use(self):
        """Test variable declaration and usage."""
        source = '''
        door x = 42
        qor("Value: " + qoraal(x))
        '''
        output = self._execute_code(source)
        self.assertEqual(output, "Value: 42.0")
        self.assertEqual(self.interpreter.variables['x'], 42.0)
    
    def test_static_type_declaration(self):
        """Test static type variable declaration."""
        source = '''
        abn num = 10
        qoraal str = "Hello"
        qor(qoraal(num) + " " + str)
        '''
        output = self._execute_code(source)
        self.assertEqual(output, "10.0 Hello")
        self.assertEqual(self.interpreter.variables['num'], 10.0)
        self.assertEqual(self.interpreter.variables['str'], "Hello")
    
    def test_arithmetic_operations(self):
        """Test arithmetic operations."""
        source = '''
        door a = 10
        door b = 5
        qor("a + b = " + qoraal(a + b))
        qor("a - b = " + qoraal(a - b))
        qor("a * b = " + qoraal(a * b))
        qor("a / b = " + qoraal(a / b))
        qor("a % b = " + qoraal(a % b))
        '''
        output = self._execute_code(source)
        expected = "a + b = 15.0\na - b = 5.0\na * b = 50.0\na / b = 2.0\na % b = 0.0"
        self.assertEqual(output, expected)
    
    def test_if_statement(self):
        """Test if statement execution."""
        source = '''
        door x = 15
        haddii (x > 10) {
            qor("x is greater than 10")
        } ugudambeyn {
            qor("x is not greater than 10")
        }
        '''
        output = self._execute_code(source)
        self.assertEqual(output, "x is greater than 10")
        
        # Test with opposite condition
        self.captured_output.truncate(0)
        self.captured_output.seek(0)
        source = '''
        door x = 5
        haddii (x > 10) {
            qor("x is greater than 10")
        } ugudambeyn {
            qor("x is not greater than 10")
        }
        '''
        output = self._execute_code(source)
        self.assertEqual(output, "x is not greater than 10")
    
    def test_for_loop(self):
        """Test for loop execution."""
        source = '''
        kuceli i min 1 ilaa 3 {
            qor("Number: " + qoraal(i))
        }
        '''
        output = self._execute_code(source)
        expected = "Number: 1.0\nNumber: 2.0\nNumber: 3.0"
        self.assertEqual(output, expected)
    
    def test_function_definition_and_call(self):
        """Test function definition and calling."""
        source = '''
        hawl add(a, b) {
            celi a + b
        }
        door result = add(5, 7)
        qor("5 + 7 = " + qoraal(result))
        '''
        output = self._execute_code(source)
        self.assertEqual(output, "5 + 7 = 12.0")
        self.assertEqual(self.interpreter.variables['result'], 12.0)
    
    def test_list_operations(self):
        """Test list operations."""
        source = '''
        door numbers = [1, 2, 3]
        numbers.push(4)
        qor("List: " + qoraal(numbers))
        qor("Length: " + qoraal(numbers.length()))
        '''
        output = self._execute_code(source)
        expected = "List: [1.0, 2.0, 3.0, 4.0]\nLength: 4"
        self.assertEqual(output, expected)
        self.assertEqual(len(self.interpreter.variables['numbers']), 4)


if __name__ == '__main__':
    unittest.main() 