use crate::*;
use core::borrow::Borrow;
use core::fmt;
use core::hash::{BuildHasher, Hash, Hasher};
use std::collections::hash_map::{DefaultHasher, HashMap};

#[derive(Clone, Copy)]
pub struct DetState;

impl BuildHasher for DetState {
    type Hasher = DefaultHasher;

    #[inline]
    fn build_hasher(&self) -> DefaultHasher {
        return DefaultHasher::new();
    }
}

#[derive(Clone, Copy)]
pub enum HashRefSlot<Key, Value>
where
    Key: Copy,
    Value: Copy,
{
    Some(Key, Value),
    None,
}

#[derive(Clone, Copy)]
pub struct HashRef<'a, Key, Value, State = DetState>
where
    Key: Eq + Hash + Copy + 'a,
    Value: Copy + 'a,
    State: BuildHasher,
{
    pub slots: &'a [HashRefSlot<Key, Value>],
    pub size: usize,
    pub state: State,
}

impl<'a, K, V> HashRef<'a, K, V, DetState>
where
    K: Eq + Hash + Copy + 'a,
    V: Copy + 'a,
{
    pub fn new(frame: impl Allocator, data: &HashMap<K, V>) -> Self {
        return Self::with_state(frame, data, DetState);
    }

    pub fn new_iter<I>(frame: impl Allocator, capa: usize, data: I) -> Self
    where
        I: Iterator<Item = (K, V)>,
    {
        return Self::with_state_iter(frame, capa, data, DetState);
    }

    pub fn empty() -> Self {
        Self {
            slots: &mut [],
            size: 0,
            state: DetState,
        }
    }
}

impl<'a, K, V, State> HashRef<'a, K, V, State>
where
    K: Eq + Hash + Copy + 'a,
    V: Copy + 'a,
    State: BuildHasher,
{
    pub fn with_state(frame: impl Allocator, data: &HashMap<K, V>, state: State) -> Self {
        let capa = data.len() * 3 / 2;
        return Self::with_state_iter(frame, capa, data.iter().map(|(&k, &v)| (k, v)), state);
    }

    pub fn with_state_iter<I>(frame: impl Allocator, capa: usize, data: I, state: State) -> Self
    where
        I: Iterator<Item = (K, V)>,
    {
        let mut slots_array = pod![HashRefSlot::None; capa; &frame];
        let slots = &mut *slots_array;
        let mut size = 0;

        for (key, value) in data {
            if size == capa {
                panic!(
                    "allocated too little capacity for size (size = capacity = {})",
                    size
                );
            }

            let mut hasher = state.build_hasher();
            key.hash(&mut hasher);
            let mut slot_idx = hasher.finish() as usize % slots.len();

            loop {
                match &mut slots[slot_idx] {
                    HashRefSlot::Some(slot_key, slot_value) => {
                        if slot_key == &key {
                            *slot_key = key;
                            *slot_value = value;
                            break;
                        }
                    }
                    slot @ HashRefSlot::None => {
                        *slot = HashRefSlot::Some(key, value);
                        size += 1;
                        break;
                    }
                }

                slot_idx += 1;
                slot_idx = slot_idx % slots.len();
            }
        }

        let slots: &'a mut [_] = slots_array.leak();
        Self { slots, size, state }
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.size;
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        return self.slots.len();
    }

    fn get_index<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut hasher = self.state.build_hasher();
        key.hash(&mut hasher);

        let mut slot_idx = hasher.finish() as usize % self.slots.len();
        let original_slot_idx = slot_idx;
        match &self.slots[slot_idx] {
            HashRefSlot::None => return None,
            HashRefSlot::Some(slot_key, slot_value) => {
                if slot_key.borrow() == key {
                    return Some(slot_idx);
                }
            }
        }

        loop {
            slot_idx += 1;
            slot_idx = slot_idx % self.slots.len();

            if slot_idx == original_slot_idx {
                return None;
            }

            match &self.slots[slot_idx] {
                HashRefSlot::None => return None,
                HashRefSlot::Some(slot_key, slot_value) => {
                    if slot_key.borrow() == key {
                        return Some(slot_idx);
                    }
                }
            }
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = self.get_index(key)?;

        match &self.slots[idx] {
            HashRefSlot::None => return None,
            HashRefSlot::Some(slot_key, slot_value) => {
                if slot_key.borrow() == key {
                    return Some(slot_value);
                }
            }
        };

        return None;
    }
}

pub struct HashRefIter<'a, Key, Value>
where
    Key: Copy,
    Value: Copy,
{
    pub slots: &'a [HashRefSlot<Key, Value>],
    pub slot_idx: usize,
}

impl<'a, Key, Value> Iterator for HashRefIter<'a, Key, Value>
where
    Key: Eq + Hash + Copy,
    Value: Copy,
{
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.slot_idx == self.slots.len() {
                return None;
            } else if let HashRefSlot::Some(key, value) = &self.slots[self.slot_idx] {
                self.slot_idx += 1;
                return Some((key, value));
            }

            self.slot_idx += 1;
        }
    }
}

impl<'a, K, V, State> IntoIterator for &HashRef<'a, K, V, State>
where
    K: Eq + Hash + Copy,
    V: Copy,
    State: BuildHasher,
{
    type Item = (&'a K, &'a V);
    type IntoIter = HashRefIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        HashRefIter {
            slots: self.slots,
            slot_idx: 0,
        }
    }
}

impl<'a, K, V, State> IntoIterator for HashRef<'a, K, V, State>
where
    K: Eq + Hash + Copy,
    V: Copy,
    State: BuildHasher,
{
    type Item = (&'a K, &'a V);
    type IntoIter = HashRefIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        HashRefIter {
            slots: self.slots,
            slot_idx: 0,
        }
    }
}

impl<'a, Key, Value, State> fmt::Debug for HashRef<'a, Key, Value, State>
where
    Key: Eq + Hash + Copy + fmt::Debug,
    Value: fmt::Debug + Copy,
    State: BuildHasher,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_map().entries(self.into_iter()).finish()
    }
}
