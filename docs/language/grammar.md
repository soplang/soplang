# Soplang grammar (EBNF)

Formal grammar for Soplang syntax. This is a compact reference; see [keywords.md](keywords.md) for usage.

## Program

```
program     = { statement } .
statement   = ( declaration | assignment | control | expression_stmt | return_stmt | import_stmt ) .
```

## Declarations and assignment

```
declaration = ( "door" | "madoor" | "abn" | "qoraal" | "tiro" | "walax" ) identifier [ type_annot ] "=" expression .
assignment  = ( identifier | member | subscript ) "=" expression .
```

## Control flow

```
control     = if_stmt | for_stmt | while_stmt | try_catch | dooro_stmt .
if_stmt     = "haddii" expression "{" block "}" [ "laakiin" "{" block "}" ] .
for_stmt    = "weli" identifier "=" expression "::" expression "{" block "}" .
while_stmt  = "ilaa" expression "{" block "}" .
```

## Functions and classes

```
function_def = "hawl" identifier "(" [ params ] ")" [ ":" type_annot ] "{" block "}" .
function_expr = "hawl" "(" [ params ] ")" [ ":" type_annot ] "{" block "}" .
params      = param { "," param } .
param       = identifier [ ":" type_annot ] .
class_def   = "nooc" identifier "{" { member_def } "}" .
```

## Expressions

```
expression  = binary | unary | primary .
binary      = primary ( op primary )* .
primary     = literal | identifier | call | member | subscript | function_expr | "(" expression ")" .
call        = ( identifier | primary "." identifier ) "(" [ args ] ")" .
```

(Simplified; full grammar may be extended in the language spec.)
