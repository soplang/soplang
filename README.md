# 📌 Soplang Programming Language  
Soplang is a programming language designed with a **Somali-based syntax**, inspired by **Python and JavaScript**.  
This document covers **Soplang's syntax** and a comparison with **Python & JavaScript**.  

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
```somali
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
```somali
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
```somali
inta_ay (x < 10) {
    qor(x)
    x = x + 1
}
```

### **Loop Control**
- **Break**: `jooji`
- **Continue**: `sii_wad`

```somali
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
```somali
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

```somali
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
```somali
nuuc(value)  // Returns type as a string: "tiro", "qoraal", etc.
```

### **Type Conversion**
```somali
tiro("123")     // Converts to number
qoraal(456)     // Converts to string
labadaran(1)    // Converts to boolean
```

---

## **📌 Classes and Inheritance**
```somali
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
```somali
ka_keen "myfile.so"
```

---

## **📌 Comments**
Soplang supports both single-line and multi-line comments:

```somali
// This is a single-line comment

/* This is a 
   multi-line comment */
```

---

## **📌 Example Soplang Program**
```somali
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