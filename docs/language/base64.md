# Base64

`b64_qor` and `b64_akhris` convert between binary data and Base64 text.
Decode failures and unknown modes are catchable and use error code `E048`.

`b64_qor` accepts a `qoraal` or a `baytyo`. Text is encoded as UTF-8 bytes.
The result is a `qoraal`. Standard encoding uses `+`, `/`, and `=` padding.

`b64_akhris` accepts Base64 text and returns a `baytyo`. Pass that value to
`qoraal` when the payload is UTF-8 text.

Pass `"url"` as the second argument for the URL-safe alphabet `-` and `_`
without padding.

```soplang
qor(b64_qor("Man"))
door bytes = b64_akhris("TWFu")
qor(qoraal(bytes))
qor(b64_qor(baytyo([251, 255]), "url"))
```
