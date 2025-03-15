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
        elif ttype == TokenType.JOOJI:
            return self.parse_break_statement()
        elif ttype == TokenType.SII_WAD:
            return self.parse_continue_statement()
        elif ttype == TokenType.ISKU_DAY:
            return self.parse_try_catch()
        elif ttype == TokenType.KA_KEEN:
            return self.parse_import_statement()
        elif ttype == TokenType.FASALKA:
            return self.parse_class_definition()

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
        func_name = self.current_token.value
        self.advance()  # consume QOR or AKHRI
        self.expect(TokenType.LEFT_PAREN)

        args = []
        while self.current_token.type != TokenType.RIGHT_PAREN:
            args.append(self.parse_expression())
            if self.current_token.type == TokenType.COMMA:
                self.advance()

        self.expect(TokenType.RIGHT_PAREN)
        return ASTNode(NodeType.FUNCTION_CALL, value=func_name, children=args)

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
        condition = self.parse_comparison_expression()
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
            elif_condition = self.parse_comparison_expression()
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

    def parse_expression(self):
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

        while self.current_token.type in (TokenType.STAR, TokenType.SLASH):
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
        if token.type == TokenType.IDENTIFIER:
            self.advance()
            node = ASTNode(NodeType.IDENTIFIER, value=token.value)

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

        raise SyntaxError(f"Unexpected token in factor: {token.type}")
