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
   overwrite on copy and optional recursive directory removal are implemented;
   symlink semantics and cross-volume moves remain design gates.

Filesystem APIs should live in a dedicated standard-library module with short,
consistent Somali names. Where operations match existing collection APIs,
reuse their vocabulary and contracts where practical. Module names and method
names remain design decisions until the terminology review is complete.

### Binary files and advanced file access

After the initial text API and byte-array type are available, add:

1. Whole-file byte reads and writes — implemented as `akhrif_baytyo`, `qor_baytyo`, and `kudar_baytyo`; see [binary files](docs/language/binary-files.md).
2. Explicit open handles, access modes, flush, and close operations.
3. Line-by-line reading, bounded chunks, and streaming large files.
4. File positions and seeking.
5. Atomic replacement where supported, with documented platform guarantees.

Handles require guaranteed cleanup on success and failure, including early
returns. File watching is a later 3.x extension. Basic text access does not
wait for handles, streaming, or file watching.

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
2. Add explicit public exports. Names that start with `_` are already private to a namespaced import.
3. Add selective imports and import aliases. `sida` is the namespace alias. Selective imports remain later work.
4. Define relative and package imports.
5. Detect conflicting names and cycles with clear diagnostics. A repeated or cyclic namespaced import is an error.
6. Specify module initialization and caching rules. A namespaced file is initialized when the import runs, and only once.

### Package system

1. Define a project and package manifest.
2. Record package names, versions, and supported Soplang versions.
3. Add dependency declarations and local path dependencies.
4. Create a deterministic lockfile.
5. Add a package cache and integrity hashes.
6. Add install, update, remove, and publish commands.
7. Make dependency resolution and builds reproducible.

Package downloads will use the HTTP and archive foundations from 3.3. Whether
the first package source is Git repositories or a dedicated registry remains a
design decision.

### Web application facilities

After the initial JSON server and module interfaces are stable, develop these
as standard-library or package additions across 3.4 and later 3.x releases:

1. Composable routers and middleware with a defined execution order and error
   propagation contract.
2. Static file responses with a configured document root and path containment.
3. HTML responses, template rendering, and form parsing.
4. Bounded multipart uploads and downloads, with streaming dependent on 3.6.
5. Cookies and sessions, with documented storage and expiration behavior.
6. Database drivers, beginning with a separately selected database and covering
   parameterized queries, transactions, connection lifetimes, and cleanup.

Database support is a separate deliverable from serving JSON APIs. Database
selection, session storage, and template engines need their own proposals;
the initial server does not commit the language to a particular framework or
database abstraction.

## Soplang 3.5 — developer tooling

### Command-line tools

1. `soplang fmt` for deterministic formatting.
2. `soplang check` for analysis without execution.
3. `soplang test` for native test discovery and execution.
4. `soplang doc` for API documentation generation.
5. Package add, remove, and update commands.
6. Consistent run and build commands for package projects.

### Testing

1. Test declaration conventions or syntax.
2. Assertions and expected-error assertions.
3. Test discovery and filtering.
4. Setup and cleanup support.
5. Parallel execution.
6. Coverage reporting.

### Editor and debugging support

1. Editor syntax packages.
2. A Language Server Protocol implementation.
3. Live diagnostics, completion, hover help, and semantic highlighting.
4. Go to definition, find references, and rename.
5. Formatting and code actions.
6. Source-level debugger information and breakpoints.
7. Stack and local-variable inspection.
8. Profiler integration.

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
7. Packages: Git-based sources, a dedicated registry, or a staged combination.
8. Filesystem: Somali vocabulary, access modes, cleanup, overwrite behavior,
   directory recursion, path rules, and platform-specific guarantees.
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

