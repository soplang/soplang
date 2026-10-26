//! Explicit roots held by generated stack frames and Rust integrations.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use super::abi::{invalid_handle, SoplangValue};
use super::context::{RuntimeLocal, RuntimeState};
use crate::error::SoplangError;

/// Generated storage stays valid until its matching leave operation.
#[derive(Clone, Copy)]
pub(super) struct RootFrame {
    pub tags: *const i64,
    pub payloads: *const i64,
    pub len: usize,
    /// Distance between slots, measured in 64-bit words.
    pub stride: usize,
}

const FRAMES: RuntimeLocal<RefCell<Vec<RootFrame>>> = RuntimeLocal::new(|state| &state.roots);

pub(super) fn enter(tags: *const i64, payloads: *const i64, len: usize, stride: usize) {
    super::debugger::enter();
    FRAMES.with(|frames| {
        frames.borrow_mut().push(RootFrame {
            tags,
            payloads,
            len,
            stride,
        })
    });
}

pub(super) fn leave() {
    super::debugger::leave();
    FRAMES.with(|frames| {
        frames.borrow_mut().pop();
    });
}

/// A tagged value pinned in its owning runtime. It does not extend a JIT
/// backend's lifetime: once the owner is dropped, `get` returns an error.
pub struct RootedValue {
    state: Weak<RuntimeState>,
    pin: i64,
}

impl RootedValue {
    pub(super) fn new(state: &Rc<RuntimeState>, value: SoplangValue) -> Self {
        let pin = state.pins.borrow_mut().insert(value);
        Self {
            state: Rc::downgrade(state),
            pin,
        }
    }

    pub fn get(&self) -> Result<SoplangValue, SoplangError> {
        let state = self.state.upgrade().ok_or_else(invalid_handle)?;
        let value = state
            .pins
            .borrow()
            .get(self.pin)
            .copied()
            .ok_or_else(invalid_handle);
        value
    }
}

impl Drop for RootedValue {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state.pins.borrow_mut().remove(self.pin);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{soplang_int, RuntimeContext};

    #[test]
    fn roots_are_scoped_and_cannot_keep_dead_execution_state_alive() {
        let context = RuntimeContext::new();
        let root = context.root(soplang_int(7)).unwrap();
        assert_eq!(root.get().unwrap().payload, 7);
        assert_eq!(context.state.pins.borrow().len(), 1);
        drop(root);
        assert_eq!(context.state.pins.borrow().len(), 0);
        let root = context.root(soplang_int(8)).unwrap();
        drop(context);
        assert_eq!(root.get().unwrap_err().code(), Some("E035"));
    }
}
