from src.utils.errors import TypeError, ValueError
import math
import random


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
                return "abn"
            else:
                return "jajab"
        elif isinstance(value, list):
            return "teed"
        elif isinstance(value, dict):
            return "walax"
        elif value is None:
            return "maran"
        else:
            return "aan la aqoon"

    @staticmethod
    def abn(value):
        """
        Convert a value to an integer number
        """
        try:
            return int(float(value))
        except (ValueError, TypeError) as err:
            raise TypeError(f"{value!r} ma badali karo abn") from err

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
    def teed(*args):
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
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")
        lst.append(item)
        return lst

    @staticmethod
    def list_pop(lst):
        """
        Remove and return the last item from a list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")
        if len(lst) == 0:
            raise ValueError(
                "Ma saari kartid teed madhan (Cannot pop from an empty list)"
            )
        return lst.pop()

    @staticmethod
    def list_length(lst):
        """
        Return the length of a list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")
        return len(lst)

    @staticmethod
    def list_concat(lst1, lst2):
        """
        Concatenate two lists and return a new list
        Or add a single item to a list if the second parameter is not a list
        """
        if not isinstance(lst1, list):
            raise TypeError("Qiimaha koowaad ma ahan teed (First value is not a list)")

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
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Return True if item exists in list, False otherwise
        return item in lst

    @staticmethod
    def list_copy(lst):
        """
        Return a shallow copy of the list
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Create a new list that is a shallow copy of the original
        return lst.copy()

    @staticmethod
    def list_clear(lst):
        """
        Remove all items from the list (in-place)
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Clear all items from the list
        lst.clear()
        return lst

    @staticmethod
    def list_get(lst, index):
        """
        Get an item from a list at the specified index
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Convert index to integer if it's a string
        if isinstance(index, str):
            try:
                index = int(index)
            except ValueError as err:
                raise TypeError(
                    "Index waa inuu noqdaa abn (Index must be a number)"
                ) from err

        if not isinstance(index, (int, float)):
            raise TypeError("Index waa inuu noqdaa abn (Index must be a number)")

        index = int(index)  # Convert float to int if needed

        if index < 0 or index >= len(lst):
            raise ValueError(
                f"Index {index} waa ka baxsan xadka teedka (Index out of range)"
            )

        return lst[index]

    @staticmethod
    def list_set(lst, index, value):
        """
        Set an item in a list at the specified index
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Convert index to integer if it's a string
        if isinstance(index, str):
            try:
                index = int(index)
            except ValueError as err:
                raise TypeError(
                    "Index waa inuu noqdaa abn (Index must be a number)"
                ) from err

        if not isinstance(index, (int, float)):
            raise TypeError("Index waa inuu noqdaa abn (Index must be a number)")

        index = int(index)  # Convert float to int if needed

        if index < 0 or index >= len(lst):
            raise ValueError(
                f"Index {index} waa ka baxsan xadka teedka (Index out of range)"
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
    def object_copy(obj):
        """
        Return a shallow copy of the object
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        # Create a new dictionary that is a shallow copy of the original
        return obj.copy()

    @staticmethod
    def object_clear(obj):
        """
        Remove all properties from the object (in-place)
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        # Clear all keys from the object
        obj.clear()
        return obj

    @staticmethod
    def list_reverse(lst):
        """
        Reverse a list in-place
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Reverse the list in-place
        lst.reverse()
        return lst

    @staticmethod
    def list_sort(lst):
        """
        Sort a list in-place (ascending order)
        """
        if not isinstance(lst, list):
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

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
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

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
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Convert indices to integers
        if not isinstance(start, (int, float)) or not isinstance(end, (int, float)):
            raise TypeError(
                "Bilowga iyo dhamaadka waa inay noqdaan abn (Start and end must be numbers)"
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
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

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
    def list_muuji(lst, item):
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
            raise TypeError("Qiimahu ma ahan teed (Value is not a list)")

        # Manually search for the item to avoid using list.index() which throws an exception
        for i in range(len(lst)):
            if lst[i] == item:
                return i

        # Return None (maran in Soplang) if the item is not in the list
        return None

    @staticmethod
    def object_values(obj):
        """
        Get all values from an object as a list
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        # Return all values as a list
        return list(obj.values())

    @staticmethod
    def object_entries(obj):
        """
        Get all key-value pairs from an object as a list of [key, value] pairs
        """
        if not isinstance(obj, dict):
            raise TypeError("Qiimahu ma ahan walax (Value is not an object)")

        # Return all key-value pairs as a list of [key, value] lists
        return [[key, value] for key, value in obj.items()]

    @staticmethod
    def string_split(s, delimiter):
        """
        Split a string by a delimiter and return a list of substrings.
        Similar to split() in Python or JavaScript.

        Args:
            s: The string to split
            delimiter: The delimiter to split by

        Returns:
            A list of substrings
        """
        if not isinstance(s, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        if not isinstance(delimiter, str):
            raise TypeError("Kala qeybiyuhu ma ahan qoraal (Delimiter is not a string)")

        return s.split(delimiter)

    @staticmethod
    def daji(value):
        """
        Round a number down to the nearest integer (floor function).
        Similar to Math.floor() in JavaScript or math.floor() in Python.

        Args:
            value: The number to round down

        Returns:
            The largest integer less than or equal to the input
        """
        if not isinstance(value, (int, float)):
            raise TypeError("Qiimaha ma ahan abn ama jajab (Value is not a number)")

        return math.floor(value)

    @staticmethod
    def kor(value):
        """
        Round a number up to the nearest integer (ceiling function).
        Similar to Math.ceil() in JavaScript or math.ceil() in Python.

        Args:
            value: The number to round up

        Returns:
            The smallest integer greater than or equal to the input
        """
        if not isinstance(value, (int, float)):
            raise TypeError("Qiimaha ma ahan abn ama jajab (Value is not a number)")

        return math.ceil(value)

    @staticmethod
    def string_contains(s, substring):
        """
        Check if a string contains a substring.
        Similar to 'includes()' in JavaScript or 'in' operator in Python.

        Args:
            s: The string to check
            substring: The substring to look for

        Returns:
            Boolean: True if the substring is found, False otherwise
        """
        if not isinstance(s, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        if not isinstance(substring, str):
            raise TypeError("Substring-ka ma ahan qoraal (Substring is not a string)")

        return substring in s

    @staticmethod
    def string_endswith(s, suffix):
        """
        Check if a string ends with a specified suffix.
        Similar to 'endsWith()' in JavaScript or 'endswith()' in Python.

        Args:
            s: The string to check
            suffix: The suffix to check for

        Returns:
            Boolean: True if the string ends with the specified suffix, False otherwise
        """
        if not isinstance(s, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        if not isinstance(suffix, str):
            raise TypeError("Suffix-ka ma ahan qoraal (Suffix is not a string)")

        return s.endswith(suffix)

    @staticmethod
    def string_startswith(s, prefix):
        """
        Check if a string starts with a specified prefix.
        Similar to 'startsWith()' in JavaScript or 'startswith()' in Python.

        Args:
            s: The string to check
            prefix: The prefix to check for

        Returns:
            Boolean: True if the string starts with the specified prefix, False otherwise
        """
        if not isinstance(s, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        if not isinstance(prefix, str):
            raise TypeError("Prefix-ka ma ahan qoraal (Prefix is not a string)")

        return s.startswith(prefix)

    @staticmethod
    def string_replace(s, target, replacement):
        """
        Replace the first occurrence of a substring with another string.
        Similar to 'replace()' in JavaScript or 'replace()' in Python.

        Args:
            s: The original string
            target: The substring to replace
            replacement: The string to substitute in place of the target

        Returns:
            String: A new string with the first occurrence of target replaced by replacement
        """
        if not isinstance(s, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        if not isinstance(target, str):
            raise TypeError("Target-ka ma ahan qoraal (Target is not a string)")

        if not isinstance(replacement, str):
            raise TypeError("Baddalka ma ahan qoraal (Replacement is not a string)")

        # Replace only the first occurrence (1 is the count parameter)
        return s.replace(target, replacement, 1)

    @staticmethod
    def string_join(separator, items):
        """
        Join a list of strings using the specified separator.
        Similar to 'sep.join(list)' in Python or 'array.join(sep)' in JavaScript.

        Args:
            separator: The string to use as a separator
            items: A list of strings to join

        Returns:
            String: A single string containing all items joined by the separator
        """
        if not isinstance(separator, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        if not isinstance(items, list):
            raise TypeError("Qiimaha labaad ma ahan teed (Second value is not a list)")

        # Convert all items to strings before joining
        string_items = []
        for item in items:
            if not isinstance(item, str):
                raise TypeError(
                    "teedka mid ka mid ah qiimihiisa ma ahan qoraal (One of the list items is not a string)")
            string_items.append(item)

        return separator.join(string_items)

    @staticmethod
    def string_jar(s, start, end=None):
        """
        Extract a substring from start index up to (but not including) end index.
        Similar to JavaScript's string.slice() or Python's string slicing.

        Args:
            s: The string to slice
            start: The starting index (inclusive)
            end: The ending index (exclusive), optional (defaults to end of string)

        Returns:
            String: A substring containing characters from start to end (exclusive)
        """
        if not isinstance(s, str):
            raise TypeError("Qiimahu ma ahan qoraal (Value is not a string)")

        # Convert indices to integers
        if not isinstance(start, (int, float)):
            raise TypeError(
                "Bilowga waa inuu noqdaa abn (Start must be a number)"
            )

        start = int(start)

        # Handle optional end parameter
        if end is None:
            end = len(s)
        elif not isinstance(end, (int, float)):
            raise TypeError(
                "Dhamaadka waa inuu noqdaa abn (End must be a number)"
            )
        else:
            end = int(end)

        # Handle negative indexing
        if start < 0:
            start = max(0, len(s) + start)
        start = min(start, len(s))

        if end < 0:
            end = max(0, len(s) + end)
        end = min(end, len(s))

        # Return the substring
        return s[start:end]

    @staticmethod
    def dherer(value):
        """
        Return the length of a value (list, string, or object).
        Similar to len() in Python or .length in JavaScript.

        Args:
            value: The value to get the length of (list, string, or object)

        Returns:
            Integer: The length of the value

        Raises:
            TypeError: If the value is not a list, string, or object
        """
        if isinstance(value, list):
            return len(value)  # Number of items in the list
        elif isinstance(value, str):
            return len(value)  # Number of characters in the string
        elif isinstance(value, dict):
            return len(value)  # Number of key-value pairs in the object
        else:
            raise TypeError(
                "Qiimaha ma ahan teed, qoraal, ama walax (Value is not a list, string, or object)"
            )

    @staticmethod
    def xul(*args):
        """
        Generate random numbers or select random items from a list.

        Behavior depends on arguments:
        - No args: Returns random float between 0.0 and 1.0
        - Two numbers: Returns random number between a and b (inclusive)
        - One list: Returns random item from the list

        Args:
            *args: Variable arguments based on the desired behavior

        Returns:
            Random float, integer, or list item based on arguments

        Raises:
            TypeError: If the arguments are of invalid types
            ValueError: If the arguments are invalid (e.g., a > b)
        """
        # Case 1: No arguments - return random float between 0.0 and 1.0
        if len(args) == 0:
            return random.random()

        # Case 2: One argument - must be a list
        elif len(args) == 1:
            if not isinstance(args[0], list):
                raise TypeError("Qiimaha ma ahan teed (Value is not a list)")

            if len(args[0]) == 0:
                raise ValueError("teedka waa madhan (List is empty)")

            return random.choice(args[0])

        # Case 3: Two arguments - must be numbers
        elif len(args) == 2:
            a, b = args

            if not isinstance(a, (int, float)) or not isinstance(b, (int, float)):
                raise TypeError("Qiimayaasha waa inay noqdaan abn ama jajab (Values must be numbers)")

            if a > b:
                raise ValueError("Qiimaha koowaad waa in uu ka yar yahay ama la mid yahay qiimaha labaad (First value must be less than or equal to second value)")

            # If both are integers, return an integer
            if isinstance(a, int) and isinstance(b, int):
                return random.randint(a, b)

            # Otherwise, return a float
            return random.uniform(a, b)

        # Case 4: More than two arguments - error
        else:
            raise TypeError("xul() waxay qaadataa 0, 1, ama 2 qiimo (xul() takes 0, 1, or 2 arguments)")


def get_builtin_functions():
    """
    Returns a dictionary of all built-in functions
    """
    builtins = {
        "qor": SoplangBuiltins.qor,
        "gelin": SoplangBuiltins.gelin,
        "nooc": SoplangBuiltins.nooc,
        "abn": SoplangBuiltins.abn,
        "jajab": SoplangBuiltins.jajab,
        "qoraal": SoplangBuiltins.qoraal,
        "bool": SoplangBuiltins.bool,
        "teed": SoplangBuiltins.teed,
        "walax": SoplangBuiltins.walax,
        "daji": SoplangBuiltins.daji,
        "kor": SoplangBuiltins.kor,
        "dherer": SoplangBuiltins.dherer,
        "xul": SoplangBuiltins.xul,
    }

    return builtins


def get_object_methods():
    """
    Returns a dictionary of object methods
    """
    methods = {
        "fure": SoplangBuiltins.object_keys,
        "leeyahay": SoplangBuiltins.object_has,
        "tir": SoplangBuiltins.object_remove,
        "kudar": SoplangBuiltins.object_merge,
        "nuqul": SoplangBuiltins.object_copy,
        "nadiifi": SoplangBuiltins.object_clear,
        "qiime": SoplangBuiltins.object_values,
        "lamaane": SoplangBuiltins.object_entries,
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
        "muuji": SoplangBuiltins.list_muuji,
    }

    return methods


def get_string_methods():
    """
    Returns a dictionary of string methods
    """
    methods = {
        "qeybi": SoplangBuiltins.string_split,
        "leeyahay": SoplangBuiltins.string_contains,
        "dhamaad": SoplangBuiltins.string_endswith,
        "bilow": SoplangBuiltins.string_startswith,
        "beddel": SoplangBuiltins.string_replace,
        "kudar": SoplangBuiltins.string_join,
        "jar": SoplangBuiltins.string_jar,
    }

    return methods
