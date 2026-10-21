# JSON

`json_akhris` and `json_qor` convert between JSON text and Soplang values.
Parse and serialization failures are catchable and use error code `E045`.

## Value mapping

| JSON | Soplang |
| --- | --- |
| `null` | `maran` |
| `true` / `false` | `bool` |
| integer without a fraction or exponent, inside 64-bit range | `abn` |
| other finite numbers | `jajab` |
| string | `qoraal` |
| array | `teed` |
| object | `walax` |

Object keys stay ordinary strings. A key named `__class__` does not turn the
value into a class instance. Class dispatch and class type checks use constructor-owned
identity in the active runtime context. Copying fields or parsing serialized
instance data cannot create that identity.

`json_qor` writes object keys in sorted order. It accepts `maran`, `bool`,
`abn`, finite `jajab`, `qoraal`, `teed`, and `walax`. Functions, bytes, tuples,
sets, errors, iterators, non-finite floats, and cyclic collections are rejected.

```soplang
door data = json_akhris("{\"magac\":\"Ayaan\",\"da\":7}")
qor(data.magac)
qor(json_qor({magac: "Ayaan", da: 7}))
```

Whole-number floats are written with a decimal point so a later parse keeps
`jajab`.

JSON whitespace is limited to space, tab, carriage return, and line feed, as
defined by [RFC 8259, section 2](https://www.rfc-editor.org/rfc/rfc8259#section-2).
Other control characters outside strings are rejected.
