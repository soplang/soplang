//! Trace generated roots and host references, then break unreachable Rc cycles.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::abi::{
    ClosureData, SoplangValue, TAG_BYTES, TAG_CLOSURE, TAG_ERROR, TAG_ITERATOR, TAG_LIST,
    TAG_OBJECT, TAG_STR, TAG_TUPLE, TAG_SET,
};
use super::context::RuntimeState;
use super::value::Value;

const MIN_ALLOCATIONS: usize = 1024;
const MIN_BYTES: usize = 4 * 1024 * 1024;

pub(super) struct CollectorState {
    allocations: Cell<usize>,
    bytes: Cell<usize>,
    next_allocations: Cell<usize>,
    next_bytes: Cell<usize>,
    collections: Cell<usize>,
    reclaimed: Cell<usize>,
}

impl Default for CollectorState {
    fn default() -> Self {
        Self {
            allocations: Cell::new(0),
            bytes: Cell::new(0),
            next_allocations: Cell::new(MIN_ALLOCATIONS),
            next_bytes: Cell::new(MIN_BYTES),
            collections: Cell::new(0),
            reclaimed: Cell::new(0),
        }
    }
}

impl CollectorState {
    pub fn allocated(&self, bytes: usize) {
        self.allocations
            .set(self.allocations.get().saturating_add(1));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
    }

    pub fn due(&self) -> bool {
        self.allocations.get() >= self.next_allocations.get()
            || self.bytes.get() >= self.next_bytes.get()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryStats {
    pub strings: usize,
    pub lists: usize,
    pub bytes: usize,
    pub tuples: usize,
    pub sets: usize,
    pub objects: usize,
    pub closures: usize,
    pub errors: usize,
    pub iterators: usize,
    /// Number of shared collection identities with element contracts.
    pub type_contracts: usize,
    pub active_frames: usize,
    pub pinned_values: usize,
    pub collections: usize,
    pub reclaimed: usize,
}

impl MemoryStats {
    pub fn values(&self) -> usize {
        self.strings
            + self.lists
            + self.bytes
            + self.tuples
            + self.sets
            + self.objects
            + self.closures
            + self.errors
            + self.iterators
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollectionReport {
    pub before: MemoryStats,
    pub after: MemoryStats,
    pub reclaimed: usize,
    /// Borrowed collections are conservatively retained until another pass.
    pub deferred: usize,
}

pub(super) fn stats(state: &RuntimeState) -> MemoryStats {
    MemoryStats {
        strings: state.strings.borrow().len(),
        lists: state.lists.borrow().len(),
        bytes: state.bytes.borrow().len(),
        tuples: state.tuples.borrow().len(),
        sets: state.sets.borrow().len(),
        objects: state.objects.borrow().len(),
        closures: state.closures.borrow().len(),
        errors: state.errors.borrow().len(),
        iterators: state.iterators.borrow().len(),
        type_contracts: state.contracts.borrow().len(),
        active_frames: state.roots.borrow().len(),
        pinned_values: state.pins.borrow().len(),
        collections: state.collector.collections.get(),
        reclaimed: state.collector.reclaimed.get(),
    }
}

#[derive(Clone)]
enum Node {
    List(Rc<std::cell::RefCell<Vec<Value>>>),
    Bytes(Rc<std::cell::RefCell<Vec<u8>>>),
    Tuple(Rc<super::TupleData>),
    Set(Rc<super::SetData>),
    Object(Rc<std::cell::RefCell<HashMap<String, Value>>>),
    Closure(Rc<ClosureData>),
    Iterator(Rc<super::IteratorData>),
}

type Key = (u8, usize);

impl Node {
    fn key(&self) -> Key {
        match self {
            Self::List(value) => (TAG_LIST, Rc::as_ptr(value) as usize),
            Self::Bytes(value) => (TAG_BYTES, Rc::as_ptr(value) as usize),
            Self::Tuple(value) => (TAG_TUPLE, Rc::as_ptr(value) as usize),
            Self::Set(value) => (TAG_SET, Rc::as_ptr(value) as usize),
            Self::Object(value) => (TAG_OBJECT, Rc::as_ptr(value) as usize),
            Self::Closure(value) => (TAG_CLOSURE, Rc::as_ptr(value) as usize),
            Self::Iterator(value) => (TAG_ITERATOR, Rc::as_ptr(value) as usize),
        }
    }

    fn strong_count(&self) -> usize {
        match self {
            Self::List(value) => Rc::strong_count(value),
            Self::Bytes(value) => Rc::strong_count(value),
            Self::Tuple(value) => Rc::strong_count(value),
            Self::Set(value) => Rc::strong_count(value),
            Self::Object(value) => Rc::strong_count(value),
            Self::Closure(value) => Rc::strong_count(value),
            Self::Iterator(value) => Rc::strong_count(value),
        }
    }

    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::List(value) => Some(Self::List(Rc::clone(value))),
            Value::Bytes(value) => Some(Self::Bytes(Rc::clone(value))),
            Value::Tuple(value) => Some(Self::Tuple(Rc::clone(value))),
            Value::Set(value) => Some(Self::Set(Rc::clone(value))),
            Value::Object(value) => Some(Self::Object(Rc::clone(value))),
            Value::Closure(value) => Some(Self::Closure(Rc::clone(value))),
            Value::Iterator(value) => Some(Self::Iterator(Rc::clone(value))),
            _ => None,
        }
    }

    fn children(&self, teardown: bool) -> (Vec<Self>, Vec<SoplangValue>, bool) {
        match self {
            Self::Tuple(value) => match value.children() {
                Some(values) => (
                    values.iter().filter_map(Self::from_value).collect(),
                    Vec::new(),
                    false,
                ),
                None => (Vec::new(), Vec::new(), true),
            },
            Self::Set(value) => match value.children() {
                Some(values) => (
                    values.iter().filter_map(Self::from_value).collect(),
                    Vec::new(),
                    false,
                ),
                None => (Vec::new(), Vec::new(), true),
            },
            Self::Bytes(_) => (Vec::new(), Vec::new(), false),
            Self::List(value) => match value.try_borrow_mut() {
                Ok(values) => (
                    values.iter().filter_map(Self::from_value).collect(),
                    Vec::new(),
                    false,
                ),
                Err(_) => (Vec::new(), Vec::new(), true),
            },
            Self::Object(value) => match value.try_borrow_mut() {
                Ok(values) => (
                    values.values().filter_map(Self::from_value).collect(),
                    Vec::new(),
                    false,
                ),
                Err(_) => (Vec::new(), Vec::new(), true),
            },
            // Callable handles cannot outlive their runtime. At teardown their
            // opaque captures no longer own otherwise unreachable collections.
            Self::Closure(value) if !teardown => (Vec::new(), value.env.clone(), false),
            Self::Closure(_) => (Vec::new(), Vec::new(), false),
            Self::Iterator(value) => match value.children() {
                Some(values) => (
                    values.iter().filter_map(Self::from_value).collect(),
                    Vec::new(),
                    false,
                ),
                None => (Vec::new(), Vec::new(), true),
            },
        }
    }

    fn clear(&self) {
        match self {
            Self::List(value) => {
                if let Ok(mut values) = value.try_borrow_mut() {
                    values.clear();
                }
            }
            Self::Object(value) => {
                if let Ok(mut values) = value.try_borrow_mut() {
                    values.clear();
                }
            }
            Self::Bytes(_) | Self::Closure(_) => (),
            Self::Tuple(value) => value.clear(),
            Self::Set(value) => value.clear(),
            Self::Iterator(value) => value.clear(),
        }
    }
}

#[derive(Clone, Copy)]
enum Edge {
    Node(usize),
    Handle(SoplangValue),
}

struct Entry {
    node: Node,
    arena_refs: usize,
    incoming_refs: usize,
    edges: Vec<Edge>,
    borrowed: bool,
    marked: bool,
}

#[derive(Default)]
struct Graph {
    nodes: Vec<Entry>,
    indices: HashMap<Key, usize>,
    handles: HashMap<(u8, i64), usize>,
}

impl Graph {
    fn node(&mut self, node: Node) -> usize {
        let key = node.key();
        if let Some(index) = self.indices.get(&key) {
            return *index;
        }
        let index = self.nodes.len();
        self.indices.insert(key, index);
        self.nodes.push(Entry {
            node,
            arena_refs: 0,
            incoming_refs: 0,
            edges: Vec::new(),
            borrowed: false,
            marked: false,
        });
        index
    }

    fn arena(&mut self, handle: i64, node: Node) {
        let tag = node.key().0;
        let index = self.node(node);
        self.nodes[index].arena_refs += 1;
        self.handles.insert((tag, handle), index);
    }

    fn build(state: &RuntimeState, teardown: bool) -> Self {
        let mut graph = Self::default();
        for (id, value) in state.tuples.borrow().iter() {
            graph.arena(id, Node::Tuple(Rc::clone(value)));
        }
        for (id, value) in state.sets.borrow().iter() {
            graph.arena(id, Node::Set(Rc::clone(value)));
        }
        for (id, value) in state.bytes.borrow().iter() {
            graph.arena(id, Node::Bytes(Rc::clone(value)));
        }
        for (id, value) in state.lists.borrow().iter() {
            graph.arena(id, Node::List(Rc::clone(value)));
        }
        for (id, value) in state.objects.borrow().iter() {
            graph.arena(id, Node::Object(Rc::clone(value)));
        }
        for (id, value) in state.closures.borrow().iter() {
            graph.arena(id, Node::Closure(Rc::clone(value)));
        }
        for (id, value) in state.iterators.borrow().iter() {
            graph.arena(id, Node::Iterator(Rc::clone(value)));
        }
        let mut index = 0;
        while index < graph.nodes.len() {
            let (references, handles, borrowed) = graph.nodes[index].node.children(teardown);
            graph.nodes[index].borrowed = borrowed;
            for reference in references {
                let child = graph.node(reference);
                graph.nodes[child].incoming_refs += 1;
                graph.nodes[index].edges.push(Edge::Node(child));
            }
            graph.nodes[index]
                .edges
                .extend(handles.into_iter().map(Edge::Handle));
            index += 1;
        }
        graph
    }
}

pub(super) fn collect(state: &RuntimeState, teardown: bool) -> CollectionReport {
    let before = stats(state);
    if teardown {
        // Cursors are opaque and unusable after their owner dies. Release their
        // owned graph edges even if Rust still retains the iterator handle.
        for (_, iterator) in state.iterators.borrow().iter() {
            iterator.clear();
        }
    }
    let mut graph = Graph::build(state, teardown);
    let mut work = Vec::new();
    for (index, entry) in graph.nodes.iter().enumerate() {
        // One strong reference is our graph snapshot. Every other reference
        // outside arenas and collection edges is held by native or Rust code.
        if entry.borrowed || entry.node.strong_count() > entry.arena_refs + entry.incoming_refs + 1
        {
            work.push(Edge::Node(index));
        }
    }
    if !teardown {
        work.extend(state.globals.borrow().values().copied().map(Edge::Handle));
        work.extend(
            state
                .pins
                .borrow()
                .iter()
                .map(|(_, value)| Edge::Handle(*value)),
        );
        for frame in state.roots.borrow().iter() {
            for slot in 0..frame.len {
                let offset = slot * frame.stride;
                // Generated frames own this aligned storage until leave_roots;
                // collection only runs at safepoints after slot writes finish.
                let value = unsafe {
                    SoplangValue {
                        tag: *frame.tags.add(offset) as u8,
                        _pad: [0; 7],
                        payload: *frame.payloads.add(offset),
                    }
                };
                work.push(Edge::Handle(value));
            }
        }
    }
    let mut handles = HashSet::new();
    while let Some(edge) = work.pop() {
        match edge {
            Edge::Handle(value) => {
                if value.tag == TAG_STR || value.tag == TAG_ERROR {
                    handles.insert((value.tag, value.payload));
                }
                if let Some(index) = graph.handles.get(&(value.tag, value.payload)) {
                    work.push(Edge::Node(*index));
                }
            }
            Edge::Node(index) => {
                if graph.nodes[index].marked {
                    continue;
                }
                graph.nodes[index].marked = true;
                work.extend(graph.nodes[index].edges.iter().copied());
            }
        }
    }
    for entry in &graph.nodes {
        if !entry.marked {
            entry.node.clear();
        }
    }
    let keep = |tag, id| {
        graph
            .handles
            .get(&(tag, id))
            .is_some_and(|index| graph.nodes[*index].marked)
    };
    state
        .strings
        .borrow_mut()
        .retain(|id, _| handles.contains(&(TAG_STR, id)));
    state
        .errors
        .borrow_mut()
        .retain(|id, _| handles.contains(&(TAG_ERROR, id)));
    state.lists.borrow_mut().retain(|id, _| keep(TAG_LIST, id));
    state.bytes.borrow_mut().retain(|id, _| keep(TAG_BYTES, id));
    state
        .tuples
        .borrow_mut()
        .retain(|id, _| keep(TAG_TUPLE, id));
    state
        .tuple_ids
        .borrow_mut()
        .retain(|_, id| state.tuples.borrow().contains(*id));
    state.sets.borrow_mut().retain(|id, _| keep(TAG_SET, id));
    state
        .set_ids
        .borrow_mut()
        .retain(|_, id| state.sets.borrow().contains(*id));
    state
        .objects
        .borrow_mut()
        .retain(|id, _| keep(TAG_OBJECT, id));
    state
        .closures
        .borrow_mut()
        .retain(|id, _| keep(TAG_CLOSURE, id));
    state
        .iterators
        .borrow_mut()
        .retain(|id, _| keep(TAG_ITERATOR, id));
    let swept = before.values().saturating_sub(stats(state).values());
    state
        .byte_ids
        .borrow_mut()
        .retain(|_, id| state.bytes.borrow().contains(*id));
    state
        .list_ids
        .borrow_mut()
        .retain(|_, id| state.lists.borrow().contains(*id));
    state
        .object_ids
        .borrow_mut()
        .retain(|_, id| state.objects.borrow().contains(*id));
    if !teardown {
        // Track live nested collections first seen through a Rust-created value
        // graph so a later pass can reclaim their cycles after its parent dies.
        for entry in graph
            .nodes
            .iter()
            .filter(|entry| entry.marked && entry.arena_refs == 0)
        {
            match &entry.node {
                Node::Tuple(value) => {
                    let id = state.tuples.borrow_mut().insert(Rc::clone(value));
                    state
                        .tuple_ids
                        .borrow_mut()
                        .insert(Rc::as_ptr(value) as usize, id);
                }
                Node::Set(value) => {
                    let id = state.sets.borrow_mut().insert(Rc::clone(value));
                    state
                        .set_ids
                        .borrow_mut()
                        .insert(Rc::as_ptr(value) as usize, id);
                }
                Node::Bytes(value) => {
                    let id = state.bytes.borrow_mut().insert(Rc::clone(value));
                    state
                        .byte_ids
                        .borrow_mut()
                        .insert(Rc::as_ptr(value) as usize, id);
                }
                Node::List(value) => {
                    let id = state.lists.borrow_mut().insert(Rc::clone(value));
                    state
                        .list_ids
                        .borrow_mut()
                        .insert(Rc::as_ptr(value) as usize, id);
                }
                Node::Object(value) => {
                    let id = state.objects.borrow_mut().insert(Rc::clone(value));
                    state
                        .object_ids
                        .borrow_mut()
                        .insert(Rc::as_ptr(value) as usize, id);
                }
                Node::Closure(_) | Node::Iterator(_) => (), // Opaque values stay in their owning context.
            }
        }
    }
    let deferred = graph.nodes.iter().filter(|entry| entry.borrowed).count();
    drop(graph);
    state.contracts.borrow_mut().prune();
    state
        .collector
        .collections
        .set(state.collector.collections.get().saturating_add(1));
    state
        .collector
        .reclaimed
        .set(state.collector.reclaimed.get().saturating_add(swept));
    let after = stats(state);
    state.collector.next_allocations.set(
        state
            .collector
            .allocations
            .get()
            .saturating_add(after.values().max(MIN_ALLOCATIONS)),
    );
    let live_bytes = state
        .strings
        .borrow()
        .iter()
        .map(|(_, value)| value.len())
        .sum::<usize>()
        .saturating_add(
            state
                .bytes
                .borrow()
                .iter()
                .map(|(_, value)| {
                    value
                        .try_borrow()
                        .map(|bytes| bytes.capacity())
                        .unwrap_or(0)
                })
                .sum::<usize>(),
        )
        .saturating_add(
            state
                .tuples
                .borrow()
                .iter()
                .map(|(_, value)| value.len().saturating_mul(std::mem::size_of::<Value>()))
                .sum::<usize>(),
        )
        .saturating_add(
            state
                .sets
                .borrow()
                .iter()
                .map(|(_, value)| value.len().saturating_mul(std::mem::size_of::<Value>()))
                .sum::<usize>(),
        )
        .saturating_add(after.values().saturating_mul(64));
    state.collector.next_bytes.set(
        state
            .collector
            .bytes
            .get()
            .saturating_add(live_bytes.max(MIN_BYTES)),
    );
    CollectionReport {
        before,
        after,
        reclaimed: swept,
        deferred,
    }
}
