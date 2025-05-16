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
VariableDeclaration ::= DynamicDeclaration | StaticDeclaration

DynamicDeclaration ::= "door" Identifier "=" Expression

StaticDeclaration ::= StaticType Identifier "=" Expression

StaticType ::= "tiro"      // Number
             | "qoraal"    // String
             | "labadaran" // Boolean
             | "liis"      // List/Array
             | "shey"      // Object
```

## Function Declarations

```ebnf
FunctionDeclaration ::= "howl" Identifier "(" [ParameterList] ")" Block

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

ElseClause ::= "haddii_kalena" Block
```

## Loops

```ebnf
LoopStatement ::= "ku_celi" Identifier "min" Expression "ilaa" Expression ["by" Expression] Block

WhileStatement ::= "inta_ay" "(" Expression ")" Block

BreakStatement ::= "jooji"

ContinueStatement ::= "sii_wad"
```

## Return Statement

```ebnf
ReturnStatement ::= "soo_celi" [Expression]
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

NullLiteral ::= "null" | "waxba"

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
             | "iskuxir"     // concat
             | "ka_kooban"   // contains
```

## Object Methods

```ebnf
ObjectMethod ::= "furaha"    // keys
               | "haystaa"   // has
               | "tir"    // remove
               | "iskudar"   // merge
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
