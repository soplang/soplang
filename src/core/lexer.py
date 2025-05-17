from src.core.tokens import TokenType
from src.utils.errors import LexerError


class Token:
    def __init__(self, type_, value, line=None, position=None):
        self.type = type_
        self.value = value
        self.line = line
        self.position = position

    def __repr__(self):
        return f"Token({self.type}, {repr(self.value)}, line={self.line}, pos={self.position})"


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
            "madoor": TokenType.MADOOR,
            "hawl": TokenType.HAWL,
            "celi": TokenType.CELI,
            "qor": TokenType.qor,
            "gelin": TokenType.GELIN,
            "haddii": TokenType.HADDII,
            "haddii_kale": TokenType.HADDII_KALE,
            "ugudambeyn": TokenType.UGUDAMBEYN,
            "dooro": TokenType.DOORO,
            "xaalad": TokenType.XAALAD,
            "kuceli": TokenType.kuceli,
            "intay": TokenType.INTAY,
            "jooji": TokenType.JOOJI,
            "soco": TokenType.soco,
            "isku_day": TokenType.ISKU_DAY,
            "qabo": TokenType.QABO,
            "ka_keen": TokenType.KA_KEEN,
            "fasalka": TokenType.FASALKA,
            "ka_dhaxal": TokenType.KA_DHAXAL,
            "cusub": TokenType.CUSUB,
            "nafta": TokenType.NAFTA,
            # Static types
            "abn": TokenType.abn,
            "qoraal": TokenType.QORAAL,
            "bool": TokenType.BOOL,
            "teed": TokenType.teed,
            "walax": TokenType.WALAX,
            # Boolean literals
            "run": TokenType.TRUE,
            "been": TokenType.FALSE,
            "null": TokenType.NULL,
            "jajab": TokenType.JAJAB,
        }

    def advance(self):
        # Update line and column tracking
        if self.current_char == "\n":
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
            raise LexerError(
                "unterminated_comment", position=self.column, line=self.line
            )

        return False

    def peek(self):
        """Look at the next character without advancing"""
        peek_pos = self.position + 1
        if peek_pos >= len(self.source):
            return None
        return self.source[peek_pos]

    def tokenize_identifier(self):
        identifier = ""
        start_position = self.column
        start_line = self.line
        while self.current_char and (
            self.current_char.isalnum() or self.current_char == "_"
        ):
            identifier += self.current_char
            self.advance()

        # Check if it's a keyword
        token_type = self.KEYWORDS.get(identifier, TokenType.IDENTIFIER)
        return Token(token_type, identifier, line=start_line, position=start_position)

    def tokenize_number(self):
        number = ""
        start_position = self.column
        start_line = self.line
        has_decimal = False

        while self.current_char and (
            self.current_char.isdigit() or self.current_char == "."
        ):
            if self.current_char == ".":
                has_decimal = True
            number += self.current_char
            self.advance()

        # Convert to integer if no decimal point, otherwise float
        value = int(number) if not has_decimal else float(number)

        return Token(
            TokenType.NUMBER, value, line=start_line, position=start_position
        )

    def tokenize_string(self):
        quote_char = self.current_char
        start_position = self.column
        start_line = self.line
        self.advance()
        string_value = ""

        while self.current_char and self.current_char != quote_char:
            string_value += self.current_char
            self.advance()

        if self.current_char == quote_char:
            self.advance()
            return Token(
                TokenType.STRING, string_value, line=start_line, position=start_position
            )
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

            # Store current position and line before advancing
            current_position = self.column
            current_line = self.line

            if self.current_char == "+":
                self.advance()
                return Token(
                    TokenType.PLUS, "+", line=current_line, position=current_position
                )
            if self.current_char == "-":
                self.advance()
                return Token(
                    TokenType.MINUS, "-", line=current_line, position=current_position
                )
            if self.current_char == "*":
                self.advance()
                return Token(
                    TokenType.STAR, "*", line=current_line, position=current_position
                )
            if self.current_char == "/":
                self.advance()
                return Token(
                    TokenType.SLASH, "/", line=current_line, position=current_position
                )
            if self.current_char == "%":
                self.advance()
                return Token(
                    TokenType.MODULO, "%", line=current_line, position=current_position
                )
            if self.current_char == "=":
                self.advance()
                return Token(
                    TokenType.EQUAL, "=", line=current_line, position=current_position
                )
            if self.current_char == "(":
                self.advance()
                return Token(
                    TokenType.LEFT_PAREN,
                    "(",
                    line=current_line,
                    position=current_position,
                )
            if self.current_char == ")":
                self.advance()
                return Token(
                    TokenType.RIGHT_PAREN,
                    ")",
                    line=current_line,
                    position=current_position,
                )
            if self.current_char == "{":
                self.advance()
                return Token(
                    TokenType.LEFT_BRACE,
                    "{",
                    line=current_line,
                    position=current_position,
                )
            if self.current_char == "}":
                self.advance()
                return Token(
                    TokenType.RIGHT_BRACE,
                    "}",
                    line=current_line,
                    position=current_position,
                )
            if self.current_char == ">":
                self.advance()
                if self.current_char == "=":
                    self.advance()
                    return Token(
                        TokenType.GREATER_EQUAL,
                        ">=",
                        line=current_line,
                        position=current_position,
                    )
                return Token(
                    TokenType.GREATER, ">", line=current_line, position=current_position
                )
            if self.current_char == "<":
                self.advance()
                if self.current_char == "=":
                    self.advance()
                    return Token(
                        TokenType.LESS_EQUAL,
                        "<=",
                        line=current_line,
                        position=current_position,
                    )
                return Token(
                    TokenType.LESS, "<", line=current_line, position=current_position
                )
            if self.current_char == "!":
                self.advance()
                if self.current_char == "=":
                    self.advance()
                    return Token(
                        TokenType.NOT_EQUAL,
                        "!=",
                        line=current_line,
                        position=current_position,
                    )
                return Token(
                    TokenType.NOT, "!", line=current_line, position=current_position
                )
            if self.current_char == "&":
                self.advance()
                if self.current_char == "&":
                    self.advance()
                    return Token(
                        TokenType.AND,
                        "&&",
                        line=current_line,
                        position=current_position,
                    )
                raise LexerError(
                    "unexpected_char",
                    position=current_position,
                    line=current_line,
                    char="&",
                )
            if self.current_char == "|":
                self.advance()
                if self.current_char == "|":
                    self.advance()
                    return Token(
                        TokenType.OR, "||", line=current_line, position=current_position
                    )
                raise LexerError(
                    "unexpected_char",
                    position=current_position,
                    line=current_line,
                    char="|",
                )
            if self.current_char == ",":
                self.advance()
                return Token(
                    TokenType.COMMA, ",", line=current_line, position=current_position
                )
            if self.current_char == ":":
                self.advance()
                return Token(
                    TokenType.COLON, ":", line=current_line, position=current_position
                )
            if self.current_char == ";":
                self.advance()
                return Token(
                    TokenType.SEMICOLON,
                    ";",
                    line=current_line,
                    position=current_position,
                )

            # New tokens for lists and objects
            if self.current_char == "[":
                self.advance()
                return Token(
                    TokenType.LEFT_BRACKET,
                    "[",
                    line=current_line,
                    position=current_position,
                )
            if self.current_char == "]":
                self.advance()
                return Token(
                    TokenType.RIGHT_BRACKET,
                    "]",
                    line=current_line,
                    position=current_position,
                )
            if self.current_char == ".":
                self.advance()
                return Token(
                    TokenType.DOT, ".", line=current_line, position=current_position
                )

            raise LexerError(
                "unexpected_char",
                position=current_position,
                line=current_line,
                char=self.current_char,
            )

        return Token(TokenType.EOF, None, line=self.line, position=self.column)

    def tokenize(self):
        tokens = []
        while self.position < len(self.source):
            token = self.next_token()
            tokens.append(token)
            if token.type == TokenType.EOF:
                break
        return tokens
