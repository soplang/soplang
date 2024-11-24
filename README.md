# 📌 Soplang Programming Language  
Soplang is a programming language designed with a **Somali-based syntax**, inspired by **Python and JavaScript**.  
This document covers **Soplang's syntax** and a comparison with **Python & JavaScript**.  

---

## **🚀 Variable Declaration (`door`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `let x = 10;` (JS) | `door x = 10` |
| `x = 10` (Python) | `door x = 10` |

---

## **📌 Print Statement (`qor()`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `console.log("Hello")` (JS) | `qor("Hello")` |
| `print("Hello")` (Python) | `qor("Hello")` |

---

## **📌 If Statements (`haddii`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `if x > 5:` (Python) | `haddii (x > 5) {` |
| `if (x > 5) {` (JS) | `haddii (x > 5) {` |
| `elif x == 5:` (Python) | `haddii_kale (x == 5) {` |
| `else if (x == 5) {` (JS) | `haddii_kale (x == 5) {` |
| `else:` (Python) | `haddii_kalena {` |
| `else {` (JS) | `haddii_kalena {` |

---

## **📌 Loops (`ku_celi`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `for i in range(1, 6):` (Python) | `ku_celi i min 1 ilaa 5 {` |
| `for (let i = 1; i <= 5; i++)` (JS) | `ku_celi i min 1 ilaa 5 {` |

---

## **📌 Functions (`howl`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `def greet(name):` (Python) | `howl salaam(magac) {` |
| `function greet(name) {` (JS) | `howl salaam(magac) {` |

---

## **📌 Lists / Arrays (`liis`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `colors = ["red", "blue"]` (Python) | `liis midabada = ["casaan", "buluug"]` |
| `let colors = ["red", "blue"];` (JS) | `liis midabada = ["casaan", "buluug"]` |

---

## **📌 Objects / Dictionaries (`shey`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `person = {"name": "Ali", "age": 25}` (Python) | `shey qof = { "magac": "Ali", "da'": 25 }` |
| `let person = { name: "Ali", age: 25 };` (JS) | `shey qof = { "magac": "Ali", "da'": 25 }` |

---

## **📌 Error Handling (`qabo`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `try:` (Python) | `isku_day {` |
| `except:` (Python) | `qabo (k) {` |
| `try {}` (JS) | `isku_day {` |
| `catch (error) {}` (JS) | `qabo (k) {` |

---

## **📌 Null / None / Undefined (`waxba`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `None` (Python) | `waxba` |
| `null` (JS) | `waxba` |

---

## **📌 Inheritance (`ka_dhaxal`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `class Dog(Animal):` (Python) | `fasalka Ey ka_dhaxal Xoolo {` |
| `class Dog extends Animal {` (JS) | `fasalka Ey ka_dhaxal Xoolo {` |

---

## **📌 File Importing (`ka_keen`)**
| **Python / JavaScript** | **Soplang** |
|--------------------------|------------|
| `import math` (Python) | `ka_keen "math"` |
| `import fs from 'fs'` (JS) | `ka_keen "fs"` |

---

## **📌 Example Soplang Program**
```somali
door x = 10

haddii (x > 5) {
    qor("X waa weyn yahay!")
} haddii_kale (x == 5) {
    qor("X waa shan!")
} haddii_kalena {
    qor("X waa yar yahay!")
}

howl salaam(magac) {
    qor("Salaan, " + magac)
}

salaam("Sharafdin")
