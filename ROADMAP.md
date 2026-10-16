# Soplang 3 Roadmap

This roadmap defines the planned direction of Soplang after the 3.0 release.
It orders work by dependency and product value. It does not assign calendar
deadlines; a release is ready when its design, implementation, documentation,
and tests are complete.

## Version policy

Soplang is on the **version 3 product line**. The current release target is
`3.0.0`, and planned releases use `3.x.y` versions.

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

## Soplang 3.0 — compiled language foundation

Soplang 3.0 establishes the release baseline:

- Somali syntax with optional semicolons.
- Dynamic variables and statically typed primitives and class instances.
- Functions, anonymous functions, closures, callbacks, and recursion.
- Classes, inheritance, constructors, and `waalid`.
- Lists, objects, Unicode strings, and short-circuit logic.
- Numeric ranges, conditionals, switch statements, and loops.
- Catchable runtime failures with `fasax` and `qabo`.
- File imports with `keen`.
- Cranelift JIT execution and a persistent interactive REPL.
- LLVM AOT compilation to standalone native binaries.
- Cross-platform packages and automated release infrastructure.

## Soplang 3.1 — runtime stability and diagnostics

Every later feature depends on a runtime that supports long-running programs
and reports failures precisely.

### Memory and runtime

1. Reclaim runtime strings, lists, objects, instances, and closure data.
2. Reuse handle slots safely and detect stale handles.
3. Preserve cycle-safe collection formatting and equality.
4. Add stress tests for long-running programs and REPL sessions.
5. Keep Cranelift JIT and LLVM AOT runtime behavior identical.

The memory design must compare reference counting, reusable handle tables, and
tracing collection before an implementation is selected.

### Diagnostics and errors

1. Preserve filenames, lines, and columns through HIR and both backends.
2. Produce compiled function stack traces.
3. Add user-raised structured errors.
4. Add guaranteed cleanup blocks equivalent to `finally`.
5. Preserve error causes when one error wraps another.

### Strings and calls

1. Add escaped strings, including `\n`, `\t`, `\\`, quotes, and Unicode escapes.
2. Add raw and multiline strings.
3. Add string interpolation and value formatting.
4. Allow any callable expression to be invoked, including a returned closure.
5. Replace or extend the eight-value dynamic call ABI.

## Soplang 3.2 — iteration, functions, and data types

### Iteration

1. Iterate directly over lists and ranges.
2. Iterate over Unicode string characters.
3. Iterate over object keys, values, and key-value pairs.
4. Define an iterator protocol for future user-defined classes.
5. Add enumerate and zip operations.

The Somali syntax for “for each item in collection” must be designed before the
parser and iterator protocol are implemented.

### Function arguments

1. Default arguments.
2. Variable-length positional arguments.
3. Named arguments.
4. Variable-length named arguments if they fit the object model cleanly.
5. Better static and runtime validation for callback signatures.

### Data types

1. Sets for unique values.
2. Immutable tuples.
3. Byte arrays.
4. Explicit structured error values.
5. Optional and union types.
6. Typed lists, objects, sets, and tuples.

### Collection methods

| Value | Planned operations |
| --- | --- |
| String | character classification, capitalization, title casing, padding, partitioning, lines, words, and formatting |
| List | occurrence count, remove by value, extend in place, unique values, enumerate, zip, sum, and callback-based sorting |
| Object | insert when absent, merge in place, map entries, filter entries, and nested-path lookup |
| Number | logarithms, trigonometry, constants, clamping, and seeded random generation |

## Soplang 3.3 — files, structured data, and networking

This release grows the standard library after runtime resource handling is in
place.

### Files and paths

1. Read, write, and append text files.
2. Read and write byte arrays.
3. Guarantee safe resource cleanup.
4. Query file existence and metadata.
5. Copy, move, rename, and delete files.
6. Create, inspect, and remove directories.
7. Join and normalize cross-platform paths.

### Structured data and utilities

1. JSON parsing and generation.
2. CSV reading and writing.
3. URL encoding and decoding.
4. Base64 encoding and decoding.
5. Regular expressions.
6. Date, time, duration, parsing, and formatting APIs.
7. Program arguments and environment variables.
8. Child processes, exit codes, and captured command output.

### Synchronous HTTP client

The first network API should provide:

1. GET, POST, PUT, PATCH, and DELETE requests.
2. Request headers and text, byte, and JSON bodies.
3. Response status, headers, text, bytes, and JSON.
4. Configurable connection and total timeouts.
5. Bounded redirects and response sizes.
6. TLS using trusted system certificates.
7. Clear DNS, connection, TLS, timeout, and protocol errors.
8. Test support through local servers so the suite does not depend on public
   network availability.

The HTTP API should use a dedicated module. Its Somali module and method names
must be decided before becoming public.

### Lower-level networking

After the HTTP client is stable, 3.3 may add:

- DNS lookup.
- TCP client and server sockets.
- UDP sockets.
- A WebSocket client.

An HTTP server framework waits for the concurrency and asynchronous I/O work in
3.6.

## Soplang 3.4 — modules and packages

### Module system

1. Give imported files their own namespaces.
2. Add explicit public exports.
3. Add selective imports and import aliases.
4. Define relative and package imports.
5. Detect conflicting names and cycles with clear diagnostics.
6. Specify module initialization and caching rules.

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

### Server capabilities

1. HTTP server primitives.
2. Routing and middleware foundations.
3. Request and response objects.
4. Streaming bodies.
5. WebSocket servers.
6. Graceful shutdown.
7. Connection, request, and resource limits.

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

## Release requirements

Every Soplang 3 release must:

1. Keep Cranelift JIT and LLVM AOT behavior consistent.
2. Add tests for syntax, semantics, runtime behavior, and generated binaries.
3. Test networking against controlled local services rather than public hosts.
4. Pass the locked Rust suite and the dedicated LLVM integration suite.
5. Document new syntax and public APIs in the language reference.
6. Include migration notes for changed behavior.
7. Build and verify supported platform and container artifacts.

