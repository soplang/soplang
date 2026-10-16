# Soplang 3 terminology review

Status: **Stage 1 review draft**. This glossary inventories the existing public
vocabulary and records recommendations for review. Candidate spellings are not
implemented or approved canonical replacements.

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
release, and decision status. Approval of this review process does not approve
individual replacements.

The inventory uses these labels:

| Label   | Meaning                                                                               |
| ------- | ------------------------------------------------------------------------------------- |
| Retain  | Recommend retaining the current identifier; no new SLS computing standard is claimed. |
| Explain | Retain for now and clarify the technical meaning or abbreviation.                     |
| Review  | A public naming decision is required; see the candidate table.                        |
| Alias   | An alternative is already implemented.                                                |

## Keywords, types, and special syntax

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

## Collection methods

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

## Public-name decisions for review

All entries below are **pending**. Alternatives are Soplang proposals based on
the cited senses; none is presented as an SLS-approved computing term.

| ID  | Current name or concept           | Candidate and recommendation                                                             | Evidence and remaining decision                                                                                                                                                                   |
| --- | --------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T01 | Constructor `dhaw`                | Prefer reviewing `dhis()`; compare `bilow()`.                                            | [Dictionary Dh][dict-dh] associates `dhaw` with unrelated senses and `dhis` with building; [Dictionary B][dict-b] attests beginning. Define old/new constructor precedence if both methods exist. |
| T02 | Try keyword `fasax`               | Review `tijaabi`.                                                                        | [Dictionary F][dict-f]: permission or leave; [Dictionary T][dict-t]: `tijaabi`. A new reserved keyword could invalidate an existing user identifier.                                              |
| T03 | Exponentiation `awood`            | Prefer `jibbaar`.                                                                        | [Math glossary][math]: Power — Jibbaar. [Dictionary A][dict-a] distinguishes ability and physical power. Keep `awood` callable if an alias is approved.                                           |
| T04 | Rounding `dhammeystir`            | Choose a rounding-specific form after reviewing *duubid*; no exact replacement selected. | [Math glossary][math]: Rounding off — Duubid. [Dictionary Dh][dict-dh]: `dhammaystir` means completion. Do not treat a spelling-only change as a complete semantic fix.                           |
| T05 | List index lookup `muuji`         | Review `raadi(item)`.                                                                    | [Dictionary M][dict-m]: showing; [Dictionary R][dict-r]: searching. Decide whether list misses remain `maran` or become `-1`; a terminology alias must preserve existing behavior.                |
| T06 | String `dhamaad`, `masax_dhamaad` | Review `dhammaad`, `masax_dhammaad`.                                                     | [Dictionary Dh][dict-dh] records doubled `m`. Preserve current names as aliases if adopted.                                                                                                       |
| T07 | Square root `xidid`               | Keep with an explicit definition, or introduce an explicit square-root name.             | [Dictionary X][dict-x] supports general root; [Math glossary][math] gives square-root phrases. Exact identifier spelling remains open.                                                            |
| T08 | Object `fure`, `qiime`, `lamaane` | Review plural naming as one family; `furayaasha()` is a candidate for keys.              | [Dictionary F][dict-f] and [math glossary][math] support key terminology, not approval of a complete computing API family. Confirm morphology and consistent identifier style.                    |
| T09 | General fold `isku_gee`           | Select a general accumulation term, or retain with an explicit definition.               | [Math glossary][math] records addition terminology; the callback may multiply or build strings or objects. Literal mathematical reduce does not automatically name a programming fold.            |
| T10 | Else/default `ugudambeyn`         | Review together with future cleanup vocabulary; no replacement selected.                 | Resolve meaning, spelling, if/switch grammar, and the future equivalent of `finally` together. No mechanical split or vowel rewrite.                                                              |
| T11 | `abn`, `door`, `madoor`           | Retain; document as project short forms with proposed long-form explanations.            | [Dictionary A][dict-a], [Dictionary D][dict-d], [Dictionary M][dict-m]. Do not assert an unrecorded historical etymology.                                                                         |
| T12 | Diagnostic `Khalad` / `qalad`     | Choose a consistent editorial form after speaker review.                                 | [Dictionary Q][dict-q] attests `qalad`; absence of `khalad` from that source alone does not prove invalidity.                                                                                     |

## CLI and diagnostic inventory

The inventory covers the public surfaces below. Individual message revisions
belong to Stage 3; the examples are proposals, not newly approved templates.

| Surface             | Current vocabulary                                                                                                                           | Planned review                                                                                 |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| REPL help           | `/caawi`, `/help`                                                                                                                            | Retain both commands; review explanatory Somali.                                               |
| REPL exit           | `/bixi`, `/exit`                                                                                                                             | Retain both commands.                                                                          |
| REPL file loading   | `/akhrifayl`, `/load`                                                                                                                        | Review joined spelling only with a compatibility plan.                                         |
| Compiler inspection | `/ast`, `/hir`, `--ast`, `--hir`                                                                                                             | Retain acronyms; explain AST and HIR rather than presenting them as translated Somali words.   |
| CLI flags           | `--build`, `--output`, `--opt-level`, `--quiet`, `--command`, `--file`, `--example`, `--interactive`, `--strict`, `--no-color`, version/help | Preserve command spellings; review human-readable help.                                        |
| Error headings      | `Khalad`, `lexer`, `parser`, `runtime`, `nooc`, `import`                                                                                     | Agree Somali labels and technical explanations; retain machine-readable error codes.           |
| Argument validation | `dood`, `qiimo`, `param`                                                                                                                     | Use `dood` for call arguments and `qiime` for values; define parameter terminology separately. |
| Type checking       | `dynamic`, `strict mode`, `abn`, `jajab`                                                                                                     | Refer to actual source type names and explain static/dynamic behavior in plain Somali.         |
| Arithmetic errors   | `Habka`, `Tixgacaysi`                                                                                                                        | Replace unclear operation wording with direct explanations of what failed.                     |
| Method errors       | `No method`, `Unknown list/object/string method`                                                                                             | Use consistent Somali sentences including receiver type and requested name.                    |
| File/build messages | `fayl`, `Barnaamijka waa la dhisayaa`, build failures                                                                                        | Review spelling, agreement, and useful technical context.                                      |

Suggested templates for the next editing stage:

| Existing wording                                     | Candidate wording                                                                      |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `Habka waa in ay noqdaan tiro`                       | `Hawlgalkani wuxuu u baahan yahay tirooyin.`                                           |
| `Tixgacaysi (-) waa in uu noqdo tiro`                | `Hawlgalka (-) wuxuu u baahan yahay tiro.`                                             |
| `No method '{}' on this type`                        | `Noocan ma laha hawl la yiraahdo '{}'.`                                                |
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
is made. Do not reassign a name to a different meaning, merge `kasaar()` with
`ka_saar(index)`, or change return values as a side effect of a rename. Aliases
must share type checking, arity, mutation, and error behavior.

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
