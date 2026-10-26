//! Reusable slots with process-wide allocation identities.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

// Zero is invalid. Exhaustion is checked so identities can never wrap or alias.
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

pub(super) struct HandleArena<T> {
    slots: Vec<Option<(i64, T)>>,
    free: Vec<usize>,
    indices: HashMap<i64, usize>,
}

impl<T> Default for HandleArena<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            indices: HashMap::new(),
        }
    }
}

impl<T> HandleArena<T> {
    pub fn insert(&mut self, value: T) -> i64 {
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .expect("Aqoonsiyada qiimayaasha way dhammaadeen.");
        let slot = match self.free.pop() {
            Some(slot) => {
                self.slots[slot] = Some((id, value));
                slot
            }
            None => {
                self.slots.push(Some((id, value)));
                self.slots.len() - 1
            }
        };
        self.indices.insert(id, slot);
        id
    }

    pub fn get(&self, id: i64) -> Option<&T> {
        let slot = *self.indices.get(&id)?;
        let (generation, value) = self.slots.get(slot)?.as_ref()?;
        (*generation == id).then_some(value)
    }

    pub fn get_mut(&mut self, id: i64) -> Option<&mut T> {
        let slot = *self.indices.get(&id)?;
        let (generation, value) = self.slots.get_mut(slot)?.as_mut()?;
        (*generation == id).then_some(value)
    }

    pub fn remove(&mut self, id: i64) -> Option<T> {
        let slot = self.indices.remove(&id)?;
        let (_, value) = self.slots[slot].take()?;
        self.free.push(slot);
        Some(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (i64, &T)> {
        self.slots
            .iter()
            .filter_map(|slot| slot.as_ref().map(|(id, value)| (*id, value)))
    }

    pub fn retain(&mut self, mut keep: impl FnMut(i64, &T) -> bool) {
        let removed: Vec<_> = self
            .iter()
            .filter_map(|(id, value)| (!keep(id, value)).then_some(id))
            .collect();
        for id in removed {
            self.remove(id);
        }
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn contains(&self, id: i64) -> bool {
        self.get(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reused_slots_reject_every_old_generation() {
        let mut arena = HandleArena::default();
        let mut expired = Vec::new();
        for value in 0..1000 {
            let id = arena.insert(value);
            for old in &expired {
                assert!(arena.get(*old).is_none());
            }
            assert_eq!(arena.get(id), Some(&value));
            assert_eq!(arena.remove(id), Some(value));
            assert!(arena.remove(id).is_none());
            expired.push(id);
        }
        assert_eq!(arena.slots.len(), 1);
        assert!(arena.indices.is_empty());
    }

    #[test]
    fn identities_are_distinct_across_arenas_and_lifetimes() {
        let first = {
            let mut arena = HandleArena::default();
            arena.insert("first")
        };
        let mut second = HandleArena::default();
        let id = second.insert("second");
        assert_ne!(first, id);
        for invalid in [0, -1, i64::MIN, i64::MAX, first] {
            assert!(second.get(invalid).is_none());
        }
        assert_eq!(second.get(id), Some(&"second"));
    }
}
