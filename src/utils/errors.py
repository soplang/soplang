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
        "lexer": "Khalad lexer",
        "parser": "Khalad parser",
        "type": "Khalad nooc",
        "runtime": "Khalad runtime",
        "import": "Khalad import",
    }

    # Lexer errors
    LEXER_ERRORS = {
        "unexpected_char": "Xaraf aan la filayn: {char}",
        "unterminated_string": "Qoraal aan la dhammaystirin",
        "unterminated_comment": "Faallo aan la dhammaystirin",
    }

    # Parser errors
    PARSER_ERRORS = {
        "expected_token": "Waxaa la filayay {expected}, laakiin waxaa la helay {found}",
        "unexpected_token": "Calaamad aan la filayn: {token}",
        "invalid_syntax": "Qoraalka syntax-kiisa waa khalad: {detail}",
        "missing_paren": "Waxaa ka maqan hal ')'",
        "missing_brace": "Waxaa ka maqan hal '}'",
        "missing_bracket": "Waxaa ka maqan hal ']'",
    }

    # Type errors
    TYPE_ERRORS = {
        "type_mismatch": "'{var_name}' waa {expected_type} laakin qiimaheeda '{value}' ma ahan {expected_type}",
        "cannot_convert": "'{value}' ma badali karo {target_type}",
        "invalid_operand": "Ma isticmaali karo '{operator}' oo ku shaqeeya {type_name}",
        "property_access": "Ma heli karo astaanta '{prop}' ee qiimaha aan ahayn walax",
        "index_access": "Ma heli karo tirooyinka ee qiimaha aan ahayn teed",
        "invalid_method": "Ma wici karo habka '{method}' ee qiimaha {type_name}",
    }

    # Runtime errors
    RUNTIME_ERRORS = {
        "undefined_variable": "Doorsame aan la qeexin: '{name}'",
        "undefined_function": "Hawl aan la qeexin: '{name}'",
        "division_by_zero": "Ma suurtogali karto qeybinta eber",
        "modulo_by_zero": "Ma suurtogali karto modulo eber",
        "index_out_of_range": "Tirada fihris-ku waa ka baxsan xadka: {index}",
        "property_not_found": "Astaanta '{prop_name}' kuma jirto walaxga",
        "method_not_found": "Habka '{method_name}' kuma jirto {type_name}",
        "missing_argument": "Howsha '{func_name}' waxay u baahan tahay {expected} dood, laakiin waxaa la siiyay {provided}",
        "parent_class_not_found": "Fasalka waalidka '{parent_name}' ma jiro",
        "break_outside_loop": "Jooji waa in ay ku jiraan xalqad",
        "continue_outside_loop": "soco waa in ay ku jiraan xalqad",
        "return_outside_function": "celi waa in ay ku jirto hawl",
        "invalid_for_loop": "kuceli billowga, dhamaadka iyo tallaabada waa in ay yihiin abn",
        "unknown_node_type": "Nooca cladka aan la aqoon: {node_type}",
        "unknown_operator": "Hawl-gal aan la aqoon: {operator}",
        "constant_reassignment": "Ma bedeli kartid qiimaha doorsamaha madoor '{name}'. Doorsooyin madoor ah ma dib loo qiimeyn karo.",
    }

    # Import errors
    IMPORT_ERRORS = {
        "file_not_found": "Faylka '{module}' ma helin",
        "import_error": "Qalad baa ka jira file-ka {filename}: {error}",
    }

    @classmethod
    def _get_error_dict(cls, error_type):
        """Get the appropriate error dictionary based on the error type.

        Args:
            error_type (str): The type of error (lexer, parser, runtime, type, import)

        Returns:
            dict: The dictionary containing error messages for the specified type
        """
        if error_type == "lexer":
            return cls.LEXER_ERRORS
        elif error_type == "parser":
            return cls.PARSER_ERRORS
        elif error_type == "type":
            return cls.TYPE_ERRORS
        elif error_type == "runtime":
            return cls.RUNTIME_ERRORS
        elif error_type == "import":
            return cls.IMPORT_ERRORS
        else:
            return cls.RUNTIME_ERRORS  # Default to runtime errors

    @classmethod
    def format_error(cls, error_type, code, line=None, position=None, **kwargs):
        """
        Format an error message with the given code, line, and position.

        Args:
            error_type (str): The type of error (lexer, parser, runtime, type)
            code (str): The error code to look up in the error dictionaries
                        or an already formatted error message
            line (int, optional): The line number where the error occurred
            position (int, optional): The position in the line where the error occurred
            **kwargs: Placeholder variables to substitute in the error message

        Returns:
            str: The formatted error message
        """
        # Validate the error type
        if error_type not in ["lexer", "parser", "runtime", "type", "import"]:
            error_type = "runtime"  # Default to runtime if type is not recognized

        # Get the error message - if code is in the dictionary, format it
        # otherwise use the code as the message (may already be formatted)
        error_dict = cls._get_error_dict(error_type)
        if code in error_dict:
            error_message = error_dict[code]
            # Format the error message with the provided kwargs
            if kwargs:
                error_message = error_message.format(**kwargs)
        else:
            # Use the code as the error message (may already be formatted)
            error_message = code

        # Format the final message in the exact format requested
        final_message = f"Khalad {error_type}: {error_message}"

        # Add location information if available
        if line is not None:
            final_message += f" sadar {line}"
            if position is not None:
                final_message += f", goobta {position}"

        return final_message

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
        return f"Khalad nooca ah: {error_code}"

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
        # Make a copy of kwargs and add position and line
        params = kwargs.copy()
        if position is not None:
            params["position"] = position
        if line is not None:
            params["line"] = line

        if error_code in ErrorMessageManager.LEXER_ERRORS:
            # Get the template from the dictionary
            message_template = ErrorMessageManager.LEXER_ERRORS[error_code]
            # Format the template with the provided parameters
            message = message_template.format(**params) if params else message_template
            # Create the final message
            self.message = ErrorMessageManager.format_error(
                "lexer", message, line=line, position=position
            )
        else:
            # Direct message
            self.message = ErrorMessageManager.format_error(
                "lexer", error_code, line=line, position=position
            )

        super().__init__(self.message)


class ParserError(SoplangError):
    def __init__(self, error_code, token=None, line=None, position=None, **kwargs):
        # Make a copy of kwargs and add token, position and line
        params = kwargs.copy()
        if token is not None:
            params["token"] = token
        if position is not None:
            params["position"] = position
        if line is not None:
            params["line"] = line

        if error_code in ErrorMessageManager.PARSER_ERRORS:
            # Get the template from the dictionary
            message_template = ErrorMessageManager.PARSER_ERRORS[error_code]
            # Format the template with the provided parameters
            message = message_template.format(**params) if params else message_template
            # Create the final message
            self.message = ErrorMessageManager.format_error(
                "parser", message, line=line, position=position
            )
        else:
            # Direct message
            self.message = ErrorMessageManager.format_error(
                "parser", error_code, line=line, position=position
            )

        super().__init__(self.message)


class TypeError(SoplangError):
    def __init__(self, error_code, line=None, position=None, **kwargs):
        # Make a copy of kwargs and add position and line
        params = kwargs.copy()
        if position is not None:
            params["position"] = position
        if line is not None:
            params["line"] = line

        if error_code in ErrorMessageManager.TYPE_ERRORS:
            # Get the template from the dictionary
            message_template = ErrorMessageManager.TYPE_ERRORS[error_code]
            # Format the template with the provided parameters
            message = message_template.format(**params) if params else message_template
            # Create the final message
            self.message = ErrorMessageManager.format_error(
                "type", message, line=line, position=position
            )
        else:
            # Direct message
            self.message = ErrorMessageManager.format_error(
                "type", error_code, line=line, position=position
            )

        super().__init__(self.message)


class ValueError(SoplangError):
    def __init__(self, message, line=None, position=None):
        self.message = ErrorMessageManager.format_error(
            "runtime",
            f"Khalad qiimaha ah (Value Error): {message}",
            line=line,
            position=position,
        )
        super().__init__(self.message)


class NameError(SoplangError):
    def __init__(self, name, line=None, position=None):
        self.message = ErrorMessageManager.format_error(
            "runtime",
            f"Khalad magaca ah (Name Error): '{name}' ma jiro",
            line=line,
            position=position,
        )
        super().__init__(self.message)


class ImportError(SoplangError):
    def __init__(self, error_code, line=None, position=None, **kwargs):
        # Make a copy of kwargs and add position and line
        params = kwargs.copy()
        if position is not None:
            params["position"] = position
        if line is not None:
            params["line"] = line

        if error_code in ErrorMessageManager.IMPORT_ERRORS:
            # Get the template from the dictionary
            message_template = ErrorMessageManager.IMPORT_ERRORS[error_code]
            # Format the template with the provided parameters
            message = message_template.format(**params) if params else message_template
            # Create the final message
            self.message = ErrorMessageManager.format_error(
                "import", message, line=line, position=position
            )
        else:
            # Direct message
            self.message = ErrorMessageManager.format_error(
                "import", error_code, line=line, position=position
            )

        super().__init__(self.message)


class RuntimeError(SoplangError):
    def __init__(self, error_code, line=None, position=None, **kwargs):
        # Make a copy of kwargs and add position and line
        params = kwargs.copy()
        if position is not None:
            params["position"] = position
        if line is not None:
            params["line"] = line

        if error_code in ErrorMessageManager.RUNTIME_ERRORS:
            # Get the template from the dictionary
            message_template = ErrorMessageManager.RUNTIME_ERRORS[error_code]
            # Format the template with the provided parameters
            message = message_template.format(**params) if params else message_template
            # Create the final message
            self.message = ErrorMessageManager.format_error(
                "runtime", message, line=line, position=position
            )
        else:
            # Direct message
            self.message = ErrorMessageManager.format_error(
                "runtime", error_code, line=line, position=position
            )

        super().__init__(self.message)


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
