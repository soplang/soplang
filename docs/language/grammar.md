# Soplang grammar (EBNF)

This compact grammar describes the syntax accepted by the Soplang parser. See
[keywords.md](keywords.md) for keyword meanings and examples.

## Program and statements

```ebnf
program          = { ";" } { statement { ";" } } .
statement        = declaration
                 | assignment
                 | function_def
                 | class_def
                 | if_stmt
                 | switch_stmt
                 | for_stmt
                 | while_stmt
                 | try_catch
                 | return_stmt
                 | import_stmt
                 | "jooji"
                 | "soco"
                 | expression_stmt .
block            = { ";" } { statement { ";" } } .
```

Semicolons are optional statement separators.

## Declarations and assignment

```ebnf
declaration      = "door" identifier "=" expression
                 | [ "madoor" ] type identifier "=" expression
                 | "madoor" identifier "=" expression .
type             = "abn" | "jajab" | "qoraal" | "bool" | "teed" | "walax"
                 | identifier .
assignment       = ( identifier | member | subscript ) "=" expression .
```

An identifier used as a type is a class name.

## Control flow

```ebnf
if_stmt          = "haddii" "(" expression ")" "{" block "}"
                   { ( "kale" | "haddii_kale" ) "(" expression ")" "{" block "}" }
                   [ ( "kale" | "ugudambeyn" ) "{" block "}" ] .
switch_stmt      = "dooro" "(" expression ")" "{"
                   { "xaalad" expression "{" block "}" }
                   [ ( "kale" | "ugudambeyn" ) "{" block "}" ]
                   "}" .
for_stmt         = "kuceli" "(" identifier expression "ilaa" expression
                   [ "::" expression ] ")" "{" block "}" .
while_stmt       = "intay" "(" expression ")" "{" block "}" .
try_catch        = ( "tijaabi" | "fasax" ) "{" block "}" "qabo" "(" identifier ")"
                   "{" block "}" .
return_stmt      = "celi" [ expression ] .
```

## Functions and classes

```ebnf
function_def     = "hawl" identifier function_tail .
function_expr    = "hawl" function_tail .
function_tail    = "(" [ parameters ] ")" [ ":" type ] "{" block "}" .
parameters       = parameter { "," parameter } .
parameter        = [ type ] identifier .
class_def        = "qaab" identifier [ "dhaxal" identifier ] "{"
                   { function_def }
                   "}" .
new_expr         = "cusub" identifier "(" [ arguments ] ")" .
```

## Imports and expressions

```ebnf
import_stmt      = "keen" string .
expression_stmt  = expression .
expression       = logical .
logical          = comparison { ( "&&" | "||" ) comparison } .
comparison       = additive { ( "==" | "!=" | ">" | "<" | ">=" | "<=" ) additive } .
additive         = multiplicative { ( "+" | "-" ) multiplicative } .
multiplicative   = unary { ( "*" | "/" | "%" ) unary } .
unary            = [ "!" | "+" | "-" ] postfix .
postfix          = primary { member | method_call | subscript } .
primary          = literal
                 | identifier
                 | call
                 | list_literal
                 | object_literal
                 | new_expr
                 | function_expr
                 | "(" expression ")" .
call             = identifier "(" [ arguments ] ")" .
method_call      = member "(" [ arguments ] ")" .
member           = "." ( identifier | "kuceli" ) .
subscript        = "[" expression "]" .
arguments        = expression { "," expression } .
list_literal     = "[" [ arguments ] "]" .
object_literal   = "{" [ object_entry { "," object_entry } ] "}" .
object_entry     = ( identifier | string ) ":" expression .
literal          = number | string | "run" | "been" | "maran" .
```

`tijaabi` is contextual: only `tijaabi {` starts a try block. It remains an
identifier in variable, function, call, and property positions.

`kale` is contextual after an if branch or inside a switch. `kale (condition)`
starts an additional conditional branch only when the closing parenthesis is
followed by `{`; `kale {` starts a fallback branch. Elsewhere `kale` remains an
identifier. `haddii_kale` and `ugudambeyn` remain compatibility spellings.

`kuceli` remains the numeric loop keyword. After a dot it is also a member
name, so `text.kuceli(3)` repeats a string. This exception does not reserve
other method names or change user-defined identifiers.
