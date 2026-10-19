# Runtime ownership and execution contexts

Implemented in the runtime-foundations stages after 3.0.1. Execution
environments own their allocations and validate opaque handles at ABI boundaries.

## State ownership

Each Cranelift backend owns a `RuntimeContext`. Its state contains:

- Strings, list handles, and object handles.
- Globals, constant bindings, and class metadata.
- Built-in function indices and compiled function registrations.
- Closure environments and their captured tagged values.
- Current source location, try depth, and pending errors.
- Structured error handles, suspended cleanup errors, and active function frames.

Compiling a module, compiling another REPL snippet, and running generated code
activate that backend's context for the duration of the operation. The native
ABI resolves its stores through the active context. Scoped activation restores
the caller's context on normal return, compilation errors, and Rust unwinding.
Rust panics must never unwind across generated-code or C ABI boundaries.

Nested contexts retain independent state. Activating a nested environment does
not reset the caller's globals or pending errors. Stores have separate interior
borrows; the active-context selector is not borrowed across callbacks.

Contexts are confined to their owning thread. This implementation does not
make `Rc<RefCell<...>>` values transferable between workers.

## JIT programs and the REPL

`run_source` creates a backend and therefore a fresh execution context for each
program. A later invocation cannot inherit globals, constants, classes, or
function registrations from a backend that has already been dropped.

A `ReplSession` retains one backend and context across snippets. Two sessions
on the same thread remain independent, including when their function and class
names match. Each execution starts with a fresh pending-error state and restores
the surrounding boundary's state when it finishes.

Rust callers can inspect a session or backend with `get_global(name)`, which
returns `Option<Value>`. Missing globals return `None`. The lookup uses the
owner's context and restores the caller's active environment. Callable IDs
and closure identities in returned values still belong to their original
context; moving a value does not transfer its callable registrations.

## Standalone LLVM and direct ABI calls

Standalone LLVM executables and low-level ABI callers use a lazily initialized
default context on their current thread. They use the same runtime stores as
the JIT. The textual LLVM value representation and exported C ABI signatures
are unchanged.

Rust integrations can use `RuntimeContext::with` to scope low-level ABI work
explicitly. Tagged handles must be used in their owning context. Leaving a
scope restores its caller; dropping a context releases its owned tables.

## Recoverable execution boundaries

`RuntimeContext::execute(operation)` returns `Result<R, SoplangError>`. Runtime
failures retain the original structured diagnostic, including its source span,
and return to this boundary without terminating the host process. Nested
boundaries preserve surrounding pending errors and try depth, including during
Rust unwinding outside the C ABI. Rust panics are not caught as language errors.

The JIT executes `main` inside this boundary. An uncaught error becomes the
`Err` returned by `run_source`, `run_main`, or `ReplSession::eval`. A session
retains bindings that were initialized before a failure, then accepts another
snippet. Definitions that execution never reached are not remembered as globals.

HIR checks pending errors after each fallible operation, including operations
inside expressions. It jumps to the nearest local handler or returns from the
current function. Callers continue that propagation until a handler or execution
boundary is reached. Callback failures retain the original error metadata.
No Rust unwinding crosses generated code or an extern boundary.

Errors retain codes, explicit causes, and compiled function traces. A user
function records its name and caller location on entry and removes that frame
on every normal or error return. The first failure freezes the trace, so
propagation and rethrowing cannot replace it with an outer caller's location.
`qabo (fariin, xog)` retrieves message text and immutable structured details
before clearing the pending error. The one-binding catch form is unchanged.

Cleanup temporarily suspends pending errors so native operations and callbacks
can execute normally. Successful cleanup restores the original error. A new
cleanup failure takes precedence and retains the suspended error as its cause
when no explicit cause exists. Abrupt cleanup exits discard the suspended
error. Execution boundaries also restore surrounding suspended errors and
function frames, including during Rust unwinding outside the native ABI.

Standalone LLVM programs use the same propagation. Their `main` calls
`soplang_finish_execution`, which reports an uncaught Somali diagnostic and
returns status 1. Successful programs return 0. CLI reporting remains at the
outer boundary, so embedding applications can choose how to report failures.

## Allocation lifetime

### Compiled call representation

User functions in both backends receive a pointer to a contiguous array of
16-byte tagged values and its length. The runtime checks argument counts before
indirect calls. Known direct calls use the same representation. This removes
the earlier eight-value ABI limit for functions, methods, and closures.
Method calls prepend the instance; captured closures prepend their environment
handle. These implicit values no longer reduce the available user arguments.

Argument vectors belong to the caller and remain valid for the duration of the
call. Callees copy parameters and captured values into their own slots rather
than retaining vector pointers. LLVM allocates fixed scratch buffers once at
function entry; loops overwrite those buffers instead of growing the stack.
Cranelift uses fixed stack slots for the same purpose. The tagged value layout
and platform return bridges remain unchanged.

### Runtime allocations

Closure environments are held in the context instead of leaked with
`Box::leak`. Their identities are looked up in the active context before a
capture is read or a closure is called. Capture lookup does not dereference
an arbitrary or already-freed pointer. The environment parameter now carries
an opaque handle rather than a memory address in both backends.

Strings, collections, errors, native functions, compiled functions, and closures use
reusable arena slots. Each insertion receives a fresh positive 64-bit identity
from a process-wide counter. The arena maps that identity to a slot and verifies
it on lookup. A reused slot never accepts its previous identity; different
contexts and different value kinds cannot accidentally share identities.
Counter exhaustion is checked instead of wrapping. Zero and negative handles
are invalid.

Invalid handles produce `E035` with a Somali explanation. Runtime entry points
validate heap operands, callable arguments, and captured values. With an active
`tijaabi` block the error is catchable; Rust conversions return `Err`.
The tagged value layout remains 16 bytes. Primitive payloads are unchanged.

## Collection model

The runtime combines reusable arenas, reference-counted collection data, and
tracing at explicit generated-code safepoints.

| Mechanism | Contribution | Remaining requirement |
| --- | --- | --- |
| Reference counting | Keeps Rust-owned values and native callback temporaries alive. | Collection cycles need tracing. |
| Reusable handle arenas | Preserve tagged identities and reject expired handles. | Reachability determines which allocations can be removed. |
| Tracing | Follows globals, stack roots, pins, collection edges, and captures. | Generated code must publish its live slots before collection. |

Cranelift maintains tagged shadow slots alongside its SSA variables. LLVM
publishes its tag and payload arrays directly. Both register initialized slots
at function entry, update them after assignments, and unregister the frame on
normal and error returns. Collection occurs after operations that can allocate,
when temporary native operations have returned and generated results are rooted.
Nested callbacks register their own frames.

The collector builds an iterative graph of lists, objects, and closure data.
It counts arena references and collection edges, then treats remaining strong
references as roots held by native or Rust code. This preserves callback
snapshots even when a callback clears the original collection. Borrowed
collections are retained conservatively, with their deferred count included in
the collection report. Unreachable list and object edges are cleared before
arena entries are removed, breaking cycles without recursive traversal.

Lists and objects retain one handle per collection identity. Converting the
same collection repeatedly reuses that handle. Reachable nested collections
first discovered through Rust-owned value graphs are registered for future
passes. Strings and structured-error handles are swept by tagged reachability;
their Rust values own their contents separately. Host-held closure values own
references to closure data, so their captures remain reachable while the owner
is alive.

The first automatic pass is due after 1,024 managed allocations or about 4 MiB
of recorded allocation volume. Subsequent intervals grow with the live set.
The byte count is an estimate, not a process-memory quota. `collect()` and
`memory_stats()` are available on runtime contexts, JIT backends, and REPL
sessions. Counts cover managed values; compiler metadata and retained JIT code
are separate allocations. The JIT backend retains immutable HIR buffers for
its lifetime because generated code embeds pointers to their strings and source
spans. Public borrowed compilation APIs copy input into the backend; callers may
modify or drop their original HIR after compilation. The REPL transfers each
snippet into the same owner without retaining a second copy. Buffers remain
owned even when compilation fails partway through. Dropping the backend frees
code before these buffers.

## Rust roots and owner lifetimes

Use `RuntimeContext::root(tagged_value)` when Rust code retains a tagged value
across collections. The returned `RootedValue` pins that value until dropped.
`get()` returns the tagged value while its owner is alive and reports `E035`
after the owner is gone. Pins use weak ownership so they cannot extend the
lifetime of runtime function pointers beyond their JIT backend.

Rust `Value` collections and closures can also keep data reachable through
owned references. Keep their context alive and drop these values before their
owner, especially for cyclic graphs. Raw graphs constructed and retained by
Rust code remain subject to Rust ownership rules. Generated tagged values
cannot be retained safely through an unregistered raw pointer or an unpinned
copy alone.

Dropping a backend runs teardown collection, releasing unreachable cycles and
closure environments, then frees the JIT module's code and data. Cranelift
normally retains code allocations to preserve external function pointers. This
backend keeps those pointers private and executes synchronously, so it can
free its module when its runtime owner is dropped.

## Verification

`tests/runtime_context_tests.rs` covers:

- Nested activation and restoration after a Rust panic outside the C ABI.
- Independent pending errors and constant bindings.
- Release of acyclic collections when their context is dropped.
- Interleaved REPL globals, collections, classes, and captured functions.
- Continued execution after another session is dropped.
- Repeated program execution inside a host context without contaminating it.
- Foreign, expired, and malformed handles, including stale operands and calls.
- Recovery after uncaught failures, preserved error locations, and bindings
  initialized before a failed REPL snippet.
- Nested execution boundaries and restoration after a Rust panic.

Arena unit tests repeatedly reuse one slot and reject all its old identities,
and verify that independently created arenas cannot alias one another.

A runtime unit test verifies that closure environments survive their owning
scope, are inaccessible through a different context, and are released when
their owner is dropped. Existing LLVM integration tests and diagnostic
fixtures verify that standalone execution and Somali output retain their
behavior.

`tests/gc_tests.rs` verifies pinned handles, slot reuse after collection,
collection identity, Rust-held values, deferred borrows, closure captures,
cyclic teardown, and bounded REPL heaps across successful and failed snippets.
A shared JIT/LLVM stress program exercises cyclic lists, objects, closures,
repeated errors, and native callbacks that mutate their source collection.
The same program runs against optimized Rust runtimes in both backends.

`tests/jit_input_ownership.rs` mutates compiler input after compilation, drops
original modules, grows incremental storage, and invokes older functions.
Literal values, format descriptors, function names, and diagnostic source text
must remain unchanged.

## External resource ownership contract

File and network APIs belong to 3.3. Their implementations must use the same
context ownership rules established here:

- A context owns each native resource. Language values carry opaque handles,
  never file descriptors, raw pointers, or transferable Rust references.
- An explicit close operation releases the resource deterministically and is
  idempotent. Operations on a closed resource return a catchable error. Invalid
  or foreign handles remain distinct from valid handles whose resource is closed.
- Use `dhammee` when a close must happen before control leaves a block. Garbage
  collection provides fallback reclamation and cannot promise when it runs.
- A reachable stream, callback, or child view keeps its underlying resource
  reachable. Rust integrations pin retained tagged handles across safepoints.
- Unreachable resources close during collection; all remaining resources close
  when their context is dropped. Destruction releases native resources without
  executing Soplang callbacks or unwinding through generated code.
- Explicit close and flush operations report I/O failures. Fallback destruction
  cannot raise an error into already-completed language execution. Programs that
  need confirmation must flush and close explicitly inside protected code.
- Resource handles remain confined to their owner. Later task/channel APIs must
  define transfer or sharing explicitly instead of copying a tagged handle to a
  different context or thread.

Native resource variants will need collector graph edges and lifecycle tests
when introduced. This contract does not add file or network operations to 3.1.
