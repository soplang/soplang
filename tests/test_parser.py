import unittest
from src.core.lexer import Lexer
from src.core.parser import Parser
from src.core.ast import NodeType


class TestParser(unittest.TestCase):
    def test_variable_declaration(self):
        """Test parsing of variable declarations."""
        source = 'door x = 42;'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        # Check if AST is generated correctly
        self.assertEqual(ast.type, NodeType.PROGRAM)
        self.assertEqual(len(ast.children), 1)
        
        var_decl = ast.children[0]
        self.assertEqual(var_decl.type, NodeType.VARIABLE_DECLARATION)
        self.assertEqual(var_decl.value, 'x')
        
        # Value should be a number
        expr = var_decl.children[0]
        self.assertEqual(expr.type, NodeType.NUMBER)
        self.assertEqual(expr.value, 42.0)
    
    def test_static_type_declaration(self):
        """Test parsing of static type declarations."""
        source = 'tiro y = 10;'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        # Check if AST is generated correctly
        self.assertEqual(ast.type, NodeType.PROGRAM)
        self.assertEqual(len(ast.children), 1)
        
        var_decl = ast.children[0]
        self.assertEqual(var_decl.type, NodeType.VARIABLE_DECLARATION)
        self.assertEqual(var_decl.value, 'y')
        self.assertTrue(hasattr(var_decl, 'var_type'))
        
        # Value should be a number
        expr = var_decl.children[0]
        self.assertEqual(expr.type, NodeType.NUMBER)
        self.assertEqual(expr.value, 10.0)
    
    def test_function_call(self):
        """Test parsing of function calls."""
        source = 'qor("Hello");'
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        # Check if AST is generated correctly
        self.assertEqual(ast.type, NodeType.PROGRAM)
        self.assertEqual(len(ast.children), 1)
        
        func_call = ast.children[0]
        self.assertEqual(func_call.type, NodeType.FUNCTION_CALL)
        self.assertEqual(func_call.value, 'qor')
        
        # Check argument
        self.assertEqual(len(func_call.children), 1)
        arg = func_call.children[0]
        self.assertEqual(arg.type, NodeType.STRING)
        self.assertEqual(arg.value, 'Hello')
    
    def test_if_statement(self):
        """Test parsing of if statements."""
        source = '''
        haddii (x > 10) {
            qor("x is greater than 10");
        } haddii_kalena {
            qor("x is not greater than 10");
        }
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        # Check if AST is generated correctly
        self.assertEqual(ast.type, NodeType.PROGRAM)
        self.assertEqual(len(ast.children), 1)
        
        if_stmt = ast.children[0]
        self.assertEqual(if_stmt.type, NodeType.IF_STATEMENT)
        
        # Should have condition, if body, and else block
        self.assertGreaterEqual(len(if_stmt.children), 3)
        
        # Check condition
        condition = if_stmt.children[0]
        self.assertEqual(condition.type, NodeType.BINARY_EXPRESSION)
        self.assertEqual(condition.value, '>')
        
        # Check if body
        if_body = if_stmt.children[1]
        self.assertEqual(if_body.type, NodeType.BLOCK)
        
        # Check else body
        else_body = if_stmt.children[2]
        self.assertEqual(else_body.type, NodeType.BLOCK)
    
    def test_for_loop(self):
        """Test parsing of for loops."""
        source = '''
        ku_celi i min 0 ilaa 5 {
            qor(i);
        }
        '''
        lexer = Lexer(source)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        
        # Check if AST is generated correctly
        self.assertEqual(ast.type, NodeType.PROGRAM)
        self.assertEqual(len(ast.children), 1)
        
        loop_stmt = ast.children[0]
        self.assertEqual(loop_stmt.type, NodeType.LOOP_STATEMENT)
        self.assertEqual(loop_stmt.value, 'i')  # loop variable
        
        # Should have start, end, and body
        self.assertGreaterEqual(len(loop_stmt.children), 3)
        
        # Check start expression
        start = loop_stmt.children[0]
        self.assertEqual(start.type, NodeType.NUMBER)
        self.assertEqual(start.value, 0.0)
        
        # Check end expression
        end = loop_stmt.children[1]
        self.assertEqual(end.type, NodeType.NUMBER)
        self.assertEqual(end.value, 5.0)
        
        # Check body
        body = loop_stmt.children[2]
        self.assertEqual(body.type, NodeType.BLOCK)


if __name__ == '__main__':
    unittest.main() 