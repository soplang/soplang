from src.lexer import Lexer
from src.parser import Parser
from src.interpreter import Interpreter

code = r"""
// This is a Soplang program showcasing its features

// ---------- Dynamic typing example ----------
door x = 10
qor("Dynamic x = " + qoraal(x))
x = "Now x is a string"  // Valid in dynamic typing
qor("Now x = " + x)

// ---------- Static typing examples ---------- 
tiro y = 20
qor("Static y = " + qoraal(y))

// Error will occur if uncommented - Type mismatch
/* 
y = "Cannot assign string to tiro"
*/

qoraal naam = "Sharafdin"
qor("Static naam = " + naam)

// Example of type conversion
tiro z = tiro("42")  // Convert string to number
qor("z = " + qoraal(z))

// ---------- Conditional examples ----------
haddii (x == "Now x is a string") {
    qor("Conditional: x is a string now!")
} haddii_kale (x == 10) {
    qor("Conditional: x is 10!")
} haddii_kalena {
    qor("Conditional: x is something else!")
}

// ---------- Try-catch error handling ----------
isku_day {
    tiro err_var = tiro("not a number")  // This will cause an error
    qor("This won't be printed!")
} qabo (err) {
    qor("Error caught: " + err)
}

// ---------- Loop example ----------
qor("Loop example:")
ku_celi i min 1 ilaa 5 {
    qor("i = " + qoraal(i))
    haddii (i == 3) {
        qor("Found 3, breaking loop")
        jooji
    }
}

// ---------- Function example ----------
howl labo_jibbaar(num) {
    qor("Doubling: " + qoraal(num))
    soo_celi num * 2
}

door result = labo_jibbaar(12)
qor("Result: " + qoraal(result))

// ---------- Class examples ----------
fasalka Xayawaan {
    howl dhawaaq() {
        qor("Generic animal sound")
    }
}

fasalka Ey ka_dhaxal Xayawaan {
    howl dhawaaq() {
        qor("Woof woof!")
    }
}

// Import example - uncomment to test
// ka_keen "examples/test.so"
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
try:
    inter.interpret(ast)
except Exception as e:
    print(f"\n❌ Error: {e}")
    print("Program execution failed.")
