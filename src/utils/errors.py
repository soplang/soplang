class SoplangError(Exception):
    """Base class for all Soplang errors."""

    pass


class ErrorMessageManager:
    """Centralized manager for all Soplang error messages in Somali.

    This class stores all error message templates in Somali and
    provides methods to format them with the appropriate details.
    """

    # Error type prefixes
    ERROR_PREFIXES = {
        "lexer": "Khalad lexer (Lexer Error)",
        "parser": "Khalad parser (Parser Error)",
        "type": "Khalad nuuc (Type Error)",
        "runtime": "Khalad runtime (Runtime Error)",
        "import": "Khalad import (Import Error)",
    }

    # Lexer errors
    LEXER_ERRORS = {
        "unexpected_char": "Xaraf aan la filayn (Unexpected character): {char}",
        "unterminated_string": "Qoraal aan la dhammaystirin (Unterminated string)",
        "unterminated_comment": "Faallo aan la dhammaystirin (Unterminated comment)",
    }

    # Parser errors
    PARSER_ERRORS = {
        "expected_token": "Waxaa la filayay (Expected) {expected}, laakiin waxaa la helay (found) {found}",
        "unexpected_token": "Calaamad aan la filayn (Unexpected token): {token}",
        "invalid_syntax": "Qoraalka syntax-kiisa waa khalad (Invalid syntax): {detail}",
        "missing_paren": "Waxaa ka maqan hal ')' (Missing closing parenthesis)",
        "missing_brace": "Waxaa ka maqan hal '}' (Missing closing brace)",
        "missing_bracket": "Waxaa ka maqan hal ']' (Missing closing bracket)",
    }

    # Type errors
    TYPE_ERRORS = {
        "type_mismatch": "'{var_name}' waa {expected_type} laakin qiimaheeda '{value}' ma ahan {expected_type} (Type mismatch)",
        "cannot_convert": "'{value}' ma badali karo {target_type} (Cannot convert to this type)",
        "invalid_operand": "Ma isticmaali karo '{operator}' oo ku shaqeeya {type_name} (Invalid operand for this operator)",
        "property_access": "Ma heli karo astaanta '{prop}' ee qiimaha aan ahayn shey (Cannot access property on non-object)",
        "index_access": "Ma heli karo tirooyinka ee qiimaha aan ahayn liis (Cannot use index on non-list)",
        "invalid_method": "Ma wici karo habka '{method}' ee qiimaha {type_name} (Cannot call method on this type)",
    }

    # Runtime errors
    RUNTIME_ERRORS = {
        "undefined_variable": "Doorsame aan la qeexin (Undefined variable): '{name}'",
        "undefined_function": "Howl aan la qeexin (Undefined function): '{name}'",
        "division_by_zero": "Ma suurtogali karto qeybinta eber (Division by zero)",
        "modulo_by_zero": "Ma suurtogali karto modulo eber (Modulo by zero)",
        "index_out_of_range": "Tirada fihris-ku waa ka baxsan xadka (Index out of range): {index}",
        "property_not_found": "Astaanta '{prop_name}' kuma jirto sheyga (Property not found on object)",
        "method_not_found": "Habka '{method_name}' kuma jirto {type_name} (Method not found on object)",
        "missing_argument": "Howsha '{func_name}' waxay u baahan tahay {expected} dood, laakiin waxaa la siiyay {provided} (Missing argument)",
        "parent_class_not_found": "Fasalka waalidka '{parent_name}' ma jiro (Parent class not found)",
        "break_outside_loop": "Jooji waa in ay ku jiraan xalqad (Break outside loop)",
        "continue_outside_loop": "Sii_wad waa in ay ku jiraan xalqad (Continue outside loop)",
        "return_outside_function": "Soo_celi waa in ay ku jirto howl (Return outside function)",
        "invalid_for_loop": "Ku_celi billowga, dhamaadka iyo tallaabada waa in ay yihiin tiro (Invalid for loop parameters)",
        "unknown_node_type": "Nooca cladka aan la aqoon (Unknown node type): {node_type}",
        "unknown_operator": "Hawl-gal aan la aqoon (Unknown operator): {operator}",
    }

    # Import errors
    IMPORT_ERRORS = {
        "file_not_found": "Faylka '{module}' ma helin (File not found)",
        "import_error": "Qalad baa ka jira file-ka {filename}: {error} (Error in imported file)",
    }

    @classmethod
    def format_error(cls, error_type, message, line=None, position=None):
        """Format an error message.

        Args:
            error_type (str): The type of error (lexer, parser, type, runtime).
            message (str): The error message.
            line (int, optional): The line number where the error occurred.
            position (int, optional): The position in the line where the error occurred.

        Returns:
            str: The formatted error message.
        """
        if error_type not in cls.ERROR_PREFIXES:
            error_type = "runtime"

        error_prefix = cls.ERROR_PREFIXES[error_type]

        if line is not None and position is not None:
            return (
                f"{error_prefix}: {message} ee sadar (line) {line}, "
                f"goobta (position) {position}"
            )
        else:
            return f"{error_prefix}: {message}"

    @classmethod
    def get_lexer_error(cls, error_code, **kwargs):
        """Get a formatted lexer error message."""
        if error_code in cls.LEXER_ERRORS:
            message = cls.LEXER_ERRORS[error_code]
            return cls.format_error("lexer", message, **kwargs)
        return f"Khalad markii loo qaybinayay: {error_code}"

    @classmethod
    def get_parser_error(cls, error_code, **kwargs):
        """Get a formatted parser error message."""
        if error_code in cls.PARSER_ERRORS:
            message = cls.PARSER_ERRORS[error_code]
            return cls.format_error("parser", message, **kwargs)
        return f"Khalad markii la falanqaynayay: {error_code}"

    @classmethod
    def get_type_error(cls, error_code, **kwargs):
        """Get a formatted type error message."""
        if error_code in cls.TYPE_ERRORS:
            message = cls.TYPE_ERRORS[error_code]
            return cls.format_error("type", message, **kwargs)
        return f"Khalad nuuca ah: {error_code}"

    @classmethod
    def get_runtime_error(cls, error_code, **kwargs):
        """Get a formatted runtime error message."""
        if error_code in cls.RUNTIME_ERRORS:
            message = cls.RUNTIME_ERRORS[error_code]
            return cls.format_error("runtime", message, **kwargs)
        return f"Khalad fulinta ah: {error_code}"

    @classmethod
    def get_import_error(cls, error_code, **kwargs):
        """Get a formatted import error message."""
        if error_code in cls.IMPORT_ERRORS:
            message = cls.IMPORT_ERRORS[error_code]
            return cls.format_error("import", message, **kwargs)
        return f"Khalad soo dejinta ah: {error_code}"


class LexerError(SoplangError):
    def __init__(self, error_code, position=None, line=None, **kwargs):
        kwargs.update({"position": position, "line": line})

        if error_code in ErrorMessageManager.LEXER_ERRORS:
            # Get error message from error code
            message = ErrorMessageManager.LEXER_ERRORS[error_code]
            self.message = ErrorMessageManager.format_error("lexer", message, **kwargs)
        else:
            # Handle direct error message strings (fallback)
            self.message = error_code

        # Add type prefix only once with English translation
        self.full_message = f"Khalad lexer (Lexer Error): {self.message}"
        super().__init__(self.full_message)


class ParserError(SoplangError):
    def __init__(self, error_code, token=None, line=None, position=None, **kwargs):
        kwargs.update({"token": token, "line": line, "position": position})

        if error_code in ErrorMessageManager.PARSER_ERRORS:
            # Get error message from error code
            message = ErrorMessageManager.PARSER_ERRORS[error_code]
            self.message = ErrorMessageManager.format_error("parser", message, **kwargs)
        else:
            # Handle direct error message strings (fallback)
            self.message = error_code

        # Add type prefix only once with English translation
        self.full_message = f"Khalad parser (Parser Error): {self.message}"
        super().__init__(self.full_message)


class TypeError(SoplangError):
    def __init__(self, error_code, **kwargs):
        if error_code in ErrorMessageManager.TYPE_ERRORS:
            # Get error message from error code
            message = ErrorMessageManager.TYPE_ERRORS[error_code]
            self.message = ErrorMessageManager.format_error("type", message, **kwargs)
        else:
            # Handle direct error message strings (fallback)
            self.message = error_code

        # Add type prefix only once with English translation
        self.full_message = f"Khalad type (Type Error): {self.message}"
        super().__init__(self.full_message)


class ValueError(SoplangError):
    def __init__(self, message):
        self.message = f"Khalad qiimaha ah (Value Error): {message}"
        super().__init__(self.message)


class NameError(SoplangError):
    def __init__(self, name):
        self.message = f"Khalad magaca ah (Name Error): '{name}' ma jiro"
        super().__init__(self.message)


class ImportError(SoplangError):
    def __init__(self, error_code, **kwargs):
        if error_code in ErrorMessageManager.IMPORT_ERRORS:
            # Get error message from error code
            message = ErrorMessageManager.IMPORT_ERRORS[error_code]
            self.message = ErrorMessageManager.format_error("import", message, **kwargs)
        else:
            # Handle direct error message strings (fallback)
            self.message = error_code

        # Add type prefix only once with English translation
        self.full_message = f"Khalad import (Import Error): {self.message}"
        super().__init__(self.full_message)


class RuntimeError(SoplangError):
    def __init__(self, error_code, **kwargs):
        if error_code in ErrorMessageManager.RUNTIME_ERRORS:
            # Get error message from error code
            message = ErrorMessageManager.RUNTIME_ERRORS[error_code]
            self.message = ErrorMessageManager.format_error(
                "runtime", message, **kwargs
            )
        else:
            # Handle direct error message strings (fallback)
            self.message = error_code

        # Add type prefix only once with English translation
        self.full_message = f"Khalad runtime (Runtime Error): {self.message}"
        super().__init__(self.full_message)


# Signal exceptions (not errors, but control flow)


class BreakSignal(Exception):
    """Signal to break out of a loop."""

    pass


class ContinueSignal(Exception):
    """Signal to continue to the next iteration of a loop."""

    pass


class ReturnSignal(Exception):
    """Signal to return from a function with a value."""

    def __init__(self, value=None):
        self.value = value
        super().__init__()
