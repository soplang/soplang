from enum import Enum


class TokenType(Enum):
    # Existing
    DOOR = "door"
    MADOOR = "madoor"  # Constant variable
    HAWL = "hawl"
    CELI = "celi"
    qor = "qor"
    GELIN = "gelin"
    HADDII = "haddii"
    HADDII_KALE = "haddii_kale"
    UGUDAMBEYN = "ugudambeyn"
    DOORO = "dooro"  # switch
    XAALAD = "xaalad"  # case
    kuceli = "kuceli"
    INTAY = "intay"
    JOOJI = "jooji"  # break
    soco = "soco"  # continue
    ISKU_DAY = "isku_day"  # try
    QABO = "qabo"  # catch
    KA_KEEN = "ka_keen"  # import
    FASALKA = "fasalka"  # class
    KA_DHAXAL = "ka_dhaxal"  # extends
    CUSUB = "cusub"  # new
    NAFTA = "nafta"  # self/this

    # Static types
    TIRO = "tiro"  # integer
    JAJAB = "jajab"  # float/decimal
    QORAAL = "qoraal"  # string
    BOOL = "bool"  # boolean
    teed = "teed"  # list
    WALAX = "walax"  # object/dict

    # Data & Operators
    IDENTIFIER = "IDENTIFIER"
    NUMBER = "NUMBER"
    STRING = "STRING"
    PLUS = "+"
    MINUS = "-"
    STAR = "*"
    SLASH = "/"
    MODULO = "%"  # Added modulo operator for remainder
    EQUAL = "="
    NOT_EQUAL = "!="
    GREATER = ">"
    LESS = "<"
    GREATER_EQUAL = ">="
    LESS_EQUAL = "<="
    AND = "&&"
    OR = "||"
    NOT = "!"
    ASSIGN = "="
    COMMA = ","
    COLON = ":"
    SEMICOLON = ";"
    LEFT_PAREN = "("
    RIGHT_PAREN = ")"
    LEFT_BRACE = "{"
    RIGHT_BRACE = "}"

    # Added for lists and objects support
    LEFT_BRACKET = "["  # For list literals
    RIGHT_BRACKET = "]"  # For list literals
    DOT = "."  # For object property access
    TRUE = "true"  # Boolean literal
    FALSE = "false"  # Boolean literal
    NULL = "null"  # Null literal

    EOF = "EOF"
