# Data

`schema.sop` describes tables. `keen "xog" sida xog` opens a pool and runs
queries through a real database client. Failures use `E059`.

```soplang
jadwal qof {
  id abn furaha
  magac qoraal
  da abn?
}
```

A table has one `abn furaha` column. Other columns are `abn`, `qoraal`,
`jajab`, or `bool`, and `?` marks an optional column. Names are lowercase
identifiers. `#` starts a comment.

```soplang
keen "xog" sida xog
door db = xog.fur("sqlite", "app.db", 2)
xog.schema(db, akhrifayl("schema.sop"))
xog.guuri(db)
xog.ku_geli(db, "qof", { magac: "Amina" })
door rows = xog.raadi(db, "qof", { magac: "Amina" })
```

`fur(kind, target, wadarta?)` opens one to eight connections. The default is
one. `guuri` creates missing tables and adds missing columns. It does not drop
columns. The schema hash is stored in `soplang_guuri`.

`ku_geli`, `raadi`, `beddel`, and `tirtir` accept objects whose fields are
columns. `raadi` returns at most 1000 rows. String values are escaped, and a
name that is not in the schema is rejected.

`macaamil(db, hawl() { ... })` runs the callback on one pinned connection and
commits when it returns. A failure rolls the transaction back. The callback
must not start another transaction.

`xog.awood(kind)` reports the operations that kind supports.

## Connectors

| `nooc` | Client | Target |
| --- | --- | --- |
| `sqlite` | bundled SQLite | filesystem path |
| `postgres` | Postgres wire protocol | `postgres://...` |
| `cockroach` | the same Postgres client | `postgres://...` |
| `mysql` | MySQL wire protocol | `mysql://...` |
| `mariadb` | the same MySQL client | `mysql://...` |
| `sqlserver` | TDS, without TLS | `server=tcp:host,1433;User Id=...;Password=...;Database=...` |
| `mongo` | official MongoDB client | `mongodb://host/database` |

SQLite is tested by the default suite, including rollback and a pool of two.
The other connectors are the same client code paths. Their live tests are
ignored until the matching variable is set: `SOPLANG_POSTGRES`,
`SOPLANG_COCKROACH`, `SOPLANG_MYSQL`, `SOPLANG_MARIADB`,
`SOPLANG_SQLSERVER`, or `SOPLANG_MONGO`.

```sh
cargo test --locked --test data_tests -- --ignored --nocapture
```

MongoDB multi-document transactions call `start_transaction`. A standalone
server rejects that call, and the error is returned as `E059`.

`soplang check` validates `schema.sop` beside a program, and `soplang lsp`
diagnoses a `schema.sop` document directly.
