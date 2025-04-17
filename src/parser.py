from src.tokens import TokenType
from src.ast import ASTNode, NodeType
from src.errors import ParserError


class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.current_token_index = 0
        self.current_token = self.tokens[self.current_token_index]

    def advance(self):
        self.current_token_index += 1
        if self.current_token_index < len(self.tokens):
            self.current_token = self.tokens[self.current_token_index]

    def expect(self, token_type):
        if self.current_token.type == token_type:
            token = self.current_token
            self.advance()
            return token
        else:
            raise ParserError(
                f"Expected {token_type}, got {self.current_token.type}", self.current_token)

    def parse(self):
        statements = []
        while self.current_token.type != TokenType.EOF:
            statements.append(self.parse_statement())
        return ASTNode(NodeType.PROGRAM, children=statements)

    def parse_statement(self):
        ttype = self.current_token.type

        # Handle dynamic typing (door)
        if ttype == TokenType.DOOR:
            return self.parse_variable_declaration(is_static=False)
        # Handle static typing (tiro, qoraal, labadaran, liis, shey)
        elif ttype in (TokenType.TIRO, TokenType.QORAAL, TokenType.LABADARAN, TokenType.LIIS, TokenType.SHEY):
            return self.parse_variable_declaration(is_static=True)
        elif ttype == TokenType.HOWL:
            return self.parse_function_definition()
        elif ttype == TokenType.QOR or ttype == TokenType.AKHRI:
            # 'qor' or 'akhri' -> function call
            return self.parse_function_call()
        elif ttype == TokenType.HADDII:
            return self.parse_if_statement()
        elif ttype == TokenType.KU_CELI:
            return self.parse_loop_statement()
        elif ttype == TokenType.INTA_AY:
            return self.parse_while_statement()
        elif ttype == TokenType.JOOJI:
            return self.parse_break_statement()
        elif ttype == TokenType.SII_WAD:
            return self.parse_continue_statement()
        elif ttype == TokenType.SOO_CELI:
            return self.parse_return_statement()
        elif ttype == TokenType.ISKU_DAY:
            return self.parse_try_catch()
        elif ttype == TokenType.KA_KEEN:
            return self.parse_import_statement()
        elif ttype == TokenType.FASALKA:
            return self.parse_class_definition()
        elif ttype == TokenType.IDENTIFIER:
            # Check if this is a function call (identifier followed by parentheses)
            if (self.current_token_index + 1 < len(self.tokens) and
                    self.tokens[self.current_token_index + 1].type == TokenType.LEFT_PAREN):
                return self.parse_function_call()

            # Check if this is an object method call (obj.method(...))
            # or a variable assignment (obj.prop = value) or (obj[idx] = value)
            if (self.current_token_index + 1 < len(self.tokens) and
                (self.tokens[self.current_token_index + 1].type == TokenType.DOT or
                 self.tokens[self.current_token_index + 1].type == TokenType.LEFT_BRACKET)):
                token_value = self.current_token.value
                self.advance()  # consume the identifier

                # Handle method call or property access
                if self.current_token.type == TokenType.DOT:
                    self.advance()  # consume the dot

                    # Get the property or method name
                    property_name = self.current_token.value
                    self.advance()  # consume property name

                    # Handle method call (obj.method(...))
                    if self.current_token.type == TokenType.LEFT_PAREN:
                        # Parse arguments
                        args = []
                        self.advance()  # consume left paren

                        if self.current_token.type != TokenType.RIGHT_PAREN:
                            args.append(self.parse_expression())
                            while self.current_token.type == TokenType.COMMA:
                                self.advance()
                                args.append(self.parse_expression())

                        self.expect(TokenType.RIGHT_PAREN)

                        # Create function call node with obj.method as function name
                        return ASTNode(NodeType.FUNCTION_CALL, value=f"{token_value}.{property_name}", children=args)

                    # Handle property assignment (obj.prop = value)
                    elif self.current_token.type == TokenType.EQUAL:
                        self.advance()  # consume equals
                        value_expr = self.parse_expression()

                        # Create an object to represent the target
                        target = ASTNode(NodeType.PROPERTY_ACCESS, value=property_name,
                                         children=[ASTNode(NodeType.IDENTIFIER, value=token_value)])

                        # Create an assignment node
                        return ASTNode(NodeType.ASSIGNMENT, children=[target, value_expr])

                # Handle index access and assignment (obj[idx])
                elif self.current_token.type == TokenType.LEFT_BRACKET:
                    self.advance()  # consume left bracket
                    index_expr = self.parse_expression()
                    self.expect(TokenType.RIGHT_BRACKET)

                    # Handle assignment (obj[idx] = value)
                    if self.current_token.type == TokenType.EQUAL:
                        self.advance()  # consume equals
                        value_expr = self.parse_expression()

                        # Create an object to represent the target
                        target = ASTNode(NodeType.INDEX_ACCESS,
                                         children=[ASTNode(NodeType.IDENTIFIER, value=token_value), index_expr])

                        # Create an assignment node
                        return ASTNode(NodeType.ASSIGNMENT, children=[target, value_expr])

                # If we get here, something is wrong
                raise ParserError(
                    f"Expected '(' or '=' after property access", self.current_token)

            # Handle regular variable assignment (var = value)
            elif (self.current_token_index + 1 < len(self.tokens) and
                  self.tokens[self.current_token_index + 1].type == TokenType.EQUAL):
                var_name = self.current_token.value
                self.advance()  # consume identifier
                self.advance()  # consume equals
                value_expr = self.parse_expression()

                # Create an assignment node
                return ASTNode(NodeType.ASSIGNMENT,
                               children=[ASTNode(NodeType.IDENTIFIER, value=var_name), value_expr])

        # Top-level 'haddii_kale', 'haddii_kalena' are invalid
        if ttype in (TokenType.HADDII_KALE, TokenType.HADDII_KALENA):
            raise ParserError(
                f"Unexpected token: {ttype} at top-level.", self.current_token)

        raise ParserError(f"Unexpected token: {ttype}", self.current_token)

    # -----------------------------
    #  door x = 5  (dynamic typing)
    #  tiro y = 10 (static typing)
    # -----------------------------
    def parse_variable_declaration(self, is_static=False):
        var_type = None

        if is_static:
            # Store the type information for static typing
            var_type = self.current_token.type
            self.advance()
        else:
            # For dynamic typing (door)
            self.expect(TokenType.DOOR)

        var_name = self.current_token.value
        self.expect(TokenType.IDENTIFIER)
        self.expect(TokenType.EQUAL)
        expr = self.parse_expression()

        # Include type information in the AST node for static typing
        node = ASTNode(NodeType.VARIABLE_DECLARATION,
                       value=var_name, children=[expr])
        if is_static:
            node.var_type = var_type

        return node

    # -----------------------------
    #  howl foo(a, b) { ... }
    # -----------------------------
    def parse_function_definition(self):
        self.expect(TokenType.HOWL)
        func_name = self.current_token.value
        self.expect(TokenType.IDENTIFIER)
        self.expect(TokenType.LEFT_PAREN)

        params = []
        while self.current_token.type != TokenType.RIGHT_PAREN:
            params.append(self.current_token.value)
            self.expect(TokenType.IDENTIFIER)
            if self.current_token.type == TokenType.COMMA:
                self.advance()

        self.expect(TokenType.RIGHT_PAREN)
        self.expect(TokenType.LEFT_BRACE)

        body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            body.append(self.parse_statement())

        self.expect(TokenType.RIGHT_BRACE)
        return ASTNode(NodeType.FUNCTION_DEFINITION, value=func_name,
                       children=[ASTNode(NodeType.IDENTIFIER, value=p) for p in params] + body)

    # -----------------------------
    #  Function calls: qor("Hi") or akhri("Enter name:")
    # -----------------------------
    def parse_function_call(self):
        """Parse a function call like 'qor("Hello")' """
        func_name = self.current_token.value
        if self.current_token.type != TokenType.IDENTIFIER and self.current_token.type not in (TokenType.QOR, TokenType.AKHRI, TokenType.QORAAL, TokenType.TIRO, TokenType.LABADARAN, TokenType.LIIS, TokenType.SHEY):
            raise ParserError(
                f"Expected function name, got {self.current_token.type}")

        # For non-identifier function names (like type names), get the value from the type
        if self.current_token.type != TokenType.IDENTIFIER:
            func_name = self.current_token.type.value

        self.advance()
        func_call = self.parse_function_call_helper(func_name)

        # If this is a function call as a statement (not part of an expression),
        # consume the semicolon if present, but don't require it
        if self.current_token.type == TokenType.SEMICOLON:
            self.advance()  # consume semicolon

        return func_call

    # -----------------------------
    #  Import statement: ka_keen "file.sp"
    # -----------------------------
    def parse_import_statement(self):
        self.expect(TokenType.KA_KEEN)
        if self.current_token.type != TokenType.STRING:
            raise SyntaxError("Expected a string after ka_keen")
        filename = self.current_token.value
        self.advance()  # consume the STRING
        return ASTNode(NodeType.IMPORT_STATEMENT, value=filename)

    # -----------------------------
    #  If statement:
    #  haddii (cond) { ... }
    #  [haddii_kale (cond2) { ... }]
    #  [haddii_kalena { ... }]
    # -----------------------------
    def parse_if_statement(self):
        self.expect(TokenType.HADDII)
        self.expect(TokenType.LEFT_PAREN)
        condition = self.parse_logical_expression()
        self.expect(TokenType.RIGHT_PAREN)
        self.expect(TokenType.LEFT_BRACE)

        if_body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            if_body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        children = [condition] + if_body

        # Parse zero or more 'haddii_kale'
        while self.current_token.type == TokenType.HADDII_KALE:
            self.advance()
            self.expect(TokenType.LEFT_PAREN)
            elif_condition = self.parse_logical_expression()
            self.expect(TokenType.RIGHT_PAREN)
            self.expect(TokenType.LEFT_BRACE)
            elif_body = []
            while self.current_token.type != TokenType.RIGHT_BRACE:
                elif_body.append(self.parse_statement())
            self.expect(TokenType.RIGHT_BRACE)
            elif_node = ASTNode(NodeType.IF_STATEMENT, children=[
                                elif_condition] + elif_body)
            children.append(elif_node)

        # Optionally parse 'haddii_kalena'
        if self.current_token.type == TokenType.HADDII_KALENA:
            self.advance()
            self.expect(TokenType.LEFT_BRACE)
            else_body = []
            while self.current_token.type != TokenType.RIGHT_BRACE:
                else_body.append(self.parse_statement())
            self.expect(TokenType.RIGHT_BRACE)
            # We'll treat else_body as a BLOCK node
            else_block = ASTNode(NodeType.BLOCK, children=else_body)
            children.append(else_block)

        return ASTNode(NodeType.IF_STATEMENT, children=children)

    # -----------------------------
    #  Loops: ku_celi i min 1 ilaa 5 { ... }
    # -----------------------------
    def parse_loop_statement(self):
        self.expect(TokenType.KU_CELI)
        loop_var = self.current_token.value
        self.expect(TokenType.IDENTIFIER)  # e.g. i
        self.expect(TokenType.IDENTIFIER)  # "min"
        start_expr = self.parse_expression()
        self.expect(TokenType.IDENTIFIER)  # "ilaa"
        end_expr = self.parse_expression()
        self.expect(TokenType.LEFT_BRACE)

        body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        children = [start_expr, end_expr] + body
        return ASTNode(NodeType.LOOP_STATEMENT, value=loop_var, children=children)

    # -----------------------------
    #  While loop: inta_ay (condition) { ... }
    # -----------------------------
    def parse_while_statement(self):
        self.expect(TokenType.INTA_AY)
        self.expect(TokenType.LEFT_PAREN)
        condition = self.parse_logical_expression()
        self.expect(TokenType.RIGHT_PAREN)
        self.expect(TokenType.LEFT_BRACE)

        body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        children = [condition] + body
        return ASTNode(NodeType.WHILE_STATEMENT, children=children)

    # -----------------------------
    #  Break statement: jooji
    # -----------------------------
    def parse_break_statement(self):
        self.expect(TokenType.JOOJI)
        return ASTNode(NodeType.BREAK_STATEMENT)

    # -----------------------------
    #  Continue statement: sii_wad
    # -----------------------------
    def parse_continue_statement(self):
        self.expect(TokenType.SII_WAD)
        return ASTNode(NodeType.CONTINUE_STATEMENT)

    # -----------------------------
    #  try/catch: isku_day { ... } qabo (err) { ... }
    # -----------------------------
    def parse_try_catch(self):
        self.expect(TokenType.ISKU_DAY)
        self.expect(TokenType.LEFT_BRACE)

        try_body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            try_body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        # parse 'qabo (errName)'
        self.expect(TokenType.QABO)
        self.expect(TokenType.LEFT_PAREN)
        error_var = self.current_token.value
        self.expect(TokenType.IDENTIFIER)
        self.expect(TokenType.RIGHT_PAREN)
        self.expect(TokenType.LEFT_BRACE)

        catch_body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            catch_body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        return ASTNode(NodeType.TRY_CATCH, value=error_var,
                       children=[ASTNode(NodeType.BLOCK, children=try_body),
                                 ASTNode(NodeType.BLOCK, children=catch_body)])

    # -----------------------------
    #  Class Definition: fasalka Ey ka_dhaxal Xayawaan { ... }
    # -----------------------------
    def parse_class_definition(self):
        self.expect(TokenType.FASALKA)
        class_name = self.current_token.value
        self.expect(TokenType.IDENTIFIER)

        parent_name = None
        if self.current_token.type == TokenType.KA_DHAXAL:
            self.advance()  # consume 'ka_dhaxal'
            parent_name = self.current_token.value
            self.expect(TokenType.IDENTIFIER)

        self.expect(TokenType.LEFT_BRACE)

        # For simplicity, parse the body as statements
        class_body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            class_body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        node = ASTNode(NodeType.CLASS_DEFINITION,
                       value=class_name, children=class_body)
        # if parent, store it in node.value or create a separate property
        if parent_name:
            node.value = (class_name, parent_name)
        return node

    # -----------------------------
    #  List Literal: [1, 2, 3]
    # -----------------------------
    def parse_list_literal(self):
        self.expect(TokenType.LEFT_BRACKET)

        elements = []
        while self.current_token.type != TokenType.RIGHT_BRACKET:
            elements.append(self.parse_expression())
            if self.current_token.type == TokenType.COMMA:
                self.advance()
            else:
                break

        self.expect(TokenType.RIGHT_BRACKET)
        return ASTNode(NodeType.LIST_LITERAL, children=elements)

    # -----------------------------
    #  Object Literal: {name: "value", age: 30}
    # -----------------------------
    def parse_object_literal(self):
        self.expect(TokenType.LEFT_BRACE)

        properties = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            # Property key
            if self.current_token.type == TokenType.IDENTIFIER or self.current_token.type == TokenType.STRING:
                key = self.current_token.value
                self.advance()
            else:
                raise ParserError(
                    "Expected property name as identifier or string", self.current_token)

            # Colon separator
            self.expect(TokenType.COLON)

            # Property value
            value = self.parse_expression()

            # Create a property node with key as value and expression as child
            property_node = ASTNode(
                NodeType.LITERAL, value=key, children=[value])
            properties.append(property_node)

            if self.current_token.type == TokenType.COMMA:
                self.advance()
            else:
                break

        self.expect(TokenType.RIGHT_BRACE)
        return ASTNode(NodeType.OBJECT_LITERAL, children=properties)

    # -----------------------------
    #  Property Access: obj.property
    # -----------------------------
    def parse_property_access(self, left):
        self.expect(TokenType.DOT)

        if self.current_token.type != TokenType.IDENTIFIER:
            raise ParserError(
                "Expected property name after dot", self.current_token)

        property_name = self.current_token.value
        self.advance()

        # Special handling for method calls
        if self.current_token.type == TokenType.LEFT_PAREN:
            # This is a method call like object.method()
            args = []
            self.advance()  # consume left paren

            if self.current_token.type != TokenType.RIGHT_PAREN:
                args.append(self.parse_expression())
                while self.current_token.type == TokenType.COMMA:
                    self.advance()
                    args.append(self.parse_expression())

            self.expect(TokenType.RIGHT_PAREN)
            return ASTNode(NodeType.METHOD_CALL, value=property_name, children=[left] + args)

        return ASTNode(NodeType.PROPERTY_ACCESS, value=property_name, children=[left])

    # -----------------------------
    #  Index Access: arr[index]
    # -----------------------------
    def parse_index_access(self, left):
        self.expect(TokenType.LEFT_BRACKET)
        index = self.parse_expression()
        self.expect(TokenType.RIGHT_BRACKET)

        return ASTNode(NodeType.INDEX_ACCESS, children=[left, index])

    # -----------------------------
    #  Expression Parsing
    # -----------------------------
    def parse_expression(self):
        """Parse an arithmetic expression"""
        left = self.parse_term()

        while self.current_token.type in (TokenType.PLUS, TokenType.MINUS):
            op = self.current_token
            self.advance()
            right = self.parse_term()
            left = ASTNode(NodeType.BINARY_OPERATION,
                           value=op.value, children=[left, right])

        return left

    def parse_term(self):
        left = self.parse_factor()

        while self.current_token.type in (TokenType.STAR, TokenType.SLASH, TokenType.MODULO):
            op = self.current_token
            self.advance()
            right = self.parse_factor()
            left = ASTNode(NodeType.BINARY_OPERATION,
                           value=op.value, children=[left, right])

        return left

    def parse_factor(self):
        token = self.current_token

        if token.type == TokenType.NUMBER:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=token.value)
        if token.type == TokenType.STRING:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=token.value)
        if token.type == TokenType.TRUE:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=True)
        if token.type == TokenType.FALSE:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=False)
        if token.type == TokenType.NULL:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=None)
        if token.type == TokenType.IDENTIFIER or token.type in (TokenType.QORAAL, TokenType.TIRO, TokenType.LABADARAN, TokenType.LIIS, TokenType.SHEY):
            # Allow type names to be used as function names
            token_value = token.value if token.type == TokenType.IDENTIFIER else token.type.value
            self.advance()

            # Check if this is a function call (followed by left parenthesis)
            if self.current_token.type == TokenType.LEFT_PAREN:
                return self.parse_function_call_helper(token_value)

            node = ASTNode(NodeType.IDENTIFIER, value=token_value)

            # Handle property access (obj.prop) and index access (arr[idx])
            while self.current_token.type in (TokenType.DOT, TokenType.LEFT_BRACKET):
                if self.current_token.type == TokenType.DOT:
                    node = self.parse_property_access(node)
                elif self.current_token.type == TokenType.LEFT_BRACKET:
                    node = self.parse_index_access(node)

            return node

        if token.type == TokenType.LEFT_PAREN:
            self.advance()
            expr = self.parse_expression()
            self.expect(TokenType.RIGHT_PAREN)
            return expr
        if token.type == TokenType.LEFT_BRACKET:
            return self.parse_list_literal()
        if token.type == TokenType.LEFT_BRACE:
            return self.parse_object_literal()

        raise ParserError(f"Unexpected token in factor: {token.type}")

    def parse_function_call_helper(self, func_name):
        """Helper method to parse a function call once we've identified the function name"""
        self.expect(TokenType.LEFT_PAREN)
        args = []

        # Parse arguments
        if self.current_token.type != TokenType.RIGHT_PAREN:
            args.append(self.parse_expression())
            while self.current_token.type == TokenType.COMMA:
                self.advance()
                args.append(self.parse_expression())

        self.expect(TokenType.RIGHT_PAREN)

        # Create function call node
        function_call = ASTNode(NodeType.FUNCTION_CALL, value=func_name, children=args)

        # If this is a function call as a statement (not part of an expression),
        # consume the semicolon if present, but don't require it
        if self.current_token.type == TokenType.SEMICOLON:
            self.advance()  # consume semicolon

        return function_call

    def parse_logical_expression(self):
        """Parse a logical expression like 'a > 5 && b < 10'"""
        left = self.parse_comparison_expression()

        while self.current_token.type in (TokenType.AND, TokenType.OR):
            op_token = self.current_token
            self.advance()
            right = self.parse_comparison_expression()
            left = ASTNode(NodeType.BINARY_OPERATION,
                           value=op_token.value, children=[left, right])

        return left

    def parse_comparison_expression(self):
        left = self.parse_expression()

        while self.current_token.type in (
            TokenType.GREATER, TokenType.LESS,
            TokenType.GREATER_EQUAL, TokenType.LESS_EQUAL,
            TokenType.EQUAL, TokenType.NOT_EQUAL
        ):
            op_token = self.current_token
            self.advance()

            if op_token.type == TokenType.EQUAL and self.current_token.type == TokenType.EQUAL:
                operator_value = "=="
                self.advance()
            else:
                operator_value = op_token.value

            right = self.parse_expression()
            left = ASTNode(NodeType.BINARY_OPERATION,
                           value=operator_value, children=[left, right])

        return left

    def parse_return_statement(self):
        self.expect(TokenType.SOO_CELI)
        # If there is an expression after soo_celi, parse it
        if self.current_token.type != TokenType.SEMICOLON:
            expr = self.parse_expression()
            return ASTNode(NodeType.RETURN_STATEMENT, children=[expr])
        # Otherwise, it's a return with no value
        return ASTNode(NodeType.RETURN_STATEMENT)
