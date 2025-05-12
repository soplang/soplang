# Soplang Formal Grammar

This document provides the formal grammar specification for the Soplang programming language in Extended Backus-Naur Form (EBNF).

## Grammar Conventions

- `|` denotes alternatives
- `[ ]` denotes optional elements
- `{ }` denotes repetition (zero or more)
- `( )` denotes grouping
- `" "` denotes literals/terminals
- `'a'..'z'` denotes character ranges

## Program Structure

```ebnf
Program ::= Statement {Statement}
```

## Statements

```ebnf
Statement ::= VariableDeclaration
            | FunctionDeclaration
            | FunctionCall
            | IfStatement
            | LoopStatement
            | WhileStatement
            | BreakStatement
            | ContinueStatement
            | ReturnStatement
            | TryCatchStatement
            | ImportStatement
            | ClassDeclaration
            | AssignmentStatement
            | Block
            | ThrowStatement

Block ::= "{" {Statement} "}"
```

## Variable Declarations

```ebnf
VariableDeclaration ::= DynamicDeclaration | StaticDeclaration | ConstantDeclaration

DynamicDeclaration ::= "door" Identifier "=" Expression

StaticDeclaration ::= StaticType Identifier "=" Expression

ConstantDeclaration ::= "madoor" [StaticType] Identifier "=" Expression

StaticType ::= "tiro"      // Number
             | "jajab"     // Decimal/Float
             | "qoraal"    // String
             | "bool"      // Boolean
             | "liis"      // List/Array
             | "walax"     // Object
```

## Function Declarations

```ebnf
FunctionDeclaration ::= "hawl" Identifier "(" [ParameterList] ")" Block

ParameterList ::= Identifier {"," Identifier}
```

## Function Calls

```ebnf
FunctionCall ::= Identifier "(" [ArgumentList] ")"
               | MethodCall

MethodCall ::= ObjectProperty "(" [ArgumentList] ")"

ArgumentList ::= Expression {"," Expression}
```

## Control Flow

```ebnf
IfStatement ::= "haddii" "(" Expression ")" Block
               {ElseIfClause}
               [ElseClause]

ElseIfClause ::= "haddii_kale" "(" Expression ")" Block

ElseClause ::= "ugudambeyn" Block
```

## Switch Statement

```ebnf
SwitchStatement ::= "dooro" "(" Expression ")" "{" {CaseClause} [DefaultClause] "}"

CaseClause ::= "xaalad" Expression Block

DefaultClause ::= "ugudambeyn" Block
```

The `dooro` statement evaluates an expression and matches its value against different `xaalad` clauses. If a match is found, the code block associated with that case is executed. The optional `ugudambeyn` clause acts as a default case that executes when no other cases match.

Example:
```
dooro (language) {
  xaalad "Somali" {
    bandhig("Salaan, Adduunka!")
  }
  xaalad "English" {
    bandhig("Hello, World!")
  }
  ugudambeyn {
    bandhig("Unknown language")
  }
}
```

## Loops

```ebnf
LoopStatement ::= "kuceli" "(" Identifier Expression "ilaa" Expression ["::" Expression] ")" Block

WhileStatement ::= "intay" "(" Expression ")" Block

BreakStatement ::= "jooji"

ContinueStatement ::= "soco"
```

## Return Statement

```ebnf
ReturnStatement ::= "celi" [Expression]
```

## Throw Statement

```ebnf
ThrowStatement ::= "throw" Expression
```

## Try-Catch

```ebnf
TryCatchStatement ::= "isku_day" Block "qabo" "(" Identifier ")" Block
```

## Import

```ebnf
ImportStatement ::= "ka_keen" StringLiteral
```

## Classes

```ebnf
ClassDeclaration ::= "fasalka" Identifier ["ka_dhaxal" Identifier] Block
```

## Object-Oriented Features

```ebnf
ObjectProperty ::= Expression "." Identifier

IndexAccess ::= Expression "[" Expression "]"
```

## Assignment

```ebnf
AssignmentStatement ::= (Identifier | ObjectProperty | IndexAccess) "=" Expression
```

## Expressions

```ebnf
Expression ::= LogicalExpression

LogicalExpression ::= ComparisonExpression {LogicalOperator ComparisonExpression}

ComparisonExpression ::= ArithmeticExpression {ComparisonOperator ArithmeticExpression}

ArithmeticExpression ::= Term {AdditiveOperator Term}

Term ::= Factor {MultiplicativeOperator Factor}

Factor ::= NumberLiteral
        | StringLiteral
        | BooleanLiteral
        | NullLiteral
        | Identifier
        | FunctionCall
        | ObjectProperty
        | IndexAccess
        | ListLiteral
        | ObjectLiteral
        | NewExpression
        | "(" Expression ")"
        | UnaryOperator Factor
```

### Expression Evaluation Rules

1. Expressions can include comparison operators (`>`, `<`, `>=`, `<=`, `==`, `!=`) without requiring additional parentheses
2. Logical expressions with `&&` (AND) and `||` (OR) can be used directly in assignments and conditions
3. Complex expressions can be used in any context where a value is expected:
   - Variable assignments: `door x = a > b`
   - Function arguments: `my_function(a >= 10)`
   | Switch conditions: `dooro (score >= 80)`
   - Array indices: `my_list[i > 0 ? 1 : 0]`
   - Object property values: `walax obj = { is_valid: x == 10 }`

Example of a complex expression in an assignment:

```
door is_adult = age >= 18 && has_id == run
```

Example of a comparison expression in a switch statement:

```
dooro (score >= 90) {
  xaalad run {
    bandhig("Excellent!")
  }
  xaalad been {
    bandhig("Keep trying!")
  }
}
```

## New Expression

```ebnf
NewExpression ::= "cusub" Identifier "(" [ArgumentList] ")"
```

## Operators

```ebnf
LogicalOperator ::= "&&" | "||"

ComparisonOperator ::= "==" | "!=" | ">" | "<" | ">=" | "<="

AdditiveOperator ::= "+" | "-"

MultiplicativeOperator ::= "*" | "/" | "%"

UnaryOperator ::= "!" | "-"
```

## Literals

```ebnf
NumberLiteral ::= Digit {Digit} ["." Digit {Digit}]

StringLiteral ::= '"' {Character} '"' | "'" {Character} "'"

BooleanLiteral ::= "true" | "false" | "run" | "been"

NullLiteral ::= "null" | "maran"

ListLiteral ::= "[" [Expression {"," Expression}] "]"

ObjectLiteral ::= "{" [PropertyAssignment {"," PropertyAssignment}] "}"

PropertyAssignment ::= (Identifier | StringLiteral) ":" Expression
```

## Basic Elements

```ebnf
Identifier ::= (Letter | "_") {Letter | Digit | "_"}

Letter ::= 'a'..'z' | 'A'..'Z'

Digit ::= '0'..'9'

Character ::= Letter | Digit | Symbol | Whitespace

Symbol ::= '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '-' | '+' | '=' | '{' | '}' | '[' | ']' | ':' | ';' | '"' | "'" | '<' | '>' | ',' | '.' | '?' | '/' | '\\' | '|' | '`' | '~'

Whitespace ::= ' ' | '\t' | '\n' | '\r'
```

## List Methods

```ebnf
ListMethod ::= "kudar"       // push
             | "kasaar"      // pop
             | "dherer"      // length
             | "kudar"       // concat
             | "leeyahay"    // contains
             | "nuqul"       // copy
             | "nadiifi"     // clear
             | "rog"         // reverse
             | "habee"       // sort
             | "jar"         // slice
             | "aaddin"      // map/transform
             | "shaandhee"   // filter
             | "raadso"      // find index
```

## Object Methods

```ebnf
ObjectMethod ::= "fure"      // keys
               | "leeyahay"  // has
               | "tirtir"    // remove
               | "kudar"     // merge
               | "nuqul"     // copy
               | "nadiifi"   // clear
```

## Error Messages

```ebnf
ErrorMessage ::= "Khalad" ErrorType ":" ErrorDetail ["ee" "sadar" LineNumber "," "goobta" Position]

ErrorType ::= "lexer" | "parser" | "runtime" | "type" | "import"

ErrorDetail ::= {Character}

LineNumber ::= Digit {Digit}

Position ::= Digit {Digit}
```

## Comments

```ebnf
Comment ::= SingleLineComment | MultiLineComment

SingleLineComment ::= "//" {Character} '\n'

MultiLineComment ::= "/*" {Character} "*/"
```
