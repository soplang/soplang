# Soplang 3 Roadmap

This roadmap defines the planned direction of Soplang after the 3.0 release.
It orders work by dependency and product value. It does not assign calendar
deadlines; a release is ready when its design, implementation, documentation,
and tests are complete.

## Version policy

Soplang is on the **version 3 product line**. The released baseline is
`3.0.1`, and planned releases use `3.x.y` versions.

- `3.x.0` releases may add compatible language, runtime, library, and tooling
  capabilities.
- `3.x.y` patch releases fix defects and make compatible documentation or
  tooling improvements.
- Compatibility aliases and migration notes should accompany changed syntax or
  public APIs.
- There is no approved or scheduled version 4. Plans, issues, and release
  material must keep future work on version 3 unless the project makes a
  separate, explicit decision to change the major version.

Somali vocabulary for new public syntax and APIs must be agreed and documented
before implementation. This prevents temporary names from becoming permanent
compatibility obligations.

The [terminology review](docs/language/terminology.md) records the existing
vocabulary, evidence, proposed changes, and compatibility decisions required
before introducing new canonical names.

## Implementation priorities

File access and web application development are planned standard-library and
runtime capabilities. Their dependency order is:

| Order | Capability | Planned milestone |
| --- | --- | --- |
| 1 | Memory reclamation, resource cleanup, and recoverable execution errors | 3.1 runtime foundations |
| 2 | Iteration, byte arrays, and stronger function and data contracts | 3.2 language foundations |
| 3 | Text files, directories, paths, JSON, and application configuration | First part of 3.3 |
| 4 | HTTP client and a limited HTTP server for JSON APIs | After the file and data foundations in 3.3 |
| 5 | Modules, packages, and web application facilities such as middleware and database access | 3.4 and subsequent 3.x additions |
| 6 | Concurrent request handling, asynchronous I/O, streaming, and WebSockets | 3.6 after its runtime prerequisites |

Whole-file text operations can be implemented independently of binary streams.
A limited sequential HTTP server can precede public async syntax. Concurrent
servers require an explicit execution and isolation model before release.

The initial web API should use simple route registration and request/response
objects inspired by Express, with explicit server configuration and shutdown
inspired by Go. Its implementation belongs in the Rust runtime and must work
with both Cranelift JIT execution and LLVM-built executables. The first server
targets JSON APIs; HTML rendering, forms, uploads, and database integrations
follow separately.

These milestones express order and intended scope, not delivery dates. Public
Somali names, signatures, and the underlying Rust HTTP library require focused
design review before implementation. Documentation remains in English, with
Soplang syntax and actual Somali diagnostic output preserved.

## Soplang 3.0 — compiled language foundation

Soplang 3.0 establishes the release baseline:

- Somali syntax with optional semicolons.
- Dynamic variables and statically typed primitives and class instances.
- Functions, anonymous functions, closures, callbacks, and recursion.
- Classes, inheritance, constructors, and `waalid`.
- Lists, objects, Unicode strings, and short-circuit logic.
- Numeric ranges, conditionals, switch statements, and loops.
- Catchable runtime failures with `tijaabi` and `qabo` (`fasax` remains supported).
- File imports with `keen`.
- Cranelift JIT execution and a persistent interactive REPL.
- LLVM AOT compilation to standalone native binaries.
- Canonical collection names, safe defaults, integer slicing bounds, occurrence counting, and Unicode string endpoints.
- Somali diagnostics with original files, token locations, and Unicode-aware carets in both backends.
- Cross-platform packaging scripts and automated release infrastructure.

## Soplang 3.1 — runtime stability and diagnostics

Implementation complete; publication pending. See the [milestone notes](docs/releases/3.1.0.md)
and [local verification](docs/testing/RUNTIME_3_1.md). Later milestones below
remain planned; their scope is not implied by the completed 3.1 foundation.

Every later feature depends on a runtime that supports long-running programs
and reports failures precisely.

### Memory and runtime

1. Reclaim runtime strings, lists, objects, instances, and closure data.
2. Reuse handle slots safely and detect stale handles.
3. Preserve cycle-safe collection formatting and equality.
4. Add stress tests for long-running programs and REPL sessions.
5. Keep Cranelift JIT and LLVM AOT runtime behavior identical.
6. Define ownership and cleanup for external resources such as file handles
   and network connections.

The [ownership design](docs/architecture/RUNTIME_CONTEXTS.md) compares reference
counting, reusable handle tables, and tracing collection. The implementation
combines them with explicit stack roots and context ownership.

### Diagnostics and errors

1. Preserve filenames, lines, and columns through HIR and both backends. **Completed for 3.0**, including imported files and source carets.
2. Produce compiled function stack traces.
3. Add user-raised structured errors.
4. Add guaranteed cleanup blocks equivalent to `finally`.
5. Preserve error causes when one error wraps another.

### Foundations for long-running applications

1. Add an execution boundary that lets a server contain a request-handler
   failure, return an error response, and continue serving requests.
2. Reclaim temporary request values and retained callbacks according to the
   memory model; test repeated successful and failing requests for unbounded
   memory growth.
3. Define the lifetime of globals, closures, request data, and application state.
4. Keep existing CLI behavior: an uncaught top-level failure still reports a
   Somali diagnostic and exits unsuccessfully.

The current runtime's thread-local heaps and values must not be passed to
arbitrary worker threads. Concurrent execution needs the isolation and value
ownership design planned for 3.6.

### Strings and calls

1. Add escaped strings, including `\n`, `\t`, `\\`, quotes, and Unicode escapes.
2. Add raw and multiline strings.
3. Add string interpolation and value formatting.
4. Allow any callable expression to be invoked, including a returned closure.
5. Replace or extend the eight-value dynamic call ABI.

## Soplang 3.2 — iteration, functions, and data types

The iteration stage is implemented in both backends; see the
[iteration contract](docs/language/iteration.md). [Default arguments](docs/language/function-arguments.md)
and variadic positional parameters are also implemented, with callback signature
validation for empty and nonempty collections. Named arguments and `**`
keyword collectors are implemented too. Compact [byte arrays](docs/language/bytes.md)
are implemented with checked mutation, strict UTF-8 conversion, type annotations,
and snapshot iteration. [Optional and union annotations](docs/language/optional-union-types.md)
are implemented too. [Typed lists and string-keyed objects](docs/language/typed-collections.md)
now enforce shared element contracts through aliases and nested mutations.
[Immutable tuples](docs/language/tuples.md) and positional annotations are also
implemented. [Sets](docs/language/sets.md) support immutable keys, hashing,
ordered iteration, and set algebra. [Typed sets](docs/language/typed-sets.md)
now enforce shared element contracts through aliases and nested collections.
Expanded [text](docs/language/text-methods.md), [list](docs/language/list-methods.md),
[object](docs/language/object-methods.md), and [numeric](docs/language/numeric-functions.md)
library operations are implemented in both backends. The 3.2 language scope is
complete locally; a version tag and distribution builds remain pending.

### Iteration

1. Iterate directly over lists and ranges.
2. Iterate over Unicode string characters.
3. Iterate over object keys, values, and key-value pairs.
4. Define an iterator protocol for future user-defined classes.
5. Add enumerate and zip operations.

Collection syntax is `kuceli (item ku source)`, with row bindings such as
`kuceli (index, item ku tirso(source))`. `ku` remains contextual. Classes and
objects can implement `wareeg()` and `xiga()`; native cursors share the same
step representation.

### Function arguments

1. Default arguments.
2. Variable-length positional arguments.
3. Named arguments.
4. Variable-length named arguments if they fit the object model cleanly.
5. Better static and runtime validation for callback signatures.

### Data types

1. Sets for unique immutable values: implemented as `urur`.
2. Immutable tuples: implemented as `xirmo`, including positional annotations.
3. Byte arrays: implemented as `baytyo`, with shared collection vocabulary.
4. Explicit structured error values: implemented as immutable `khalad` values with
   codes, causes, traces, and optional structured catches (see [3.1 errors](docs/releases/3.1.0.md#errors-and-cleanup)).
5. Optional and union types: implemented for variables, parameters, and returns.
6. Typed lists, objects, and positional tuples: implemented with shared nested
   contracts. Typed sets are also implemented as `urur[T]`.

### Collection methods

| Value | Planned operations |
| --- | --- |
| String | Implemented: character classification, capitalization, title casing, padding, partitioning, lines, words, and formatting; see [text methods](docs/language/text-methods.md). |
| List | Implemented: remove by value, extend in place, unique values, enumerate, zip, sum, and callback-based sorting; see [list methods](docs/language/list-methods.md). |
| Object | Implemented: insert when absent, merge in place, map entries, filter entries, and nested-path lookup; see [object methods](docs/language/object-methods.md). |
| Number | Implemented: logarithms, trigonometry, constants, clamping, and context-local seeded random generation; see [numeric functions](docs/language/numeric-functions.md). |

## Soplang 3.3 — files, structured data, and networking

This release grows the standard library after runtime resource handling is in
place. Implement file and JSON foundations first, followed by the HTTP client
and limited JSON server. Each part must pass its own release gates before the
next depends on it.

### Initial text file access

Implemented locally as `akhrifayl`, `qor_fayl`, and `kudar_fayl`; see
[text files](docs/language/text-files.md).

1. Read an entire UTF-8 text file into a string — implemented.
2. Create or overwrite a text file with an explicit write operation — implemented.
3. Append text with a separate operation that preserves existing content — implemented.
4. Open and close resources automatically for these convenience operations,
   including when an operation fails — implemented for these convenience calls.
5. Report missing files, permission failures, invalid UTF-8, and other I/O
   failures as catchable Somali errors with source locations — implemented.

Writing replaces existing content; appending is a distinct operation. Text
operations must not silently replace invalid UTF-8. Relative filesystem paths
are resolved from the process working directory. `keen` keeps its existing
rule of resolving imports relative to the importing source file.

### File management, directories, and paths

Implemented locally; see [filesystem](docs/language/filesystem.md).

1. Query existence, file type, size, and modification time — `fayl_jiraa`, `fayl_xog`.
2. Copy, move, rename, and delete files — `nuqul_fayl`, `wareeji_fayl`, `tirtir_fayl`.
3. Create directories, list their entries, and remove directories — `abuur_gal`, `tir_gal`, `tirtir_gal`.
4. Join paths and resolve relative paths across Linux, Windows, and macOS — `isku_xidh`, `baabi_jid`.
5. Specify overwrite behavior, recursive directory operations, symbolic-link
   handling, and cross-filesystem moves before publishing the API — optional
   overwrite on copy and optional recursive directory removal are implemented.
   Copy and delete act on a symlink itself. `wareeji_fayl` copies a file or
   symlink across filesystems and then removes the source. A directory move
   across filesystems is rejected.

Filesystem APIs should live in a dedicated standard-library module with short,
consistent Somali names. Where operations match existing collection APIs,
reuse their vocabulary and contracts where practical. Module names and method
names remain design decisions until the terminology review is complete.

### Binary files and advanced file access

After the initial text API and byte-array type are available, add:

1. Whole-file byte reads and writes — implemented as `akhrif_baytyo`, `qor_baytyo`, and `kudar_baytyo`; see [binary files](docs/language/binary-files.md).
2. Explicit open handles, access modes, flush, and close operations — implemented as `fayl_fur`, `fayl_qor`, `fayl_nadiifi`, and `fayl_xidh`; see [file handles](docs/language/file-handles.md).
3. Line-by-line reading and bounded chunks — implemented as `fayl_xariiq` and `fayl_akhris`. Unbounded streaming of request bodies remains with 3.6.
4. File positions and seeking — implemented as `fayl_raad` and `fayl_meel`.
5. Atomic replacement — implemented as `fayl_beddel`. POSIX rename replaces a file in the same directory atomically. Windows uses `MoveFileExW` with replace-existing and write-through.

`fayl_xidh` closes the handle. Process exit closes any handle still open. File
watching is a later 3.x extension.

### Structured data and utilities

1. JSON parsing and generation, available before JSON HTTP handlers — implemented as `json_akhris` and `json_qor`; see [JSON](docs/language/json.md).
2. CSV reading and writing — implemented as `csv_akhris` and `csv_qor`; see [CSV](docs/language/csv.md).
3. URL encoding and decoding — implemented as `url_qor` and `url_akhris`; see [URL encoding](docs/language/url.md).
4. Base64 encoding and decoding — implemented as `b64_qor` and `b64_akhris`; see [Base64](docs/language/base64.md).
5. Regular expressions — implemented as `regex_raadi`, `regex_dhammaan`, and `regex_beddel`; see [regular expressions](docs/language/regular-expressions.md).
6. Date, time, duration, parsing, and formatting APIs — implemented for UTC seconds as `waqti`, `waqti_akhris`, `waqti_qor`, `waqti_qayb`, `waqti_ka_qayb`, and `waqti_kudar`; see [date and time](docs/language/time.md). Time zones other than `Z` remain later work.
7. Program arguments and environment variables — implemented as `amarro`, `deegaan`, `deegaan_qor`, and `deegaan_tirtir`; see [program arguments](docs/language/process.md).
8. Child processes, exit codes, and captured command output — implemented as `fuli`; see [program arguments](docs/language/process.md#child-processes). The command is not passed through a shell.

JSON conversion must define how Soplang values map to JSON, how malformed
input is reported, and which values cannot be serialized. JSON parse and
serialization failures must be catchable. Parsed maps must preserve arbitrary
string keys, including `__class__`, without acquiring class identity from input
fields. Program arguments and environment
variables provide configuration for file-based tools and server applications.

### Synchronous HTTP client

Implemented locally as `http_codso`; see [HTTP client](docs/language/http.md).
A namespace module still waits for the 3.4 package work.

1. GET, POST, PUT, PATCH, and DELETE requests — implemented.
2. Request headers and text, byte, and JSON bodies — implemented.
3. Response status, headers, text, bytes, and JSON — implemented.
4. Configurable connection and total timeouts — implemented as `waqti`.
5. Bounded redirects and response sizes — implemented as `leexasho` and `cabbir`.
6. TLS using trusted system certificates — implemented for `https`.
7. Clear DNS, connection, TLS, timeout, and protocol errors — catchable as `E053`.
8. Test support through local servers so the suite does not depend on public
   network availability — covered by the local HTTP tests.

### Initial HTTP server for JSON APIs

The first server should provide a small application API backed by a maintained
Rust HTTP implementation. Implemented locally as `server_fur`, `server_aqbal`,
and `server_xidh`; see [HTTP server](docs/language/http-server.md). It handles
one request at a time.

1. Register handlers for GET, POST, PUT, PATCH, and DELETE routes — implemented.
2. Match fixed paths and named path parameters such as `/users/:id` — implemented.
3. Expose request method, path, query parameters, headers, and bounded text or
   JSON request bodies — implemented.
4. Construct responses with status codes, headers, text, JSON, and redirects — implemented.
5. Return predictable 404 responses and contain handler failures as 500
   responses while keeping the server running — implemented.
6. Configure host and port, request-body limits, connection limits, and read
   and write timeouts — host, port, body limit, and timeouts are implemented.
   The server accepts one connection at a time.
7. Stop accepting requests and release server resources on graceful shutdown — `server_xidh`.
8. Run the same application through JIT during development and as an LLVM-built
   native executable for deployment — covered by the server tests.

The initial execution model handles one request at a time. Public async/await
syntax is not part of this milestone.

### Lower-level networking

After the HTTP client is stable, 3.3 may add:

- DNS lookup — implemented as `dns_raadi`; see [DNS and TCP](docs/language/networking.md).
- TCP client and server sockets — implemented as `tcp_fur`, `tcp_dhegeysi`, `tcp_aqbal`, `tcp_qor`, `tcp_akhris`, and `tcp_xidh`. One connection is accepted at a time.
- UDP sockets — implemented as `udp_fur`, `udp_qor`, `udp_akhris`, and `udp_xidh`.
- A WebSocket client — implemented as `ws_fur`, `ws_qor`, `ws_akhris`, and `ws_xidh`; see [DNS and TCP](docs/language/networking.md).

Advanced concurrent servers, streaming bodies, and WebSocket servers follow
the concurrency and asynchronous I/O work in 3.6.

## Soplang 3.4 — modules and packages

### Module system

1. Give imported files their own namespaces — implemented as `keen "file.sop" sida name`; see [modules](docs/language/modules.md). Flat `keen` imports stay unchanged.
2. Add explicit public exports — implemented as `soosaar name, ...` inside a namespaced file, including classes. Callers construct and type-check an exported class through its namespace or a selective import. Without `soosaar`, names that start with `_` stay private.
3. Add selective imports and import aliases — implemented as `keen "file.sop" ka name` and `name sida local`. A namespace `sida` and `ka` cannot share one import.
4. Define relative and package imports — relative `keen` resolves from the importing file. `keen "xisaab"` and `keen "xisaab/util.sop"` load a dependency from the nearest manifest; see [packages](docs/language/packages.md).
5. Detect conflicting names and cycles with clear diagnostics — implemented. Cycles are errors; repeated namespaced imports share one initializer.
6. Specify module initialization and caching rules — implemented. A namespaced file is initialized when the import runs, and only once.

### Package system

Project creation is implemented as `soplang init [directory] [--lib] [--name name]`,
with `soplang.toml` and a root `main.sop` or `lib.sop` starter. Existing files
are preserved. See [packages](docs/language/packages.md#create-a-project).

1. Define a project and package manifest: implemented with TOML syntax and schema validation; see [packages](docs/language/packages.md).
2. Record names, versions, and supported Soplang versions: `magac`, `nusxa`, and `soplang`.
3. Resolve Git repository URLs, hosting-service short forms, and local paths: implemented. Revisions accept branches, tags, and full commits.
4. Keep one version per package: implemented as one repository/commit selection for each name, with conflicts rejected across the full graph.
5. Pin exact commits and content hashes in a deterministic root lock: implemented. Dependency-owned locks cannot override the root selection.
6. Verify a package cache and support offline builds: implemented as immutable staged trees, checksum verification, concurrent publication, and `--offline`.
7. Add, remove and explicitly update dependencies: implemented through the package flags. Git publication uses the repository's own commit/tag workflow; no central package host is required.

The directory-registry and `.sopkg` providers remain compatibility sources.
Standard command forms are implemented in 3.5. Public class exports, the
sequential web framework, TLS, multipart uploads, templates, and the ORM are
implemented. Concurrent serving remains 3.6.

### Web application facilities

Build a bundled, importable web framework on the server and module foundations.
The minimal sequential JSON server alone does not complete this requirement.

1. Routers, method/path matching, route groups, middleware order and error propagation — implemented by `keen "web"`; see [web](docs/language/web.md). An uncaught handler or middleware error is still status 500.
2. Path/query/header/cookie access, JSON and form parsing, validation, response status and JSON/HTML/redirect/file helpers — implemented. URL-encoded forms use `foom`. Multipart bodies use `qayb`.
3. Static-file containment and CORS — implemented. `fayl_gudaha` rejects paths that leave the root. `habee` renders HTML templates. Ranged downloads remain later web work.
4. Sessions with expiration, authentication hooks, logging, request limits and timeouts — implemented on the `Web` value. Limits and timeouts use the existing `waqti` and `cabbir` options.
5. TLS and a test client — implemented. `server_fur(host, port, {shahaado, fure})` serves one TLS connection at a time, and `Web.tijaabi` is an in-memory client using the same HTTP codec and route pipeline. Graceful concurrent shutdown, streaming, and WebSocket servers remain 3.6.
6. A Prisma-style ORM with required `schema.sop`, schema validation, typed query/editor support, migrations, pools and transactions — implemented. See [data](docs/language/data.md). `check` and the language server validate `schema.sop`.
7. Real connector implementations and tests for SQLite, PostgreSQL, MySQL, MariaDB, MongoDB, SQL Server and CockroachDB, with a documented capability matrix — implemented. SQLite runs in the default suite. The other six run when `SOPLANG_POSTGRES`, `SOPLANG_COCKROACH`, `SOPLANG_MYSQL`, `SOPLANG_MARIADB`, `SOPLANG_SQLSERVER`, or `SOPLANG_MONGO` is set. SQL Server uses TDS with TLS.

The ORM is required before the final release. These are implementation
requirements; database selection and a decision about whether to include an ORM
are already settled. Document each connector's supported operations explicitly.

## Soplang 3.5 — developer tooling

### Command-line tools

1. `soplang fmt` for deterministic formatting — implemented. `--qor` writes the file. A second format leaves the text unchanged.
2. `soplang check` for analysis without execution — implemented. `--adag` is strict mode, and a sibling `schema.sop` is validated.
3. `soplang test` for native test discovery and execution — implemented.
4. `soplang doc` for API documentation generation — implemented. `--qor` writes Markdown.
5. Package add, remove, and update commands — implemented as `--ku-dar`, `--ka-saar`, and `--cusbooneysii`.
6. Consistent run and build commands for package projects — implemented as `soplang run` and `soplang dhis`. `run` executes `gelid/main.sop` or `main.sop`. `dhis` writes `dhis/<magac>`.

### Testing

1. Test declaration conventions or syntax — implemented as top-level `tijaabo "name" { }` and `tijaabo "name" qalad { }`. Ordinary execution skips them.
2. Assertions and expected-error assertions — implemented. `hubi(value)` requires `run`, and `hubi(left, right)` requires equality. `qalad` passes only for a runtime failure in the body; `qalad("CODE")` additionally checks the error code. Compiler and fixture errors fail the test.
3. Test discovery and filtering — implemented. `--shaandhe` keeps names that contain the text. `--lambarka` selects one test. Directory discovery recognizes `*_test.sop`, `*.test.sop`, and `.sop` under `tests`.
4. Setup and cleanup support — implemented as top-level `diyaar { }` and `nadiifi { }`. Cleanup runs after the test body, including when the body fails.
5. Parallel execution — implemented as `--wada`. Each test owns a runtime; parallel runs use a bounded pool of isolated child processes (`--shaqaale` selects the limit).
6. Coverage reporting — implemented as `--dabool`. It reports reached/executable counts, percentages, and missed lines separately for each file, including uncalled functions, with bounded hit storage and equivalent parallel aggregation.

### Editor and debugging support

1. Editor syntax packages — implemented in `editors/vscode` for `.sop` files.
2. A Language Server Protocol implementation — implemented as `soplang lsp` over stdio.
3. Live diagnostics, completion, hover help, and semantic highlighting — implemented.
4. Go to definition, find references, and rename — implemented.
5. Formatting and code actions — implemented. The code action is `Hagaaji qaabka`.
6. Source-level debugger information and breakpoints — implemented in JIT and instrumented LLVM executables. `soplang debug` supports file/line breakpoints, statement stepping, step-over and return; ordinary builds omit instrumentation.
7. Stack and local inspection — implemented. The source console selects caller frames and reads initialized local and captured values from rooted compiled storage without evaluating source. Inspection depth and output are bounded.
8. Profiler integration — implemented as `soplang profile file.sop`. It reports per-function calls and inclusive/exclusive time, with compilation timed separately from execution.

## Soplang 3.6 — concurrency, asynchronous I/O, and servers

This work follows memory reclamation, resource cleanup, and synchronous network
APIs.

### Concurrency

1. Lightweight tasks.
2. Channels for communication.
3. Thread-safe shared runtime values.
4. Cancellation and timeouts.
5. Structured task lifetimes.
6. Protection against data races.

### Asynchronous I/O

1. Async file operations.
2. Async HTTP requests.
3. Async TCP and UDP.
4. Timers and event-loop integration.
5. Streaming request and response bodies.

The project must decide whether Soplang exposes explicit async/await syntax or
uses another task model before this work begins.

### Advanced server capabilities

Build on the sequential JSON server and request/response API from 3.3:

1. Concurrent request handling with explicit isolation and safe application
   state sharing.
2. Cancellation and deadlines propagated through handlers and I/O.
3. Streaming request and response bodies with backpressure.
4. WebSocket servers and defined connection lifetimes.
5. Graceful draining of active requests and connections during shutdown.
6. Bounded queues and limits on concurrent requests and retained resources.
7. Stress and load tests covering slow clients, disconnects, failures, and
   sustained memory use.

## Later Soplang 3.x — advanced ecosystem

These items remain on the version 3 line but are not assigned to a specific
minor release:

- Generic user-defined types.
- Interfaces, traits, or protocols.
- Pattern matching and destructuring.
- Immutable collection variants.
- Decimal and arbitrary-precision integers.
- A stable foreign-function interface.
- Native extension packages.
- An API for embedding Soplang in other applications.
- Sandbox and capability controls.
- Additional compiler optimizations.
- WebAssembly as another build target.

## Open design decisions

The following choices must be resolved through focused design proposals before
their implementation release begins:

1. Memory management: reference counting, reusable handles, or tracing GC.
2. Iteration: Somali syntax and the user-defined iterator protocol.
3. Errors: Somali vocabulary for raising errors and guaranteed cleanup.
4. Modules: namespace, export, selective import, and alias syntax.
5. Networking: the HTTP module API and lower-level socket abstractions.
6. Concurrency: tasks and channels, async/await, or another model.
7. Packages: Git repositories and local paths, one version per package, an exact commit/content lock, verified caching, offline builds, and explicit updates. A central registry is not required.
8. Filesystem: decided for this stage. See [filesystem](docs/language/filesystem.md) and [file handles](docs/language/file-handles.md) for symlink, cross-filesystem, and atomic-replace behavior.
9. Web APIs: route registration, handler callbacks, request/response types,
   middleware, error containment, and the Rust HTTP implementation.
10. Web application integrations: database drivers, templates, and session
    storage, delivered independently after the initial JSON server.

## Release requirements

Every Soplang 3 release must:

1. Keep Cranelift JIT and LLVM AOT behavior consistent.
2. Add tests for syntax, semantics, runtime behavior, and generated binaries.
3. Test networking against controlled local services rather than public hosts.
4. Pass the locked Rust suite and the dedicated LLVM integration suite.
5. Document new syntax and public APIs in the language reference.
6. Include migration notes for changed behavior.
7. Build and verify supported platform and container artifacts.
8. Test filesystem APIs in temporary directories on supported platforms,
   including UTF-8, binary data where supported, failed operations, path
   behavior, and resource cleanup.
9. Test servers with controlled local clients through both backends, covering
   routing, JSON, request limits, handler failures, shutdown, and repeated
   requests without unbounded memory growth.

