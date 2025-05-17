import unittest
from src.core.lexer import Lexer
from src.core.tokens import TokenType


class TestLexer(unittest.TestCase):
    def test_basic_tokens(self):
        """Test basic token recognition."""
        source = 'qor("Hello World")'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Check if tokens are generated correctly
        self.assertEqual(tokens[0].type, TokenType.qor)
        self.assertEqual(tokens[1].type, TokenType.LEFT_PAREN)
        self.assertEqual(tokens[2].type, TokenType.STRING)
        self.assertEqual(tokens[2].value, "Hello World")
        self.assertEqual(tokens[3].type, TokenType.RIGHT_PAREN)
    
    def test_variable_declaration(self):
        """Test token generation for variable declarations."""
        source = 'door magac = "Sharafdin";'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Check tokens for variable declaration
        self.assertEqual(tokens[0].type, TokenType.DOOR)
        self.assertEqual(tokens[1].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[1].value, "magac")
        self.assertEqual(tokens[2].type, TokenType.EQUAL)
        self.assertEqual(tokens[3].type, TokenType.STRING)
        self.assertEqual(tokens[3].value, "Sharafdin")
        self.assertEqual(tokens[4].type, TokenType.SEMICOLON)
    
    def test_static_type_declaration(self):
        """Test token generation for static type declarations."""
        source = 'abn da = 10;'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Check tokens for static type declaration
        self.assertEqual(tokens[0].type, TokenType.abn)
        self.assertEqual(tokens[1].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[1].value, "da")
        self.assertEqual(tokens[2].type, TokenType.EQUAL)
        self.assertEqual(tokens[3].type, TokenType.NUMBER)
        self.assertEqual(tokens[3].value, 10.0)
        self.assertEqual(tokens[4].type, TokenType.SEMICOLON)
    
    def test_arithmetic_operators(self):
        """Test token generation for arithmetic operators."""
        source = 'a + b - c * d / e % f'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Check tokens for operators
        self.assertEqual(tokens[0].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[1].type, TokenType.PLUS)
        self.assertEqual(tokens[2].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[3].type, TokenType.MINUS)
        self.assertEqual(tokens[4].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[5].type, TokenType.STAR)
        self.assertEqual(tokens[6].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[7].type, TokenType.SLASH)
        self.assertEqual(tokens[8].type, TokenType.IDENTIFIER)
        self.assertEqual(tokens[9].type, TokenType.MODULO)
        self.assertEqual(tokens[10].type, TokenType.IDENTIFIER)
    
    def test_comments(self):
        """Test comment handling in lexer."""
        source = '''
            // This is a single line comment
            door x = 10; // Comment after code
            /* This is a 
               multi-line comment */
            door y = 20;
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        
        # Comments should be skipped, so check that only code tokens remain
        var_tokens = [token for token in tokens if token.type == TokenType.DOOR]
        self.assertEqual(len(var_tokens), 2)
        
        identifier_tokens = [token for token in tokens if token.type == TokenType.IDENTIFIER]
        self.assertEqual(identifier_tokens[0].value, "x")
        self.assertEqual(identifier_tokens[1].value, "y")


if __name__ == '__main__':
    unittest.main() 