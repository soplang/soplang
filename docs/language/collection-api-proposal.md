# Shared collection vocabulary for Soplang 3

Status: **Approved vocabulary and operation contracts implemented for Soplang 3**.

The shared collection core, addition and combination methods, and simplified
string, numeric, and object names below are implemented. Earlier spellings
remain supported. Lists and strings also share integer slicing bounds,
first/last access, and occurrence counting. Slicing rejects fractional bounds
instead of truncating them; reversed string slices return an empty string.

This proposal organizes public methods around operations shared by lists,
objects, and strings. It records fewer preferred names, predictable return
values, and small argument-based variations. Earlier public names remain
supported throughout the Soplang 3 line.
Behavioral corrections are recorded in the migration guide.

The [language reference](keywords.md) describes supported behavior. The
[terminology review](terminology.md) records source evidence and previously
approved names. This proposal does not silently supersede those decisions.

The behavior audit uses Soplang commit `1c2ed05`, particularly
[collection implementations](../../src/runtime/stdlib.rs),
[runtime dispatch and indexing](../../src/runtime/abi.rs), and
[member parsing](../../src/frontend/parser.rs).

## Naming principles

1. Use one preferred name for the same operation across receiver types.
2. Prefer a short, familiar Somali word. Join a phrase when its additional
   meaning is necessary; deleting underscores alone is not sufficient.
3. Use arguments for variations of one operation, such as removing the last
   list item or removing an item at a particular index.
4. Preserve distinctions between reading, testing, assigning, inserting, and
   removing. Similar implementation code does not make these the same task.
5. State mutation and return behavior as part of the method contract.
6. Do not add an operation to a receiver where it has no useful meaning.
7. Continue allowing underscores in user-defined identifiers.

These are programming API conventions. Joined identifiers are not claims about
standard Somali prose spelling. SLS sources in the terminology review provide
linguistic evidence, not approval of these complete computing APIs.

## Operation matrix

An em dash means the method is outside this proposal for that receiver.
The shared methods below are implemented. Lists and strings both support
`hore()`, `dambe()`, and `tiri(value)`. Earlier list endpoint spellings remain
available.

| Operation | Preferred name | List | Object | String |
| --- | --- | --- | --- | --- |
| Count entries or characters | `dherer()` | Item count | Key count | Unicode character count |
| Test membership | `leeyahay(x)` | Value exists | Key exists | Substring exists |
| Read an entry | `hel(x[, default])` | Item at integer index | Value at string key | Character at integer index |
| Assign an entry | `dhig(x, value)` | Replace item at an existing index | Insert or replace key/value | —; strings remain immutable |
| Remove an entry | `saar([x])` | Pop last item, or remove indexed item | Remove keyed entry; key required | —; text editing returns new strings |
| Clear contents | `nadiifi()` | Clear in place | Clear in place | —; assigning an empty string is sufficient |
| Copy | `nuqul()` | New shallow list | New shallow object | —; strings are immutable values |
| Read first/last | `hore()`, `dambe()` | First/last item | —; no positional entry order | First/last Unicode character |
| Search for a position | `raadi(x)` | First matching value's index | —; `hel` already reads by key | First matching substring's character index |
| Slice | `jar(start?, end?)` | New shallow list slice | — | New character-based string slice |
| Count matches | `tiri(x)` | Number of equal values | —; a unique key needs membership only | Non-overlapping substring matches |

Shared names have shared purposes. Receiver differences must be explicit:
objects use string keys, lists and strings use integer positions, and object
membership concerns keys rather than values.

Implementation coverage:

- `dherer()` exists on all three receivers; global `dherer(value)` remains.
- `hel()` exists on all three receivers; lists and objects support `dhig()`.
  Existing bracket access remains available.
- `saar()` and `saar(index)` remove list items; `saar(key)` removes object keys.
- `nadiifi()`, `nuqul()`, and `leeyahay()` already share vocabulary where
  applicable.
- List `tiri(value)` counts equal items; string `tiri(sub)` counts non-overlapping
  substring matches. Empty substring counting remains an error.
- `hore()` and `dambe()` return a list value or Unicode character, and `maran`
  for empty receivers. Neither method mutates its receiver.

## Approved shared core contracts

### Read: `hel`

- Accept one position/key and an optional default value.
- Return the entry when present, including an entry whose value is `maran`.
- Return the supplied default, or `maran`, when the position/key is absent.
- Accept negative list/string indexes, counted from the end.
- Require integer indexes and string object keys; invalid types raise errors.
- Preserve the distinction between an absent key and a key storing `maran`:
  use `leeyahay(key)` to test presence.
- Evaluate arguments normally; the default expression is evaluated even if an
  entry exists. Lazy defaults would be a separate API decision.

Lists, objects, and strings support `hel`. The methods supplement bracket reads.

### Assign: `dhig`

- Mutate the list or object and return the receiver.
- A list position must already exist; support negative indexes.
- An object key may be new or already present.
- Reject invalid types and out-of-range list positions before mutation.
- Do not resize a list implicitly. Insertion shifts items and remains a
  separate operation, `geli(index, value)`.

Lists and objects support this method. Bracket assignment remains available.

### Remove: one `saar` family

| Call | Result | Failure behavior |
| --- | --- | --- |
| `list.saar()` | Remove and return the last item | Empty list raises a catchable error, preserving current pop behavior. |
| `list.saar(index)` | Remove and return the indexed item; later items shift left | Invalid index type or out-of-range position raises a catchable error. |
| `object.saar(key)` | Remove and return the keyed value | Missing key returns `maran`; invalid key type raises an error. |

Negative list indexes work as they do for existing indexed removal. This is one
operation with receiver-specific addressing, not separate canonical names for
pop and indexed removal. No removal-by-value overload is proposed: a numeric
list item would otherwise be ambiguous with an index.

Both `kasaar()` and `ka_saar(index)` remain supported with their existing
argument counts. Their shared replacement is `saar`; `kasaar` retains its
existing contract.

### Clear and copy

`nadiifi()` mutates and returns the receiver. `nuqul()` returns a different
outer collection while retaining shared nested lists and objects. This matches
existing list and object behavior; deep copying requires a separate design,
including cycles.

### Read, search, and presence are distinct

| Call | Question answered | Result |
| --- | --- | --- |
| `object.leeyahay("magac")` | Is this key present? | Boolean |
| `object.hel("magac")` | What value is stored? | Value or default |
| `object.saar("magac")` | What value was removed? | Removed value or `maran` |
| `list.raadi(value)` | At which index does this value first occur? | Index or `maran` |

List `raadi` is implemented alongside `muuji`. String
`raadi` currently returns `-1` when missing, whereas list lookup returns
`maran`. Preserve both behaviors for compatibility and document this exception.
Do not change either return value as part of naming consolidation.

### Slicing: `jar`

Both lists and strings accept `jar()`, `jar(start)`, and `jar(start, end)`.
Omitted bounds mean the beginning and the end. Bounds must be integers; negative
values count from the end and both endpoints are clamped to the collection.
The start is included and the end excluded. Equal or reversed normalized bounds
return an empty result. String positions count Unicode characters.

Lists return a new shallow outer list, even for `jar()`; strings return a new
string value. Neither receiver is mutated. Previously accepted fractional bounds
now report catchable errors, and reversed string bounds no longer panic. The
migration guide records this intentional consistency change.

## Addition and combination

| Preferred call | Effect | Result |
| --- | --- | --- |
| `list.geli(value)` | Append exactly one value, including a nested list | Mutated receiver |
| `list.geli(index, value)` | Insert at an integer position; shift later items | Mutated receiver |
| `list.midee(other_list)` | Combine list contents without changing either input | New shallow list |
| `object.midee(other_object)` | Merge keys without changing either input; right side wins | New shallow object |
| `separator.xiriiri(list)` | Join string representations using the separator | New string |

`geli` describes entry insertion; the optional position changes where it occurs.
`midee` describes combining collections and has the same copy behavior for lists
and objects. `xiriiri` describes connecting text with a separator. Calling
`midee` with the receiver itself is supported. Copies share nested collections.

Legacy `kudar` retains its behavior: a scalar appends to a list in place, a list
argument returns a concatenated copy, an object argument returns a merged copy,
and a string receiver joins a list. It is not a direct alias of `geli` because
passing a list has different meaning.

## Simplified names and variations

| Earlier API | Preferred API | Contract |
| --- | --- | --- |
| `haddii_kale`, `ugudambeyn` | Contextual `kale` | Additional conditional branch or fallback; ordinary identifiers preserved. |
| `ugu_yar`, `ugu_weyn` | `yar`, `weyn` | Numeric minimum and maximum. |
| List `kasaar`, `ka_saar`; object `ka_qaad` | `saar` | Removal by receiver-specific addressing; contracts above. |
| `koowaad`, `ugu_dambeeya` | `hore`, `dambe` | First/last list value or `maran`. |
| `isku_gee(callback, initial)` | `ururi(callback, initial)` | General left fold; callback receives accumulator and next value. |
| `xarfaha_waaweyn`, `xarafaha_weyn` | `weynee()` | Unicode uppercase conversion. |
| `xarfaha_yaryar` | `yaree()` | Unicode lowercase conversion. |
| `masax_bilow()` | `masax("hore")` | Leading whitespace only. |
| `masax_dhamaad()`, `masax_dhammaad()` | `masax("dambe")` | Trailing whitespace only; `masax()` still trims both ends. |
| `raadi_dambe(sub)` | `raadi(sub, "dambe")` | Last character position or `-1`; omitted direction or `"hore"` searches first. |
| `beddel_dhammaan(old, new)` | `beddel(old, new, maran)` | Replace all; omitted count replaces first, nonnegative integer limits replacements. |
| `ku_celi(count)` | `kuceli(count)` | Repeat text; parser accepts the loop keyword after a dot. |
| `dhammeystir(value)` | `duub(value)` | Round nearest, with halfway values away from zero. |
| `fure()`, `qiime()`, `lamaane()` | `furayaal()`, `qiimayaal()`, `lammaanayaal()` | Return all keys, values, or pairs in sorted-key order. |

Direction arguments must be `"hore"` or `"dambe"`; other values report catchable
errors. Replacement and repetition counts must be nonnegative integers. A zero
replacement count leaves the text unchanged. Strings remain immutable. `xidid`
keeps its spelling and is documented specifically as square root.

`ururi` can multiply numbers, build strings, or return other values. Map,
filter, fold, every, and some validate callbacks even for empty lists. They
iterate a shallow snapshot, so a callback may change the original collection.
Errors propagate unchanged and stop further callback calls. Every/some short
circuit normally. Callbacks still share nested mutable values.

## Compatibility and parsing

Every earlier spelling remains available; no removal release is scheduled.
`tir(key)` continues to return the receiver, while `saar(key)` returns the
removed value. Canonical methods preserve existing custom class method lookup
precedence. New method names do not become globally reserved identifiers.
`tijaabi` and `kale` are contextual; `kuceli` retains its loop meaning and is
also accepted as a member name after a dot.

These are Soplang API decisions informed by descriptive SLS evidence. They do
not claim approval by an SLS computing standard. The terminology review records
rationale and sources for the selected names.

## Example

```soplang
door tirooyin = [10, 20]
tirooyin.geli([30, 40])  // one nested item
qor(tirooyin.midee([50]))
qor(tirooyin.saar())
qor(tirooyin.ururi(hawl(a, b) { celi a + b }, 0))
door qof = { magac: "Ayaan", da: 25 }
qof.dhig("da", 26)
qor(qof.furayaal())
qor(" / ".xiriiri(["Ayaan", "Soplang"]))
qor("  Salaan  ".masax("hore"))
```

## Verification

Shared fixtures run against Cranelift JIT and standalone LLVM binaries. Coverage
includes earlier spellings, negative positions, nested values, self-combination,
Unicode text, invalid arguments, callback errors and mutation, contextual keyword
collisions, inheritance, and custom class methods.

The consistency fixtures cover shared slicing bounds, search-miss compatibility,
first/last characters, and occurrence counting. Global-update fixtures also
check changes from functions, callbacks, methods, and loop indexes, plus typed
writes and rejected assignments. REPL coverage checks updates across snippets.
All work belongs to Soplang 3.
