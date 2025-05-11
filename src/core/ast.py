from enum import Enum


class NodeType(Enum):
    PROGRAM = "PROGRAM"
    VARIABLE_DECLARATION = "VARIABLE_DECLARATION"
    FUNCTION_DEFINITION = "FUNCTION_DEFINITION"
    FUNCTION_CALL = "FUNCTION_CALL"
    IF_STATEMENT = "IF_STATEMENT"
    SWITCH_STATEMENT = "SWITCH_STATEMENT"
    LOOP_STATEMENT = "LOOP_STATEMENT"
    WHILE_STATEMENT = "WHILE_STATEMENT"
    BLOCK = "BLOCK"
    BINARY_OPERATION = "BINARY_OPERATION"
    UNARY_OPERATION = "UNARY_OPERATION"  # For unary operators like ! (NOT)
    LITERAL = "LITERAL"
    IDENTIFIER = "IDENTIFIER"
    CLASS_DEFINITION = "CLASS_DEFINITION"
    IMPORT_STATEMENT = "IMPORT_STATEMENT"
    TRY_CATCH = "TRY_CATCH"
    BREAK_STATEMENT = "BREAK_STATEMENT"
    CONTINUE_STATEMENT = "CONTINUE_STATEMENT"
    RETURN_STATEMENT = "RETURN_STATEMENT"

    # New node types for lists and objects
    LIST_LITERAL = "LIST_LITERAL"  # For list creation [1, 2, 3]
    OBJECT_LITERAL = "OBJECT_LITERAL"  # For object creation {a: 1, b: 2}
    PROPERTY_ACCESS = "PROPERTY_ACCESS"  # For object.property
    METHOD_CALL = "METHOD_CALL"  # For object.method()
    INDEX_ACCESS = "INDEX_ACCESS"  # For list[index]
    # For explicit assignment (separate from declaration)
    ASSIGNMENT = "ASSIGNMENT"


class ASTNode:
    def __init__(self, type_, value=None, children=None, line=None, position=None):
        self.type = type_
        self.value = value
        self.children = children if children else []
        self.var_type = None  # For static typing
        self.is_constant = False  # For constant variables (madoor)
        self.line = line  # Store line number
        self.position = position  # Store position/column number

    def __repr__(self):
        type_info = ""
        if (
            self.type == NodeType.VARIABLE_DECLARATION and
            hasattr(self, "var_type") and
            self.var_type is not None
        ):
            type_info = f", var_type={self.var_type}"
            if self.is_constant:
                type_info += ", constant=True"
        elif self.type == NodeType.VARIABLE_DECLARATION and self.is_constant:
            type_info = ", constant=True"

        line_pos = ""
        if self.line is not None:
            line_pos = f", line={self.line}"
            if self.position is not None:
                line_pos += f", pos={self.position}"
        return f"ASTNode({self.type}, value={self.value}{type_info}{line_pos}, children={self.children})"
