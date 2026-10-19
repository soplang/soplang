# Runtime ownership and execution contexts

Implemented in the first runtime-foundations stage after 3.0.1. This stage
isolates execution environments and gives their allocations an owner.

## State ownership

Each Cranelift backend owns a `RuntimeContext`. Its state contains:

- Strings, list handles, and object handles.
- Globals, constant bindings, and class metadata.
- Built-in function indices and compiled function registrations.
- Closure environments and their captured tagged values.
- Current source location, try depth, and pending errors.

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
names match. A new execution resets that context's leftover try state.

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

## Allocation lifetime

Closure environments are held in the context instead of leaked with
`Box::leak`. Their identities are looked up in the active context before a
capture is read or a closure is called. Capture lookup does not dereference
an arbitrary or already-freed pointer. This retains the existing environment
parameter representation used by both backends.

Dropping a backend first releases its runtime, including closure environments
and acyclic collections whose references have not escaped. It then explicitly
frees the JIT module's code and data allocations. Cranelift normally retains
those allocations to preserve external function pointers. This backend keeps
its code pointers private and all execution is synchronous, so its module can
release them once the backend is dropped.

This is ownership at the execution-environment boundary, not a garbage
collector. Unreachable allocations inside a live context are not reclaimed
yet, and cyclic `Rc` collection graphs still need cycle reclamation. Slot reuse,
generation checks, and validation of foreign or stale tagged handles belong to
the following handle-management stage. Resource cleanup and recoverable
uncaught execution errors also remain separate stages.

## Verification

`tests/runtime_context_tests.rs` covers:

- Nested activation and restoration after a Rust panic outside the C ABI.
- Independent pending errors and constant bindings.
- Release of acyclic collections when their context is dropped.
- Interleaved REPL globals, collections, classes, and captured functions.
- Continued execution after another session is dropped.
- Repeated program execution inside a host context without contaminating it.

A runtime unit test verifies that closure environments survive their owning
scope, are inaccessible through a different context, and are released when
their owner is dropped. Existing LLVM integration tests and diagnostic
fixtures verify that standalone execution and Somali output retain their
behavior.
