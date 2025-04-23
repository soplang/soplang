from src.utils.errors import TypeError, ValueError


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
        elif isinstance(value, bool):
            return "labadaran"
        elif isinstance(value, (int, float)):
            return "tiro"
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
        # Convert boolean values to Soplang equivalents
        if isinstance(value, bool):
            return "run" if value else "been"

        if isinstance(value, dict):
            try:
                # Simple JSON-like stringification for dictionaries
                pairs = []
                for k, v in value.items():
                    pairs.append(f'"{k}": {SoplangBuiltins.qoraal(v)}')
                return "{" + ", ".join(pairs) + "}"
            except Exception:
                # Fallback for circular references
                return "{...}"
        elif isinstance(value, list):
            try:
                # Simple JSON-like stringification for lists
                items = [SoplangBuiltins.qoraal(item) for item in value]
                return "[" + ", ".join(items) + "]"
            except Exception:
                # Fallback for circular references
                return "[...]"
        return str(value)

    @staticmethod
    def labadaran(value):
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

    @staticmethod
    def list_push(lst, item):
        """
        Add an item to the end of a list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")
        lst.append(item)
        return lst

    @staticmethod
    def list_pop(lst):
        """
        Remove and return the last item from a list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")
        if len(lst) == 0:
            raise ValueError(
                "Ma saari kartid liis madhan (Cannot pop from an empty list)"
            )
        return lst.pop()

    @staticmethod
    def list_length(lst):
        """
        Return the length of a list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")
        return len(lst)

    @staticmethod
    def list_get(lst, index):
        """
        Get an item from a list at the specified index
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Convert index to integer if it's a string
        if isinstance(index, str):
            try:
                index = int(index)
            except ValueError:
                raise TypeError("Index waa inuu noqdaa tiro (Index must be a number)")

        if not isinstance(index, (int, float)):
            raise TypeError("Index waa inuu noqdaa tiro (Index must be a number)")

        index = int(index)  # Convert float to int if needed

        if index < 0 or index >= len(lst):
            raise ValueError(
                f"Index {index} waa ka baxsan xadka liiska (Index out of range)"
            )

        return lst[index]

    @staticmethod
    def list_set(lst, index, value):
        """
        Set an item in a list at the specified index
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Convert index to integer if it's a string
        if isinstance(index, str):
            try:
                index = int(index)
            except ValueError:
                raise TypeError("Index waa inuu noqdaa tiro (Index must be a number)")

        if not isinstance(index, (int, float)):
            raise TypeError("Index waa inuu noqdaa tiro (Index must be a number)")

        index = int(index)  # Convert float to int if needed

        if index < 0 or index >= len(lst):
            raise ValueError(
                f"Index {index} waa ka baxsan xadka liiska (Index out of range)"
            )

        lst[index] = value
        return value

    @staticmethod
    def object_get(obj, key):
        """
        Get a property from an object
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan shey (Value is not an object)")

        if key not in obj:
            return None

        return obj[key]

    @staticmethod
    def object_set(obj, key, value):
        """
        Set a property on an object
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan shey (Value is not an object)")

        obj[key] = value
        return value

    @staticmethod
    def object_keys(obj):
        """
        Get all keys from an object as a list
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan shey (Value is not an object)")

        return list(obj.keys())

    @staticmethod
    def object_has(obj, key):
        """
        Check if an object has a specific property
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan shey (Value is not an object)")

        return key in obj


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
        "labadaran": SoplangBuiltins.labadaran,
        "liis": SoplangBuiltins.liis,
        "shey": SoplangBuiltins.shey,
    }

    return builtins


def get_object_methods():
    """
    Returns a dictionary of object methods
    """
    methods = {"keys": SoplangBuiltins.object_keys, "has": SoplangBuiltins.object_has}

    return methods


def get_list_methods():
    """
    Returns a dictionary of list methods
    """
    methods = {
        "push": SoplangBuiltins.list_push,
        "pop": SoplangBuiltins.list_pop,
        "length": SoplangBuiltins.list_length,
    }

    return methods
