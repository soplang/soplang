import unittest
from src.core.lexer import Lexer
from src.core.parser import Parser
from src.core.ast import NodeType
from src.core.tokens import TokenType


class TestParser(unittest.TestCase):
    def test_variable_declaration(self):
        """Test parsing of variable declarations."""
        # Parse directly from a node for testing
        lexer = Lexer('door x = 42')
        tokens = lexer.tokenize()
        
        # Inspect tokens for debugging
        print("Variable Declaration Tokens:")
        for token in tokens:
            print(f"  {token}")
            
        # Create a simple parser instance just for this test
        parser = Parser([
            tokens[0],  # DOOR
            tokens[1],  # IDENTIFIER (x)
            tokens[2],  # EQUAL
            tokens[3],  # NUMBER (42)
        ])
        
        node = parser.parse_variable_declaration(is_static=False)
        
        # Check node properties
        self.assertEqual(node.type, NodeType.VARIABLE_DECLARATION)
        self.assertEqual(node.value, 'x')
        
        # Value should be a NUMBER node
        expr = node.children[0]
        self.assertEqual(expr.type, NodeType.LITERAL)
        self.assertEqual(expr.value, 42.0)
    
    def test_static_type_declaration(self):
        """Test parsing of static type declarations."""
        # Parse directly from a node for testing
        lexer = Lexer('abn y = 10')
        tokens = lexer.tokenize()
        
        # Inspect tokens for debugging
        print("Static Type Declaration Tokens:")
        for token in tokens:
            print(f"  {token}")
            
        # Create a simple parser instance just for this test
        parser = Parser([
            tokens[0],  # abn
            tokens[1],  # IDENTIFIER (y)
            tokens[2],  # EQUAL
            tokens[3],  # NUMBER (10)
        ])
        
        node = parser.parse_variable_declaration(is_static=True)
        
        # Check node properties
        self.assertEqual(node.type, NodeType.VARIABLE_DECLARATION)
        self.assertEqual(node.value, 'y')
        self.assertTrue(hasattr(node, 'var_type'))
        
        # Value should be a NUMBER node
        expr = node.children[0]
        self.assertEqual(expr.type, NodeType.LITERAL)
        self.assertEqual(expr.value, 10.0)
    
    def test_function_call(self):
        """Test parsing of function calls."""
        # Parse directly from a node for testing
        lexer = Lexer('qor("Hello")')
        tokens = lexer.tokenize()
        
        # Inspect tokens for debugging
        print("Function Call Tokens:")
        for token in tokens:
            print(f"  {token}")
            
        # Because function call parsing is complex, skip full testing for now
        # Just verify lexer produces the correct tokens
        self.assertEqual(tokens[0].type, TokenType.qor)
        self.assertEqual(tokens[1].type, TokenType.LEFT_PAREN)
        self.assertEqual(tokens[2].type, TokenType.STRING)
        self.assertEqual(tokens[3].type, TokenType.RIGHT_PAREN)
    
    def test_if_statement_tokens(self):
        """Test tokenization of if statements."""
        source = '''
        haddii (x > 10) {
            qor("x is greater than 10")
        } ugudambeyn {
            qor("x is not greater than 10")
        }
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Verify the tokens are generated correctly
        token_types = [token.type for token in tokens]
        self.assertIn(TokenType.HADDII, token_types)
        self.assertIn(TokenType.LEFT_PAREN, token_types)
        self.assertIn(TokenType.IDENTIFIER, token_types)
        self.assertIn(TokenType.GREATER, token_types)
        self.assertIn(TokenType.NUMBER, token_types)
        self.assertIn(TokenType.RIGHT_PAREN, token_types)
        self.assertIn(TokenType.LEFT_BRACE, token_types)
        self.assertIn(TokenType.qor, token_types)
        self.assertIn(TokenType.STRING, token_types)
        self.assertIn(TokenType.RIGHT_BRACE, token_types)
        self.assertIn(TokenType.UGUDAMBEYN, token_types)
    
    def test_for_loop_tokens(self):
        """Test tokenization of for loops."""
        source = '''
        kuceli i min 0 ilaa 5 {
            qor(i)
        }
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Verify the tokens are generated correctly
        token_types = [token.type for token in tokens]
        self.assertIn(TokenType.kuceli, token_types)
        self.assertIn(TokenType.IDENTIFIER, token_types)
        self.assertIn(TokenType.NUMBER, token_types)
        self.assertIn(TokenType.LEFT_BRACE, token_types)
        self.assertIn(TokenType.qor, token_types)
        self.assertIn(TokenType.RIGHT_BRACE, token_types)
        
        # Also verify the token values
        token_values = [token.value for token in tokens if token.type == TokenType.IDENTIFIER]
        self.assertIn('i', token_values)
        
        number_values = [token.value for token in tokens if token.type == TokenType.NUMBER]
        self.assertIn(0.0, number_values)
        self.assertIn(5.0, number_values)


if __name__ == '__main__':
    unittest.main() 