import os
from src.core.ast import ASTNode, NodeType
from src.core.tokens import TokenType
from src.utils.errors import RuntimeError, TypeError, BreakSignal, ContinueSignal, ReturnSignal, ImportError
from src.stdlib.builtins import get_builtin_functions, get_list_methods, get_object_methods


class Interpreter:
    def __init__(self):
        self.variables = {}             # Global variables
        self.variable_types = {}        # Store static types
        self.functions = get_builtin_functions()  # Built-in functions
        self.list_methods = get_list_methods()
        self.object_methods = get_object_methods()
        self.classes = {}               # Store class definitions
        self.call_stack = []            # Track function calls if needed

    def interpret(self, root):
        if root.type != NodeType.PROGRAM:
            raise RuntimeError("Root node must be PROGRAM.")
        for statement in root.children:
            try:
                result = self.execute(statement)
            except (BreakSignal, ContinueSignal):
                raise RuntimeError(
                    "Jooji ama sii_wad waa in ay ku jiraan xalqad.")
            except ReturnSignal:
                raise RuntimeError("Soo_celi waa in ay ku jirto howl.")
            # If needed, handle global returns

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
        else:
            raise RuntimeError(f"Unknown statement type: {node.type}")

    # -----------------------------
    #  Variable Declaration
    # -----------------------------
    def execute_var_declaration(self, node):
        var_name = node.value
        var_value = self.evaluate(node.children[0])  # expression

        # If this is a static type declaration, store the type and enforce it
        if hasattr(node, 'var_type') and node.var_type is not None:
            # Store the type information
            self.variable_types[var_name] = node.var_type

            # Validate the value against the declared type
            self.validate_type(var_name, var_value, node.var_type)

        self.variables[var_name] = var_value
        return var_value

    # -----------------------------
    #  Type validation
    # -----------------------------
    def validate_type(self, var_name, value, expected_type):
        """Validates that the value matches the expected static type"""

        if expected_type == TokenType.TIRO:
            if not isinstance(value, (int, float)):
                raise TypeError(
                    f"'{var_name}' waa tiro laakin qiimaheeda '{value}' ma ahan tiro")

        elif expected_type == TokenType.QORAAL:
            if not isinstance(value, str):
                raise TypeError(
                    f"'{var_name}' waa qoraal laakin qiimaheeda '{value}' ma ahan qoraal")

        elif expected_type == TokenType.LABADARAN:
            if not isinstance(value, bool):
                raise TypeError(
                    f"'{var_name}' waa labadaran laakin qiimaheeda '{value}' ma ahan labadaran")

        elif expected_type == TokenType.LIIS:
            if not isinstance(value, list):
                raise TypeError(
                    f"'{var_name}' waa liis laakin qiimaheeda '{value}' ma ahan liis")

        elif expected_type == TokenType.SHEY:
            if not isinstance(value, dict):
                raise TypeError(
                    f"'{var_name}' waa shey laakin qiimaheeda '{value}' ma ahan shey")

    # -----------------------------
    #  Variable Assignment
    # -----------------------------
    def assign_variable(self, var_name, value):
        """Assign a value to a variable, with type checking if it's statically typed"""
        if var_name not in self.variables:
            raise RuntimeError(f"Ma jiro doorsame '{var_name}'")

        # If it's a statically typed variable, validate the type
        if var_name in self.variable_types:
            self.validate_type(var_name, value, self.variable_types[var_name])

        self.variables[var_name] = value
        return value

    # -----------------------------
    #  Assignment Statement
    # -----------------------------
    def execute_assignment(self, node):
        """Execute an assignment node (identifier = expression)"""
        target = node.children[0]  # Target of assignment
        value = self.evaluate(node.children[1])  # Expression to assign

        # Simple variable assignment
        if target.type == NodeType.IDENTIFIER:
            return self.assign_variable(target.value, value)

        # Property assignment (obj.prop = value)
        elif target.type == NodeType.PROPERTY_ACCESS:
            obj = self.evaluate(target.children[0])
            if not isinstance(obj, dict):
                raise TypeError("Cannot set property of non-object value")

            prop_name = target.value
            obj[prop_name] = value
            return value

        # Index assignment (arr[idx] = value)
        elif target.type == NodeType.INDEX_ACCESS:
            arr = self.evaluate(target.children[0])
            if not isinstance(arr, list):
                raise TypeError("Cannot set index of non-list value")

            idx = self.evaluate(target.children[1])
            if not isinstance(idx, (int, float)) or int(idx) != idx:
                raise TypeError("List index must be an integer")

            idx = int(idx)
            if idx < 0 or idx >= len(arr):
                raise RuntimeError(f"List index out of range: {idx}")

            arr[idx] = value
            return value

        else:
            raise RuntimeError(f"Invalid assignment target: {target.type}")

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
                params = user_func['params']
                body = user_func['body']

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
                raise RuntimeError(f"Object '{obj_name}' is not defined")

            if isinstance(obj, list) and method_name in self.list_methods:
                # Call list method
                return self.list_methods[method_name](obj, *args)
            elif isinstance(obj, dict) and method_name in self.object_methods:
                # Call object method
                return self.object_methods[method_name](obj, *args)
            else:
                raise RuntimeError(
                    f"Method '{method_name}' not found on {type(obj).__name__}")
        else:
            raise RuntimeError(f"Function '{func_name}' is not defined")

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
    #  Loop Statement (for/ku_celi)
    # -----------------------------
    def execute_loop_statement(self, node):
        # node.value = loop_var name
        # node.children[0] = start
        # node.children[1] = end
        # node.children[2..] = body
        loop_var = node.value
        start_value = self.evaluate(node.children[0])
        end_value = self.evaluate(node.children[1])

        # Ensure start and end are numbers
        if not isinstance(start_value, (int, float)) or not isinstance(end_value, (int, float)):
            raise TypeError(
                "Ku_celi billowga iyo dhamaadka waa in ay yihiin tiro")

        i = start_value
        while i <= end_value:
            # Set the loop variable in scope
            self.variables[loop_var] = i

            # Execute the body
            try:
                for stmt_index in range(2, len(node.children)):
                    self.execute(node.children[stmt_index])
            except BreakSignal:
                break  # Exit the loop
            except ContinueSignal:
                pass  # Skip to the next iteration

            i += 1

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
            with open(filename, 'r') as f:
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
            raise ImportError(f"{filename}")
        except Exception as e:
            raise RuntimeError(
                f"Qalad baa ka jira file-ka {filename}: {str(e)}")

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
            raise RuntimeError(f"Fasalka waalidka '{parent_name}' ma jiro")

        # Create the class definition
        class_def = {
            'name': class_name,
            'parent': parent_name,
            'methods': {},
            'fields': {}
        }

        # Process class body
        for child in node.children:
            if child.type == NodeType.FUNCTION_DEFINITION:
                method_name = child.value
                class_def['methods'][method_name] = child
            elif child.type == NodeType.VARIABLE_DECLARATION:
                field_name = child.value
                field_value = self.evaluate(child.children[0])
                class_def['fields'][field_name] = field_value
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
        if node.type == NodeType.LITERAL:
            return node.value
        if node.type == NodeType.IDENTIFIER:
            if node.value in self.variables:
                return self.variables[node.value]
            else:
                raise RuntimeError(f"Undefined variable: {node.value}")
        if node.type == NodeType.BINARY_OPERATION:
            left_val = self.evaluate(node.children[0])
            right_val = self.evaluate(node.children[1])
            return self.apply_operator(node.value, left_val, right_val)
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
                raise TypeError(
                    f"Cannot access property '{node.value}' of non-object value")

            # Get the property name and return the value
            prop_name = node.value
            if prop_name not in obj:
                raise RuntimeError(
                    f"Property '{prop_name}' does not exist on object")

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

            raise RuntimeError(
                f"Method '{method_name}' does not exist on {type(obj).__name__}")
        if node.type == NodeType.INDEX_ACCESS:
            # Evaluate the array expression
            arr = self.evaluate(node.children[0])
            if not isinstance(arr, list):
                raise TypeError("Cannot access index of non-list value")

            # Evaluate the index expression
            idx = self.evaluate(node.children[1])
            if not isinstance(idx, (int, float)) or int(idx) != idx:
                raise TypeError("List index must be an integer")

            idx = int(idx)
            if idx < 0 or idx >= len(arr):
                raise RuntimeError(f"List index out of range: {idx}")

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
                raise RuntimeError("Expected class name after 'cusub'")

            className = classNameNode.value
            if className not in self.classes:
                raise RuntimeError(f"Class not defined: {className}")

            # We won't do a full OOP system, just store as dict for now
            instance = {
                "__class__": className
                # Could store fields, call init, etc.
            }
            return instance

        raise RuntimeError(f"Cannot evaluate node type: {node.type}")

    def apply_operator(self, operator, left, right):
        """Apply an operator to two values."""
        if operator == "+":
            # Handle string concatenation
            if isinstance(left, str) or isinstance(right, str):
                return str(left) + str(right)
            return left + right
        elif operator == "-":
            return left - right
        elif operator == "*":
            return left * right
        elif operator == "/":
            if right == 0:
                raise RuntimeError("Ma suurtogali karto qeybinta eber.")
            return left / right
        elif operator == "%":
            if right == 0:
                raise RuntimeError(
                    "Ma suurtogali karto modulo eber.")
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
            raise RuntimeError(f"Unknown operator: {operator}")

    # Define a function and store it in the functions dictionary
    def define_function(self, node):
        if node.type != NodeType.FUNCTION_DEFINITION:
            raise RuntimeError("Expected function definition node.")

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
            'params': [param.value for param in param_nodes],
            'body': body_nodes
        }
