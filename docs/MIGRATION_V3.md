# Migrating to Soplang 3

Soplang 3 establishes the Rust compiler, the Cranelift JIT, and the LLVM AOT backend as the supported implementation. This release also makes runtime behavior stricter and finalizes the public vocabulary.

## Vocabulary

Update programs that still use names from earlier Soplang 2 releases or documentation:

| Earlier spelling | Soplang 3 |
| --- | --- |
| `howl` | `hawl` |
| `soo_celi` | `celi` |
| `bandhig` | `qor` |
| `akhri` | `gelin` |
| `ku_celi` | `kuceli` |
| `inta_ay` | `intay` |
| `sii_wad` | `soco` |
| `tiro` | `abn` |
| `liis` | `teed` |
| `shey` | `walax` |
| `labadaran` | `bool` |
| `waxba` | `maran` |
| `kudhow` | `xul` |
| `raadso` | `muuji` |
| `null` | `maran` |

`null` remains accepted as a compatibility spelling. New code should use `maran`.

## Calls and constructors

Functions, closures, methods, built-ins, and constructors now enforce their argument counts. Calls that previously ignored extra arguments or filled missing arguments with `maran` report a catchable error.

```soplang
hawl laban(x) { celi x * 2 }

fasax {
    laban()
} qabo (qalad) {
    qor(qalad)
}
```

Constructor arguments are also checked against the inherited or local `dhaw()` definition.

## Numeric conversion and ranges

`abn()` and `jajab()` accept trimmed numeric strings:

```soplang
door count = abn(" 42 ")
door price = jajab("3.5")
```

`abn()` rejects decimal strings such as `"3.5"`. `baaxad()` accepts integer arguments only and reports an error when its step is zero.

## Indexing

String indexes count Unicode characters. Lists and strings support negative indexes from the end. An out-of-range read returns `maran`; an out-of-range list write reports a catchable error.

Objects now accept computed string keys:

```soplang
door qof = {}
qof["magac"] = "Ayaan"
qor(qof["magac"])
```

## Logical operators

`&&` and `||` now short circuit. The right expression runs only when its result is needed.

## Semicolons

Semicolons are optional and supported consistently. Existing newline-oriented source remains valid.

```soplang
door x = 1;
qor(x);
```

## New standard library APIs

- Numbers: `dhammeystir`, `sugan`, `ugu_yar`, `ugu_weyn`, `awood`, `xidid`
- Lists: `geli`, `ka_saar`, `koowaad`, `ugu_dambeeya`, `isku_gee`, `dhammaan`, `qaar`
- Objects: `hel`, `dhig`, `ka_qaad`, plus bracket access
- Strings: `dherer`, `masax_bilow`, `masax_dhamaad`, `tiri`, `ku_celi`, `raadi_dambe`

See the [language reference](language/keywords.md) for signatures and examples.
