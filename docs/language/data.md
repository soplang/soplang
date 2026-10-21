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
Column names must be unique. Names starting with `soplang_` are reserved for
database metadata.

```soplang
keen "xog" sida xog
door db = xog.fur("sqlite", "app.db", 2)
xog.schema(db, akhrifayl("schema.sop"))
xog.guuri(db)
xog.ku_geli(db, "qof", { magac: "Amina" })
door rows = xog.raadi(db, "qof", { magac: "Amina" })
xog.xidh(db)
```

`fur(kind, target, wadarta?)` opens one to eight connections. The default is
one. `guuri` creates missing tables and adds missing columns. It does not drop
columns. The schema hash is stored in `soplang_guuri`.
Connections belong to the execution context and close when it is destroyed.
`xidh(db)` closes a pool explicitly. A pool cannot be used from another context
or closed during a transaction. SQLite `:memory:` pools share one private
in-memory database across their connections.

`ku_geli`, `raadi`, `beddel`, and `tirtir` accept objects whose fields are
columns. `raadi` returns at most 1000 rows. SQL drivers bind values as parameters;
quotes and backslashes in application data cannot become SQL syntax. Unknown
columns, missing required fields, nulls in required fields, and incorrect value
types are rejected. An `abn` value may be widened to a `jajab` column. Optional
fields omitted on insertion become null, and null filters use `IS NULL`.

`macaamil(db, hawl() { ... })` runs the callback on one pinned connection and
commits when it returns. A failure rolls the transaction back. The callback
must not start another transaction, run a migration, or close the pool.
SQLite, PostgreSQL, CockroachDB and SQL Server migrations run in a transaction.
MySQL and MariaDB DDL uses the server's implicit commit behavior. MongoDB
migrations create or update collection validators and unique key indexes while
preserving existing documents. MongoDB integer keys use an atomic counter.

`xog.awood(kind)` reports the operations that kind supports.

## Connectors

| `nooc` | Client | Target |
| --- | --- | --- |
| `sqlite` | bundled SQLite | filesystem path |
| `postgres` | Postgres wire protocol | `postgres://...` |
| `cockroach` | the same Postgres client | `postgres://...` |
| `mysql` | MySQL wire protocol | `mysql://...` |
| `mariadb` | the same MySQL client | `mysql://...` |
| `sqlserver` | TDS with rustls TLS | `server=tcp:host,1433;User Id=...;Password=...;Database=...;Encrypt=true` |
| `mongo` | official MongoDB client | `mongodb://host/database` |

SQLite is tested by the default suite, including rollback and a pool of two.
Live tests cover parameter binding, Unicode text, nulls, typed reads, pooled
connections, rollback, commit, updates, deletes and repeated migrations that
preserve existing records. Their tests are ignored by default. Set the matching
connection variable and explicitly select the test: `SOPLANG_POSTGRES`,
`SOPLANG_COCKROACH`, `SOPLANG_MYSQL`, `SOPLANG_MARIADB`,
`SOPLANG_SQLSERVER`, or `SOPLANG_MONGO`.

```sh
cargo test --locked --test data_tests live_postgres -- --ignored --exact --nocapture
```

MongoDB multi-document transactions call `start_transaction`. A standalone
server cannot execute these transactions; the error is returned as `E059`.
Use a replica set or sharded cluster when testing transactions.

Database-driver details are attached as technical details to Somali `E059`
diagnostics. They can contain server information and should be handled as
diagnostic data.

`soplang check` validates `schema.sop` beside a program, and `soplang lsp`
diagnoses a `schema.sop` document directly.

## Generated typed clients

Generate a source module from the schema before compiling the application:

```sh
soplang xog abuur schema.sop --qor client.sop
soplang xog abuur schema.sop --hubi client.sop
```

Without an output option, the command prints the client to standard output.
Generation does not connect to a database or run application code. `--qor`
atomically replaces only files carrying the generated-client header and
preserves existing permissions and symbolic links. `--hubi` checks exact source
equality without writing; use it in CI to catch a stale client. Commit the
schema and generated source together. A client embeds its schema and hash;
changing `schema.sop` alone does not change an already compiled application.

For the `qof` schema above:

```soplang
keen "client.sop" sida c
c.MacmiilXog db = cusub c.MacmiilXog("sqlite", "app.db", 2)
tijaabi {
    db.guuri()
    db.qof.geli(cusub c.QofGelin(magac: "Amina", da: 22))
    c.QofShaandhe shuruud = cusub c.QofShaandhe().halkaMagac("Amina")
    teed[c.Qof] safaf = db.qof.raadi(shuruud)
    kuceli (qof ku safaf) { qor(qof.helMagac(), qof.helDa()) }
    db.qof.beddel(shuruud, cusub c.QofBeddel().dhigDa(23))
} dhammee { db.xidh() }
```

A table produces five public classes:

| Class for `qof` | Contract |
| --- | --- |
| `Qof` | Typed record getters such as `helId()` and `helMagac()`; `xog()` returns a copy for JSON conversion. |
| `QofGelin` | Required fields first, followed by nullable fields and an optional integer key. Named arguments are supported. |
| `QofShaandhe` | Typed equality filters such as `halkaMagac(value)` and `halkaDa(maran)`. Multiple fields are combined with AND. |
| `QofBeddel` | Typed setters such as `dhigMagac(value)`. Primary keys cannot be patched. |
| `QofJadwal` | `geli`, `raadi`, `mid`, `beddel`, `tirtir`, `beddelDhammaan`, and `tirtirDhammaan`. |

`raadi()` returns at most 1000 typed records; ordering is not guaranteed.
`mid(filter)` returns a record or `maran`. Mutations return affected counts,
not generated keys. `beddel` and `tirtir` reject an empty filter; intentional
bulk changes use the explicitly named `Dhammaan` methods.

`MacmiilXog` exposes each model through `jadwalQof()` and, when its name is a
valid non-conflicting Soplang identifier, `.qof`. A table named `qor` uses
`jadwalQor()`. Reserved column names receive safe constructor parameter names,
such as `qiimeHawl`. Class and method suffixes capitalize the first character
of the schema name; underscores are retained.

`db.macaamil(hawl(): maran { ... })` uses the same pinned transaction as the
native data API. `db.xidh()` is idempotent; closing during a transaction fails.
`db.xidhiidh()` exposes the native pool handle, and `db.sha256()` returns the
schema fingerprint. The generator validates its output in strict mode; JIT
and LLVM enforce the same argument and record contracts. Editor completion
shows model methods, input types and field argument types even after an
unfinished member access.

Generation is limited to 128 tables, 2048 columns and one MiB of source per
client. Split larger schemas into separate clients. The current query API
supports typed equality filters and scalar fields, without joins, relations,
aggregate expressions or arbitrary SQL. A typed client does not add operations
that its connector cannot perform.
