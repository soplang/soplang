# Soplang 3 terminology review

Status: **Approved vocabulary and shared collection contracts implemented**.
The approved names below and their compatibility spellings are available in
Soplang 3. The original inventory records the audit baseline; the language
reference describes the current API. The original inventory below is historical; completed decisions follow it.

The [shared collection API proposal](collection-api-proposal.md) reviews
operation families and underscore simplification. Its shared `saar`, `hel`,
`dhig`, and length contracts are implemented. The additional simplifications and addition/combination contracts are also
implemented; the design record states their exact behavior.

All work belongs to Soplang 3. No version 4 transition or removal of existing
names is scheduled. The [language reference](keywords.md) describes the current
API; the [roadmap](../../ROADMAP.md) describes future capabilities.

## Evidence and decision policy

The source snapshot is Somali Language Standard (SLS), commit
`9bcab5e420d67406cb021c8fcf89df3aeb270675`, inspected locally. Links below are
pinned to that revision so subsequent source changes do not alter this review.
The Soplang inventory was checked against commit `bb3a0e1`.

At this snapshot, SLS has no Stable standards. SLS-0002 orthography is Proposed;
SLS-0100 dictionary rules are Draft. Computer science (SLS-0204), software
engineering (SLS-0205), networking (SLS-0207), and mathematics (SLS-0216)
terminology standards are Planned. See the [SLS registry][sls-registry].

The [resource library][sls-resources] is descriptive evidence. A word appearing
there establishes an attestation, not approval of a programming meaning.
Historical spellings and OCR remnants must not become canonical terms merely
because a search finds them. Absence from a headword list does not establish
that a form is incorrect. Proposed [orthography rules][sls-orthography] also
reject automatic joining, splitting, or vowel changes without evidence.

Each decision should record the concept, receiver or syntax context, current
name, chosen canonical name, evidence, rationale, aliases, implementation
release, and decision status. Approval is recorded per public name. SLS
attestation and a Soplang naming decision remain separate kinds of evidence.

The inventory uses these labels:

| Label   | Meaning                                                                               |
| ------- | ------------------------------------------------------------------------------------- |
| Retain  | Recommend retaining the current identifier; no new SLS computing standard is claimed. |
| Explain | Retain for now and clarify the technical meaning or abbreviation.                     |
| Review  | A public naming decision is required; see the candidate table.                        |
| Alias   | An alternative is already implemented.                                                |

## Baseline inventory: keywords, types, and special syntax

Sources: [lexer](../../src/frontend/lexer.rs), [parser](../../src/frontend/parser.rs),
and [grammar](grammar.md).

| Current form  | Programming concept          | Recommendation and evidence                                                                                                                                             |
| ------------- | ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `door`        | Dynamic variable declaration | Explain as a Soplang short form related to *doorsoome*; the expansion is a project terminology proposal, not a verified historical abbreviation. [Dictionary D][dict-d] |
| `madoor`      | Constant declaration         | Explain as a Soplang short form related to *madoorsoome*. Same abbreviation caveat. [Dictionary M][dict-m]                                                              |
| `hawl`        | Function                     | Retain; the dictionary supports work or purposeful activity. Its programming use is Soplang terminology. [Dictionary H][dict-h]                                         |
| `celi`        | Return from a function       | Retain; document its control-flow role.                                                                                                                                 |
| `qor`         | Print                        | Retain; also a callable built-in.                                                                                                                                       |
| `gelin`       | Read console input           | Explain as input; do not describe it as general file reading. [Dictionary G][dict-g] records *gelin* and *geli*.                                                        |
| `haddii`      | If                           | Retain.                                                                                                                                                                 |
| `haddii_kale` | Else if                      | Retain; underscore is an identifier convention, not a claim about prose spacing.                                                                                        |
| `ugudambeyn`  | Else and switch default      | Review both meaning and spelling before adding a distinct cleanup construct.                                                                                            |
| `dooro`       | Switch                       | Retain.                                                                                                                                                                 |
| `xaalad`      | Switch case                  | Retain.                                                                                                                                                                 |
| `kuceli`      | Numeric for loop             | Explain the language-specific joined spelling; distinguish the string method `ku_celi`.                                                                                 |
| `ilaa`        | Numeric loop endpoint marker | Retain; parsed contextually as an identifier, not a globally reserved keyword.                                                                                          |
| `::`          | Numeric loop step marker     | Retain; describe as *tallaabo* in explanations.                                                                                                                         |
| `intay`       | While loop                   | Retain for now; do not mechanically expand a contraction without review.                                                                                                |
| `jooji`       | Break                        | Retain.                                                                                                                                                                 |
| `soco`        | Continue                     | Retain.                                                                                                                                                                 |
| `fasax`       | Try block                    | Review: dictionary senses concern permission or leave. [Dictionary F][dict-f]                                                                                           |
| `qabo`        | Catch                        | Retain; the capture or holding metaphor has dictionary support. [Dictionary Q][dict-q]                                                                                  |
| `keen`        | Import a file                | Retain; explain current file-flattening behavior separately from future modules.                                                                                        |
| `qaab`        | Class                        | Retain as a Soplang technical use of form or structure. [Dictionary Q][dict-q]                                                                                          |
| `dhaxal`      | Inherit                      | Retain.                                                                                                                                                                 |
| `cusub`       | Create an instance           | Retain.                                                                                                                                                                 |
| `nafta`       | Current instance             | Retain.                                                                                                                                                                 |
| `waalid`      | Parent method lookup         | Retain.                                                                                                                                                                 |
| `abn`         | Signed integer               | Explain its relationship to *abyoone* as a project short form; no evidence establishes the historical origin of `abn`. [Dictionary A][dict-a]                           |
| `jajab`       | Floating-point number        | Explain: binary floating point, not exact decimal arithmetic. [Math glossary][math] records decimal fraction terminology.                                               |
| `qoraal`      | String                       | Retain.                                                                                                                                                                 |
| `bool`        | Boolean                      | Retain as technical vocabulary; the math glossary attests *Aljebrada bool*. [Math glossary][math]                                                                       |
| `teed`        | Mutable list                 | Retain; the glossary attests Array — Teed. Soplang's list behavior must still be specified. [Math glossary][math]                                                       |
| `walax`       | Object                       | Retain; dictionary meaning includes a thing or object. [Dictionary W][dict-w]                                                                                           |
| `run`         | True                         | Retain. [Dictionary R][dict-r]                                                                                                                                          |
| `been`        | False                        | Retain. [Dictionary B][dict-b]                                                                                                                                          |
| `maran`       | Null                         | Retain; distinguish absence of a value from an empty string or collection. [Dictionary M][dict-m]                                                                       |
| `null`        | Null compatibility spelling  | Alias of `maran`; already implemented.                                                                                                                                  |

`dhaw` is an automatically selected constructor method, not a lexer keyword.
Its naming review appears below.

## Global built-ins

Source: [built-in registration and implementation](../../src/runtime/stdlib.rs).
Types that also act as conversion or construction functions are repeated here
because their callable behavior needs a separate definition.

| Current name  | Actual behavior                                          | Recommendation                                                                                                         |
| ------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `qor`         | Print a value                                            | Retain.                                                                                                                |
| `gelin`       | Read a line, optionally displaying a prompt              | Explain.                                                                                                               |
| `nooc`        | Return a type name                                       | Retain.                                                                                                                |
| `abn`         | Convert to integer                                       | Explain; never label this an integer-or-float conversion.                                                              |
| `jajab`       | Convert to floating point                                | Explain; avoid implying an exact decimal type.                                                                         |
| `qoraal`      | Convert to string                                        | Retain.                                                                                                                |
| `bool`        | Convert using truthiness                                 | Retain; document truthiness.                                                                                           |
| `teed`        | Construct a list from arguments                          | Retain.                                                                                                                |
| `walax`       | Construct an object from alternating keys and values     | Retain.                                                                                                                |
| `daji`        | Floor to integer                                         | Explain rounding toward negative infinity.                                                                             |
| `kor`         | Ceiling to integer                                       | Explain rounding toward positive infinity.                                                                             |
| `dherer`      | String character count, list length, or object key count | Retain. [Math glossary][math]: Length — Dherer.                                                                        |
| `xul`         | Random number or random list item                        | Explain the random-selection meaning; retain for now.                                                                  |
| `baaxad`      | Materialize an integer range as a list                   | Explain the Soplang extension of a word for breadth. Do not claim an established iterator term. [Dictionary B][dict-b] |
| `dhammeystir` | Round to nearest integer                                 | Review spelling and mathematical meaning.                                                                              |
| `sugan`       | Absolute value                                           | Retain. [Math glossary][math]: absolute value uses *qiimo sugan*.                                                      |
| `ugu_yar`     | Numeric minimum                                          | Retain.                                                                                                                |
| `ugu_weyn`    | Numeric maximum                                          | Retain.                                                                                                                |
| `awood`       | Exponentiation                                           | Review mathematical sense.                                                                                             |
| `xidid`       | Square root                                              | Explain the square-root restriction; review whether the name should be explicit.                                       |

## Baseline inventory: collection methods

Sources: [method dispatch](../../src/runtime/abi.rs) and
[method implementations](../../src/runtime/stdlib.rs). Names are scoped to the
receiver type. A shared spelling does not guarantee shared mutation behavior.

### Lists

| Current name   | Actual behavior                                                | Recommendation                                                                 |
| -------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `dherer`       | Length                                                         | Retain.                                                                        |
| `kudar`        | Append an item, or return concatenated copy when passed a list | Explain both overloads and their different mutation behavior.                  |
| `kasaar`       | Pop and return the last item                                   | Retain for now; distinguish from indexed removal.                              |
| `leeyahay`     | Membership                                                     | Retain.                                                                        |
| `nuqul`        | Shallow copy                                                   | Explain that nested values are shared.                                         |
| `nadiifi`      | Clear in place                                                 | Retain.                                                                        |
| `rog`          | Reverse in place                                               | Retain.                                                                        |
| `habee`        | Sort in place                                                  | Retain.                                                                        |
| `jar`          | Slice into a new list                                          | Retain.                                                                        |
| `aaddin`       | Map a callback over items                                      | Retain. [Math glossary][math]: Mapping — Aaddin.                               |
| `shaandhee`    | Filter by predicate                                            | Retain; dictionary includes filtering. [Dictionary Sh][dict-sh]                |
| `muuji`        | Find an item's index                                           | Review; the dictionary meaning is show or make visible. [Dictionary M][dict-m] |
| `geli`         | Insert at an index                                             | Retain.                                                                        |
| `ka_saar`      | Remove and return an indexed item                              | Retain for now; not an alias of `kasaar()`.                                    |
| `koowaad`      | First item or `maran`                                          | Retain.                                                                        |
| `ugu_dambeeya` | Last item or `maran`                                           | Retain.                                                                        |
| `isku_gee`     | Fold with an arbitrary callback and initial value              | Review; addition wording is narrower than its behavior.                        |
| `dhammaan`     | Test every item against a predicate                            | Retain.                                                                        |
| `qaar`         | Test whether any item matches                                  | Retain.                                                                        |

### Objects

| Current name | Actual behavior                    | Recommendation                              |
| ------------ | ---------------------------------- | ------------------------------------------- |
| `fure`       | Return all keys                    | Review singular naming for a plural result. |
| `qiime`      | Return all values                  | Review plural naming together with `fure`.  |
| `lamaane`    | Return all key-value pairs         | Review plural naming together with `fure`.  |
| `leeyahay`   | Test key presence                  | Retain.                                     |
| `tir`        | Delete a key and return the object | Retain; distinguish from count `tiri`.      |
| `kudar`      | Return a merged copy               | Explain that the receiver is not mutated.   |
| `nuqul`      | Shallow copy                       | Retain.                                     |
| `nadiifi`    | Clear in place                     | Retain.                                     |
| `hel`        | Lookup with optional default       | Retain.                                     |
| `dhig`       | Assign a key and value             | Retain.                                     |
| `ka_qaad`    | Remove a key and return its value  | Retain; distinguish from `tir`.             |

### Strings

| Current name      | Actual behavior                             | Recommendation                                                        |
| ----------------- | ------------------------------------------- | --------------------------------------------------------------------- |
| `qeybi`           | Split by delimiter                          | Retain for now; do not convert vowel variants by analogy alone.       |
| `leeyahay`        | Substring membership                        | Retain.                                                               |
| `dhamaad`         | Test suffix                                 | Review spelling and predicate clarity.                                |
| `bilow`           | Test prefix                                 | Retain for now; review predicate style alongside suffix naming.       |
| `beddel`          | Replace the first occurrence                | Explain first occurrence explicitly.                                  |
| `beddel_dhammaan` | Replace all occurrences                     | Retain.                                                               |
| `kudar`           | Join a list using the receiver as separator | Explain this receiver-specific meaning.                               |
| `jar`             | Character-based slice                       | Retain.                                                               |
| `xarfaha_waaweyn` | Uppercase conversion                        | Retain as the documented canonical form.                              |
| `xarafaha_weyn`   | Uppercase conversion                        | Alias; already implemented, not the preferred documentation spelling. |
| `xarfaha_yaryar`  | Lowercase conversion                        | Retain.                                                               |
| `masax`           | Trim whitespace at both ends                | Explain whitespace-only behavior.                                     |
| `raadi`           | First matching character index or `-1`      | Retain.                                                               |
| `dherer`          | Unicode character count                     | Retain.                                                               |
| `masax_bilow`     | Trim leading whitespace                     | Retain.                                                               |
| `masax_dhamaad`   | Trim trailing whitespace                    | Review doubled consonant with `dhamaad`.                              |
| `tiri`            | Count non-overlapping occurrences           | Retain. [Dictionary T][dict-t]                                        |
| `ku_celi`         | Repeat text                                 | Retain; distinct from the loop keyword `kuceli`.                      |
| `raadi_dambe`     | Last matching character index or `-1`       | Retain.                                                               |

## Approved public-name decisions

The decisions below are implemented as Soplang 3 terminology. These are
project decisions informed by SLS evidence, not SLS computing standards.
Implementation is included in the current Soplang 3 development tree. Existing
spellings remain supported; no removal release is scheduled.

| ID | Current spelling to retain as an alias | Approved canonical spelling | Behavior to preserve |
| --- | --- | --- | --- |
| T01 | Constructor `dhaw()` | `dhis()` | Construction, argument validation, inheritance, and parent constructor calls. |
| T02 | Try block `fasax` | `tijaabi` | The existing `qabo` grammar and runtime error handling. |
| T03 | Exponentiation `awood()` | `jibbaar()` | Numeric conversion, arity, and floating-point exponentiation. |
| T04 | Rounding `dhammeystir()` | `duub()` | Nearest integer, halfway away from zero. |
| T05 | List `muuji(item)` | List `raadi(item)` | First matching index; `maran` when no item matches. |
| T06 | String `dhamaad(sub)` | String `dhammaad(sub)` | Suffix predicate and argument validation. |
| T08 | Object `fure()`, `qiime()`, `lamaane()` | `furayaal()`, `qiimayaal()`, `lammaanayaal()` | Keys, values, and pairs in sorted-key order. |
| T09 | List `isku_gee(callback, initial)` | `ururi(callback, initial)` | General left fold, including nonnumeric accumulators. |
| T10 | Branch `haddii_kale`, fallback `ugudambeyn` | Contextual `kale (condition)` and `kale` | Existing if/switch branching; preserve ordinary identifiers named `kale`. |
| T13 | List `koowaad()` | `hore()` | First item or `maran` for an empty list. |
| T14 | List `ugu_dambeeya()` | `dambe()` | Last item or `maran` for an empty list. |
| T15 | `ugu_yar(...)`, `ugu_weyn(...)` | `yar(...)`, `weyn(...)` | Numeric minimum/maximum; preserve arity and result types. |
| T16 | List/object `kudar(other)` | `midee(other)` | New shallow combined collection; right object keys win. |
| T17 | String `kudar(list)` | `xiriiri(list)` | Join string representations using the receiver as separator. |
| T18 | `xarfaha_waaweyn()`, `xarafaha_weyn()`, `xarfaha_yaryar()` | `weynee()`, `yaree()` | Unicode uppercase/lowercase conversion. |
| T19 | `masax_bilow()`, `masax_dhamaad()`, `masax_dhammaad()` | `masax("hore")`, `masax("dambe")` | Directional whitespace trimming; no argument trims both ends. |
| T20 | `raadi_dambe(sub)` | `raadi(sub, "dambe")` | Last Unicode character index or `-1`; default search stays first. |
| T21 | `beddel_dhammaan(old, new)` | `beddel(old, new, maran)` | Replace all; optional nonnegative integer limits count, omitted count replaces first. |
| T22 | String `ku_celi(count)` | `kuceli(count)` | Nonnegative integer repetition; allowed after dot without changing loops. |

`tijaabi` is contextual before `{`, preserving existing identifier uses.
Automatic construction checks each class for `dhis`, then `dhaw`, before its
parent. Explicit method and parent calls prefer the requested spelling within
each class, then its counterpart, before the next ancestor. Both methods remain
explicitly callable when a class defines both. Tests exercise these rules in
Cranelift JIT and LLVM AOT binaries.

The additional short names `hore`, `dambe`, `yar`, and `weyn` are contextual
API choices: lists and strings supply first/last position context and numeric selection
supplies minimum/maximum context. These choices do not claim that an adjective
alone is a full mathematical translation. Contextual `kale` leaves future
cleanup/finally terminology open.

## Implemented shared collection contracts

| Operation | List | Object | String |
| --- | --- | --- | --- |
| Length | `dherer()` | `dherer()` | `dherer()` |
| Membership | `leeyahay(value)` | `leeyahay(key)` | `leeyahay(substring)` |
| Read | `hel(index, default?)` | `hel(key, default?)` | `hel(index, default?)` |
| Assign | `dhig(index, value)` | `dhig(key, value)` | Not applicable; immutable |
| Remove | `saar()` or `saar(index)` | `saar(key)` | Not applicable; immutable |
| Clear | `nadiifi()` | `nadiifi()` | Not applicable; immutable |
| Shallow copy | `nuqul()` | `nuqul()` | Not needed for immutable values |
| Combine | `midee(list)` | `midee(object)` | `xiriiri(list)` joins using the separator receiver |
| Insert | `geli(value)` or `geli(index, value)` | `dhig(key, value)` | Not applicable; immutable |
| Endpoints | `hore()`, `dambe()` | Not positional | `hore()`, `dambe()` |
| Count matches | `tiri(value)` | Use membership for a unique key | `tiri(substring)` |
| Slice | `jar(start?, end?)` | Not positional | `jar(start?, end?)` |

Removal returns the removed value. Legacy `kasaar()`, `ka_saar(index)`, and
`ka_qaad(key)` preserve their signatures. `tir(key)` still returns the object
and therefore remains a distinct compatibility wrapper. The receiver supplies
addressing context: integer positions for lists/strings and string object keys.
See the [language reference](keywords.md#shared-collection-operations) for
defaults, bounds, and mutation returns. Shared slicing uses integer bounds,
negative offsets, clamping, and empty results for reversed bounds. First/last
methods return `maran` for empty lists and strings. List counting uses the same
value equality as membership and search; string counting uses non-overlapping
substring matches. Search-miss results retain their receiver-specific behavior
for compatibility.

## Decision rationale

The historical inventory above identifies the original questions. The following
records resolve them for this stage. T07 retains `xidid` specifically for square
root; T11 retains the short type/declaration identifiers; T12 retains `Khalad`
without imposing a new spelling on diagnostic headings.

| ID  | Current name or concept           | Candidate and recommendation                                                             | Evidence and remaining decision                                                                                                                                                                   |
| --- | --------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T01 | Constructor `dhaw` | Approved: `dhis()`; retain `dhaw()` as a compatibility alias. | [Dictionary Dh][dict-dh] associates `dhaw` with unrelated senses and `dhis` with building. Implemented precedence is specified in the approved decision above. |
| T02 | Try keyword `fasax` | Approved: `tijaabi`; retain `fasax` as a compatibility spelling. | [Dictionary F][dict-f]: permission or leave; [Dictionary T][dict-t]: `tijaabi`. Contextual parsing preserves existing identifier uses. |
| T03 | Exponentiation `awood` | Approved: `jibbaar()`; retain `awood()` as a compatibility alias. | [Math glossary][math]: Power — Jibbaar. [Dictionary A][dict-a] distinguishes ability and physical power. |
| T04 | Rounding `dhammeystir` | Implemented: `duub`; earlier spelling retained. | [Math glossary][math] records rounding off as *duubid*. The verb form and API use are Soplang decisions; nearest rounding uses halfway away from zero. |
| T05 | List index lookup `muuji` | Approved: list `raadi(item)`; retain `muuji(item)` as a compatibility alias. | [Dictionary M][dict-m]: showing; [Dictionary R][dict-r]: searching. Preserve `maran` for a missing list item; string search retains its existing `-1` result. |
| T06 | String `dhamaad`, `masax_dhamaad` | Implemented: `dhammaad` and the corrected trim alias `masax_dhammaad`; directional trim now prefers `masax("dambe")` (T19). | [Dictionary Dh][dict-dh] records doubled `m`. Alias behavior must match existing suffix and whitespace operations. |
| T07 | Square root `xidid` | Retain with an explicit square-root definition. | [Dictionary X][dict-x] supports general root; the [math glossary][math] has square-root phrases. No arbitrary-root overload is added. |
| T08 | Object `fure`, `qiime`, `lamaane` | Implemented: `furayaal`, `qiimayaal`, `lammaanayaal`; earlier names retained. | [Dictionary F][dict-f] records the key plural `-rayaal`; [Dictionary Q][dict-q] records *qiime* and [Dictionary L][dict-l] records *lammaane*. The complete plural API family is a Soplang choice, not an SLS standard. |
| T09 | General fold `isku_gee` | Implemented: `ururi`; earlier spelling retained. | [Dictionary U][dict-u] records gathering. Accumulation covers arbitrary callbacks, including multiplication and string building. |
| T10 | Else/default `ugudambeyn` | Implemented: contextual `kale`; `kale (condition)` also replaces `haddii_kale`. | Additional branches and fallbacks share one name. Existing spellings remain supported. This does not define future cleanup/finally syntax. |
| T11 | `abn`, `door`, `madoor`           | Retain; document as project short forms with proposed long-form explanations.            | [Dictionary A][dict-a], [Dictionary D][dict-d], [Dictionary M][dict-m]. Do not assert an unrecorded historical etymology.                                                                         |
| T12 | Diagnostic `Khalad` / `qalad` | Retain `Khalad` headings and existing machine-readable codes. | [Dictionary Q][dict-q] attests `qalad`; this does not establish that `khalad` is invalid. Further editorial changes require their own evidence. |

### Collection and string choices

- **T16–T17:** [Dictionary M][dict-m] attests `midee` for unifying; [Dictionary
  X][dict-x] attests `xiriiri` for connecting. Collections combine into a new
  shallow value; separator strings join text. `geli(value)` appends exactly one
  entry, including a nested list, while `geli(index, value)` keeps indexed
  insertion. Legacy list `kudar` still selects append or concatenation based on
  the argument type and therefore is not a direct alias of `geli`.
- **T18:** `weynee` and `yaree` use the string receiver to establish letter-case
  context. These are project API choices; the numeric globals remain `weyn`
  and `yar`.
- **T19–T21:** Explicit `"hore"`/`"dambe"` directions and a replacement count
  keep related operations under one name. `maran` means unlimited replacements;
  no count means one. Invalid directions and fractional/negative counts report
  catchable errors. Earlier separate methods keep their signatures.
- **T22:** `kuceli` is recognized as a member name after a dot as well as a loop
  keyword in statement position. No new global keyword is introduced.

## CLI and diagnostic inventory

The inventory covers the public surfaces below. The wording changes in this
stage clarify arithmetic, method lookup, conversion, argument counts, and
strict-mode messages. Existing error headings are retained; additional translations can be reviewed
in a separate editorial stage.

| Surface             | Current vocabulary                                                                                                                           | Planned review                                                                                 |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| REPL help           | `/caawi`, `/help`                                                                                                                            | Retain both commands; review explanatory Somali.                                               |
| REPL exit           | `/bixi`, `/exit`                                                                                                                             | Retain both commands.                                                                          |
| REPL file loading   | `/akhrifayl`, `/load`                                                                                                                        | Review joined spelling only with a compatibility plan.                                         |
| Compiler inspection | `/ast`, `/hir`, `--ast`, `--hir`                                                                                                             | Retain acronyms; explain AST and HIR rather than presenting them as translated Somali words.   |
| CLI flags           | `--build`, `--output`, `--opt-level`, `--quiet`, `--command`, `--file`, `--example`, `--interactive`, `--strict`, `--no-color`, version/help | Preserve command spellings; review human-readable help.                                        |
| Error headings      | `Khalad`, `ereyayn`, `naxwe`, `fulin`, `nooc`, `soo-dejin`                                                                                     | Agree Somali labels and technical explanations; retain machine-readable error codes.           |
| Argument validation | `dood`, `qiimo`, `param`                                                                                                                     | Use `dood` for call arguments and `qiime` for values; define parameter terminology separately. |
| Type checking       | `dynamic`, `strict mode`, `abn`, `jajab`                                                                                                     | Refer to actual source type names and explain static/dynamic behavior in plain Somali.         |
| Arithmetic errors   | `Habka`, `Tixgacaysi`                                                                                                                        | Replace unclear operation wording with direct explanations of what failed.                     |
| Method errors       | `No method`, `Unknown list/object/string method`                                                                                             | Use consistent Somali sentences including receiver type and requested name.                    |
| File/build messages | `fayl`, `Barnaamijka waa la dhisayaa`, build failures                                                                                        | Review spelling, agreement, and useful technical context.                                      |

Applied wording in this stage:

| Existing wording                                     | Current wording                                                                      |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `Habka waa in ay noqdaan tiro`                       | `Hawlgalka (%) wuxuu u baahan yahay tirooyin.`                                           |
| `Tixgacaysi (-) waa in uu noqdo tiro`                | `Hawlgalka (-) wuxuu u baahan yahay tiro.`                                             |
| `No method '{}' on this type`                        | `Nooca '{}' ma laha hawl la yiraahdo '{}'.`                                                |
| `hawl '{}' waxay leedahay param dynamic strict mode` | `Habka adag wuxuu u baahan yahay in dood kasta oo hawsha '{}' leedahay loo qoro nooc.` |
| `baaxad waa hawl dhisay`                             | `baaxad() waa hawl ku dhex jirta Soplang.`                                             |

Review agreement against the actual subject; do not globally replace `wuxuu`
with `waxay` or the reverse. Keep natural-language prose spacing distinct from
snake_case identifiers. Descriptions must say that `abn` is integer, `jajab` is
floating point, copies are shallow, and `maran` is distinct from an empty
collection.

## Compatibility and implementation order

1. **Glossary:** review this inventory and record decisions for T01–T12.
2. **Documentation and diagnostics:** apply approved wording and clarify current
   behavior without silently introducing candidate names as supported APIs.
3. **Public names:** add approved canonical names and compatibility aliases in a
   compatible Soplang 3 release. Scope each alias by receiver and call behavior.
4. **Verification:** cover old and new forms in parser, semantic, runtime, JIT,
   and LLVM AOT tests; update examples, changelog, and migration notes.

Existing vocabulary remains supported unless a separate compatibility decision
is made. Do not reassign a name to a different meaning or change return values
as a side effect of a rename. The approved shared collection contracts combine
pop and indexed removal under `saar` while preserving both existing signatures.
Aliases must share type checking, arity, mutation, and
error behavior; wrappers with different return values must be labelled as such.

Keyword aliases need an explicit identifier-collision strategy. Constructor
aliases need a deterministic lookup rule across inheritance and `waalid`.
Avoid warnings on every legacy call unless a warning policy is separately
approved. Each implementation stage must leave the tree green and end with a
commit handoff.

## Terminology for future releases

Before implementing new public syntax or APIs, use the same evidence and
decision record for:

- **3.1:** raise/throw, cleanup/finally, error causes, stack traces, and string
  formatting.
- **3.2:** collection iteration, sets, tuples, bytes, parameter kinds, and typed
  collections.
- **3.3:** file/path operations, HTTP requests and responses, sockets, timeouts,
  DNS, TLS, serialization, and process operations.
- **3.4:** module namespaces, exports, import aliases, packages, and lockfiles.
- **3.5:** formatter, checker, test runner, editor, debugger, and profiler text.
- **3.6:** tasks, channels, async operations, cancellation, and server APIs.

Protocol names and identifiers such as HTTP, DNS, TLS, JSON, and WebSocket may
remain recognizable technical names. Their Somali explanations and public API
names still require review. Planned SLS domains do not supply approved terms
yet; future selections must be labelled as Soplang decisions unless SLS review
actually establishes the mapping.

## Source attribution

Evidence is paraphrased from Somali Language Standard (SLS), © 2026 Somali
Language Standard contributors, licensed under [CC BY 4.0][cc-by]. This glossary
is Soplang's review and does not imply SLS endorsement. Historical source
provenance and its limitations are recorded in the [dictionary source
registry][dict-sources] and [technical glossary source registry][math-sources].

[sls-registry]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/standards/registry.json
[sls-resources]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/README.md
[sls-orthography]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/spec/orthography/0002-spelling-rules.md
[math]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/erey-bixin/05-xisaab.md
[dict-a]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/22-a.md
[dict-b]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/01-b.md
[dict-d]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/06-d.md
[dict-dh]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/10-dh.md
[dict-f]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/13-f.md
[dict-g]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/12-g.md
[dict-h]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/20-h.md
[dict-m]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/17-m.md
[dict-q]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/14-q.md
[dict-r]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/07-r.md
[dict-sh]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/09-sh.md
[dict-t]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/02-t.md
[dict-w]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/19-w.md
[dict-x]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/04-x.md
[dict-sources]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/00-sources.md
[math-sources]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/erey-bixin/00-sources.md
[cc-by]: https://creativecommons.org/licenses/by/4.0/

[dict-l]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/16-l.md
[dict-u]: https://github.com/somastllc/somali-language-standard/blob/9bcab5e420d67406cb021c8fcf89df3aeb270675/resources/qaamuus/26-u.md

## Diagnostic location implementation

The phase headings use Somali explanatory terms: `ereyayn` (lexing), `naxwe`
(parsing), `nooc` (semantic/type validation), `fulin` (runtime), and `soo-dejin`
(imports). `Khalad` remains the shared heading. These are Soplang editorial
choices; no SLS computing-standard approval is implied.

Original filenames and token positions pass through parsing, HIR, Cranelift,
and LLVM. Errors show the relevant source line and a caret, including inside
imported functions. Columns count Unicode scalar values, while caret spacing
uses display width and four-column tab stops. The checked error corpus covers
these details. Existing codes and caught-message string behavior remain intact.

## Runtime error and cleanup forms

| Form | Meaning | Compatibility |
| --- | --- | --- |
| `tuur fariin` | Raise an error; `tuur` uses the imperative sense of throwing. | Contextual before a same-line value. Ordinary calls, assignments, and property names remain valid. |
| `dhammee { ... }` | Finish protected work with guaranteed cleanup. | Contextual after a try body or handler; ordinary identifier uses remain valid. |

`dhammee` runs on success, errors, returns, and loop exits. It can follow a
`tijaabi` body with or without `qabo`. Existing handler variables continue to
receive error-message strings.

Structured errors add `khalad`, `fariin`, `summad`, `sabab`, `fayl`, `saf`,
`tiir`, and `raad`. Their contracts and exact diagnostic text are recorded in
the [runtime error wording inventory](runtime-error-wording.md) for review
alongside the earlier full wording inventory.

`qaabee` is the shared formatting verb. The global function formats one value;
the string method applies a template. Both use the same descriptor vocabulary.
