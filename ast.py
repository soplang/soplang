from enum import Enum

class NodeType(Enum):
    PROGRAM = "PROGRAM"
    VARIABLE_DECLARATION = "VARIABLE_DECLARATION"
    FUNCTION_DEFINITION = "FUNCTION_DEFINITION"
    FUNCTION_CALL = "FUNCTION_CALL"
    IF_STATEMENT = "IF_STATEMENT"
    LOOP_STATEMENT = "LOOP_STATEMENT"
    BLOCK = "BLOCK"
    BINARY_OPERATION = "BINARY_OPERATION"
    LITERAL = "LITERAL"
    IDENTIFIER = "IDENTIFIER"
    CLASS_DEFINITION = "CLASS_DEFINITION"
    IMPORT_STATEMENT = "IMPORT_STATEMENT"
    TRY_CATCH = "TRY_CATCH"
    BREAK_STATEMENT = "BREAK_STATEMENT"
    CONTINUE_STATEMENT = "CONTINUE_STATEMENT"

class ASTNode:
    def __init__(self, type_, value=None, children=None):
        self.type = type_
        self.value = value
        self.children = children if children else []

    def __repr__(self):
        return f"ASTNode({self.type}, value={self.value}, children={self.children})"
