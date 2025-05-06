import os

from src.core.ast import ASTNode, NodeType
from src.core.tokens import TokenType
from src.stdlib.builtins import (
    get_builtin_functions,
    get_list_methods,
    get_object_methods,
    SoplangBuiltins,
)
from src.utils.errors import (
    BreakSignal,
    ContinueSignal,
    ImportError,
    ReturnSignal,
    RuntimeError,
    TypeError,
)


class Interpreter:
    def __init__(self):
        self.variables = {}  # Global variables
        self.variable_types = {}  # Store static types
        self.functions = get_builtin_functions()  # Built-in functions
        self.list_methods = get_list_methods()
        self.object_methods = get_object_methods()
        self.classes = {}  # Store class definitions
        self.call_stack = []  # Track function calls if needed

    def interpret(self, root):
        if root.type != NodeType.PROGRAM:
            raise RuntimeError("invalid_syntax", detail="Root node must be PROGRAM")
        for statement in root.children:
            try:
                result = self.execute(statement)
            except BreakSignal:
                raise RuntimeError("break_outside_loop")
            except ContinueSignal:
                raise RuntimeError("continue_outside_loop")
            except ReturnSignal:
                raise RuntimeError("return_outside_function")

    # -----------------------------
    #  Execute Statement
    # -----------------------------
    def execute(self, node):
        if node.type == NodeType.PROGRAM:
            for child in node.children:
                self.execute(child)
        elif node.type == NodeType.VARIABLE_DECLARATION:
            self.execute_var_declaration(node)
        elif node.type == NodeType.FUNCTION_DEFINITION:
            self.define_function(node)
        elif node.type == NodeType.FUNCTION_CALL:
            return self.execute_function_call(node)
        elif node.type == NodeType.IF_STATEMENT:
            return self.execute_if_statement(node)
        elif node.type == NodeType.LOOP_STATEMENT:
            return self.execute_loop_statement(node)
        elif node.type == NodeType.WHILE_STATEMENT:
            return self.execute_while_statement(node)
        elif node.type == NodeType.BREAK_STATEMENT:
            raise BreakSignal()
        elif node.type == NodeType.CONTINUE_STATEMENT:
            raise ContinueSignal()
        elif node.type == NodeType.RETURN_STATEMENT:
            if node.children:
                # Return the evaluated expression
                return_value = self.evaluate(node.children[0])
                raise ReturnSignal(return_value)
            else:
                # Return with no value
                raise ReturnSignal(None)
        elif node.type == NodeType.BLOCK:
            return self.execute_block(node)
        elif node.type == NodeType.IMPORT_STATEMENT:
            return self.execute_import_statement(node)
        elif node.type == NodeType.TRY_CATCH:
            return self.execute_try_catch(node)
        elif node.type == NodeType.CLASS_DEFINITION:
            return self.execute_class_definition(node)
        elif node.type == NodeType.ASSIGNMENT:
            return self.execute_assignment(node)
        # Handle expressions that might be statements
        elif node.type in (NodeType.BINARY_OPERATION, NodeType.UNARY_OPERATION,
                           NodeType.PROPERTY_ACCESS, NodeType.METHOD_CALL,
                           NodeType.INDEX_ACCESS, NodeType.IDENTIFIER,
                           NodeType.LITERAL):
            # Just evaluate the expression and discard the result
            self.evaluate(node)
            return None
        else:
            raise RuntimeError("unknown_node_type", node_type=node.type)

    # -----------------------------
    #  Variable Declaration
    # -----------------------------
    def execute_var_declaration(self, node):
        var_name = node.value
        var_value = self.evaluate(node.children[0])  # expression

        # If this is a static type declaration, store the type and enforce it
        if hasattr(node, "var_type") and node.var_type is not None:
            # Store the type information
            self.variable_types[var_name] = node.var_type

            # Validate the value against the declared type
            self.validate_type(var_name, var_value, node.var_type, node)

        self.variables[var_name] = var_value
        return var_value

    # -----------------------------
    #  Type validation
    # -----------------------------
    def validate_type(self, var_name, value, expected_type, node=None):
        """Validates that the value matches the expected static type"""
        # Get line and position from node if available
        line = getattr(node, 'line', None)
        position = getattr(node, 'position', None)

        if expected_type == TokenType.TIRO:
            if not isinstance(value, (int, float)):
                raise TypeError("type_mismatch",
                                var_name=var_name,
                                value=value,
                                expected_type="tiro",
                                line=line,
                                position=position)

        elif expected_type == TokenType.QORAAL:
            if not isinstance(value, str):
                raise TypeError("type_mismatch",
                                var_name=var_name,
                                value=value,
                                expected_type="qoraal",
                                line=line,
                                position=position)

        elif expected_type == TokenType.BOOL:
            if not isinstance(value, bool):
                raise TypeError("type_mismatch",
                                var_name=var_name,
                                value=value,
                                expected_type="bool",
                                line=line,
                                position=position)

        elif expected_type == TokenType.LIIS:
            if not isinstance(value, list):
                raise TypeError("type_mismatch",
                                var_name=var_name,
                                value=value,
                                expected_type="liis",
                                line=line,
                                position=position)

        elif expected_type == TokenType.SHEY:
            if not isinstance(value, dict):
                raise TypeError("type_mismatch",
                                var_name=var_name,
                                value=value,
                                expected_type="shey",
                                line=line,
                                position=position)

    # -----------------------------
    #  Variable Assignment
    # -----------------------------
    def assign_variable(self, var_name, value, line=None, position=None):
        """Assign a value to a variable, with type checking if it's statically typed"""
        if var_name not in self.variables:
            raise RuntimeError("undefined_variable", name=var_name,
                               line=line, position=position)

        # If it's a statically typed variable, validate the type
        if var_name in self.variable_types:
            # Create a temporary node with line/position for validation
            temp_node = ASTNode(NodeType.ASSIGNMENT, line=line, position=position)
            self.validate_type(
                var_name, value, self.variable_types[var_name], temp_node)

        self.variables[var_name] = value
        return value

    # -----------------------------
    #  Assignment Statement
    # -----------------------------
    def execute_assignment(self, node):
        """Execute an assignment node (identifier = expression)"""
        target = node.children[0]  # Target of assignment
        value = self.evaluate(node.children[1])

        # Get line and position from node
        line = getattr(node, 'line', None)
        position = getattr(node, 'position', None)

        # Simple variable assignment
        if target.type == NodeType.IDENTIFIER:
            return self.assign_variable(target.value, value, line, position)

        # Property assignment (obj.prop = value)
        elif target.type == NodeType.PROPERTY_ACCESS:
            obj = self.evaluate(target.children[0])
            if not isinstance(obj, dict):
                raise TypeError("property_access", prop=target.value,
                                line=line, position=position)

            prop_name = target.value
            obj[prop_name] = value
            return value

        # Index assignment (arr[idx] = value)
        elif target.type == NodeType.INDEX_ACCESS:
            arr = self.evaluate(target.children[0])
            if not isinstance(arr, list):
                raise TypeError("index_access", line=line, position=position)

            idx = self.evaluate(target.children[1])
            if not isinstance(idx, (int, float)) or int(idx) != idx:
                raise TypeError("invalid_operand", operator="[]",
                                type_name="tiro", line=line, position=position)

            idx = int(idx)
            if idx < 0 or idx >= len(arr):
                raise RuntimeError("index_out_of_range", index=idx,
                                   line=line, position=position)

            arr[idx] = value
            return value

        else:
            raise RuntimeError(
                "invalid_syntax", detail=f"Invalid assignment target: {target.type}",
                line=line, position=position)

    # -----------------------------
    #  Function Call
    # -----------------------------
    def execute_function_call(self, node):
        func_name = node.value
        args = [self.evaluate(arg) for arg in node.children]

        # Check if it's a built-in function
        if func_name in self.functions:
            if callable(self.functions[func_name]):
                # Built-in function (Python function)
                return self.functions[func_name](*args)
            else:
                # User-defined function (Soplang function)
                user_func = self.functions[func_name]
                params = user_func["params"]
                body = user_func["body"]

                # Create a new scope for function execution
                old_vars = self.variables.copy()

                # Bind arguments to parameters
                for i, param in enumerate(params):
                    if i < len(args):
                        self.variables[param] = args[i]
                    else:
                        # Default to None if not enough arguments
                        self.variables[param] = None

                # Execute function body
                result = None
                try:
                    for statement in body:
                        result = self.execute(statement)
                except ReturnSignal as ret:
                    result = ret.value

                # Restore the previous scope
                self.variables = old_vars

                return result

        # Check if it's a method call on an object or list
        elif "." in func_name:
            obj_name, method_name = func_name.split(".", 1)
            obj = self.variables.get(obj_name)

            if obj is None:
                raise RuntimeError("undefined_variable", name=obj_name)

            if isinstance(obj, list) and method_name in self.list_methods:
                # Call list method
                return self.list_methods[method_name](obj, *args)
            elif isinstance(obj, dict) and method_name in self.object_methods:
                # Call object method
                return self.object_methods[method_name](obj, *args)
            else:
                raise RuntimeError(
                    "method_not_found", method_name=method_name, type_name=SoplangBuiltins.nuuc(obj))
        else:
            raise RuntimeError("undefined_function", name=func_name)

    # -----------------------------
    #  If Statement
    # -----------------------------
    def execute_if_statement(self, node):
        # children[0]: condition
        # rest: if-body, elif-blocks, else-block
        condition_value = self.evaluate(node.children[0])
        index = 1

        if condition_value:
            # Execute statements until we reach an IF_STATEMENT or BLOCK
            while index < len(node.children):
                child = node.children[index]
                if child.type in (NodeType.IF_STATEMENT, NodeType.BLOCK):
                    break
                self.execute(child)
                index += 1
            return

        # If the main if is false, check elif
        executed_elif = False
        while index < len(node.children):
            child = node.children[index]
            if child.type == NodeType.IF_STATEMENT:
                # Found an elif block
                elif_condition = self.evaluate(child.children[0])
                if elif_condition:
                    # Execute the elif body and exit
                    for stmt in child.children[1:]:
                        if stmt.type in (NodeType.IF_STATEMENT, NodeType.BLOCK):
                            break
                        self.execute(stmt)
                    executed_elif = True
                    break
            elif child.type == NodeType.BLOCK:
                # Found the else block, execute unconditionally
                for stmt in child.children:
                    self.execute(stmt)
                return
            index += 1

    # -----------------------------
    #  Loop Statement (for/kuceli)
    # -----------------------------
    def execute_loop_statement(self, node):
        # node.value = loop_var name
        # node.children[0] = start
        # node.children[1] = end
        # node.children[2] = step (optional)
        # node.children[2...] or node.children[3...] = body
        loop_var = node.value
        start_value = self.evaluate(node.children[0])
        end_value = self.evaluate(node.children[1])

        # Check if there's a step parameter
        step_value = 1  # Default step
        body_start_index = 2

        # If we have at least 3 children before the body, the 3rd one is the step
        if len(node.children) > 2 and (
            node.children[2].type == NodeType.LITERAL or
            node.children[2].type == NodeType.IDENTIFIER or
            node.children[2].type == NodeType.BINARY_OPERATION
        ):
            step_value = self.evaluate(node.children[2])
            body_start_index = 3

        # Ensure all values are numbers
        if (
            not isinstance(start_value, (int, float)) or
            not isinstance(end_value, (int, float)) or
            not isinstance(step_value, (int, float))
        ):
            raise TypeError("invalid_for_loop")

        i = start_value
        # Check step direction to determine the appropriate comparison
        if step_value > 0:

            def condition():
                return i <= end_value

        else:

            def condition():
                return i >= end_value

        while condition():
            # Set the loop variable in scope
            self.variables[loop_var] = i

            # Execute the body
            try:
                for stmt_index in range(body_start_index, len(node.children)):
                    self.execute(node.children[stmt_index])
            except BreakSignal:
                break  # Exit the loop
            except ContinueSignal:
                pass  # Skip to the next iteration

            i += step_value

    # -----------------------------
    #  While Statement
    # -----------------------------
    def execute_while_statement(self, node):
        # node.children[0] = condition
        # node.children[1..] = body

        while self.evaluate(node.children[0]):
            # Execute the body
            try:
                for stmt_index in range(1, len(node.children)):
                    self.execute(node.children[stmt_index])
            except BreakSignal:
                break  # Exit the loop
            except ContinueSignal:
                pass  # Skip to the next iteration

    # -----------------------------
    #  Block
    # -----------------------------
    def execute_block(self, node):
        # Just execute each child
        for child in node.children:
            self.execute(child)

    # -----------------------------
    #  Try-Catch
    # -----------------------------
    def execute_try_catch(self, node):
        # node.children[0] = try block (BLOCK)
        # node.children[1] = catch block (BLOCK)
        # node.value = error variable name
        error_var = node.value

        try:
            self.execute_block(node.children[0])
        except Exception as e:
            # Store the error in the variable and execute the catch block
            self.variables[error_var] = str(e)
            self.execute_block(node.children[1])

    # -----------------------------
    #  Import Statement
    # -----------------------------
    def execute_import_statement(self, node):
        filename = node.value

        try:
            with open(filename, "r") as f:
                code = f.read()

            # Import the modules only when needed
            from src.core.lexer import Lexer
            from src.core.parser import Parser

            lexer = Lexer(code)
            tokens = lexer.tokenize()
            parser = Parser(tokens)
            ast = parser.parse()

            # Execute the imported program
            for stmt in ast.children:
                self.execute(stmt)

        except FileNotFoundError:
            raise ImportError("file_not_found", module=filename)
        except Exception as e:
            raise ImportError("import_error", filename=filename, error=str(e))

    # -----------------------------
    #  Class Definition
    # -----------------------------
    def execute_class_definition(self, node):
        if isinstance(node.value, tuple):
            class_name, parent_name = node.value
        else:
            class_name = node.value
            parent_name = None

        # Validate parent class exists if specified
        if parent_name and parent_name not in self.classes:
            raise RuntimeError("parent_class_not_found", parent_name=parent_name)

        # Create the class definition
        class_def = {
            "name": class_name,
            "parent": parent_name,
            "methods": {},
            "fields": {},
        }

        # Process class body
        for child in node.children:
            if child.type == NodeType.FUNCTION_DEFINITION:
                method_name = child.value
                class_def["methods"][method_name] = child
            elif child.type == NodeType.VARIABLE_DECLARATION:
                field_name = child.value
                field_value = self.evaluate(child.children[0])
                class_def["fields"][field_name] = field_value
            else:
                # Execute any statements in the class (like qor())
                self.execute(child)

        # Store the class definition
        self.classes[class_name] = class_def

        return class_def

    # -----------------------------
    #  Evaluate expressions
    # -----------------------------
    def evaluate(self, node):
        # Get line and position from node
        line = getattr(node, 'line', None)
        position = getattr(node, 'position', None)

        if node.type == NodeType.LITERAL:
            return node.value
        if node.type == NodeType.IDENTIFIER:
            if node.value in self.variables:
                return self.variables[node.value]
            else:
                raise RuntimeError("undefined_variable",
                                   name=node.value, line=line, position=position)
        if node.type == NodeType.BINARY_OPERATION:
            left_val = self.evaluate(node.children[0])
            right_val = self.evaluate(node.children[1])
            return self.apply_operator(node.value, left_val, right_val)
        if node.type == NodeType.UNARY_OPERATION:
            # For unary operations, only evaluate the operand
            operand_val = self.evaluate(node.children[0])
            # Handle different unary operators
            if node.value == "!":
                return not bool(operand_val)
            else:
                raise RuntimeError("unknown_operator",
                                   operator=node.value, line=line, position=position)
        if node.type == NodeType.LIST_LITERAL:
            # Evaluate each element in the list
            return [self.evaluate(element) for element in node.children]
        if node.type == NodeType.OBJECT_LITERAL:
            # Create a dictionary from key-value pairs
            obj = {}
            for prop in node.children:
                key = prop.value
                value = self.evaluate(prop.children[0])
                obj[key] = value
            return obj
        if node.type == NodeType.PROPERTY_ACCESS:
            # Evaluate the object expression
            obj = self.evaluate(node.children[0])
            if not isinstance(obj, dict):
                raise TypeError("property_access", prop=node.value,
                                line=line, position=position)

            # Get the property name and return the value
            prop_name = node.value
            if prop_name not in obj:
                raise RuntimeError("property_not_found",
                                   prop_name=prop_name, line=line, position=position)

            return obj[prop_name]
        if node.type == NodeType.METHOD_CALL:
            # Evaluate the object expression (first child)
            obj = self.evaluate(node.children[0])
            method_name = node.value

            # For built-in list methods
            if isinstance(obj, list) and method_name in self.list_methods:
                # Arguments start from the second child
                args = [self.evaluate(arg) for arg in node.children[1:]]
                return self.list_methods[method_name](obj, *args)

            # For built-in object methods
            elif isinstance(obj, dict) and method_name in self.object_methods:
                # Arguments start from the second child
                args = [self.evaluate(arg) for arg in node.children[1:]]
                return self.object_methods[method_name](obj, *args)

            # For user-defined object methods
            elif isinstance(obj, dict) and method_name in obj:
                if callable(obj[method_name]):
                    # Arguments start from the second child
                    args = [self.evaluate(arg) for arg in node.children[1:]]
                    return obj[method_name](*args)

            raise RuntimeError("method_not_found", method_name=method_name,
                               type_name=SoplangBuiltins.nuuc(obj))

        if node.type == NodeType.INDEX_ACCESS:
            # Evaluate the array expression
            arr = self.evaluate(node.children[0])
            if not isinstance(arr, list):
                raise TypeError("index_access", line=line, position=position)

            # Evaluate the index expression
            idx = self.evaluate(node.children[1])
            if not isinstance(idx, (int, float)) or int(idx) != idx:
                raise TypeError("invalid_operand", operator="[]",
                                type_name="tiro", line=line, position=position)

            idx = int(idx)
            if idx < 0 or idx >= len(arr):
                raise RuntimeError("index_out_of_range", index=idx,
                                   line=line, position=position)

            return arr[idx]
        if node.type == NodeType.FUNCTION_CALL:
            # Already handled in execute_function_call, but we need to support it here
            # for expressions that include function calls
            return self.execute_function_call(node)

        # Constructing new objects: e.g. 'cusub MyClass()'
        if node.type == NodeType.FUNCTION_CALL and node.value == "cusub":
            # children[0] = className, children[1..] = constructor args
            classNameNode = node.children[0]
            if classNameNode.type != NodeType.IDENTIFIER:
                raise RuntimeError(
                    "invalid_syntax", detail="Expected class name after 'cusub'")

            className = classNameNode.value
            if className not in self.classes:
                raise RuntimeError("parent_class_not_found", parent_name=className)

            # We won't do a full OOP system, just store as dict for now
            instance = {
                "__class__": className
                # Could store fields, call init, etc.
            }
            return instance

        raise RuntimeError("unknown_node_type", node_type=node.type,
                           line=line, position=position)

    def apply_operator(self, operator, left, right):
        """Apply an operator to two values."""
        if operator == "+":
            # Handle string concatenation
            if isinstance(left, str) or isinstance(right, str):
                # Import SoplangBuiltins for proper string conversion
                from src.stdlib.builtins import SoplangBuiltins

                # Use qoraal for proper string conversion (including booleans to been/run)
                return SoplangBuiltins.qoraal(left) + SoplangBuiltins.qoraal(right)
            return left + right
        elif operator == "-":
            return left - right
        elif operator == "*":
            return left * right
        elif operator == "/":
            if right == 0:
                raise RuntimeError("division_by_zero")
            return left / right
        elif operator == "%":
            if right == 0:
                raise RuntimeError("modulo_by_zero")
            return left % right
        elif operator == "==":
            return left == right
        elif operator == "!=":
            return left != right
        elif operator == ">":
            return left > right
        elif operator == "<":
            return left < right
        elif operator == ">=":
            return left >= right
        elif operator == "<=":
            return left <= right
        elif operator == "&&":
            return bool(left) and bool(right)
        elif operator == "||":
            return bool(left) or bool(right)
        else:
            raise RuntimeError("unknown_operator", operator=operator)

    # Define a function and store it in the functions dictionary
    def define_function(self, node):
        if node.type != NodeType.FUNCTION_DEFINITION:
            raise RuntimeError(
                "invalid_syntax", detail="Expected function definition node")

        # Extract function name and parameters
        func_name = node.value
        param_nodes = []
        body_nodes = []

        # Separate parameter nodes from body nodes
        for child in node.children:
            if child.type == NodeType.IDENTIFIER:
                param_nodes.append(child)
            else:
                body_nodes.append(child)

        # Store the function definition
        self.functions[func_name] = {
            "params": [param.value for param in param_nodes],
            "body": body_nodes,
        }

    def execute_method_call(self, node):
        # Get object
        obj = self.evaluate(node.children[0])

        # Get method name
        method_name = node.value

        # Try to find method in the appropriate registry
        method = None
        if isinstance(obj, dict):
            # Object methods
            if method_name in self.object_methods:
                method = self.object_methods[method_name]
        elif isinstance(obj, list):
            # List methods
            if method_name in self.list_methods:
                method = self.list_methods[method_name]
        else:
            # Could extend to other types of objects
            pass

        if method is None:
            if isinstance(obj, dict) or isinstance(obj, list):
                raise RuntimeError(
                    "method_not_found", method_name=method_name, type_name=SoplangBuiltins.nuuc(obj))
            else:
                raise TypeError("invalid_method", method=method_name,
                                type_name=SoplangBuiltins.nuuc(obj))

    def execute_list_method(self, method_name, obj, args):
        """Execute a list method"""
        if not isinstance(obj, list):
            raise TypeError("invalid_method", method=method_name,
                            type_name=SoplangBuiltins.nuuc(obj))

        if method_name not in self.list_methods:
            raise RuntimeError("method_not_found",
                               method_name=method_name, type_name="liis")

        method = self.list_methods[method_name]
        args.insert(0, obj)  # Insert the list as the first argument
        return method(*args)

    def execute_object_method(self, method_name, obj, args):
        """Execute an object method"""
        if not isinstance(obj, dict):
            raise TypeError("invalid_method", method=method_name,
                            type_name=SoplangBuiltins.nuuc(obj))

        if method_name not in self.object_methods:
            raise RuntimeError("method_not_found",
                               method_name=method_name, type_name="shey")

        method = self.object_methods[method_name]
        args.insert(0, obj)  # Insert the object as the first argument
        return method(*args)
