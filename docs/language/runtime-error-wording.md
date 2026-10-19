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
