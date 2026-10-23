# Soplang 3.6 execution model

This document records the execution contract for tasks and asynchronous I/O.
The public model uses task handles and `sug(task)`. It does not add `async` or
`await` keywords.

## Execution and ownership

Compiled Soplang callbacks run on the thread that owns their `RuntimeContext`.
Lightweight tasks use separate stacks scheduled cooperatively on that thread.
The scheduler may move owned byte buffers and I/O results to worker threads;
tagged Soplang values, closures, class registries, and resource handles never
cross that boundary. This keeps existing `Rc` and `RefCell` values race free.
Tasks in one context can share a mutable collection, but only one task executes
Soplang instructions at a time. A callback must release every runtime borrow
before it suspends.

`shaqo(callback, arg1, arg2)` starts a child task and returns a handle. `sug(task)`
waits for its result or raises its error. `baaji(task)` requests cancellation.
`marin(xad)` creates a bounded channel; `dir` and `qaad` wait when full and empty,
respectively, and `xidh` closes it. A zero-capacity channel is a rendezvous.
`qaad` returns `{qiime, dhammaad}`. Closing a channel preserves buffered values;
after they are drained, `qaad` returns `{qiime: maran, dhammaad: run}`. Sends after
closure fail. A blocked sender or receiver resumes when the channel closes.
Timer and I/O operations return task handles that can be passed to `sug`.

## Task lifetime and cancellation

Every child belongs to the task that started it, or to the top-level execution
scope. When a scope exits, it requests cancellation of unfinished children and
drains them before returning. Cancellation is delivered at suspension points
and generated-code safepoints as a Soplang error. A `dhammee` block therefore
runs during normal cancellation. A deadline requests the same cancellation and
is reported distinctly from an explicit `baaji` request. Blocking native calls
must be moved to the I/O executor or must have an explicit deadline.

The scheduler must never drop a suspended compiled stack to cancel it. A forced
Rust unwind cannot cross generated `extern "C"` frames safely. It resumes a
cancelled task until the compiled function returns through its ordinary error
path, then releases its stack and handle. Tasks that keep executing CPU code
must observe cancellation at bounded safepoints.

## Runtime state at a switch

Before switching tasks, save the active source span, pending error, catch
depth, suspended errors, call stack, root frames, debugger frames, and active
profiling frames. Restore the destination task's state before resuming it.
The collector must trace generated root frames on every suspended stack and
the paused parent stack, including task results and queued channel values.
The stack backing each root pointer remains alive until its task completes.
No `RefCell` borrow or runtime-local selector may be held across a switch.

The top-level JIT execution and the generated LLVM executable use the same
runtime scheduler and drain their task scopes before tearing down native code
or collector state. A context drop cannot force-unwind a suspended task.

## I/O and server behavior

The I/O executor owns sockets, file operations, timers, and bounded transport
buffers. It sends completed byte buffers or errors back to the owner thread.
The owner converts them to Soplang values and resumes waiting tasks. Channels
and streams have explicit capacity; producers stop reading or writing when
their consumer cannot keep up. Disconnect and deadline events cancel the
associated operation and release its resources.

Concurrent HTTP handlers remain isolated tasks in one owner context. Shared
application state is accessed only on that thread. Request bodies, response
bodies, and WebSocket messages use bounded streams. Shutdown stops accepting,
drains active handlers and connections up to a deadline, then cancels the rest.
Tests must cover slow clients, abrupt disconnects, cancellation during cleanup,
many idle connections, and sustained memory use in both JIT and LLVM builds.
