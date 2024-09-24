import re
from token import TokenType

class Token:
    def __init__(self, type_, value):
        self.type = type_
        self.value = value

    def __repr__(self):
        return f"Token({self.type}, {repr(self.value)})"

class Lexer:
    def __init__(self, source_code):
        self.source = source_code
        self.position = 0
        self.current_char = self.source[self.position] if self.source else None

        self.KEYWORDS = {
            "door": TokenType.DOOR,
            "howl": TokenType.HOWL,
            "soo_celi": TokenType.SOO_CELI,
            "qor": TokenType.QOR,
            "akhri": TokenType.AKHRI,
            "haddii": TokenType.HADDII,
            "haddii_kale": TokenType.HADDII_KALE,
            "haddii_kalena": TokenType.HADDII_KALENA,
            "ku_celi": TokenType.KU_CELI,
            "inta_ay": TokenType.INTA_AY,
            "jooji": TokenType.JOOJI,
            "sii_wad": TokenType.SII_WAD,
            "isku_day": TokenType.ISKU_DAY,
            "qabo": TokenType.QABO,
            "ka_keen": TokenType.KA_KEEN,
            "fasalka": TokenType.FASALKA,
            "ka_dhaxal": TokenType.KA_DHAXAL,
            "cusub": TokenType.CUSUB,
            "nafta": TokenType.NAFTA
        }

    def advance(self):
        self.position += 1
        if self.position < len(self.source):
            self.current_char = self.source[self.position]
        else:
            self.current_char = None

    def tokenize_identifier(self):
        identifier = ""
        while self.current_char and (self.current_char.isalnum() or self.current_char == "_"):
            identifier += self.current_char
            self.advance()

        # Check if it's a keyword
        token_type = self.KEYWORDS.get(identifier, TokenType.IDENTIFIER)
        return Token(token_type, identifier)

    def tokenize_number(self):
        number = ""
        while self.current_char and (self.current_char.isdigit() or self.current_char == "."):
            number += self.current_char
            self.advance()
        return Token(TokenType.NUMBER, float(number))

    def tokenize_string(self):
        quote_char = self.current_char
        self.advance()
        string_value = ""

        while self.current_char and self.current_char != quote_char:
            string_value += self.current_char
            self.advance()

        if self.current_char == quote_char:
            self.advance()
            return Token(TokenType.STRING, string_value)
        raise Exception("Unterminated string literal")

    def next_token(self):
        while self.current_char:
            if self.current_char.isspace():
                self.advance()
                continue

            if self.current_char.isalpha():
                return self.tokenize_identifier()

            if self.current_char.isdigit():
                return self.tokenize_number()

            if self.current_char in "\"'":
                return self.tokenize_string()

            if self.current_char == "+":
                self.advance()
                return Token(TokenType.PLUS, "+")
            if self.current_char == "-":
                self.advance()
                return Token(TokenType.MINUS, "-")
            if self.current_char == "*":
                self.advance()
                return Token(TokenType.STAR, "*")
            if self.current_char == "/":
                self.advance()
                return Token(TokenType.SLASH, "/")
            if self.current_char == "=":
                self.advance()
                return Token(TokenType.EQUAL, "=")
            if self.current_char == "(":
                self.advance()
                return Token(TokenType.LEFT_PAREN, "(")
            if self.current_char == ")":
                self.advance()
                return Token(TokenType.RIGHT_PAREN, ")")
            if self.current_char == "{":
                self.advance()
                return Token(TokenType.LEFT_BRACE, "{")
            if self.current_char == "}":
                self.advance()
                return Token(TokenType.RIGHT_BRACE, "}")
            if self.current_char == ">":
                self.advance()
                if self.current_char == "=":
                    self.advance()
                    return Token(TokenType.GREATER_EQUAL, ">=")
                return Token(TokenType.GREATER, ">")
            if self.current_char == "<":
                self.advance()
                if self.current_char == "=":
                    self.advance()
                    return Token(TokenType.LESS_EQUAL, "<=")
                return Token(TokenType.LESS, "<")
            if self.current_char == "!":
                self.advance()
                if self.current_char == "=":
                    self.advance()
                    return Token(TokenType.NOT_EQUAL, "!=")
                return Token(TokenType.NOT, "!")

            raise Exception(f"Unexpected character: {self.current_char}")

        return Token(TokenType.EOF, None)

    def tokenize(self):
        tokens = []
        while self.position < len(self.source):
            token = self.next_token()
            tokens.append(token)
            if token.type == TokenType.EOF:
                break
        return tokens
