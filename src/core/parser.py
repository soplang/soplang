from src.core.ast import ASTNode, NodeType
from src.core.tokens import TokenType
from src.utils.errors import ParserError


class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.current_token_index = 0
        self.current_token = self.tokens[self.current_token_index]

    def get_friendly_token_name(self, token_type):
        """Convert token types to user-friendly descriptions."""
        token_descriptions = {
            TokenType.LEFT_PAREN: "opening parenthesis '('",
            TokenType.RIGHT_PAREN: "closing parenthesis ')'",
            TokenType.LEFT_BRACE: "opening brace '{'",
            TokenType.RIGHT_BRACE: "closing brace '}'",
            TokenType.LEFT_BRACKET: "opening bracket '['",
            TokenType.RIGHT_BRACKET: "closing bracket ']'",
            TokenType.COMMA: "comma ','",
            TokenType.DOT: "dot '.'",
            TokenType.COLON: "colon ':'",
            TokenType.SEMICOLON: "semicolon ';'",
            TokenType.PLUS: "plus '+'",
            TokenType.MINUS: "minus '-'",
            TokenType.STAR: "multiplication '*'",
            TokenType.SLASH: "division '/'",
            TokenType.MODULO: "modulo '%'",
            TokenType.EQUAL: "equals sign '='",
            TokenType.GREATER: "greater than '>'",
            TokenType.LESS: "less than '<'",
            TokenType.GREATER_EQUAL: "greater than or equal '>='",
            TokenType.LESS_EQUAL: "less than or equal '<='",
            TokenType.NOT_EQUAL: "not equal '!='",
            TokenType.AND: "logical AND '&&'",
            TokenType.OR: "logical OR '||'",
            TokenType.NOT: "logical NOT '!'",
            TokenType.IDENTIFIER: "variable or function name",
            TokenType.STRING: "string",
            TokenType.NUMBER: "number",
            TokenType.TRUE: "boolean 'run' (true)",
            TokenType.FALSE: "boolean 'been' (false)",
            TokenType.NULL: "null",
            # Keywords
            TokenType.DOOR: "keyword 'door'",
            TokenType.HAWL: "keyword 'hawl' (function)",
            TokenType.CELI: "keyword 'celi' (return)",
            TokenType.QOR: "keyword 'qor' (print)",
            TokenType.HADDII: "keyword 'haddii' (if)",
            TokenType.HADDII_KALE: "keyword 'haddii_kale' (else if)",
            TokenType.HADDII_KALENA: "keyword 'haddii_kalena' (else)",
            TokenType.KU_CELI: "keyword 'ku_celi' (for)",
            TokenType.INTA_AY: "keyword 'inta_ay' (while)",
            TokenType.TIRO: "keyword 'tiro' (number type)",
            TokenType.QORAAL: "keyword 'qoraal' (string type)",
            TokenType.LABADARAN: "keyword 'labadaran' (boolean type)",
            TokenType.LIIS: "keyword 'liis' (list type)",
            TokenType.SHEY: "keyword 'shey' (object type)",
        }

        return token_descriptions.get(token_type, str(token_type))

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
            # Get user-friendly descriptions for the tokens
            expected_description = self.get_friendly_token_name(token_type)
            found_description = self.get_friendly_token_name(self.current_token.type)

            # Use the "expected_token" error code from ErrorMessageManager.PARSER_ERRORS
            raise ParserError(
                "expected_token",
                expected=expected_description,
                found=found_description,
                token=self.current_token,
                line=getattr(self.current_token, 'line', None),
                position=getattr(self.current_token, 'position', None)
            )

    def parse(self):
        statements = []
        while self.current_token.type != TokenType.EOF:
            statements.append(self.parse_statement())
        return ASTNode(NodeType.PROGRAM, children=statements)

    def parse_statement(self):
        """
        Parse any statement in the language
        """
        token = self.current_token
        token_type = token.type
        line = getattr(token, 'line', None)
        position = getattr(token, 'position', None)

        # Handle haddii (if statement)
        if token_type == TokenType.HADDII:
            return self.parse_if_statement()

        # Handle variable declarations with static typing
        elif token_type in (
            TokenType.TIRO,
            TokenType.QORAAL,
            TokenType.LABADARAN,
            TokenType.LIIS,
            TokenType.SHEY,
        ):
            return self.parse_variable_declaration(is_static=True)

        # Handle variable declaration with dynamic typing (door)
        elif token_type == TokenType.DOOR:
            return self.parse_variable_declaration(is_static=False)

        # Handle function definition (hawl)
        elif token_type == TokenType.hawl:
            return self.parse_function_definition()

        # Handle return statement (celi)
        elif token_type == TokenType.CELI:
            return self.parse_return_statement()

        # Handle print statement (qor)
        elif token_type == TokenType.QOR:
            self.advance()  # Consume qor
            # Parse function call expression
            return ASTNode(
                NodeType.FUNCTION_CALL,
                value="qor",
                children=[self.parse_expression()],
                line=line,
                position=position
            )

        # Handle function calls
        elif token_type == TokenType.QOR or token_type == TokenType.GELIN:
            return self.parse_function_call()

        # Handle loops
        elif token_type == TokenType.KU_CELI:
            return self.parse_loop_statement()

        # Handle while loop
        elif token_type == TokenType.INTA_AY:
            return self.parse_while_statement()

        # Handle break statement
        elif token_type == TokenType.JOOJI:
            return self.parse_break_statement()

        # Handle continue statement
        elif token_type == TokenType.SII_WAD:
            return self.parse_continue_statement()

        # Handle try/catch
        elif token_type == TokenType.ISKU_DAY:
            return self.parse_try_catch()

        # Handle import statement
        elif token_type == TokenType.KA_KEEN:
            return self.parse_import_statement()

        # Handle class definition
        elif token_type == TokenType.FASALKA:
            return self.parse_class_definition()

        # Handle code block
        elif token_type == TokenType.LEFT_BRACE:
            # Parse code block { ... }
            self.advance()  # Consume '{'

            statements = []
            while self.current_token.type != TokenType.RIGHT_BRACE:
                statements.append(self.parse_statement())

            self.expect(TokenType.RIGHT_BRACE)
            return ASTNode(NodeType.BLOCK, children=statements)

        # Handle identifier
        elif token_type == TokenType.IDENTIFIER:
            # This could be a function call, a variable assignment, a property access, etc.
            identifier_value = self.current_token.value
            self.advance()  # Consume the identifier

            # Handle property chains (obj.prop1.prop2) or arrays (obj[idx]) for assignment
            if self.current_token.type in (TokenType.DOT, TokenType.LEFT_BRACKET):
                left = ASTNode(NodeType.IDENTIFIER, value=identifier_value)

                # Parse any chain of property accesses or array indexing
                while self.current_token.type in (TokenType.DOT, TokenType.LEFT_BRACKET):
                    if self.current_token.type == TokenType.DOT:
                        # Handle property access (obj.prop)
                        self.advance()  # Consume the dot

                        if self.current_token.type != TokenType.IDENTIFIER:
                            raise ParserError(
                                "expected_token",
                                expected="IDENTIFIER",
                                found=self.current_token.type,
                                token=self.current_token,
                                line=getattr(self.current_token, 'line', None),
                                position=getattr(self.current_token, 'position', None)
                            )

                        prop_name = self.current_token.value
                        self.advance()  # Consume the property name

                        if self.current_token.type == TokenType.LEFT_PAREN:
                            # This is a method call (obj.method())
                            args = []
                            self.advance()  # Consume left paren

                            if self.current_token.type != TokenType.RIGHT_PAREN:
                                args.append(self.parse_expression())
                                while self.current_token.type == TokenType.COMMA:
                                    self.advance()
                                    args.append(self.parse_expression())

                            self.expect(TokenType.RIGHT_PAREN)
                            left = ASTNode(NodeType.METHOD_CALL,
                                           value=prop_name, children=[left] + args)
                        else:
                            # Regular property access (obj.prop)
                            left = ASTNode(NodeType.PROPERTY_ACCESS,
                                           value=prop_name, children=[left])

                    elif self.current_token.type == TokenType.LEFT_BRACKET:
                        # Handle array indexing (arr[idx])
                        self.advance()  # Consume left bracket
                        index = self.parse_expression()
                        self.expect(TokenType.RIGHT_BRACKET)
                        left = ASTNode(NodeType.INDEX_ACCESS, children=[left, index])

                # Now check if this is an assignment (obj.prop = value or arr[idx] = value)
                if self.current_token.type == TokenType.EQUAL:
                    self.advance()  # Consume equals
                    value = self.parse_logical_expression()
                    return ASTNode(NodeType.ASSIGNMENT, children=[left, value])

                # If not an assignment, just return the property access or method call
                return left

            # Handle function call (func())
            elif self.current_token.type == TokenType.LEFT_PAREN:
                args = []
                self.advance()  # Consume left paren

                if self.current_token.type != TokenType.RIGHT_PAREN:
                    args.append(self.parse_expression())
                    while self.current_token.type == TokenType.COMMA:
                        self.advance()
                        args.append(self.parse_expression())

                self.expect(TokenType.RIGHT_PAREN)
                return ASTNode(NodeType.FUNCTION_CALL, value=identifier_value, children=args)

            # Handle simple variable assignment (var = value)
            elif self.current_token.type == TokenType.EQUAL:
                self.advance()  # Consume equals
                value = self.parse_logical_expression()
                return ASTNode(
                    NodeType.ASSIGNMENT,
                    children=[ASTNode(NodeType.IDENTIFIER,
                                      value=identifier_value), value]
                )

            # Just a variable reference
            return ASTNode(NodeType.IDENTIFIER, value=identifier_value)

        # Top-level 'haddii_kale', 'haddii_kalena' are invalid
        if token_type in (TokenType.HADDII_KALE, TokenType.HADDII_KALENA):
            raise ParserError(
                "unexpected_token",
                token=self.get_friendly_token_name(token_type),
                line=getattr(token, 'line', None),
                position=getattr(token, 'position', None)
            )

        raise ParserError(
            "unexpected_token",
            token=self.get_friendly_token_name(token_type),
            line=getattr(token, 'line', None),
            position=getattr(token, 'position', None)
        )

    # -----------------------------
    #  door x = 5  (dynamic typing)
    #  tiro y = 10 (static typing)
    # -----------------------------
    def parse_variable_declaration(self, is_static=False):
        # Get the variable type (for static typing)
        var_type = self.current_token.type if is_static else None
        token_line = getattr(self.current_token, 'line', None)
        token_position = getattr(self.current_token, 'position', None)

        self.advance()  # Consume type/door token

        # Get the variable name
        if self.current_token.type != TokenType.IDENTIFIER:
            raise ParserError(
                "expected_token",
                expected="identifier",
                found=self.get_friendly_token_name(self.current_token.type),
                token=self.current_token,
                line=getattr(self.current_token, 'line', None),
                position=getattr(self.current_token, 'position', None)
            )

        var_name = self.current_token.value
        self.advance()  # Consume identifier

        # Expect equals sign
        self.expect(TokenType.EQUAL)

        # Parse the expression to assign to the variable
        expression = self.parse_expression()

        # Create variable declaration node
        var_node = ASTNode(
            NodeType.VARIABLE_DECLARATION,
            value=var_name,
            children=[expression],
            line=token_line,
            position=token_position
        )
        var_node.var_type = var_type  # Store type for static typing

        return var_node

    # -----------------------------
    #  hawl foo(a, b) { ... }
    # -----------------------------
    def parse_function_definition(self):
        self.expect(TokenType.HAWL)
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
        return ASTNode(
            NodeType.FUNCTION_DEFINITION,
            value=func_name,
            children=[ASTNode(NodeType.IDENTIFIER, value=p) for p in params] + body,
        )

    # -----------------------------
    #  Function calls: qor("Hi") or gelin("Enter name:")
    # -----------------------------
    def parse_function_call(self):
        """Parse a function call like 'qor("Hello")'"""
        func_name = self.current_token.value
        if (
            self.current_token.type != TokenType.IDENTIFIER and
            self.current_token.type
            not in (
                TokenType.QOR,
                TokenType.GELIN,
                TokenType.QORAAL,
                TokenType.TIRO,
                TokenType.LABADARAN,
                TokenType.LIIS,
                TokenType.SHEY,
            )
        ):
            raise ParserError(
                "expected_token",
                expected="function name",
                found=self.get_friendly_token_name(self.current_token.type),
                token=self.current_token,
                line=getattr(self.current_token, 'line', None),
                position=getattr(self.current_token, 'position', None)
            )

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
            raise ParserError(
                "expected_token",
                expected="string for file path",
                found=self.get_friendly_token_name(self.current_token.type),
                token=self.current_token,
                line=getattr(self.current_token, 'line', None),
                position=getattr(self.current_token, 'position', None)
            )
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
            elif_node = ASTNode(
                NodeType.IF_STATEMENT, children=[elif_condition] + elif_body
            )
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
    #  or with step: ku_celi i min 1 ilaa 5 by 2 { ... }
    # -----------------------------
    def parse_loop_statement(self):
        self.expect(TokenType.KU_CELI)
        loop_var = self.current_token.value
        self.expect(TokenType.IDENTIFIER)  # e.g. i
        self.expect(TokenType.IDENTIFIER)  # "min"
        start_expr = self.parse_expression()
        self.expect(TokenType.IDENTIFIER)  # "ilaa"
        end_expr = self.parse_expression()

        # Check for optional step parameter
        step_expr = None
        if (
            self.current_token.type == TokenType.IDENTIFIER and
            self.current_token.value == "by"
        ):
            self.advance()  # consume "by"
            step_expr = self.parse_expression()

        self.expect(TokenType.LEFT_BRACE)

        body = []
        while self.current_token.type != TokenType.RIGHT_BRACE:
            body.append(self.parse_statement())
        self.expect(TokenType.RIGHT_BRACE)

        children = [start_expr, end_expr]
        if step_expr:
            children.append(step_expr)
        children.extend(body)

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

        return ASTNode(
            NodeType.TRY_CATCH,
            value=error_var,
            children=[
                ASTNode(NodeType.BLOCK, children=try_body),
                ASTNode(NodeType.BLOCK, children=catch_body),
            ],
        )

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

        node = ASTNode(NodeType.CLASS_DEFINITION, value=class_name, children=class_body)
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
            if (
                self.current_token.type == TokenType.IDENTIFIER or
                self.current_token.type == TokenType.STRING
            ):
                key = self.current_token.value
                self.advance()
            else:
                raise ParserError(
                    "expected_token",
                    expected="property name (identifier or string)",
                    found=self.get_friendly_token_name(self.current_token.type),
                    token=self.current_token,
                    line=getattr(self.current_token, 'line', None),
                    position=getattr(self.current_token, 'position', None)
                )

            # Colon separator
            self.expect(TokenType.COLON)

            # Property value
            value = self.parse_expression()

            # Create a property node with key as value and expression as child
            property_node = ASTNode(NodeType.LITERAL, value=key, children=[value])
            properties.append(property_node)

            if self.current_token.type == TokenType.COMMA:
                self.advance()
            else:
                break

        self.expect(TokenType.RIGHT_BRACE)
        return ASTNode(NodeType.OBJECT_LITERAL, children=properties)

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
            left = ASTNode(
                NodeType.BINARY_OPERATION, value=op.value, children=[left, right]
            )

        return left

    def parse_term(self):
        left = self.parse_factor()

        while self.current_token.type in (
            TokenType.STAR,
            TokenType.SLASH,
            TokenType.MODULO,
        ):
            op = self.current_token
            self.advance()
            right = self.parse_factor()
            left = ASTNode(
                NodeType.BINARY_OPERATION, value=op.value, children=[left, right]
            )

        return left

    def parse_factor(self):
        """Parse factors: unary operations and postfix expressions"""
        token = self.current_token

        # Handle unary operations
        if token.type in (TokenType.PLUS, TokenType.MINUS, TokenType.NOT):
            op = token
            self.advance()
            factor = self.parse_factor()

            # For unary plus, just return the factor as is
            if op.type == TokenType.PLUS:
                return factor

            # For unary minus
            if op.type == TokenType.MINUS:
                # Create a negative number directly if it's a literal
                if factor.type == NodeType.LITERAL and isinstance(factor.value, (int, float)):
                    return ASTNode(NodeType.LITERAL, value=-factor.value)

                # Otherwise create a binary operation
                minus_one = ASTNode(NodeType.LITERAL, value=-1)
                return ASTNode(NodeType.BINARY_OPERATION, value="*", children=[minus_one, factor])

            # For NOT operator
            if op.type == TokenType.NOT:
                return ASTNode(NodeType.UNARY_OPERATION, value="!", children=[factor])

        # Handle postfix expressions
        return self.parse_postfix()

    def parse_postfix(self):
        """Parse postfix expressions: property access, method calls, and array indexing"""
        expr = self.parse_primary()

        # Handle property access (obj.prop), method calls (obj.method()), and array indexing (array[index])
        while (
            self.current_token.type == TokenType.DOT or
            self.current_token.type == TokenType.LEFT_BRACKET
        ):
            if self.current_token.type == TokenType.DOT:
                # Property access
                self.advance()  # Consume the dot

                if self.current_token.type != TokenType.IDENTIFIER:
                    raise ParserError(
                        "expected_token",
                        expected="property name",
                        found=self.get_friendly_token_name(self.current_token.type),
                        token=self.current_token,
                        line=getattr(self.current_token, 'line', None),
                        position=getattr(self.current_token, 'position', None)
                    )

                property_name = self.current_token.value
                self.advance()  # Consume the property name

                if self.current_token.type == TokenType.LEFT_PAREN:
                    # This is a method call (obj.method())
                    args = []
                    self.advance()  # Consume the left paren

                    if self.current_token.type != TokenType.RIGHT_PAREN:
                        args.append(self.parse_expression())
                        while self.current_token.type == TokenType.COMMA:
                            self.advance()
                            args.append(self.parse_expression())

                    self.expect(TokenType.RIGHT_PAREN)
                    expr = ASTNode(NodeType.METHOD_CALL,
                                   value=property_name, children=[expr] + args)
                else:
                    # Regular property access (obj.prop)
                    expr = ASTNode(NodeType.PROPERTY_ACCESS,
                                   value=property_name, children=[expr])

            elif self.current_token.type == TokenType.LEFT_BRACKET:
                # Array indexing (array[index])
                self.advance()  # Consume the left bracket
                index = self.parse_expression()
                self.expect(TokenType.RIGHT_BRACKET)
                expr = ASTNode(NodeType.INDEX_ACCESS, children=[expr, index])

        return expr

    def parse_primary(self):
        """Parse a primary expression: literal, identifier, or parenthesized expression"""
        token = self.current_token

        if token.type == TokenType.NUMBER:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=token.value)
        elif token.type == TokenType.STRING:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=token.value)
        elif token.type == TokenType.TRUE:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=True)
        elif token.type == TokenType.FALSE:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=False)
        elif token.type == TokenType.NULL:
            self.advance()
            return ASTNode(NodeType.LITERAL, value=None)
        elif token.type == TokenType.IDENTIFIER or token.type in (
            TokenType.QORAAL, TokenType.TIRO, TokenType.LABADARAN, TokenType.LIIS, TokenType.SHEY,
            TokenType.QOR, TokenType.GELIN  # Added QOR and GELIN to handle them in expressions
        ):
            # Allow type names to be used as function names
            token_value = token.value if token.type == TokenType.IDENTIFIER else token.type.value
            self.advance()

            # Check if this is a function call (followed by left parenthesis)
            if self.current_token.type == TokenType.LEFT_PAREN:
                return self.parse_function_call_helper(token_value)

            # Just an identifier
            return ASTNode(NodeType.IDENTIFIER, value=token_value)
        elif token.type == TokenType.LEFT_PAREN:
            self.advance()
            expr = self.parse_logical_expression()
            self.expect(TokenType.RIGHT_PAREN)
            return expr
        elif token.type == TokenType.LEFT_BRACKET:
            return self.parse_list_literal()
        elif token.type == TokenType.LEFT_BRACE:
            return self.parse_object_literal()
        else:
            raise ParserError(
                "unexpected_token",
                token=self.get_friendly_token_name(token.type),
                line=getattr(token, 'line', None),
                position=getattr(token, 'position', None)
            )

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
            left = ASTNode(
                NodeType.BINARY_OPERATION, value=op_token.value, children=[left, right]
            )

        return left

    def parse_comparison_expression(self):
        left = self.parse_expression()

        while self.current_token.type in (
            TokenType.GREATER,
            TokenType.LESS,
            TokenType.GREATER_EQUAL,
            TokenType.LESS_EQUAL,
            TokenType.EQUAL,
            TokenType.NOT_EQUAL,
        ):
            op_token = self.current_token
            self.advance()

            if (
                op_token.type == TokenType.EQUAL and
                self.current_token.type == TokenType.EQUAL
            ):
                operator_value = "=="
                self.advance()
            else:
                operator_value = op_token.value

            right = self.parse_expression()
            left = ASTNode(
                NodeType.BINARY_OPERATION, value=operator_value, children=[left, right]
            )

        return left

    def parse_return_statement(self):
        self.expect(TokenType.CELI)
        # If there is an expression after celi, parse it
        if self.current_token.type != TokenType.SEMICOLON:
            expr = self.parse_expression()
            return ASTNode(NodeType.RETURN_STATEMENT, children=[expr])
        # Otherwise, it's a return with no value
        return ASTNode(NodeType.RETURN_STATEMENT)

    def create_node(self, node_type, value=None, children=None):
        """Create an AST node with current token's line and position information"""
        line = getattr(self.current_token, 'line', None)
        position = getattr(self.current_token, 'position', None)
        return ASTNode(node_type, value=value, children=children, line=line, position=position)
