# Runtime error vocabulary after 3.0.1

This inventory records the runtime-foundations additions to the Somali
wording review. These forms are implemented in the
compiler and runtime; their wording can be reviewed without changing the
language's error behavior. Documentation explanations remain in English.

## Syntax and public names

| Somali form | Meaning |
| --- | --- |
| `tuur fariin` | Raise an error message or structured error. |
| `dhammee { ... }` | Run guaranteed cleanup after protected execution. |
| `khalad(fariin, summad?, sabab?)` | Construct an immutable error. |
| `qabo (fariin, xog)` | Bind the message and, optionally, its structured details. |
| `fariin` | Message. |
| `summad` | Error code. |
| `sabab` | Original cause. |
| `fayl` | Source filename. |
| `saf` | One-based source line. |
| `tiir` | One-based source column. |
| `raad` | Function call trace. |
| `magac` | Function name within a trace entry. |

## Exact new diagnostic wording

| Somali wording | English meaning |
| --- | --- |
| `Qiimahan ma jiro ama wuxuu ka tirsan yahay fulin kale.` | This value no longer exists or belongs to another execution context. |
| `Aqoonsiyada qiimayaasha way dhammaadeen.` | Allocation identities have been exhausted; internal checked counter limit. |
| `Qiimaha 'tuur' waa inuu noqdaa qoraal ama khalad.` | The raised value must be a string or an error. |
| `Fariinta khalad() waa inay noqotaa qoraal.` | The error message must be a string. |
| `Summadda khalad() waa inay noqotaa qoraal ama maran.` | The error code must be a string or null. |
| `Sababta khalad() waa inay noqotaa khalad ama maran.` | The cause must be another error or null. |
| `Xogta khaladka lama beddeli karo.` | Error details cannot be modified. |
| `Furaha khaladku waa inuu noqdaa qoraal.` | An error-property key must be a string. |
| `Magacyada 'qabo' waa inay kala duwanaadaan.` | The catch binding names must differ. |
| `Waxaa la filayay 'qabo' ama 'dhammee' ka dib 'tijaabi'.` | A handler or cleanup block must follow the protected body. |
| `Raadka wicitaannada:` | Call trace heading. |
| `Sababta:` | Cause heading. |
| `Sababaha kale waa la soo gaabiyey.` | Additional causes have been truncated for bounded rendering. |
| `Liiska doodaha hawsha ma saxna.` | The compiled call's argument vector is invalid. |
| `Meesha qiime lagu qorayo waa inay noqotaa doorsame, tusmo ama sifo.` | An assignment target must be a variable, indexed item, or property. |
| `Xarafka ka dambeeya '\' lama aqoonsan: {character}.` | The character after a backslash is not a supported escape. |
| `Xaraf waa inuu ka dambeeyaa calaamadda '\'.` | A backslash must be followed by a character. |
| `Koodhka Unicode ee qoraalku ma saxna.` | A Unicode escape does not name a valid scalar value. |
| `Tallaabada kuceli ma beddelayso qiimaha hadda.` | The floating-point step cannot advance toward the endpoint. |

The former diagnostics limiting compiled functions to eight ABI values have
been removed. Argument counts are still validated against each signature.

## Formatting vocabulary and diagnostics

`qaabee` means format or give a shape to a value. The same verb names the
global value formatter and the string template method.

| Somali wording | English meaning |
| --- | --- |
| `Qaabka qaabee() waa inuu noqdaa qoraal.` | The format descriptor must be a string. |
| `Qaabka qoraalka '{format}' lama aqoonsan.` | The format descriptor is unsupported or malformed. |
| `Qaabka qoraalku wuxuu ka badan yahay xadka la oggol yahay.` | Width or precision exceeds its limit. |
| `Qaabka '{format}' wuxuu u baahan yahay tiro.` | This descriptor requires a number. |
| `Qaabka '{format}' wuxuu u baahan yahay abn.` | This descriptor requires an integer. |
| `Qaabka 's' wuxuu u baahan yahay qoraal.` | The string descriptor requires a string. |
| `Saxnaanta laguma sheegi karo qaabka abn.` | Precision cannot be specified for integer formatting. |
| `Saxnaantu waxay u baahan tahay qaab qoraal ama jajab.` | Precision requires a string or float format. |
| `Qaabka qoraalku ma aqbalo '{' gudaha meel qiime lagu gelinayo.` | Nested replacement fields are unsupported. |
| `Qaybta qiimaha lagu gelinayo qoraalka lama xidhin.` | A replacement field was not closed. |
| `Qiime looma helin meesha '{name}' ee qoraalka.` | No value was supplied for this placeholder. |
| `Qaabka qoraalku ma isku dari karo meelgelin toos ah iyo mid lambar leh.` | Automatic and numbered placeholders cannot be mixed. |
| `Calaamadda '}' ee qoraalka ma laha '{' u dhiganta.` | A closing brace has no matching opening brace. |

Interpolation reuses the brace and descriptor messages above. Its lexer also
adds these forms under `E005`:

| Somali wording | English meaning |
| --- | --- |
| `Waxaa loo baahan yahay weedh gudaha '{...}' ee qoraalka.` | An interpolation field requires an expression. |
| `Qaybaha qoraalka aad bay isugu dhex jiraan.` | Interpolated literals exceed the nesting limit. |

These definitions are in [formatting.rs](../../src/runtime/formatting.rs).

`khalad()` also uses the existing argument-count template:

```text
Hawsha 'khalad()' waxay qaadataa 1 ilaa 3 dood, laakiin {count} ayaa la bixiyay
```

Runtime definitions are in [abi.rs](../../src/runtime/abi.rs),
[stdlib.rs](../../src/runtime/stdlib.rs), and
[format.rs](../../src/error/format.rs). Contextual syntax is in
[parser.rs](../../src/frontend/parser.rs); string diagnostics are in
[lexer.rs](../../src/frontend/lexer.rs). The
[language reference](keywords.md#structured-errors) defines contracts and examples.

## Iteration vocabulary (3.2 implementation stage)

| Somali form | Meaning |
| --- | --- |
| `kuceli (qiime ku source)` | Iterate over each value in a source. |
| `ku` | Contextual collection separator; ordinary identifier elsewhere. |
| `wareeg(value)` / `value.wareeg()` | Create an iterator; also its runtime type name. |
| `xiga(iterator)` / `iterator.xiga()` | Advance to the next step. |
| `taxane(...)` | Lazy, exclusive integer sequence; see the reference for positional forms. |
| `tirso(source, start?)` | Enumerate values with their indices. |
| `lamaane(source, ...)` | Combine sources into rows, using the same word as object key/value pairs. |
| `dhammaad` | Boolean flag indicating exhaustion. |
| `qiime` | The value produced by a nonterminal step. |
| `<wareeg>` | Display text for an opaque iterator. |

New iteration failures use `E037`. Name validation and ordinary argument counts
reuse existing parser, semantic, and standard-library diagnostics.

| Exact Somali wording | English meaning |
| --- | --- |
| `Waxaa la filayay 'ku' (kuceli).` | The collection loop header requires its separator. |
| `Magacyada wareeggu waa inay kala duwanaadaan.` | Loop binding names must differ. |
| `Nooca '{type}' laguma wareegi karo.` | This value type cannot be iterated. |
| `Hawsha wareeg() waa inay celisaa wax leh hawl xiga().` | The iterator factory must return an iterator with a next method. |
| `Qaabkani ma laha hawl wareeg() ama xiga().` | The instance does not implement iteration. |
| `Hawsha xiga() waa inay celisaa walax leh dhammaad iyo qiime.` | A next method must return an object describing its step. |
| `Natiijada xiga() waxaa ka maqan qiime.` | A nonterminal step is missing its value. |
| `Sifada dhammaad ee xiga() waa inay noqotaa bool.` | The exhaustion flag must be boolean. |
| `Wareeggan mar hore ayaa la wadaa.` | This iterator is already advancing. |
| `Wareegyada aad bay isugu dhex jiraan.` | Native iterator nesting exceeds the limit. |
| `Qiimaha hadda la beddelayo laguma wareegi karo.` | A conflicting mutable borrow prevents iteration. |
| `xiga() wuxuu u baahan yahay wareeg.` | The next helper requires a native iterator. |
| `Tusmada tirso() waxay ka baxday xadka abn.` | An enumeration index cannot be represented as an integer. |
| `Tusmada bilowga ee tirso() waa inay noqotaa abn.` | The starting index must be an integer. |
| `Doodaha taxane() waa inay noqdaan abn.` | Range arguments must be integers. |
| `Tallaabada taxane() eber ma noqon karto.` | A range step cannot be zero. |
| `Wareeggu wuxuu u baahan yahay teed ama xirmo ka kooban {count} qiime si loo qaybiyo magacyada.` | A row must contain exactly as many values as there are binding names. |
| `Wareeggu wuxuu u baahan yahay walax.` | An internal protocol method receiver must be an object. |
| `Wareeggu ma laha hawl la yiraahdo '{method}'.` | This iterator has no such method; uses the ordinary runtime error category. |

The implementation is in [iterators.rs](../../src/runtime/iterators.rs), with
ABI dispatch in [abi.rs](../../src/runtime/abi.rs). The [iteration reference](iteration.md)
defines the public contracts and examples. Argument counts for `wareeg`, `xiga`,
`tirso`, and `taxane` use the existing `Hawsha ... dood ...` templates.

## Function defaults

`Qiime hordhac ah` means a default value. A default is evaluated when its
argument is omitted; it is not a declaration-time shared literal. The full
[argument contract](function-arguments.md) describes ordering and scope.

| Somali diagnostic | Meaning |
| --- | --- |
| `Doodda '{name}' mar hore ayaa lagu qeexay hawshan.` | The parameter name is duplicated. |
| `Dood aan lahayn qiime hordhac ah ma raaci karto dood leh qiime hordhac ah.` | A required parameter follows an optional parameter. |
| `Qiimaha hordhaca ahi ma adeegsan karo doodda '{name}' ka hor inta aan la diyaarin.` | A default refers to its own or a later parameter. |

Argument-count messages keep their existing wording. Optional parameters change
the count to an inclusive range, for example `1 ilaa 3 dood` (one to three
arguments). Fixed-arity functions continue to report a single count. Existing
type-mismatch, source-location, and structured-error wording also applies to
default expressions.

## Variadic positional parameters

| Somali diagnostic | Meaning |
| --- | --- |
| `Doodda '*' waxay ururisaa teed; nooc kale lama siin karo.` | A rest parameter is a list and cannot have a scalar annotation. |
| `Doodda '*' ma yeelan karto qiime hordhac ah.` | A rest parameter cannot have its own default. |
| `Doodda '*' waa inay noqotaa doodda ugu dambaysa.` | A rest parameter must be last. |

For an unbounded argument range, count messages use `N ama ka badan dood`
(N or more arguments). Duplicate names and unavailable default dependencies
keep the wording above.

## Collection callback signatures

`Hawsha la siiyay '{method}()' ma aqbasho {count} dood; waxay u baahan tahay {contract}.`
means the supplied callback cannot accept the number of arguments passed by
that collection method. This is `E022` and is checked even on empty lists.

`{contract}` is a fixed count such as `2 dood`, an inclusive range such as
`1 ilaa 3 dood`, an unbounded range such as `1 ama ka badan dood`, or
`tiro dhaban ah oo doodo ah` for an even number of arguments.

## Named arguments and keyword collectors

| Somali diagnostic | Meaning |
| --- | --- |
| `Dood boos leh ma raaci karto dood magac leh.` | A positional argument follows a named argument. |
| `Doodda '{name}' laba jeer ayaa la bixiyay.` | A parameter was supplied twice. |
| `Doodda magaca '{name}' leh hawshani ma aqbasho.` | The function does not accept that name. |
| `Doodda '{name}' ayaa ka maqan wicitaankan.` | A required argument is missing. |
| `Doodaha boosaska leh ayaa ka badan intii hawshu aqbali lahayd.` | Too many positional arguments were supplied. |
| `Doodda '**' waxay ururisaa walax; nooc kale lama siin karo.` | A keyword collector is an object. |
| `Doodda '**' ma yeelan karto qiime hordhac ah.` | A keyword collector cannot have a default. |
| `Doodda '**' waa inay noqotaa doodda ugu dambaysa.` | A keyword collector must be last. |
| `Doodda '*' waxaa ka dambayn karta '**' oo keliya.` | Only a keyword collector may follow a positional collector. |
| `Hawshani ma taageerto doodaha magacyada leh.` | No named-parameter metadata is available. |
| `Hawshani waxay aqbashaa doodaha boosaska leh oo keliya.` | The native callable accepts positional arguments only. |
| `Qiimahan hawl ahaan looma wici karo.` | The value cannot be called. |
| `Hawl '{name}' lama helin walaxda.` | The object has no such callable field or method. |
| `Qaabka '{name}' ma laha dhise aqbala doodahan.` | The class has no constructor accepting the arguments. |
| `Hawsha lama diyaarin.` | The compiled entry is unavailable. |
| `Magacyada dooduhu ma aha qoraal UTF-8 ah oo sax ah.` | A native descriptor has invalid text. |
| `Magacyada iyo qiimayaasha dooduhu isma laha.` | Descriptor labels and values do not match. |
| `Nooca wicitaanku ma saxna.` | A native descriptor has an invalid call kind. |
| `Walaxda hadda lama akhriyi karo.` | A host borrow prevents reading method metadata. |

Known signatures report bad names and collisions at the named argument and
inferable type errors at the supplied value. Missing arguments and dynamic
binding errors point to the call. Runtime binding failures use `E022`.

## Byte arrays

`bayt` means byte; `baytyo` names the byte-array type and constructor. The
[byte-array contract](bytes.md) documents storage, conversion, and mutation.
`qoraal` decodes text; `teed` creates a list copy. Other method names reuse the
existing collection vocabulary. New byte-operation failures use `E038`.

| Somali diagnostic | Meaning |
| --- | --- |
| `Qiimaha baytku waa inuu noqdaa abn u dhexeeya 0 iyo 255.` | A byte value must be an integer from 0 through 255. |
| `Tusmada baytyadu waa inay noqotaa abn.` | A byte index must be an integer. |
| `Tusmadu way ka baxsan tahay baytyada.` | The write or removal index is outside the byte array. |
| `baytyo() waxay u baahan tahay qoraal, teed ka kooban baytyo, ama baytyo kale.` | The constructor accepts text, a list of bytes, or another byte array. |
| `Baytyadu ma aha qoraal UTF-8 ah oo sax ah.` | The bytes are not valid UTF-8 text. |
| `Baytyada hadda lama akhriyi karo.` | A host borrow prevents reading bytes. |
| `Baytyada hadda lama beddeli karo.` | A host borrow prevents mutation. |
| `Teedka hadda lama akhriyi karo.` | A host borrow prevents reading the source list. |
| `Ma saari kartid baytyo madhan.` | An empty byte array has nothing to remove. |
| `midee() waxay u baahan tahay baytyo.` | Concatenation requires another byte array. |
| `Baytyadu ma laha hawl la yiraahdo '{method}'.` | The byte array has no such method. |
| `Magaca 'baytyo' waxaa loo adeegsadaa nooc hore u jiray.` | A class name conflicts with the built-in byte type; `E020`. |

Length conversion errors now include bytes among the accepted types:
`Qiimaha ma ahan teed, qoraal, walax, baytyo, xirmo, ama urur`.
Arity, slicing, stale-handle, and annotation errors reuse their existing forms.

## Optional and union types

`Nooc oggol maran` describes a nullable type. `Noocyo la isku daray` describes
union alternatives. The public syntax uses `T?`, `A | B`, and `maran`.

| Somali diagnostic | Meaning |
| --- | --- |
| `Waxaa la filayay nooc.` | A concrete type annotation was expected. |
| `Noocyada la isku daray aad bay u badan yihiin.` | The annotation exceeds the alternative limit. |
| `Qaabka noocyadu ma saxna.` | A native type-contract descriptor is malformed; `E022`. |
| `'{name}' wuxuu u baahan yahay nooca {types}, laakiin waxaa la helay {actual}.` | The value does not satisfy its runtime type alternatives; `E022`. |

`{types}` joins canonical type names with ` | `, including `maran` when null
is allowed. `{name}` is a variable name, `dood` for an argument, or `celi` for
a return value. Class alternatives use class names. Static mismatches, unknown
classes, required arguments, and bare returns reuse existing diagnostic forms.
The [type reference](optional-union-types.md) defines the contracts.

## Typed lists and objects

`Urur` describes a collection; `noocaysan` means typed. Existing method names
remain unchanged. Initial annotation failures use the existing `E022` contract
message. Rejected mutations use `E039`; missing classes retain `E021`.

| Somali diagnostic | Meaning |
| --- | --- |
| `Beddelkani ma waafaqsana nooca ururka {type}.` | The proposed mutation violates a collection contract. |
| `Teedka hadda lama akhriyi karo.` | A conflicting host borrow prevents reading the list. |
| `Teedka hadda lama beddeli karo.` | A conflicting host borrow prevents mutation. |
| `Walaxda hadda lama beddeli karo.` | A conflicting host borrow prevents object mutation. |
| `Furaha walaxda noocaysan waa inuu noqdaa qoraal.` | A typed object's key annotation must be string. |
| `Noocan ma qaato noocyo xubno ah.` | This type does not accept element arguments. |
| `Noocyadu aad bay isugu dhex jiraan.` | The annotation exceeds the nesting limit. |
| `Qaabka noocan aad buu u weyn yahay.` | The annotation exceeds its total node budget. |

`Walaxda hadda lama akhriyi karo.` and malformed-descriptor diagnostics retain
their existing wording. The [typed-collections reference](typed-collections.md)
defines shared alias, mutation, and ownership behavior.

## Tuples

`xirmo` names a fixed sequence whose positions cannot be changed. Tuple
operation errors use `E040`; annotation failures retain `E022`, nested guarded
writes retain `E039`, and ownership failures retain `E035`.

| Somali diagnostic | Meaning |
| --- | --- |
| `Xirmada lama beddeli karo.` | Tuple positions cannot be changed. |
| `Tusmada xirmadu waa inay noqotaa abn.` | A tuple index must be an integer. |
| `midee() waxay u baahan tahay xirmo.` | Tuple concatenation requires another tuple. |
| `Xirmadu ma laha hawl la yiraahdo '{method}'.` | The tuple has no such method. |
| `Magaca 'xirmo' waxaa loo adeegsadaa nooc hore u jiray.` | A class name conflicts with the tuple type; `E020`. |

List conversion reuses `Teedka hadda lama akhriyi karo.` for a conflicting
host borrow. Row-binding messages now say `teed ama xirmo` because both list
and tuple rows are accepted. See the [tuple reference](tuples.md).

## Set vocabulary and diagnostics

`urur` names the set type and constructor. `geli`, `saar`, `leeyahay`, `dherer`,
`nadiifi`, `nuqul`, `teed`, `xirmo`, and `wareeg` reuse collection vocabulary.
`midee` is union, `wadaag` intersection, `reeb` difference, `kaladuwan`
symmetric difference, and `hoosaad` subset membership.

| Somali wording | English meaning |
| --- | --- |
| `Xubinta ururku waa inay noqotaa qiime aan la beddeli karin; waxaa la helay '{type}'.` | A set key must be immutable; the given type is unsupported. |
| `Qiimaha NaN looma adeegsan karo xubin urur.` | NaN is not a valid set key. |
| `Xirmada ururka lagu darayo waxay leedahay lakabyo aad u badan.` | A tuple key exceeds the 64-level bound. |
| `Ururku ma laha tusmo; isticmaal leeyahay(), geli(), ama saar().` | Sets have no positional index. |
| `{method}() waxay u baahan tahay urur.` | This set operation requires another set. |
| `Ururku ma laha hawl la yiraahdo '{method}'.` | This method is not defined on a set. |
| `Magaca 'urur' waxaa loo adeegsadaa nooc hore u jiray.` | A class cannot reuse the builtin set type name. |

Set operation failures use `E041`; invalid annotations retain `E022`, class
name conflicts use `E020`, and stale or foreign handles retain `E035`.

## Typed set vocabulary and diagnostics

`urur[T]` annotates set keys. Initial checks use the existing collection
contract message and `E022`; rejected mutations use `E039`. Mutable or otherwise
unsupported key values still use the set diagnostic `E041`.

| Somali wording | English meaning |
| --- | --- |
| `Nooca '{type}' looma adeegsan karo xubin urur.` | This annotation admits no valid set key. |
| `Ururka hadda lama akhriyi karo.` | Conflicting native borrow prevents reading a set for validation. |
| `Ururka hadda lama beddeli karo.` | Conflicting native borrow prevents publishing a set mutation. |

Unknown classes retain `E021`. The existing annotation-depth, union-count,
and syntax diagnostics apply to set element annotations too.
