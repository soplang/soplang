//! Owner-thread task scheduler for compiled Soplang callbacks.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use corosensei::{Coroutine, CoroutineResult, Yielder};

use super::context::{RuntimeLocal, RuntimeState};
use super::roots::RootFrame;
use super::value::Value;
use crate::error::{runtime_error, SoplangError};

type Fiber = Coroutine<Signal, Event, Result<Value, SoplangError>>;
type FiberYielder = Yielder<Signal, Event>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Signal {
    Continue,
    Cancel,
}

enum Event {
    Await(i64),
}

thread_local! {
    static YIELDER: Cell<*const FiberYielder> = const { Cell::new(std::ptr::null()) };
}

fn cancelled() -> SoplangError {
    runtime_error("Shaqada waa la baajiyey.", 0, 0)
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
    waiting_for: Option<i64>,
    cancel: bool,
    parent: i64,
}

#[derive(Default)]
pub(super) struct Scheduler {
    tasks: HashMap<i64, Task>,
    ready: VecDeque<i64>,
    running: i64,
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        // Normal execution drains tasks before their context is destroyed.
        // A host may discard an inactive context; leaking a suspended stack is
        // safer than corosensei force-unwinding through generated C ABI frames.
        for (_, task) in self.tasks.drain() {
            std::mem::forget(task.fiber);
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

pub(super) fn spawn(callee: Value, args: Vec<Value>) -> Result<Value, SoplangError> {
    if !matches!(
        callee,
        Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_)
    ) {
        return Err(runtime_error("shaqo() waxay u baahan tahay hawl.", 0, 0));
    }
    STATE.with(|state| {
        let data = Rc::new(TaskData {
            id: Cell::new(0),
            result: RefCell::new(None),
        });
        let id = state.tasks.borrow_mut().insert(Rc::clone(&data));
        data.id.set(id);
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
    let (id, mut task) = {
        let mut scheduler = state.scheduler.borrow_mut();
        let Some(id) = scheduler.ready.pop_front() else {
            return Ok(false);
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
            task.waiting_for = Some(target);
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
        CoroutineResult::Return(result) => {
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
            let mut ready = Vec::new();
            for (other, waiting) in scheduler.tasks.iter_mut() {
                if waiting.waiting_for == Some(id) {
                    waiting.waiting_for = None;
                    ready.push(*other);
                }
            }
            scheduler.ready.extend(ready);
        }
    }
    Ok(true)
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

pub(super) fn cancel_task(data: &Rc<TaskData>) -> Result<(), SoplangError> {
    STATE.with(|state| {
        validate(state, data)?;
        let mut scheduler = state.scheduler.borrow_mut();
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
