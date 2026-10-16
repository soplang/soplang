# Soplang expressions

Expressions produce values and can be combined with calls, indexing, and
property access.

## Values

- Numbers: `42`, `3.14`
- Strings: `"Soomaali"`
- Booleans: `run`, `been`
- Empty value: `maran`
- Lists: `[1, 2, 3]`
- Objects: `{ magac: "Ayaan", da: 25 }`
- Anonymous functions: `hawl(x) { celi x * 2 }`

`maran` is the canonical spelling of the empty value. `null` remains accepted
as a compatibility spelling.

## Operators

| Group | Operators |
| --- | --- |
| Arithmetic | `+`, `-`, `*`, `/`, `%` |
| Comparison | `==`, `!=`, `>`, `<`, `>=`, `<=` |
| Logical | `&&`, `\|\|`, `!` |
| Unary signs | `+`, `-` |

Normal arithmetic precedence applies: unary operators bind first, followed by
`*`, `/`, and `%`, then `+` and `-`, comparisons, and logical operators.

```soplang
door dhexda = 2 + 3 * 4
door ansax = dhexda >= 10 && dhexda != 15
qor(!ansax)
```

## Calls and access

```soplang
door tirooyin = [10, 20, 30]
qor(tirooyin[0])
qor(tirooyin.dherer())

door qof = { magac: "Ayaan" }
qor(qof.magac)

door magac = qoraal(25)
```

Use methods such as `jar()` for slices; bracket syntax accepts one index. List and string indexes may be negative, and string positions count Unicode characters. Objects accept string keys in brackets for computed reads and writes.

## Classes and functions

```soplang
door labanlaab = hawl(abn x): abn {
    celi x * 2
}

door qof = cusub Qof("Ayaan")
```

For the complete syntax, see [grammar.md](grammar.md). For keyword and built-in
references, see [keywords.md](keywords.md).
