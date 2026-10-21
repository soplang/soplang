//! Owned runtime state selected at the native ABI boundary.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::abi::{ClassRegistryEntry, ClosureData, CompiledFnEntry, InstanceClass, SoplangValue};
use super::handles::HandleArena;
use super::value::{NativeFn, Value};

#[derive(Default)]
pub(super) struct RuntimeState {
    pub current_source: RefCell<Option<crate::error::SourceSpan>>,
    pub try_depth: Cell<u32>,
    pub random: Cell<Option<u64>>,
    pub reporting: RefCell<super::report::Reporting>,
    pub databases: RefCell<HashMap<i64, crate::data::Database>>,
    pub files: RefCell<Option<HashMap<i64, super::file_handles::OpenFile>>>,
    pub tcp_connections: RefCell<Option<HashMap<i64, std::net::TcpStream>>>,
    pub tcp_listeners: RefCell<Option<HashMap<i64, std::net::TcpListener>>>,
    pub udp_sockets: RefCell<Option<HashMap<i64, std::net::UdpSocket>>>,
    pub http_listeners: RefCell<Option<HashMap<i64, super::http_server::Listener>>>,
    pub websockets: RefCell<Option<HashMap<i64, super::websocket::Socket>>>,
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
    pub instances: RefCell<HashMap<usize, InstanceClass>>,
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
                profile_frames: std::mem::take(&mut self.state.reporting.borrow_mut().frames),
            };
            let result = operation();
            let error = self.state.pending_error.borrow_mut().take();
            match error {
                Some(error) => Err(error),
                None => Ok(result),
            }
        })
    }

    /// Reset instrumentation for the next run in this execution environment.
    pub fn configure_reporting(&self, coverage: super::CoverageMap, profile: bool) {
        *self.state.reporting.borrow_mut() = super::report::Reporting {
            report: super::ExecutionReport {
                coverage: super::CoverageReport {
                    executable: coverage,
                    hit: Default::default(),
                },
                ..Default::default()
            },
            profiling: profile,
            frames: Vec::new(),
        };
    }

    pub fn execution_report(&self) -> super::ExecutionReport {
        self.state.reporting.borrow().report.clone()
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
    profile_frames: Vec<super::report::ProfileFrame>,
}

impl Drop for ExecutionBoundary {
    fn drop(&mut self) {
        *self.state.pending_error.borrow_mut() = self.error.take();
        self.state.try_depth.set(self.try_depth);
        *self.state.suspended_errors.borrow_mut() = std::mem::take(&mut self.suspended);
        *self.state.call_stack.borrow_mut() = std::mem::take(&mut self.call_stack);
        *self.state.current_source.borrow_mut() = self.source.take();
        self.state.roots.borrow_mut().truncate(self.root_depth);
        self.state.reporting.borrow_mut().frames = std::mem::take(&mut self.profile_frames);
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

/// Select database ownership without exposing the rest of the runtime state.
pub(crate) fn with_databases<R>(
    operation: impl FnOnce(&RefCell<HashMap<i64, crate::data::Database>>) -> R,
) -> R {
    RuntimeLocal::new(|state| &state.databases).with(operation)
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

#[cfg(test)]
mod instance_tests {
    use super::*;

    #[test]
    fn class_identity_is_context_local_and_weak() {
        let owner = RuntimeContext::new();
        let other = RuntimeContext::new();
        let value = owner.with(|| {
            let value = super::super::abi::soplang_new_instance(
                b"Example".as_ptr(),
                7,
                std::ptr::null(),
                0,
            );
            super::super::abi::soplang_to_value(value).unwrap()
        });
        let Value::Object(object) = &value else {
            panic!("instance")
        };
        owner.collect();
        owner.with(|| {
            assert_eq!(
                super::super::abi::get_instance_class(object).as_deref(),
                Some("Example")
            )
        });
        other.with(|| assert_eq!(super::super::abi::get_instance_class(object), None));
        assert_eq!(owner.state.instances.borrow().len(), 1);
        drop(value);
        owner.collect();
        assert!(owner.state.instances.borrow().is_empty());
        assert_eq!(owner.memory_stats().values(), 0);
    }
}

#[cfg(test)]
mod resource_tests {
    use super::super::{file_handles, http_server, net, websocket};
    use super::*;
    use std::time::Duration;

    fn port(value: &Value) -> u16 {
        let Value::Object(fields) = value else {
            panic!("endpoint")
        };
        let Value::Int(port) = fields.borrow()["deked"] else {
            panic!("port")
        };
        port as u16
    }

    #[test]
    fn files_are_owned_by_the_opening_context() {
        let owner = RuntimeContext::new();
        let foreign = RuntimeContext::new();
        let path = std::env::temp_dir().join(format!(
            "soplang_owned_file_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let handle = owner.with(|| {
            let handle = file_handles::open(path.to_str().unwrap(), "qor").unwrap();
            file_handles::write(&handle, b"abc").unwrap();
            file_handles::seek(&handle, 0, "bilow").unwrap();
            handle
        });
        foreign.with(|| {
            assert_eq!(
                file_handles::read_bytes(&handle, 3).unwrap_err().code(),
                Some("E058")
            );
            assert!(file_handles::close(&handle).is_err());
        });
        owner.with(|| assert_eq!(file_handles::position(&handle).unwrap(), Value::Int(0)));
        let state = Rc::downgrade(&owner.state);
        drop(owner);
        assert!(state.upgrade().is_none());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn dropping_a_context_releases_listener_ports() {
        let owner = RuntimeContext::new();
        let foreign = RuntimeContext::new();
        let (tcp, http, udp) = owner.with(|| {
            (
                net::listen("127.0.0.1", 0).unwrap(),
                http_server::open("127.0.0.1", 0).unwrap(),
                net::bind_udp("127.0.0.1", 0, Duration::from_millis(10)).unwrap(),
            )
        });
        foreign.with(|| {
            assert_eq!(
                net::accept(&tcp, Duration::from_millis(1))
                    .unwrap_err()
                    .code(),
                Some("E055")
            );
            assert_eq!(net::receive_udp(&udp, 1).unwrap_err().code(), Some("E055"));
            net::close(&tcp).unwrap();
            http_server::close(&http).unwrap();
            net::close(&udp).unwrap();
        });
        for p in [port(&tcp), port(&http)] {
            assert!(std::net::TcpListener::bind(("127.0.0.1", p)).is_err());
        }
        assert!(std::net::UdpSocket::bind(("127.0.0.1", port(&udp))).is_err());
        drop(owner);
        for p in [port(&tcp), port(&http)] {
            std::net::TcpListener::bind(("127.0.0.1", p)).unwrap();
        }
        std::net::UdpSocket::bind(("127.0.0.1", port(&udp))).unwrap();
    }

    #[test]
    fn tcp_and_websocket_connections_close_on_context_drop() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let owner = RuntimeContext::new();
        let foreign = RuntimeContext::new();
        let socket = owner.with(|| {
            net::connect(
                "127.0.0.1",
                i64::from(listener.local_addr().unwrap().port()),
                Duration::from_secs(2),
            )
            .unwrap()
        });
        let (mut peer, _) = listener.accept().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        foreign.with(|| assert!(net::write(&socket, b"foreign").is_err()));
        drop(owner);
        assert_eq!(std::io::Read::read(&mut peer, &mut [0u8; 1]).unwrap(), 0);

        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (peer, _) = listener.accept().unwrap();
            peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let mut socket = tungstenite::accept(peer).unwrap();
            assert!(socket.read().is_err()); // Context destruction closes the transport.
        });
        let owner = RuntimeContext::new();
        let socket = owner.with(|| {
            websocket::connect(&format!("ws://127.0.0.1:{port}"), Duration::from_secs(2)).unwrap()
        });
        foreign.with(|| {
            assert_eq!(
                websocket::send(&socket, b"foreign", true)
                    .unwrap_err()
                    .code(),
                Some("E056")
            )
        });
        drop(owner);
        server.join().unwrap();
    }
}
