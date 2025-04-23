from src.core.tokens import TokenType
from src.utils.errors import LexerError


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
        self.line = 1  # Track line number
        self.column = 1  # Track column position
        self.current_char = self.source[self.position] if self.source else None

        self.KEYWORDS = {
            # Dynamic and control flow
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
            "nafta": TokenType.NAFTA,
            # Static types
            "tiro": TokenType.TIRO,
            "qoraal": TokenType.QORAAL,
            "labadaran": TokenType.LABADARAN,
            "liis": TokenType.LIIS,
            "shey": TokenType.SHEY,
            # Boolean literals
            "run": TokenType.TRUE,
            "been": TokenType.FALSE,
            "null": TokenType.NULL,
        }

    def advance(self):
        # Update line and column tracking
        if self.current_char == '\n':
            self.line += 1
            self.column = 1
        else:
            self.column += 1

        self.position += 1
        if self.position < len(self.source):
            self.current_char = self.source[self.position]
        else:
            self.current_char = None

    def skip_whitespace(self):
        while self.current_char and self.current_char.isspace():
            self.advance()

    def skip_comment(self):
        # Single-line comments (//)
        if self.current_char == "/" and self.peek() == "/":
            self.advance()  # Skip first '/'
            self.advance()  # Skip second '/'

            # Skip until end of line or end of file
            while self.current_char and self.current_char != "\n":
                self.advance()

            # Skip the newline character if present
            if self.current_char == "\n":
                self.advance()

            return True

        # Multi-line comments (/* ... */)
        elif self.current_char == "/" and self.peek() == "*":
            self.advance()  # Skip '/'
            self.advance()  # Skip '*'

            while self.current_char:
                if self.current_char == "*" and self.peek() == "/":
                    self.advance()  # Skip '*'
                    self.advance()  # Skip '/'
                    return True
                self.advance()

            # If we reach here, the comment was not properly closed
            raise LexerError("unterminated_comment",
                             position=self.column, line=self.line)

        return False

    def peek(self):
        """Look at the next character without advancing"""
        peek_pos = self.position + 1
        if peek_pos >= len(self.source):
            return None
        return self.source[peek_pos]

    def tokenize_identifier(self):
        identifier = ""
        while self.current_char and (
            self.current_char.isalnum() or self.current_char == "_"
        ):
            identifier += self.current_char
            self.advance()

        # Check if it's a keyword
        token_type = self.KEYWORDS.get(identifier, TokenType.IDENTIFIER)
        return Token(token_type, identifier)

    def tokenize_number(self):
        number = ""
        while self.current_char and (
            self.current_char.isdigit() or self.current_char == "."
        ):
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
        raise LexerError("unterminated_string", position=self.column, line=self.line)

    def next_token(self):
        while self.current_char:
            # Skip whitespace
            if self.current_char.isspace():
                self.skip_whitespace()
                continue

            # Handle comments
            if self.current_char == "/":
                if self.skip_comment():
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
            if self.current_char == "%":
                self.advance()
                return Token(TokenType.MODULO, "%")
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
            if self.current_char == "&":
                self.advance()
                if self.current_char == "&":
                    self.advance()
                    return Token(TokenType.AND, "&&")
                raise LexerError(
                    "unexpected_char", position=self.column, line=self.line, char="&"
                )
            if self.current_char == "|":
                self.advance()
                if self.current_char == "|":
                    self.advance()
                    return Token(TokenType.OR, "||")
                raise LexerError(
                    "unexpected_char", position=self.column, line=self.line, char="|"
                )
            if self.current_char == ",":
                self.advance()
                return Token(TokenType.COMMA, ",")
            if self.current_char == ":":
                self.advance()
                return Token(TokenType.COLON, ":")
            if self.current_char == ";":
                self.advance()
                return Token(TokenType.SEMICOLON, ";")

            # New tokens for lists and objects
            if self.current_char == "[":
                self.advance()
                return Token(TokenType.LEFT_BRACKET, "[")
            if self.current_char == "]":
                self.advance()
                return Token(TokenType.RIGHT_BRACKET, "]")
            if self.current_char == ".":
                self.advance()
                return Token(TokenType.DOT, ".")

            raise LexerError(
                "unexpected_char", position=self.column, line=self.line, char=self.current_char
            )

        return Token(TokenType.EOF, None)

    def tokenize(self):
        tokens = []
        while self.position < len(self.source):
            token = self.next_token()
            tokens.append(token)
            if token.type == TokenType.EOF:
                break
        return tokens
