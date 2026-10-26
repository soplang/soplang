//! Bounded, context-owned channels for cooperative Soplang tasks.

use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use super::context::{RuntimeLocal, RuntimeState};
use super::tasks;
use super::value::Value;
use crate::error::{runtime_error, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);
const MAX_CAPACITY: usize = 65_536;

#[derive(Default)]
struct ChannelState {
    capacity: usize,
    queue: VecDeque<Value>,
    offer: Option<(i64, Value)>,
    completed_senders: HashSet<i64>,
    senders: VecDeque<i64>,
    receivers: VecDeque<i64>,
    closed: bool,
}

pub struct ChannelData {
    id: Cell<i64>,
    state: RefCell<ChannelState>,
}

impl ChannelData {
    pub fn handle(&self) -> i64 {
        self.id.get()
    }

    pub(super) fn children(&self) -> Option<Vec<Value>> {
        let state = self.state.try_borrow().ok()?;
        let mut values: Vec<_> = state.queue.iter().cloned().collect();
        if let Some((_, value)) = &state.offer {
            values.push(value.clone());
        }
        Some(values)
    }

    pub(super) fn clear(&self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.queue.clear();
            state.offer = None;
        }
    }
}

fn error(message: &str) -> SoplangError {
    runtime_error(message, 0, 0)
}

fn validate(data: &Rc<ChannelData>) -> Result<(), SoplangError> {
    STATE.with(|state| {
        if state
            .channels
            .borrow()
            .get(data.handle())
            .is_some_and(|entry| Rc::ptr_eq(entry, data))
        {
            Ok(())
        } else {
            Err(super::abi::invalid_handle())
        }
    })
}

fn wake_one(waiters: &mut VecDeque<i64>, channel: i64) {
    while let Some(id) = waiters.pop_front() {
        if tasks::wake_channel_task(id, channel) {
            break;
        }
    }
}

fn wake_all(waiters: &mut VecDeque<i64>, channel: i64) {
    while let Some(id) = waiters.pop_front() {
        tasks::wake_channel_task(id, channel);
    }
}

pub(super) fn create(capacity: usize) -> Value {
    let data = Rc::new(ChannelData {
        id: Cell::new(0),
        state: RefCell::new(ChannelState {
            capacity,
            ..ChannelState::default()
        }),
    });
    STATE.with(|state| {
        let id = state.channels.borrow_mut().insert(Rc::clone(&data));
        data.id.set(id);
        state
            .collector
            .allocated(std::mem::size_of::<ChannelData>());
    });
    Value::Channel(data)
}

pub(super) fn send(data: &Rc<ChannelData>, value: Value) -> Result<(), SoplangError> {
    validate(data)?;
    let id = tasks::running_task();
    loop {
        let mut channel = data.state.borrow_mut();
        if channel.completed_senders.remove(&id) {
            return Ok(());
        }
        if channel.closed {
            return Err(error("Marinka waa xiran yahay."));
        }
        if channel.capacity > 0 && channel.queue.len() < channel.capacity {
            channel.queue.push_back(value);
            wake_one(&mut channel.receivers, data.handle());
            return Ok(());
        }
        if channel.capacity == 0 && channel.offer.is_none() {
            channel.offer = Some((id, value.clone()));
            wake_one(&mut channel.receivers, data.handle());
        }
        if id != 0 && !channel.senders.contains(&id) {
            channel.senders.push_back(id);
        }
        drop(channel);
        if let Err(error) = tasks::wait_channel(data.handle()) {
            let mut channel = data.state.borrow_mut();
            channel.senders.retain(|sender| *sender != id);
            if channel.completed_senders.remove(&id) {
                return Ok(());
            }
            if channel
                .offer
                .as_ref()
                .is_some_and(|(sender, _)| *sender == id)
            {
                channel.offer = None;
                wake_one(&mut channel.senders, data.handle());
            }
            return Err(error);
        }
    }
}

pub(super) fn receive(data: &Rc<ChannelData>) -> Result<Option<Value>, SoplangError> {
    validate(data)?;
    let id = tasks::running_task();
    loop {
        let mut channel = data.state.borrow_mut();
        if let Some(value) = channel.queue.pop_front() {
            wake_one(&mut channel.senders, data.handle());
            return Ok(Some(value));
        }
        if let Some((sender, value)) = channel.offer.take() {
            channel.completed_senders.insert(sender);
            tasks::wake_channel_task(sender, data.handle());
            wake_one(&mut channel.senders, data.handle());
            return Ok(Some(value));
        }
        if channel.closed {
            return Ok(None);
        }
        if id != 0 && !channel.receivers.contains(&id) {
            channel.receivers.push_back(id);
        }
        drop(channel);
        if let Err(error) = tasks::wait_channel(data.handle()) {
            data.state
                .borrow_mut()
                .receivers
                .retain(|receiver| *receiver != id);
            return Err(error);
        }
    }
}

pub(super) fn close(data: &Rc<ChannelData>) -> Result<(), SoplangError> {
    validate(data)?;
    let mut channel = data.state.borrow_mut();
    if channel.closed {
        return Ok(());
    }
    channel.closed = true;
    channel.offer = None;
    wake_all(&mut channel.senders, data.handle());
    wake_all(&mut channel.receivers, data.handle());
    Ok(())
}

fn expect_channel<'a>(value: &'a Value, name: &str) -> Result<&'a Rc<ChannelData>, SoplangError> {
    let Value::Channel(data) = value else {
        return Err(runtime_error(
            format!("{}() waxay u baahan tahay marin.", name),
            0,
            0,
        ));
    };
    Ok(data)
}

pub(super) fn builtin_marin(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [Value::Int(capacity)] = args.as_slice() else {
        return Err(error("marin() waxay u baahan tahay xad tiro dhan ah."));
    };
    if *capacity < 0 || *capacity as usize > MAX_CAPACITY {
        return Err(error("Xadka marinku waa inuu u dhexeeyaa 0 iyo 65536."));
    }
    Ok(create(*capacity as usize))
}

pub(super) fn builtin_dir(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [channel, value] = args.as_slice() else {
        return Err(error("dir() waxay u baahan tahay marin iyo qiime."));
    };
    send(expect_channel(channel, "dir")?, value.clone())?;
    Ok(Value::Null)
}

pub(super) fn builtin_qaad(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [channel] = args.as_slice() else {
        return Err(error("qaad() waxay u baahan tahay marin."));
    };
    let value = receive(expect_channel(channel, "qaad")?)?;
    let mut result = std::collections::HashMap::new();
    result.insert("dhammaad".to_owned(), Value::Bool(value.is_none()));
    result.insert("qiime".to_owned(), value.unwrap_or(Value::Null));
    Ok(Value::Object(Rc::new(RefCell::new(result))))
}

pub(super) fn builtin_xidh(args: Vec<Value>) -> Result<Value, SoplangError> {
    let [channel] = args.as_slice() else {
        return Err(error("xidh() waxay u baahan tahay marin."));
    };
    close(expect_channel(channel, "xidh")?)?;
    Ok(Value::Null)
}
