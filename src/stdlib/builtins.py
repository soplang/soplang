from src.utils.errors import TypeError, ValueError


class SoplangBuiltins:
    @staticmethod
    def bandhig(message=""):
        """
        Print a message to the console (equivalent to 'print' in many languages)

        Args:
            message: The message to print (default: empty string)
        """
        print(message)
        return message

    @staticmethod
    def gelin(prompt=""):
        """
        Read input from the user (equivalent to input/prompt)
        """
        return input(prompt)

    @staticmethod
    def nooc(value):
        """
        Return the type of a value as a string
        """
        if isinstance(value, str):
            return "qoraal"
        elif isinstance(value, bool):
            return "bool"
        elif isinstance(value, (int, float)):
            # Distinguish between integers and floats
            if isinstance(value, int):
                return "tiro"
            else:
                return "jajab"
        elif isinstance(value, list):
            return "liis"
        elif isinstance(value, dict):
            return "walax"
        elif value is None:
            return "maran"
        else:
            return "aan la aqoon"

    @staticmethod
    def tiro(value):
        """
        Convert a value to an integer number
        """
        try:
            return int(float(value))
        except (ValueError, TypeError) as err:
            raise TypeError(f"{value!r} ma badali karo tiro") from err

    @staticmethod
    def jajab(value):
        """
        Convert a value to a decimal/floating-point number
        """
        try:
            return float(value)
        except (ValueError, TypeError) as err:
            raise TypeError(f"{value!r} ma badali karo jajab") from err

    @staticmethod
    def qoraal(value):
        """
        Convert a value to a string
        """
        # Convert boolean values to Soplang equivalents
        if isinstance(value, bool):
            return "run" if value else "been"

        # Handle numeric values
        if isinstance(value, (int, float)):
            if isinstance(value, int):
                return str(value)  # Integer without decimal point
            else:
                return str(value)  # Float (always with decimal point)

        if isinstance(value, dict):
            try:
                # Simple JSON-like stringification for dictionaries
                pairs = []
                for k, v in value.items():
                    pairs.append(f"{k!r}: {SoplangBuiltins.qoraal(v)}")
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
    def walax(**kwargs):
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
        Or add a single item to a list if the second parameter is not a list
        """
        if not isinstance(lst1, list):
            raise TypeError("Qiimaha koowaad ma ahan liis (First value is not a list)")

        # If lst2 is a list, concatenate (without modifying original)
        if isinstance(lst2, list):
            # Create a new list with items from both lists
            return lst1.copy() + lst2
        # Otherwise, treat as push operation (modifies in-place)
        else:
            # Add the item to the list (modifies in-place)
            lst1.append(lst2)
            return lst1

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
    def list_copy(lst):
        """
        Return a shallow copy of the list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Create a new list that is a shallow copy of the original
        return lst.copy()

    @staticmethod
    def list_clear(lst):
        """
        Remove all items from the list (in-place)
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Clear all items from the list
        lst.clear()
        return lst

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
            except ValueError as err:
                raise TypeError(
                    "Index waa inuu noqdaa tiro (Index must be a number)"
                ) from err

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
            except ValueError as err:
                raise TypeError(
                    "Index waa inuu noqdaa tiro (Index must be a number)"
                ) from err

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
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        if key not in obj:
            return None

        return obj[key]

    @staticmethod
    def object_set(obj, key, value):
        """
        Set a property on an object
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        obj[key] = value
        return value

    @staticmethod
    def object_keys(obj):
        """
        Get all keys from an object as a list
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        return list(obj.keys())

    @staticmethod
    def object_has(obj, key):
        """
        Check if an object has a specific property
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        return key in obj

    @staticmethod
    def object_remove(obj, key):
        """
        Remove a property from an object
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

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
                "Qiimaha koowaad ma ahan walax (First value is not an object)"
            )
        if not isinstance(obj2, dict):
            raise TypeError(
                "Qiimaha labaad ma ahan walax (Second value is not an object)"
            )

        # Create a new dictionary with items from both objects
        result = obj1.copy()
        result.update(obj2)
        return result

    @staticmethod
    def list_reverse(lst):
        """
        Reverse a list in-place
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Reverse the list in-place
        lst.reverse()
        return lst

    @staticmethod
    def list_sort(lst):
        """
        Sort a list in-place (ascending order)
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Sort the list in-place (ascending order)
        lst.sort()
        return lst

    @staticmethod
    def list_filter(lst, condition_func):
        """
        Filter a list based on a condition function and return a new list
        with only the items that satisfy the condition
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        if not callable(condition_func):
            raise TypeError(
                "Qiimaha labaad ma ahan hawl (Second argument is not a function)"
            )

        # Create a new list with items that satisfy the condition
        result = []
        for item in lst:
            # Call the condition function for each item
            if condition_func(item):
                result.append(item)

        return result

    @staticmethod
    def list_jar(lst, start, end):
        """
        Return a new list containing items from the start index up to (but not including) the end index.
        Similar to JavaScript's array.slice() or Python's list slicing.

        Args:
            lst: The list to slice
            start: The starting index (inclusive)
            end: The ending index (exclusive)

        Returns:
            A new list containing elements from start to end (exclusive)
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Convert indices to integers
        if not isinstance(start, (int, float)) or not isinstance(end, (int, float)):
            raise TypeError(
                "Bilowga iyo dhamaadka waa inay noqdaan tiro (Start and end must be numbers)"
            )

        start = int(start)
        end = int(end)

        # Handle out-of-range indices
        # If start is negative, count from the end of the list
        if start < 0:
            start = max(0, len(lst) + start)
        # Make sure start is within bounds
        start = min(start, len(lst))

        # If end is negative, count from the end of the list
        if end < 0:
            end = max(0, len(lst) + end)
        # Make sure end is within bounds
        end = min(end, len(lst))

        # Create a new list with the sliced elements
        return lst[start:end]

    @staticmethod
    def list_map(lst, transform_func):
        """
        Transform a list by applying a function to each item and return a new list
        with the transformed values. Similar to map() in many languages.

        Args:
            lst: The list to transform
            transform_func: A function that takes an item and returns a transformed value

        Returns:
            A new list containing the transformed values
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        if not callable(transform_func):
            raise TypeError(
                "Qiimaha labaad ma ahan hawl (Second argument is not a function)"
            )

        # Create a new list with transformed items
        result = []
        for item in lst:
            # Apply the transform function to each item
            transformed = transform_func(item)
            result.append(transformed)

        return result

    @staticmethod
    def list_raadso(lst, item):
        """
        Find the index of an item in a list
        Returns the index of the first matching item or maran if not found

        Args:
            lst: The list to search in
            item: The item to search for

        Returns:
            The index of the first occurrence of the item, or None (maran in Soplang) if not found
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan liis (Value is not a list)")

        # Manually search for the item to avoid using list.index() which throws an exception
        for i in range(len(lst)):
            if lst[i] == item:
                return i

        # Return None (maran in Soplang) if the item is not in the list
        return None


def get_builtin_functions():
    """
    Returns a dictionary of all built-in functions
    """
    builtins = {
        "bandhig": SoplangBuiltins.bandhig,
        "gelin": SoplangBuiltins.gelin,
        "nooc": SoplangBuiltins.nooc,
        "tiro": SoplangBuiltins.tiro,
        "jajab": SoplangBuiltins.jajab,
        "qoraal": SoplangBuiltins.qoraal,
        "bool": SoplangBuiltins.bool,
        "liis": SoplangBuiltins.liis,
        "walax": SoplangBuiltins.walax,
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
        "kudar": SoplangBuiltins.object_merge,
    }

    return methods


def get_list_methods():
    """
    Returns a dictionary of list methods
    """
    methods = {
        "kasaar": SoplangBuiltins.list_pop,
        "dherer": SoplangBuiltins.list_length,
        "kudar": SoplangBuiltins.list_concat,
        "leeyahay": SoplangBuiltins.list_contains,
        "nuqul": SoplangBuiltins.list_copy,
        "nadiifi": SoplangBuiltins.list_clear,
        "rog": SoplangBuiltins.list_reverse,
        "habee": SoplangBuiltins.list_sort,
        "shaandhee": SoplangBuiltins.list_filter,
        "jar": SoplangBuiltins.list_jar,
        "aaddin": SoplangBuiltins.list_map,
        "raadso": SoplangBuiltins.list_raadso,
    }

    return methods
