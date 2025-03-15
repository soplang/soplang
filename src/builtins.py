from src.errors import TypeError, ValueError


class SoplangBuiltins:
    @staticmethod
    def qor(message):
        """
        Print a message to the console (equivalent to print/console.log)
        """
        print(message)
        return message

    @staticmethod
    def akhri(prompt=""):
        """
        Read input from the user (equivalent to input/prompt)
        """
        return input(prompt)

    @staticmethod
    def nuuc(value):
        """
        Return the type of a value as a string
        """
        if isinstance(value, str):
            return "qoraal"
        elif isinstance(value, (int, float)):
            return "tiro"
        elif isinstance(value, bool):
            return "boolean"
        elif isinstance(value, list):
            return "liis"
        elif isinstance(value, dict):
            return "shey"
        elif value is None:
            return "waxba"
        else:
            return "aan la aqoon"

    @staticmethod
    def tiro(value):
        """
        Convert a value to a number
        """
        try:
            return float(value)
        except (ValueError, TypeError):
            raise TypeError(f"'{value}' ma badali karo tiro")

    @staticmethod
    def qoraal(value):
        """
        Convert a value to a string
        """
        return str(value)

    @staticmethod
    def boole(value):
        """
        Convert a value to a boolean
        """
        if value in [0, "", False, None, "false", "False"]:
            return False
        return True

    @staticmethod
    def liis(*args):
        """
        Create a list from the arguments
        """
        return list(args)

    @staticmethod
    def shey(**kwargs):
        """
        Create a dictionary from keyword arguments
        """
        return kwargs


def get_builtin_functions():
    """
    Returns a dictionary of all built-in functions
    """
    builtins = {
        "qor": SoplangBuiltins.qor,
        "akhri": SoplangBuiltins.akhri,
        "nuuc": SoplangBuiltins.nuuc,
        "tiro": SoplangBuiltins.tiro,
        "qoraal": SoplangBuiltins.qoraal,
        "boole": SoplangBuiltins.boole,
        "liis": SoplangBuiltins.liis,
        "shey": SoplangBuiltins.shey
    }

    return builtins
