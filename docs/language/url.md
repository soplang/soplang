# URL encoding

`url_qor` and `url_akhris` percent-encode and decode UTF-8 text. Decode
failures and unknown modes are catchable and use error code `E047`.

By default, unreserved characters `A-Z`, `a-z`, `0-9`, `-`, `_`, `.`, and `~`
stay as themselves. Every other byte is written as uppercase `%HH`. A literal
`+` stays `+` when decoding.

Pass `"foom"` as the second argument for `application/x-www-form-urlencoded`
spaces: encoding writes a space as `+`, and decoding turns `+` back into a
space.

```soplang
qor(url_qor("Ayaan Cali"))
qor(url_qor("Ayaan Cali", "foom"))
qor(url_akhris("Ayaan%20Cali"))
```
