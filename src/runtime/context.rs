//! Owned runtime state selected at the native ABI boundary.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::abi::{ClassRegistryEntry, ClosureData, CompiledFnEntry, SoplangValue};
use super::handles::HandleArena;
use super::value::{NativeFn, Value};

#[derive(Default)]
pub(super) struct RuntimeState {
    pub current_source: RefCell<Option<crate::error::SourceSpan>>,
    pub try_depth: Cell<u32>,
    pub pending_error: RefCell<Option<crate::error::SoplangError>>,
    pub call_stack: RefCell<Vec<crate::error::StackFrame>>,
    pub errors: RefCell<HandleArena<Rc<crate::error::SoplangError>>>,
    pub suspended_errors: RefCell<Vec<Option<crate::error::SoplangError>>>,
    pub collector: super::gc::CollectorState,
    pub contracts: RefCell<super::contracts::ContractStore>,
    pub roots: RefCell<Vec<super::roots::RootFrame>>,
    pub pins: RefCell<HandleArena<SoplangValue>>,
    pub list_ids: RefCell<HashMap<usize, i64>>,
    pub byte_ids: RefCell<HashMap<usize, i64>>,
    pub tuple_ids: RefCell<HashMap<usize, i64>>,
    pub set_ids: RefCell<HashMap<usize, i64>>,
    pub object_ids: RefCell<HashMap<usize, i64>>,
    pub strings: RefCell<HandleArena<String>>,
    pub lists: RefCell<HandleArena<Rc<RefCell<Vec<Value>>>>>,
    pub bytes: RefCell<HandleArena<Rc<RefCell<Vec<u8>>>>>,
    pub tuples: RefCell<HandleArena<Rc<super::TupleData>>>,
    pub sets: RefCell<HandleArena<Rc<super::SetData>>>,
    pub objects: RefCell<HandleArena<Rc<RefCell<HashMap<String, Value>>>>>,
    pub native_functions: RefCell<HandleArena<NativeFn>>,
    pub builtin_indices: RefCell<Option<HashMap<String, i64>>>,
    pub globals: RefCell<HashMap<String, SoplangValue>>,
    pub compiled_functions: RefCell<HandleArena<CompiledFnEntry>>,
    pub constants: RefCell<HashSet<String>>,
    pub classes: RefCell<HashMap<String, ClassRegistryEntry>>,
    pub closures: RefCell<HandleArena<Rc<ClosureData>>>,
    pub iterators: RefCell<HandleArena<Rc<super::iterators::IteratorData>>>,
}

impl Drop for RuntimeState {
    fn drop(&mut self) {
        super::gc::collect(self, true);
    }
}

thread_local! {
    // Direct ABI callers and standalone LLVM programs use this default context.
    // JIT backends activate their own state while compiling and executing.
    static ACTIVE: RefCell<Rc<RuntimeState>> = RefCell::new(Rc::default());
}

/// State for one execution environment. Values and callable handles belong to
/// the context that created them. Contexts remain on their owning thread.
#[derive(Default)]
pub struct RuntimeContext {
    pub(super) state: Rc<RuntimeState>,
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run an ABI operation in this context. Nested scopes and Rust panics
    /// restore the caller's context. A panic must not cross an extern boundary.
    pub fn with<R>(&self, operation: impl FnOnce() -> R) -> R {
        let _scope = self.activate();
        operation()
    }

    /// Execute generated code or ABI calls and return an uncaught Soplang error.
    /// Nested execution preserves the surrounding error state. Rust panics are
    /// not language errors and must not cross an extern boundary.
    pub fn execute<R>(
        &self,
        operation: impl FnOnce() -> R,
    ) -> Result<R, crate::error::SoplangError> {
        self.with(|| {
            let _boundary = ExecutionBoundary {
                state: Rc::clone(&self.state),
                error: self.state.pending_error.borrow_mut().take(),
                try_depth: self.state.try_depth.replace(0),
                suspended: self.state.suspended_errors.replace(Vec::new()),
                call_stack: self.state.call_stack.replace(Vec::new()),
                source: self.state.current_source.replace(None),
                root_depth: self.state.roots.borrow().len(),
            };
            let result = operation();
            let error = self.state.pending_error.borrow_mut().take();
            match error {
                Some(error) => Err(error),
                None => Ok(result),
            }
        })
    }

    /// Reclaim unreachable tagged allocations and collection cycles.
    pub fn collect(&self) -> super::gc::CollectionReport {
        super::gc::collect(&self.state, false)
    }

    pub fn memory_stats(&self) -> super::gc::MemoryStats {
        super::gc::stats(&self.state)
    }

    /// Pin a tagged value while Rust code retains it across collections.
    pub fn root(
        &self,
        value: SoplangValue,
    ) -> Result<super::roots::RootedValue, crate::error::SoplangError> {
        self.with(|| {
            super::abi::soplang_to_value(value)?;
            Ok(super::roots::RootedValue::new(&self.state, value))
        })
    }

    /// Inspect a global without changing the caller's active context. Callable
    /// identifiers in the result still belong to this execution environment.
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.with(|| {
            let value = self.state.globals.borrow().get(name).copied()?;
            super::abi::soplang_to_value(value).ok()
        })
    }

    pub(crate) fn activate(&self) -> RuntimeScope {
        let previous = ACTIVE.with(|active| active.replace(Rc::clone(&self.state)));
        RuntimeScope { previous }
    }
}

struct ExecutionBoundary {
    state: Rc<RuntimeState>,
    error: Option<crate::error::SoplangError>,
    try_depth: u32,
    suspended: Vec<Option<crate::error::SoplangError>>,
    call_stack: Vec<crate::error::StackFrame>,
    source: Option<crate::error::SourceSpan>,
    root_depth: usize,
}

impl Drop for ExecutionBoundary {
    fn drop(&mut self) {
        *self.state.pending_error.borrow_mut() = self.error.take();
        self.state.try_depth.set(self.try_depth);
        *self.state.suspended_errors.borrow_mut() = std::mem::take(&mut self.suspended);
        *self.state.call_stack.borrow_mut() = std::mem::take(&mut self.call_stack);
        *self.state.current_source.borrow_mut() = self.source.take();
        self.state.roots.borrow_mut().truncate(self.root_depth);
    }
}

pub(crate) struct RuntimeScope {
    previous: Rc<RuntimeState>,
}

impl Drop for RuntimeScope {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.replace(Rc::clone(&self.previous));
        });
    }
}

/// Select a store without retaining a borrow of ACTIVE across runtime callbacks.
pub(super) struct RuntimeLocal<T> {
    select: fn(&RuntimeState) -> &T,
}

impl<T> RuntimeLocal<T> {
    pub const fn new(select: fn(&RuntimeState) -> &T) -> Self {
        Self { select }
    }

    pub fn with<R>(&self, operation: impl FnOnce(&T) -> R) -> R {
        let state = ACTIVE.with(|active| Rc::clone(&active.borrow()));
        operation((self.select)(&state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::abi::*;

    #[test]
    fn closures_are_owned_and_only_read_in_their_context() {
        extern "C" fn unused_entry() {}

        let context = RuntimeContext::new();
        let (closure, weak) = context.with(|| {
            // Only registration and capture lookup are exercised; no entry call.
            soplang_register_user_fn("f".as_ptr(), 1, unused_entry as *const u8, 1);
            let captured = soplang_str("owned".as_ptr(), 5);
            let closure = soplang_make_closure("f".as_ptr(), 1, 1, &captured);
            let value = soplang_capture_get(closure.payload, 0);
            assert_eq!(soplang_to_value(value).unwrap(), Value::Str("owned".into()));
            let heap = context.state.closures.borrow();
            (closure, Rc::downgrade(heap.get(closure.payload).unwrap()))
        });
        assert!(soplang_to_value(closure).is_err());
        assert!(weak.upgrade().is_some());
        drop(context);
        assert!(weak.upgrade().is_none());
        assert!(soplang_to_value(closure).is_err());
    }
}
