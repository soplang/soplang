//! Owner-thread task scheduler for compiled Soplang callbacks.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use corosensei::{Coroutine, CoroutineResult, Yielder};

use super::context::{RuntimeLocal, RuntimeState};
use super::roots::RootFrame;
use super::value::Value;
use crate::error::{runtime_error, SoplangError};

type Fiber = Coroutine<Signal, Event, Result<Value, SoplangError>>;
type FiberYielder = Yielder<Signal, Event>;
const MAX_ACTIVE_TASKS: usize = 4096;
const MAX_FILE_TASKS: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Signal {
    Continue,
    Cancel,
}

enum Event {
    Await(i64),
    AwaitEither(i64, i64),
    Channel(i64),
    Yield,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wait {
    Task(i64),
    Either(i64, i64),
    Channel(i64),
}

thread_local! {
    static YIELDER: Cell<*const FiberYielder> = const { Cell::new(std::ptr::null()) };
}

fn cancelled() -> SoplangError {
    runtime_error("Shaqada waa la baajiyey.", 0, 0)
}

fn timed_out() -> SoplangError {
    runtime_error("Waqtigii shaqada loo qabtay wuu dhammaaday.", 0, 0)
}

fn suspend(event: Event) -> Result<(), SoplangError> {
    let yielder = YIELDER.with(|current| current.replace(std::ptr::null()));
    if yielder.is_null() {
        return Err(runtime_error("Shaqo socota ma jirto.", 0, 0));
    }
    // The pointer is installed only while this coroutine is running. Its
    // yielder lives on that coroutine's stack until it returns normally.
    let signal = unsafe { (*yielder).suspend(event) };
    YIELDER.with(|current| current.set(yielder));
    if signal == Signal::Cancel {
        Err(cancelled())
    } else {
        Ok(())
    }
}

/// A handle belongs to the runtime context that created it.
pub struct TaskData {
    id: Cell<i64>,
    result: RefCell<Option<Result<Value, SoplangError>>>,
    finished_at: Cell<Option<Instant>>,
}

impl TaskData {
    pub fn handle(&self) -> i64 {
        self.id.get()
    }

    pub(super) fn children(&self) -> Option<Vec<Value>> {
        let result = self.result.try_borrow().ok()?;
        Some(match result.as_ref() {
            Some(Ok(value)) => vec![value.clone()],
            _ => Vec::new(),
        })
    }

    pub(super) fn clear(&self) {
        if let Ok(mut result) = self.result.try_borrow_mut() {
            *result = None;
        }
    }
}

#[derive(Default)]
struct TaskLocal {
    source: Option<crate::error::SourceSpan>,
    try_depth: u32,
    pending_error: Option<SoplangError>,
    call_stack: Vec<crate::error::StackFrame>,
    suspended_errors: Vec<Option<SoplangError>>,
    roots: Vec<RootFrame>,
    profile_frames: Vec<super::report::ProfileFrame>,
    debugger: super::debugger::Debugger,
}

impl TaskLocal {
    fn swap(&mut self, state: &RuntimeState) {
        std::mem::swap(&mut self.source, &mut state.current_source.borrow_mut());
        self.try_depth = state.try_depth.replace(self.try_depth);
        std::mem::swap(
            &mut self.pending_error,
            &mut state.pending_error.borrow_mut(),
        );
        std::mem::swap(&mut self.call_stack, &mut state.call_stack.borrow_mut());
        std::mem::swap(
            &mut self.suspended_errors,
            &mut state.suspended_errors.borrow_mut(),
        );
        std::mem::swap(&mut self.roots, &mut state.roots.borrow_mut());
        std::mem::swap(
            &mut self.profile_frames,
            &mut state.reporting.borrow_mut().frames,
        );
        std::mem::swap(&mut self.debugger, &mut state.debugger.borrow_mut());
    }
}

struct Task {
    fiber: Fiber,
    handle: Rc<TaskData>,
    local: TaskLocal,
    waiting_for: Option<Wait>,
    cancel: bool,
    parent: i64,
}

pub(super) struct Scheduler {
    tasks: HashMap<i64, Task>,
    ready: VecDeque<i64>,
    running: i64,
    pending_io: HashMap<i64, PendingIo>,
    completion_tx: Sender<Completion>,
    completion_rx: Receiver<Completion>,
}

struct PendingIo {
    abort: tokio::task::AbortHandle,
    parent: i64,
    handle: Rc<TaskData>,
    file: bool,
}

struct Completion {
    id: i64,
    result: Result<OwnedResult, OwnedFailure>,
    finished_at: Instant,
}

pub(super) enum OwnedResult {
    Unit,
    Text(String),
    Bytes(Vec<u8>),
}

pub(super) enum OwnedFailure {
    Read(PathBuf, io::Error),
    Write(PathBuf, io::Error),
    FileTooLarge(PathBuf),
}

impl Default for Scheduler {
    fn default() -> Self {
        let (completion_tx, completion_rx) = mpsc::channel();
        Self {
            tasks: HashMap::new(),
            ready: VecDeque::new(),
            running: 0,
            pending_io: HashMap::new(),
            completion_tx,
            completion_rx,
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        // Normal execution drains tasks before their context is destroyed.
        // A host may discard an inactive context; leaking a suspended stack is
        // safer than corosensei force-unwinding through generated C ABI frames.
        for (_, task) in self.tasks.drain() {
            std::mem::forget(task.fiber);
        }
        for (_, pending) in self.pending_io.drain() {
            pending.abort.abort();
        }
    }
}

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

fn validate(state: &RuntimeState, data: &Rc<TaskData>) -> Result<(), SoplangError> {
    if state
        .tasks
        .borrow()
        .get(data.handle())
        .is_some_and(|entry| Rc::ptr_eq(entry, data))
    {
        Ok(())
    } else {
        Err(super::abi::invalid_handle())
    }
}

fn executor() -> Result<&'static tokio::runtime::Runtime, SoplangError> {
    static EXECUTOR: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    EXECUTOR
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("soplang-io")
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|detail| {
            crate::error::external_error("Shaqooyinka dibadda lama bilaabi karin.", detail)
        })
}

fn new_handle(state: &RuntimeState) -> Rc<TaskData> {
    let data = Rc::new(TaskData {
        id: Cell::new(0),
        result: RefCell::new(None),
        finished_at: Cell::new(None),
    });
    let id = state.tasks.borrow_mut().insert(Rc::clone(&data));
    data.id.set(id);
    state.collector.allocated(std::mem::size_of::<TaskData>());
    data
}

fn check_limit(state: &RuntimeState) -> Result<(), SoplangError> {
    let scheduler = state.scheduler.borrow();
    let active =
        scheduler.tasks.len() + scheduler.pending_io.len() + usize::from(scheduler.running != 0);
    if active >= MAX_ACTIVE_TASKS {
        Err(runtime_error("Shaqooyin badan ayaa hal mar socda.", 0, 0))
    } else {
        Ok(())
    }
}

pub(super) fn spawn_external<F>(future: F, file: bool) -> Result<Value, SoplangError>
where
    F: Future<Output = Result<OwnedResult, OwnedFailure>> + Send + 'static,
{
    let executor = executor()?;
    STATE.with(|state| {
        while try_completion(state) {}
        check_limit(state)?;
        let mut scheduler = state.scheduler.borrow_mut();
        if file
            && scheduler
                .pending_io
                .values()
                .filter(|entry| entry.file)
                .count()
                >= MAX_FILE_TASKS
        {
            return Err(runtime_error(
                "Faylal badan ayaa hal mar la adeegsanayaa.",
                0,
                0,
            ));
        }
        let data = new_handle(state);
        let id = data.handle();
        let parent = scheduler.running;
        let sender = scheduler.completion_tx.clone();
        let handle = executor.spawn(async move {
            let result = future.await;
            let _ = sender.send(Completion {
                id,
                result,
                finished_at: Instant::now(),
            });
        });
        scheduler.pending_io.insert(
            id,
            PendingIo {
                abort: handle.abort_handle(),
                parent,
                handle: Rc::clone(&data),
                file,
            },
        );
        Ok(Value::Task(data))
    })
}

pub(super) fn timer(millis: u64) -> Result<Value, SoplangError> {
    spawn_external(
        async move {
            tokio::time::sleep(Duration::from_millis(millis)).await;
            Ok(OwnedResult::Unit)
        },
        false,
    )
}

fn wake_awaiters(scheduler: &mut Scheduler, target: i64) {
    let mut ready = Vec::new();
    for (id, task) in scheduler.tasks.iter_mut() {
        if matches!(task.waiting_for, Some(Wait::Task(id)) if id == target)
            || matches!(task.waiting_for, Some(Wait::Either(first, second)) if first == target || second == target)
        {
            task.waiting_for = None;
            ready.push(*id);
        }
    }
    scheduler.ready.extend(ready);
}

fn finish_io(state: &RuntimeState, completion: Completion) {
    let mut scheduler = state.scheduler.borrow_mut();
    let Some(pending) = scheduler.pending_io.remove(&completion.id) else {
        return;
    };
    let result = completion
        .result
        .map(|result| match result {
            OwnedResult::Unit => Value::Null,
            OwnedResult::Text(text) => Value::Str(text),
            OwnedResult::Bytes(bytes) => super::bytes::from_raw_vec(bytes),
        })
        .map_err(|error| match error {
            OwnedFailure::Read(path, error) => crate::error::file_read_error(&path, &error),
            OwnedFailure::Write(path, error) => crate::error::file_write_error(&path, &error),
            OwnedFailure::FileTooLarge(path) => runtime_error(
                format!("Faylka '{}' wuxuu ka weyn yahay 16 MiB.", path.display()),
                0,
                0,
            ),
        });
    pending.handle.finished_at.set(Some(completion.finished_at));
    *pending.handle.result.borrow_mut() = Some(result);
    wake_awaiters(&mut scheduler, completion.id);
}

fn try_completion(state: &RuntimeState) -> bool {
    let completion = state.scheduler.borrow().completion_rx.try_recv();
    match completion {
        Ok(completion) => {
            finish_io(state, completion);
            true
        }
        Err(TryRecvError::Empty | TryRecvError::Disconnected) => false,
    }
}

fn cancel_io(scheduler: &mut Scheduler, id: i64) -> bool {
    let Some(pending) = scheduler.pending_io.remove(&id) else {
        return false;
    };
    pending.abort.abort();
    pending.handle.finished_at.set(Some(Instant::now()));
    *pending.handle.result.borrow_mut() = Some(Err(cancelled()));
    wake_awaiters(scheduler, id);
    true
}

pub(super) fn spawn(callee: Value, args: Vec<Value>) -> Result<Value, SoplangError> {
    if !matches!(
        callee,
        Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_)
    ) {
        return Err(runtime_error("shaqo() waxay u baahan tahay hawl.", 0, 0));
    }
    STATE.with(|state| {
        check_limit(state)?;
        let data = new_handle(state);
        let id = data.handle();
        let fiber = Coroutine::new(move |yielder: &FiberYielder, signal| {
            if signal == Signal::Cancel {
                return Err(cancelled());
            }
            let previous = YIELDER.with(|current| current.replace(yielder));
            let result = super::abi::invoke(&callee, &args);
            YIELDER.with(|current| current.set(previous));
            result
        });
        let mut scheduler = state.scheduler.borrow_mut();
        let parent = scheduler.running;
        scheduler.tasks.insert(
            id,
            Task {
                fiber,
                handle: Rc::clone(&data),
                local: TaskLocal {
                    debugger: state.debugger.borrow().for_task(),
                    ..TaskLocal::default()
                },
                waiting_for: None,
                cancel: false,
                parent,
            },
        );
        scheduler.ready.push_back(id);
        Ok(Value::Task(data))
    })
}

fn step(state: &RuntimeState) -> Result<bool, SoplangError> {
    try_completion(state);
    let (id, mut task) = {
        let mut scheduler = state.scheduler.borrow_mut();
        let Some(id) = scheduler.ready.pop_front() else {
            if scheduler.pending_io.is_empty() {
                return Ok(false);
            }
            drop(scheduler);
            let completion = state
                .scheduler
                .borrow()
                .completion_rx
                .recv()
                .map_err(|_| runtime_error("Shaqooyinka dibadda way istaageen.", 0, 0))?;
            finish_io(state, completion);
            return Ok(true);
        };
        let task = scheduler.tasks.remove(&id).expect("ready task missing");
        scheduler.running = id;
        (id, task)
    };
    let mut caller = TaskLocal::default();
    caller.swap(state);
    state
        .parked_roots
        .borrow_mut()
        .insert(0, caller.roots.clone());
    state.parked_roots.borrow_mut().remove(&id);
    task.local.swap(state);
    let outcome = task.fiber.resume(if task.cancel {
        Signal::Cancel
    } else {
        Signal::Continue
    });
    let completed_at = Instant::now();
    task.local.swap(state);
    state
        .parked_roots
        .borrow_mut()
        .insert(id, task.local.roots.clone());
    caller.swap(state);
    state.parked_roots.borrow_mut().remove(&0);
    let mut scheduler = state.scheduler.borrow_mut();
    scheduler.running = 0;
    match outcome {
        CoroutineResult::Yield(Event::Await(target)) => {
            task.waiting_for = Some(Wait::Task(target));
            if state
                .tasks
                .borrow()
                .get(target)
                .is_some_and(|data| data.result.borrow().is_some())
            {
                task.waiting_for = None;
                scheduler.ready.push_back(id);
            }
            scheduler.tasks.insert(id, task);
        }
        CoroutineResult::Yield(Event::AwaitEither(first, second)) => {
            task.waiting_for = Some(Wait::Either(first, second));
            if [first, second].iter().any(|target| {
                state
                    .tasks
                    .borrow()
                    .get(*target)
                    .is_some_and(|data| data.result.borrow().is_some())
            }) {
                task.waiting_for = None;
                scheduler.ready.push_back(id);
            }
            scheduler.tasks.insert(id, task);
        }
        CoroutineResult::Yield(Event::Channel(channel)) => {
            task.waiting_for = Some(Wait::Channel(channel));
            scheduler.tasks.insert(id, task);
        }
        CoroutineResult::Yield(Event::Yield) => {
            scheduler.tasks.insert(id, task);
            scheduler.ready.push_back(id);
        }
        CoroutineResult::Return(result) => {
            task.handle.finished_at.set(Some(completed_at));
            *task.handle.result.borrow_mut() = Some(result);
            state.parked_roots.borrow_mut().remove(&id);
            let mut cancelled_children = Vec::new();
            for (child_id, child) in scheduler.tasks.iter_mut() {
                if child.parent == id {
                    child.cancel = true;
                    if child.waiting_for.take().is_some() {
                        cancelled_children.push(*child_id);
                    }
                }
            }
            scheduler.ready.extend(cancelled_children);
            let child_io: Vec<_> = scheduler
                .pending_io
                .iter()
                .filter_map(|(child_id, pending)| (pending.parent == id).then_some(*child_id))
                .collect();
            for child_id in child_io {
                cancel_io(&mut scheduler, child_id);
            }
            wake_awaiters(&mut scheduler, id);
        }
    }
    Ok(true)
}

pub(super) fn poll() -> Result<(), SoplangError> {
    STATE.with(|state| {
        try_completion(state);
        let scheduler = state.scheduler.borrow();
        let has_ready = !scheduler.ready.is_empty();
        let running = scheduler.running;
        drop(scheduler);
        if !has_ready {
            return Ok(());
        }
        if running == 0 {
            step(state)?;
            Ok(())
        } else {
            suspend(Event::Yield)
        }
    })
}

pub(super) fn running_task() -> i64 {
    STATE.with(|state| state.scheduler.borrow().running)
}

pub(super) fn wait_channel(channel: i64) -> Result<(), SoplangError> {
    STATE.with(|state| {
        if state.scheduler.borrow().running != 0 {
            suspend(Event::Channel(channel))
        } else if step(state)? {
            Ok(())
        } else {
            Err(runtime_error(
                "Shaqooyinka sugaya ma dhaqaaqi karaan.",
                0,
                0,
            ))
        }
    })
}

pub(super) fn wake_channel_task(id: i64, channel: i64) -> bool {
    if id == 0 {
        return true;
    }
    STATE.with(|state| {
        let mut scheduler = state.scheduler.borrow_mut();
        let Some(task) = scheduler.tasks.get_mut(&id) else {
            return false;
        };
        if task.waiting_for != Some(Wait::Channel(channel)) {
            return false;
        }
        task.waiting_for = None;
        scheduler.ready.push_back(id);
        true
    })
}

pub(super) fn await_task(data: &Rc<TaskData>) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        validate(state, data)?;
        loop {
            if let Some(result) = data.result.borrow().as_ref() {
                return result.clone();
            }
            let current = state.scheduler.borrow().running;
            if current != 0 {
                if current == data.handle() {
                    return Err(runtime_error("Shaqo isma sugi karto.", 0, 0));
                }
                suspend(Event::Await(data.handle()))?;
            } else if !step(state)? {
                return Err(runtime_error(
                    "Shaqooyinka sugaya ma dhaqaaqi karaan.",
                    0,
                    0,
                ));
            }
        }
    })
}

pub(super) fn await_with_timeout(data: &Rc<TaskData>, millis: u64) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        validate(state, data)?;
        if state.scheduler.borrow().running == data.handle() {
            return Err(runtime_error("Shaqo isma sugi karto.", 0, 0));
        }
        if let Some(result) = data.result.borrow().as_ref() {
            return result.clone();
        }
        if millis == 0 {
            cancel_task(data)?;
            let _ = await_task(data);
            return Err(timed_out());
        }
        let expires = Instant::now() + Duration::from_millis(millis);
        let Value::Task(deadline) = spawn_external(
            async move {
                tokio::time::sleep_until(tokio::time::Instant::from_std(expires)).await;
                Ok(OwnedResult::Unit)
            },
            false,
        )?
        else {
            unreachable!("timer returns a task")
        };
        let result = (|| loop {
            if let Some(result) = data.result.borrow().as_ref() {
                if data
                    .finished_at
                    .get()
                    .is_some_and(|finished| finished <= expires)
                {
                    return result.clone();
                }
                return Err(timed_out());
            }
            if Instant::now() >= expires {
                cancel_task(data)?;
                let _ = await_task(data);
                return Err(timed_out());
            }
            if state.scheduler.borrow().running != 0 {
                suspend(Event::AwaitEither(data.handle(), deadline.handle()))?;
            } else if !step(state)? {
                return Err(runtime_error(
                    "Shaqooyinka sugaya ma dhaqaaqi karaan.",
                    0,
                    0,
                ));
            }
        })();
        let _ = cancel_task(&deadline);
        result
    })
}

pub(super) fn cancel_task(data: &Rc<TaskData>) -> Result<(), SoplangError> {
    STATE.with(|state| {
        validate(state, data)?;
        let mut scheduler = state.scheduler.borrow_mut();
        if cancel_io(&mut scheduler, data.handle()) {
            return Ok(());
        }
        if let Some(task) = scheduler.tasks.get_mut(&data.handle()) {
            task.cancel = true;
            if task.waiting_for.take().is_some() {
                scheduler.ready.push_back(data.handle());
            }
        }
        Ok(())
    })
}

pub(super) fn drain() -> Result<(), SoplangError> {
    STATE.with(|state| {
        {
            let mut scheduler = state.scheduler.borrow_mut();
            let pending: Vec<_> = scheduler.pending_io.keys().copied().collect();
            for id in pending {
                cancel_io(&mut scheduler, id);
            }
            let waiting: Vec<_> = scheduler
                .tasks
                .iter_mut()
                .filter_map(|(id, task)| {
                    task.cancel = true;
                    task.waiting_for.take().map(|_| *id)
                })
                .collect();
            scheduler.ready.extend(waiting);
        }
        while step(state)? {}
        if state.scheduler.borrow().tasks.is_empty() {
            Ok(())
        } else {
            Err(runtime_error("Shaqooyinka la baajiyey ma dhammaan.", 0, 0))
        }
    })
}

pub(super) fn builtin_shaqo(mut args: Vec<Value>) -> Result<Value, SoplangError> {
    if args.is_empty() {
        return Err(runtime_error("shaqo() waxay u baahan tahay hawl.", 0, 0));
    }
    let callee = args.remove(0);
    spawn(callee, args)
}

pub(super) fn builtin_sug(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [Value::Task(data)] = args.as_slice() else {
        return Err(runtime_error("sug() waxay u baahan tahay shaqo.", 0, 0));
    };
    await_task(data)
}

pub(super) fn builtin_baaji(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [Value::Task(data)] = args.as_slice() else {
        return Err(runtime_error("baaji() waxay u baahan tahay shaqo.", 0, 0));
    };
    cancel_task(data)?;
    Ok(Value::Null)
}

pub(super) fn builtin_hurdada(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [Value::Int(millis)] = args.as_slice() else {
        return Err(runtime_error(
            "hurdada() waxay u baahan tahay millisekanno tiro dhan ah.",
            0,
            0,
        ));
    };
    if !(0..=86_400_000).contains(millis) {
        return Err(runtime_error(
            "Hurdadu waa inay u dhexeysaa 0 iyo 86400000 millisekan.",
            0,
            0,
        ));
    }
    timer(*millis as u64)
}

pub(super) fn builtin_sug_waqti(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [Value::Task(data), Value::Int(millis)] = args.as_slice() else {
        return Err(runtime_error(
            "sug_waqti() waxay u baahan tahay shaqo iyo millisekanno tiro dhan ah.",
            0,
            0,
        ));
    };
    if !(0..=86_400_000).contains(millis) {
        return Err(runtime_error(
            "Waqtigu waa inuu u dhexeeyaa 0 iyo 86400000 millisekan.",
            0,
            0,
        ));
    }
    await_with_timeout(data, *millis as u64)
}
