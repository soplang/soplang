import os
from ast import ASTNode, NodeType

class Interpreter:
    def __init__(self):
        self.variables = {}     # Global variables
        self.functions = {      # Built-in functions
            "qor": self.builtin_qor,
            "akhri": self.builtin_akhri
        }
        self.classes = {}       # Store class definitions
        self.call_stack = []    # Track function calls if needed

    def interpret(self, root):
        if root.type != NodeType.PROGRAM:
            raise RuntimeError("Root node must be PROGRAM.")
        for statement in root.children:
            result = self.execute(statement)
            # If needed, handle global returns

    # -----------------------------
    #  Execute Statement
    # -----------------------------
    def execute(self, node):
        if node.type == NodeType.VARIABLE_DECLARATION:
            return self.execute_var_declaration(node)
        elif node.type == NodeType.FUNCTION_CALL:
            return self.execute_function_call(node)
        elif node.type == NodeType.IF_STATEMENT:
            return self.execute_if_statement(node)
        elif node.type == NodeType.LOOP_STATEMENT:
            return self.execute_loop_statement(node)
        elif node.type == NodeType.BLOCK:
            return self.execute_block(node)
        elif node.type == NodeType.BREAK_STATEMENT:
            # Signal the loop to break
            raise BreakSignal()
        elif node.type == NodeType.CONTINUE_STATEMENT:
            # Signal the loop to continue
            raise ContinueSignal()
        elif node.type == NodeType.IMPORT_STATEMENT:
            return self.execute_import_statement(node)
        elif node.type == NodeType.TRY_CATCH:
            return self.execute_try_catch(node)
        elif node.type == NodeType.CLASS_DEFINITION:
            return self.execute_class_definition(node)
        else:
            raise RuntimeError(f"Unknown statement type: {node.type}")

    def execute_var_declaration(self, node):
        var_name = node.value
        var_value = self.evaluate(node.children[0])  # expression
        self.variables[var_name] = var_value
        return var_value

    # -----------------------------
    #  Function Call
    # -----------------------------
    def execute_function_call(self, node):
        func_name = node.value
        args = [self.evaluate(arg) for arg in node.children]

        # Built-in or user-defined?
        if func_name in self.functions:
            return self.functions[func_name](*args)
        else:
            # Could be user-defined if you store functions in self.variables
            raise RuntimeError(f"Undefined function: {func_name}")

    # Built-in: qor(message)
    def builtin_qor(self, msg):
        print(msg)

    # Built-in: akhri(prompt) -> input
    def builtin_akhri(self, prompt):
        return input(prompt)

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
                # This is an elif block
                elif_condition = self.evaluate(child.children[0])
                if elif_condition and not executed_elif:
                    # Execute elif body
                    ci = 1
                    while ci < len(child.children):
                        sub = child.children[ci]
                        if sub.type in (NodeType.IF_STATEMENT, NodeType.BLOCK):
                            break
                        self.execute(sub)
                        ci += 1
                    executed_elif = True
            elif child.type == NodeType.BLOCK:
                # This is the else block
                if not executed_elif:
                    self.execute_block(child)
                return
            index += 1

    # -----------------------------
    #  Loops
    # -----------------------------
    def execute_loop_statement(self, node):
        # node.value = loop_var name
        # node.children[0] = start
        # node.children[1] = end
        # node.children[2..] = body
        start_val = self.evaluate(node.children[0])
        end_val = self.evaluate(node.children[1])
        loop_var = node.value

        i = start_val
        while i <= end_val:
            self.variables[loop_var] = i
            try:
                for stmt in node.children[2:]:
                    self.execute(stmt)
            except BreakSignal:
                break
            except ContinueSignal:
                i += 1
                continue
            i += 1

    # -----------------------------
    #  Blocks
    # -----------------------------
    def execute_block(self, node):
        # Just execute each child
        for stmt in node.children:
            self.execute(stmt)

    # -----------------------------
    #  Try/Catch
    # -----------------------------
    def execute_try_catch(self, node):
        # node.children[0] = try block (BLOCK)
        # node.children[1] = catch block (BLOCK)
        # node.value = error variable name
        try:
            for stmt in node.children[0].children:
                self.execute(stmt)
        except Exception as e:
            # set error variable
            self.variables[node.value] = str(e)
            # run catch block
            for stmt in node.children[1].children:
                self.execute(stmt)

    # -----------------------------
    #  Import Statement
    # -----------------------------
    def execute_import_statement(self, node):
        filename = node.value  # e.g. "other.so"
        if not filename.endswith(".so"):
            raise RuntimeError("Soplang files must end with .so")

        if not os.path.isfile(filename):
            raise RuntimeError(f"File not found: {filename}")

        # Load the file, tokenize, parse, interpret
        from lexer import Lexer
        from parser import Parser

        with open(filename, "r", encoding="utf-8") as f:
            code = f.read()

        lx = Lexer(code)
        tokens = lx.tokenize()
        from ast import ASTNode, NodeType  # to avoid circular import
        ps = Parser(tokens)
        imported_ast = ps.parse()

        self.interpret(imported_ast)

    # -----------------------------
    #  Class Definition
    # -----------------------------
    def execute_class_definition(self, node):
        # node.value -> className or (className, parentName)
        # node.children -> body statements
        if isinstance(node.value, tuple):
            className, parentName = node.value
        else:
            className = node.value
            parentName = None

        # store the class definition in self.classes
        self.classes[className] = {
            "parent": parentName,
            "body": node.children
        }

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
        if operator == "+": return left + right
        if operator == "-": return left - right
        if operator == "*": return left * right
        if operator == "/": return left / right
        if operator == ">": return left > right
        if operator == "<": return left < right
        if operator == ">=": return left >= right
        if operator == "<=": return left <= right
        if operator == "==": return left == right
        if operator == "!=": return left != right
        raise RuntimeError(f"Unknown operator: {operator}")


# -----------------------------
#  Custom Exceptions for Break/Continue
# -----------------------------
class BreakSignal(Exception):
    pass

class ContinueSignal(Exception):
    pass