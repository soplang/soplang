# 📌 Soplang Programming Language  
Soplang is a programming language with syntax inspired by Somali language terms. The name "Soplang" combines "Somali" and "Programming Language". It features a Python-like simplicity with JavaScript-like flexibility.  

## **Overview**

Soplang is designed with the following goals:

- **Inclusive Programming**: Making coding more accessible to Somali speakers
- **Educational Tool**: Serving as a bridge for learning programming concepts
- **Modern Features**: Supporting both static and dynamic typing, functional programming, and object-oriented concepts

## **Features**

- **Dual Typing System**: Both dynamic typing (`door`) and static typing (`tiro`, `qoraal`, etc.)
- **Somali-based Syntax**: Keywords are derived from Somali words
- **Control Flow**: Conditional statements and loops using Somali keywords
- **Functions**: First-class functions with parameters and return values
- **Data Structures**: Lists and dictionaries (objects) with methods
- **Error Handling**: Try-catch blocks for exception handling
- **Modern OOP**: Class-based object-oriented programming with inheritance
- **Modular Design**: Import system for code organization

## **Installation**

```bash
git clone https://github.com/sharafdin/soplang.git
cd soplang
```

## **Docker Setup**

Soplang can be run in a Docker container for a consistent development environment:

```bash
# Build and start Docker container
docker-compose build
docker-compose up -d

# Run the shell inside the container
docker-compose exec soplang python main.py

# Run a Soplang file inside the container
docker-compose exec soplang python main.py examples/hello_world.so

# Stop the container when done
docker-compose down
```

For more details, see the [Contributing Guide](docs/CONTRIBUTING.md).

## **Usage**

To run a Soplang program:

```bash
python main.py examples/01_basics.so
```

To start the interactive shell:

```bash
python main.py
```

## **Examples**

### **Hello World**
```js
// Simple hello world program
qor("Salaan, Adduunka!")  // Prints: Hello, World!
```

### **Variables**
```js
// Dynamic typing
door magac = "Sharafdin"
door da = 25

// Static typing
qoraal name = "Sharafdin"
tiro age = 25
```

### **Control Flow**
```js
door points = 85

haddii (points >= 90) {
    qor("Waxaad heshay A")  // You got an A
} haddii_kale (points >= 80) {
    qor("Waxaad heshay B")  // You got a B
} haddii_kalena {
    qor("Waxaad heshay C")  // You got a C
}
```

### **Loops**
```js
// For loop
qor("Tirooyinka 1 ilaa 5:")  // Numbers 1 to 5:
ku_celi i min 1 ilaa 5 {
    qor(i)
}

// While loop
door i = 0
inta_ay (i < 5) {
    qor(i)
    i = i + 1
}
```

### **Functions**
```js
howl isuGee(a, b) {
    soo_celi a + b
}

door natiijo = isuGee(5, 10)
qor("Isugeynta: " + qoraal(natiijo))  // Sum: 15
```

### **Data Structures**
```js
// Lists
door numbers = [1, 2, 3, 4, 5]
qor(numbers[2])  // Prints: 3

// Objects
door person = {
    "name": "Sharafdin",
    "age": 25,
    "isStudent": true
}
qor(person.name)  // Prints: Sharafdin
```

## **Documentation**

For a complete language reference, see the documentation in the `docs` folder:

- [Documentation Home](docs/index.md) - Start here for all documentation
- [Language Reference](docs/language/keywords.md) - Complete list of keywords and their meanings
- [Grammar Specification](docs/language/grammar.md) - Formal language specification
- [Examples](examples/) - Example programs to learn from
- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute to Soplang

## **Project Structure**

```
soplang/
│── src/                    # Source code
│   │── lexer.py            # Tokenizer
│   │── parser.py           # AST Parser
│   │── interpreter.py      # Interpreter
│   │── builtins.py         # Built-in methods
│   │── errors.py           # Custom Somali error messages
│   │── tokens.py           # Token definitions
│   │── ast.py              # AST Nodes
│   └── shell.py            # Interactive shell
│── examples/               # Example programs
│── tests/                  # Unit tests
│── docs/                   # Documentation
│   │── index.md            # Documentation home
│   │── CONTRIBUTING.md     # Contributing guide
│   │── language/           # Language reference
│   │── build/              # Build documentation
│   │── testing/            # Testing documentation
│   └── examples/           # Example documentation
│── main.py                 # Main entry point
│── soplang_shell.sh        # Shell launcher script
└── README.md               # This file
```

## **Contributing**

Contributions are welcome! Whether it's adding new features, fixing bugs, or improving documentation. See [Contributing Guide](docs/CONTRIBUTING.md) for details.

## **License**

This project is licensed under the MIT License - see the LICENSE file for details.

## **Acknowledgments**

- Inspired by the Somali language and culture
- Thanks to all contributors and supporters
- Website: [https://www.soplang.org/](https://www.soplang.org/)

---

## **🚀 Variable Declaration**
Soplang supports both dynamic and static typing:

### **Dynamic Typing (`door`)**
| **Python / JavaScript** | **Soplang**   |
| ----------------------- | ------------- |
| `let x = 10;` (JS)      | `door x = 10` |
| `x = 10` (Python)       | `door x = 10` |

### **Static Typing**
| **Type**    | **Soplang**          | **Description**             |
| ----------- | -------------------- | --------------------------- |
| Number      | `tiro x = 10`        | Declares a number variable  |
| String      | `qoraal x = "Hello"` | Declares a string variable  |
| Boolean     | `labadaran x = true` | Declares a boolean variable |
| List/Array  | `liis x = [1, 2, 3]` | Declares a list variable    |
| Object/Dict | `shey x = {"a": 1}`  | Declares an object variable |

Static typing enforces type checking at runtime:
```js
tiro x = 5
x = "Hello"  // ❌ ERROR: Cannot assign string to a number
```

---

## **📌 Print and Input**

### **Print (`qor()`)**
| **Python / JavaScript**     | **Soplang**    |
| --------------------------- | -------------- |
| `console.log("Hello")` (JS) | `qor("Hello")` |
| `print("Hello")` (Python)   | `qor("Hello")` |

### **User Input (`akhri()`)**
| **Python / JavaScript**    | **Soplang**       |
| -------------------------- | ----------------- |
| `input("Prompt")` (Python) | `akhri("Prompt")` |
| `prompt("Prompt")` (JS)    | `akhri("Prompt")` |

Example:
```js
qoraal magac = akhri("Fadlan magacaaga geli: ")
qor("Salaam, " + magac)
```

---

## **📌 If Statements (`haddii`)**
| **Python / JavaScript**   | **Soplang**              |
| ------------------------- | ------------------------ |
| `if x > 5:` (Python)      | `haddii (x > 5) {`       |
| `if (x > 5) {` (JS)       | `haddii (x > 5) {`       |
| `elif x == 5:` (Python)   | `haddii_kale (x == 5) {` |
| `else if (x == 5) {` (JS) | `haddii_kale (x == 5) {` |
| `else:` (Python)          | `haddii_kalena {`        |
| `else {` (JS)             | `haddii_kalena {`        |

---

## **📌 Loops (`ku_celi`)**
| **Python / JavaScript**             | **Soplang**                |
| ----------------------------------- | -------------------------- |
| `for i in range(1, 6):` (Python)    | `ku_celi i min 1 ilaa 5 {` |
| `for (let i = 1; i <= 5; i++)` (JS) | `ku_celi i min 1 ilaa 5 {` |

### **While Loops (`inta_ay`)**
```js
inta_ay (x < 10) {
    qor(x)
    x = x + 1
}
```

### **Loop Control**
- **Break**: `jooji`
- **Continue**: `sii_wad`

```js
ku_celi i min 1 ilaa 5 {
    haddii (i == 3) {
        jooji  // Exit the loop
    }
}
```

---

## **📌 Functions (`howl`)**
| **Python / JavaScript**       | **Soplang**            |
| ----------------------------- | ---------------------- |
| `def greet(name):` (Python)   | `howl salaam(magac) {` |
| `function greet(name) {` (JS) | `howl salaam(magac) {` |

### **Return Values (`soo_celi`)**
```js
howl laboJibbaar(n) {
    soo_celi n * 2
}
```

---

## **📌 Error Handling (`isku_day` / `qabo`)**
| **Python / JavaScript** | **Soplang**  |
| ----------------------- | ------------ |
| `try:` (Python)         | `isku_day {` |
| `except:` (Python)      | `qabo (k) {` |
| `try {}` (JS)           | `isku_day {` |
| `catch (error) {}` (JS) | `qabo (k) {` |

```js
isku_day {
    door result = 10 / 0
} qabo (err) {
    qor("Error: " + err)
}
```

---

## **📌 Type Checking and Conversion**
Soplang provides built-in type checking and conversion functions:

### **Type Checking**
```js
nuuc(value)  // Returns type as a string: "tiro", "qoraal", etc.
```

### **Type Conversion**
```js
tiro("123")     // Converts to number
qoraal(456)     // Converts to string
labadaran(1)    // Converts to boolean
```

---

## **📌 Classes and Inheritance**
```js
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
```

---

## **📌 Importing Files (`ka_keen`)**
```js
ka_keen "myfile.so"
```

---

## **📌 Comments**
Soplang supports both single-line and multi-line comments:

```js
// This is a single-line comment

/* This is a 
   multi-line comment */
```

---

## **📌 Example Soplang Program**
```js
// Basic Soplang program
tiro age = 25
qoraal name = "Sharafdin"

qor("Hello, " + name)
qor("You are " + qoraal(age) + " years old")

haddii (age > 18) {
    qor("You are an adult")
} haddii_kalena {
    qor("You are a minor")
}

ku_celi i min 1 ilaa 5 {
    qor("Count: " + qoraal(i))
}

howl salaam(qof) {
    soo_celi "Salaam, " + qof + "!"
}

qor(salaam(name))
```

For more examples, check the `examples/` directory in the repository.
