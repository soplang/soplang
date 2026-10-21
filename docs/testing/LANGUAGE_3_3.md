# Local verification for the 3.3 file and data stage

These checks cover the implemented 3.3 library. Streaming HTTP bodies and a
WebSocket server remain in 3.6.

## Initial UTF-8 text file access

Commands run locally on Linux:

```sh
cargo test --locked --test text_file_tests --test interpreter_tests --test llvm_ir_tests
cargo test --locked -- --skip test_aot
```

The dedicated text-file suite covers read/write/append round trips, catchable
missing-file errors, and invalid UTF-8 reads. LLVM runs the same operations at
optimization levels 0 and 2 with paths relative to the binary working directory.

Cross-platform executables and packaging remain distribution gates.

## Paths and filesystem management

```sh
cargo test --locked --test filesystem_tests --test text_file_tests --test llvm_ir_tests
```

The filesystem suite covers metadata, join/absolute paths, directory lifecycle,
copy/move/delete guards, and LLVM execution at optimization levels 0 and 2.
File handles, bounded reads, seeking, and atomic replacement are covered by
`cargo test --locked --test file_handle_tests`.

## Whole-file binary access

```sh
cargo test --locked --test binary_file_tests --test llvm_ir_tests
```

The binary suite covers byte round trips, append, strict `baytyo` arguments, and
LLVM execution at optimization levels 0 and 2.

## JSON

```sh
cargo test --locked --test json_tests --test llvm_ir_tests
```

The JSON suite covers value mapping, sorted object keys, a preserved `__class__`
field that stays a plain object, catchable parse and serialization failures, and
LLVM execution at optimization levels 0 and 2.

## CSV

```sh
cargo test --locked --test csv_tests --test llvm_ir_tests
```

The CSV suite covers quoted fields, numeric cells, a trailing newline, catchable
parse and type failures, and LLVM execution at optimization levels 0 and 2.

## URL encoding

```sh
cargo test --locked --test url_tests --test llvm_ir_tests
```

The URL suite covers component percent-encoding, form `+` spaces, preserved
plus signs, invalid hex and modes, and LLVM execution at optimization levels
0 and 2.

## Base64

```sh
cargo test --locked --test base64_tests --test llvm_ir_tests
```

The Base64 suite covers padding lengths, binary `+` and `/` characters, URL-safe
mode without padding, catchable alphabet and mode failures, and LLVM execution
at optimization levels 0 and 2.

## Regular expressions

```sh
cargo test --locked --test regex_tests --test llvm_ir_tests
```

The regular-expression suite covers the first match, a missing match, every
match, numbered replacements, catchable invalid patterns, and LLVM execution at
optimization levels 0 and 2.

## Date and time

```sh
cargo test --locked --lib time --test time_tests --test llvm_ir_tests
```

The time suite covers the Unix epoch, a known UTC instant, leap-day rejection,
offsets other than `Z`, calendar parts, one-day durations, the current clock
round trip, and LLVM execution at optimization levels 0 and 2.

## Program arguments and environment

```sh
cargo test --locked --test process_tests --test llvm_ir_tests
```

The process suite covers arguments after `-c` and after a source file, hyphen
arguments, environment reads, writes, removals, invalid names, and LLVM
execution with an argument after the built program name.

## Child processes

```sh
cargo test --locked --test process_tests --test llvm_ir_tests
```

The child-process checks cover captured output, a non-zero exit code, a missing
program, and LLVM execution. The command arguments are passed directly and are
not interpreted by a shell.

## HTTP client

```sh
cargo test --locked --test http_tests
```

The HTTP suite uses a local server. It covers GET headers, a JSON POST, a
redirect that is not followed, the response-size limit, a short timeout,
connection failure, a rejected method, an `https` URL that fails before a
public network request, and LLVM execution.

## HTTP server

```sh
cargo test --locked --test http_server_tests
```

The server suite drives the same program through JIT and LLVM. It checks path
parameters, query values, a JSON post, a 404, a handler failure returned as
status 500, a redirect header, and shutdown of the listener.

## DNS and TCP

```sh
cargo test --locked --test network_tests
```

The networking suite checks literal and local DNS names, a TCP echo between a
listener and a connected peer, closing a socket, and the same TCP exchange
through LLVM. It also exchanges one UDP datagram through JIT and LLVM.
The WebSocket checks exchange text and binary frames with a local server
through JIT and LLVM. A WebSocket server remains later work.

## JSON class-identity regression checks

The locked debug suite passes 341 tests after the class-identity fix, including
52 LLVM tests. `json_class_identity.sop` runs through JIT and LLVM at levels 0
and 2. It checks positional and named method dispatch, scalar and nested class
contracts, ordinary object iteration, field copies, round trips, inheritance,
constructors, and retained identity across automatic collection. A host test
checks context isolation and weak metadata cleanup. JSON parser tests reject
vertical tab and form feed outside strings.

## Resource ownership and replacement regression checks

The locked debug suite passes 347 tests, including 52 LLVM tests. New host
checks cover foreign-context file and socket operations, listener ports released
on context teardown, and TCP/WebSocket peers observing transport closure.
Filesystem tests cover concurrent atomic replacements, existing scratch-looking
neighbors, failed replacement cleanup, symlink targets, and dangling metadata.
The existing file and network integration suites exercise both JIT and LLVM.

## HTTP framing regression checks

The locked debug suite passes 357 tests, including 52 LLVM tests. Four local
protocol tests exercise duplicate and conflicting framing, Host validation,
unsigned decimal lengths, header/body size limits, CRLF and control-character
injection, reserved response headers, body types, and 204/304 framing.
The JIT and LLVM server tests verify that rejected requests and invalid handler
headers do not stop subsequent requests.

## WebSocket timeout audit

The full locked debug suite passes 364 tests, including all 52 dedicated LLVM
tests. The optimized selection passes 115 tests across the library, JSON,
module, project-init, file-handle, HTTP-server and LLVM suites. The complete
diagnostic/example suites pass all 14 tests through JIT and AOT.

Local stall tests cover both plain and TLS handshakes. Redirect tests cover a
successful upgrade after a redirect and rejection after three redirect hops.
The timeout remains a socket-operation limit; DNS and multi-operation deadlines
are not implied.
