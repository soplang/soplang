from src.utils.errors import TypeError, ValueError


class SoplangBuiltins:
    @staticmethod
    def qor(message=""):
        """
        Print a message to the console (equivalent to 'print' in many languages)

        Args:
            message: The message to print (default: empty string)
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
            return "bool"
        elif isinstance(value, (int, float)):
            return "tiro"
        elif isinstance(value, list):
            return "liis"
        elif isinstance(value, dict):
            return "shey"
        elif value is None:
            return "maran"
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
    def bool(value):
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
    def list_concat(lst1, lst2):
        """
        Concatenate two lists and return a new list
        """
        if not isinstance(lst1, list):
            raise TypeError("Qiimaha koowaad ma ahan liis (First value is not a list)")
        if not isinstance(lst2, list):
            raise TypeError("Qiimaha labaad ma ahan liis (Second value is not a list)")

        # Create a new list with items from both lists
        return lst1 + lst2

    @staticmethod
    def list_contains(lst, item):
        """
        Check if an item exists in the list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Return True if item exists in list, False otherwise
        return item in lst

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

    @staticmethod
    def object_remove(obj, key):
        """
        Remove a property from an object
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan shey (Value is not an object)")

        if key in obj:
            del obj[key]

        return obj

    @staticmethod
    def object_merge(obj1, obj2):
        """
        Merge two objects into a new one
        """
        if not isinstance(obj1, dict):
            raise TypeError(
                "Qiimaha koowaad ma ahan shey (First value is not an object)")
        if not isinstance(obj2, dict):
            raise TypeError(
                "Qiimaha labaad ma ahan shey (Second value is not an object)")

        # Create a new dictionary with items from both objects
        result = obj1.copy()
        result.update(obj2)
        return result


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
        "bool": SoplangBuiltins.bool,
        "liis": SoplangBuiltins.liis,
        "shey": SoplangBuiltins.shey,
    }

    return builtins


def get_object_methods():
    """
    Returns a dictionary of object methods
    """
    methods = {
        "fure": SoplangBuiltins.object_keys,
        "leeyahay": SoplangBuiltins.object_has,
        "tirtir": SoplangBuiltins.object_remove,
        "iskudar": SoplangBuiltins.object_merge
    }

    return methods


def get_list_methods():
    """
    Returns a dictionary of list methods
    """
    methods = {
        "kudar": SoplangBuiltins.list_push,
        "kasaar": SoplangBuiltins.list_pop,
        "dherer": SoplangBuiltins.list_length,
        "iskuxir": SoplangBuiltins.list_concat,
        "ka_kooban": SoplangBuiltins.list_contains,
    }

    return methods
