# Text methods

Strings remain immutable UTF-8 text. Existing indexing, slicing, length, search,
case conversion, trimming, replacement, counting, joining, and formatting keep
their existing behavior. The following operations complete the 3.2 text-library
stage in both Cranelift JIT and LLVM-built programs.

## Classification

`text.hubi(kind)` returns a boolean. The kind must be one of these strings:

| Kind | Condition |
| --- | --- |
| `"xarfo"` | Every scalar has the Unicode Alphabetic property. |
| `"tiro"` | Every scalar is Unicode numeric, including numeric letters and fractions. |
| `"xarfo iyo tiro"` | Every scalar is alphabetic or numeric. |
| `"bannaan"` | Every scalar is Unicode whitespace. |
| `"waaweyn"` | At least one cased scalar exists and all cased scalars are uppercase. |
| `"yaryar"` | At least one cased scalar exists and all cased scalars are lowercase. |
| `"cinwaan"` | At least one cased scalar exists and title conversion leaves the text unchanged. |
| `"ascii"` | All scalars are ASCII. |

The first seven predicates return false for empty text; `"ascii"` returns true.
Uncased characters are ignored by the upper/lower predicates. Numeric
classification does not parse a number: `"½"` is numeric, while `"1.5"` and
`"-1"` are not. Combining marks follow their Unicode properties individually;
text is not normalized before classification.

```soplang
qor("Soplang".hubi("xarfo"))        // run
qor("١٢".hubi("tiro"))             // run
qor("SOPLANG 3".hubi("waaweyn"))    // run
```

## Capitalization and title casing

`weynee()` keeps its existing full-uppercase behavior. Two optional modes add
capitalization and title casing:

| Call | Result |
| --- | --- |
| `text.weynee("hore")` | Titlecase the first scalar and lowercase the remaining text. |
| `text.weynee("eray")` | For each Unicode word segment, titlecase its first cased scalar and lowercase the remainder. |

```soplang
qor("sALAAN SOPLANG".weynee("hore")) // Salaan soplang
qor("sALAAN, sOPLANG".weynee("eray")) // Salaan, Soplang
qor("ßABC".weynee("hore"))          // Ssabc
```

Capitalization begins at the first scalar, including leading punctuation or
whitespace. Title casing preserves separators and uses Unicode word boundaries,
including internal apostrophes. Case conversion is language-neutral and supports
multi-scalar mappings, titlecase digraphs, and contextual lowercase final sigma.
It does not apply locale-specific rules or normalize text. Word boundaries are
computed from the original text; a mapping that changes a combining mark into
a letter can change the boundaries seen by a later conversion.

The implementation uses Unicode 17 casing data and word segmentation. Titlecase
exceptions are generated from the official [Unicode character data](https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt)
and [special casing mappings](https://www.unicode.org/Public/17.0.0/ucd/SpecialCasing.txt).
Word boundaries follow [Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/).
The earlier uppercase aliases still accept no arguments. `yaree()` is unchanged.

## Padding

`text.buuxi(width, fill?, direction?)` pads to a minimum Unicode scalar width.
The default fill is one space; the default direction is `"dambe"`.

| Direction | Behavior |
| --- | --- |
| `"dambe"` | Add padding after the text. |
| `"hore"` | Add padding before the text. |
| `"dhexe"` | Split padding on both sides, placing an odd extra scalar after the text. |

```soplang
qor("é".buuxi(3, "-", "hore")) // --é
qor("x".buuxi(4, "-", "dhexe")) // -x--
```

Width must be an integer from 0 through 1,000,000. Fill must be exactly one
Unicode scalar. Combining sequences are multiple scalars, so a grapheme with
multiple scalars cannot be a single fill. Width counts scalars, not UTF-8 bytes,
grapheme clusters, or terminal display columns. Existing text is never truncated.
Arguments are validated even when the text already meets the requested width.
Capacity overflow and allocation failure return catchable errors.

## Partitioning

`text.kalaqaad(separator, direction?)` returns
`xirmo(before, separator, after)`. The separator must be nonempty text.
The default direction, `"hore"`, selects the first match. `"dambe"` selects
the last match, including overlapping candidate matches.

```soplang
qor("a=b=c".kalaqaad("="))          // xirmo(a, =, b=c)
qor("a=b=c".kalaqaad("=", "dambe")) // xirmo(a=b, =, c)
```

When absent, forward partitioning returns `xirmo(text, "", "")`; reverse
partitioning returns `xirmo("", "", text)`. Matching uses exact UTF-8 text
without normalization. Each result position is a string and can be annotated
with `xirmo[qoraal, qoraal, qoraal]`.

## Words and lines

`text.qeybi()` splits on Unicode whitespace, discarding empty fields.
Passing one explicit separator keeps the earlier behavior, including empty
fields and value-to-string conversion for that separator. `qeybi("")` retains
its existing scalar splitting with leading and trailing empty fields.
Whitespace tokenization differs from Unicode word segmentation used for title
casing; punctuation remains part of a whitespace token.

`text.sadarro(keep_endings?)` returns a list of lines. The optional flag is a
boolean, defaulting to `been`. It recognizes LF, CRLF, CR, vertical tab, form
feed, file/group/record separators, NEL, line separator, and paragraph separator.
CRLF is one boundary. Empty text yields an empty list; consecutive boundaries
yield blank lines. A final boundary does not create an additional trailing line.
With `run`, each line retains its exact original ending.

```soplang
qor(e" a\t b\n".qeybi())     // [a, b]
qor(e"a\r\nb\n".sadarro()) // [a, b]
```

## Errors and compatibility

Invalid classification kinds, casing modes, widths, fills, directions,
partition separators, and line flags raise catchable `E042` errors with Somali
messages and original source locations. Wrong argument counts retain the shared
method-arity diagnostics. Existing zero-argument `weynee()` and one-argument
`qeybi(separator)` calls keep their results. Formatting remains available through
`qaabee`, templates, and interpolated strings.

See the [public example](../../examples/41_text_methods.sop), the
[shared regression fixture](../../tests/fixtures/text_methods.sop), and
[verification notes](../testing/LANGUAGE_3_2.md).
