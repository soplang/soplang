# CSV

`csv_akhris` and `csv_qor` convert between CSV text and a list of rows. Each
row is a list of fields. Parse and serialization failures are catchable and use
error code `E046`.

Reading accepts commas, CR, LF, and CRLF. A quoted field may contain commas and
line breaks. A doubled quote inside a quoted field is one quote character. An
empty string parses as no rows. A final newline does not add an extra row.

Writing accepts `qoraal`, `abn`, finite `jajab`, and `maran` cells. `maran`
becomes an empty field. Fields that contain a comma, quote, or line break are
quoted. Rows are separated by `\n`.

```soplang
door rows = csv_akhris(akhrifayl("dad.csv"))
qor(rows[0][0])
qor_fayl("dad.csv", csv_qor([["magac", "da"], ["Ayaan", 7]]))
```
