from src.lexer import Lexer
from src.parser import Parser
from src.interpreter import Interpreter

code = r"""
# Dynamic typing
door x = 10
qor(x + 5)

# Static typing
tiro y = 20
qoraal naam = "Sharafdin"
qor(naam + " " + qoraal(y))

haddii (x > 5) {
    qor("X waa weyn!")
} haddii_kale (x == 5) {
    qor("X waa shan!")
} haddii_kalena {
    qor("X waa yar yahay!")
}

# Try-catch example
isku_day {
    door z = 5 / 0
    qor("Never see this!")
} qabo (err) {
    qor("Khalad baa dhacay: " + err)
}

# Loop example
ku_celi i min 1 ilaa 5 {
    qor("i="+ i)
    haddii (i == 3) {
        jooji
    }
}

# Function example
howl showDouble(num) {
    qor(num * 2)
}
showDouble(12)

# Class examples
fasalka Xayawaan {
    # In a real language, we'd define methods or fields
    qor("Xayawaan fasal")
}

fasalka Ey ka_dhaxal Xayawaan {
    qor("Ey fasal")
}

ka_keen "another.sp"
"""

# 1) Tokenize
print("\n🔹 Running Soplang...")
lexer = Lexer(code)
tokens = lexer.tokenize()
print("\n🔹 Tokens Generated:")
for t in tokens:
    print(t)

# 2) Parse
parser = Parser(tokens)
ast = parser.parse()
print("\n🔹 Abstract Syntax Tree (AST):")
print(ast)

# 3) Interpret
inter = Interpreter()
print("\n🔹 Running the Soplang Interpreter:")
inter.interpret(ast)
