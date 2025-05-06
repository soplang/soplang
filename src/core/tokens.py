from enum import Enum


class TokenType(Enum):
    # Existing
    DOOR = "door"
    HAWL = "hawl"
    CELI = "celi"
    QOR = "qor"
    GELIN = "gelin"
    HADDII = "haddii"
    HADDII_KALE = "haddii_kale"
    HADDII_KALENA = "haddii_kalena"
    KU_CELI = "ku_celi"
    INTA_AY = "inta_ay"
    JOOJI = "jooji"        # break
    SII_WAD = "sii_wad"    # continue
    ISKU_DAY = "isku_day"  # try
    QABO = "qabo"          # catch
    KA_KEEN = "ka_keen"    # import
    FASALKA = "fasalka"    # class
    KA_DHAXAL = "ka_dhaxal"  # extends
    CUSUB = "cusub"        # new
    NAFTA = "nafta"        # self/this

    # Static types
    TIRO = "tiro"          # number
    QORAAL = "qoraal"      # string
    LABADARAN = "labadaran"  # boolean
    LIIS = "liis"          # list
    SHEY = "shey"          # object/dict

    # Data & Operators
    IDENTIFIER = "IDENTIFIER"
    NUMBER = "NUMBER"
    STRING = "STRING"
    PLUS = "+"
    MINUS = "-"
    STAR = "*"
    SLASH = "/"
    MODULO = "%"           # Added modulo operator for remainder
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
    LEFT_BRACKET = "["    # For list literals
    RIGHT_BRACKET = "]"   # For list literals
    DOT = "."             # For object property access
    TRUE = "true"         # Boolean literal
    FALSE = "false"       # Boolean literal
    NULL = "null"         # Null literal

    EOF = "EOF"
