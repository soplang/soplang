class SoplangError(Exception):
    """Base class for all Soplang errors."""
    pass


class LexerError(SoplangError):
    def __init__(self, message, position=None):
        self.message = f"Khalad markii loo qaybinayay: {message}"
        if position is not None:
            self.message += f" at position {position}"
        super().__init__(self.message)


class ParserError(SoplangError):
    def __init__(self, message, token=None):
        self.message = f"Khalad markii la falanqaynayay: {message}"
        if token is not None:
            self.message += f" at token {token}"
        super().__init__(self.message)


class TypeError(SoplangError):
    def __init__(self, message):
        self.message = f"Khalad nuuca ah: {message}"
        super().__init__(self.message)


class ValueError(SoplangError):
    def __init__(self, message):
        self.message = f"Khalad qiimaha ah: {message}"
        super().__init__(self.message)


class NameError(SoplangError):
    def __init__(self, name):
        self.message = f"Khalad magaca ah: '{name}' ma jiro"
        super().__init__(self.message)


class ImportError(SoplangError):
    def __init__(self, module):
        self.message = f"Khalad soo dejinta ah: Ma soo dejin karo '{module}'"
        super().__init__(self.message)


class RuntimeError(SoplangError):
    def __init__(self, message):
        self.message = f"Khalad fulinta ah: {message}"
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
