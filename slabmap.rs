/*! This crate provides the type [`SlabMap`].
[`SlabMap`] is HashMap-like collection that automatically determines the key.

# Examples

```
use slabmap::SlabMap;

let mut s = SlabMap::new();
let key_a = s.insert("aaa");
let key_b = s.insert("bbb");

assert_eq!(s[key_a], "aaa");
assert_eq!(s[key_b], "bbb");

for (key, value) in &s {
    println!("{} -> {}", key, value);
}

assert_eq!(s.remove(key_a), Some("aaa"));
assert_eq!(s.remove(key_a), None);
```
*/

pub mod slab_map//! A fast HashMap-like collection that automatically determines the key.

use std::{
    collections::TryReserveError,
    fmt::Debug,
    iter::{Enumerate, FusedIterator},
    mem::replace,
};

use derive_ex::derive_ex;

#[cfg(test)]
mod testsuse std::time::Instant;

use crate::SlabMap;

#[test]
fn test_new() {
    let s = SlabMap::<u32>::new();
    assert_eq!(s.len(), 0);
}

#[test]
fn test_with_capacity() {
    for cap in 0..100 {
        let s = SlabMap::<u32>::with_capacity(cap);
        assert!(s.capacity() >= cap);
    }
}

#[test]
fn test_retain() {
    let mut s = SlabMap::new();
    s.insert(10);
    s.insert(15);
    s.insert(20);
    s.insert(25);

    s.retain(|_idx, x| *x % 2 == 0);

    let value: Vec<_> = s.values().cloned().collect();
    assert_eq!(value, vec![10, 20]);
    assert_eq!(s.len(), 2);
}

#[test]
fn test_len() {
    let mut s = SlabMap::new();
    assert_eq!(s.len(), 0);

    let key1 = s.insert(10);
    let key2 = s.insert(15);

    assert_eq!(s.len(), 2);

    s.remove(key1);
    assert_eq!(s.len(), 1);

    s.remove(key2);
    assert_eq!(s.len(), 0);
}

#[test]
fn test_is_empty() {
    let mut s = SlabMap::new();
    assert!(s.is_empty());

    let key = s.insert("a");
    assert!(!s.is_empty());

    s.remove(key);
    assert!(s.is_empty());
}

#[test]
fn test_get() {
    let mut s = SlabMap::new();
    let key = s.insert(100);

    assert_eq!(s.get(key), Some(&100));
    assert_eq!(s.get(key + 1), None);
}

#[test]
fn test_contains_key() {
    let mut s = SlabMap::new();
    let key = s.insert(100);

    assert!(s.contains_key(key));
    assert!(!s.contains_key(key + 1));
}

#[test]
fn test_insert() {
    let mut s = SlabMap::new();
    let key_abc = s.insert("abc");
    let key_xyz = s.insert("xyz");

    assert_eq!(s[key_abc], "abc");
    assert_eq!(s[key_xyz], "xyz");
}

#[test]
fn test_insert_with_key() {
    let mut s = SlabMap::new();
    let key = s.insert_with_key(|key| format!("my key is {}", key));

    assert_eq!(s[key], format!("my key is {}", key));
}

#[test]
fn test_remove() {
    let mut s = SlabMap::new();
    let key = s.insert("a");
    assert_eq!(s.remove(key), Some("a"));
    assert_eq!(s.remove(key), None);
}

#[test]
fn test_clear() {
    let mut s = SlabMap::new();
    s.insert(1);
    s.insert(2);

    s.clear();

    assert!(s.is_empty());
}

#[test]
fn test_drain() {
    let mut s = SlabMap::new();
    let k0 = s.insert(10);
    let k1 = s.insert(20);

    let d: Vec<_> = s.drain().collect();
    let mut e = vec![(k0, 10), (k1, 20)];
    e.sort();

    assert!(s.is_empty());
    assert_eq!(d, e);
}

#[test]
fn test_optimize() {
    let mut s = SlabMap::new();
    const COUNT: usize = 1000000;
    for i in 0..COUNT {
        s.insert(i);
    }
    let keys: Vec<_> = s.keys().take(COUNT - 1).collect();
    for key in keys {
        s.remove(key);
    }

    s.optimize(); // if comment out this line, `s.values().sum()` to be slow.

    let begin = Instant::now();
    let sum: usize = s.values().sum();
    println!("sum : {}", sum);
    println!("duration : {} ms", (Instant::now() - begin).as_millis());
}

#[test]
fn insert_remove_capacity() {
    let mut s = SlabMap::new();
    let mut keys = Vec::new();
    for _ in 0..10 {
        s.insert(11);
    }
    for _ in 0..100 {
        keys.push(s.insert(10));
    }
    let capacity = s.capacity();
    for _ in 0..1000 {
        for key in keys.drain(..) {
            s.remove(key);
        }
        for _ in 0..100 {
            keys.push(s.insert(10));
        }
    }
    assert_eq!(capacity, s.capacity());
}

#[test]
fn insert_remove_capacity_all() {
    let mut s = SlabMap::new();
    let mut keys = Vec::new();
    for _ in 0..100 {
        keys.push(s.insert(10));
    }
    let capacity = s.capacity();
    for _ in 0..1000 {
        for key in keys.drain(..) {
            s.remove(key);
        }
        for _ in 0..100 {
            keys.push(s.insert(10));
        }
    }
    assert_eq!(capacity, s.capacity());
}

#[test]
fn into_iter() {
    let mut s = SlabMap::new();
    let k0 = s.insert(0);
    let k1 = s.insert(1);
    let k2 = s.insert(2);
    s.remove(k1);

    let a: Vec<_> = s.into_iter().collect();
    let mut e = vec![(k0, 0), (k2, 2)];
    e.sort();

    assert_eq!(a, e);
}

#[test]
fn clone_from() {
    let mut s0 = SlabMap::new();
    let mut s1 = SlabMap::new();
    for _ in 0..10 {
        s0.insert(0);
    }
    for _ in 0..1000 {
        s1.insert(0);
    }
    let cap_old = s1.capacity();
    s1.clone_from(&s0);
    let cap_new = s1.capacity();
    assert_eq!(cap_old, cap_new);
}

#[test]
fn from_iter() {
    let s: SlabMap<usize> = [(5, 1), (0, 3)].into_iter().collect();
    assert_eq!(s.len(), 2, "len");
    assert_eq!(s[5], 1);
    assert_eq!(s[0], 3);
}

#[test]
fn merge_vacant() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13)].into_iter().collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    let e = vec![(0, 10), (3, 13)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_insert() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13)].into_iter().collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    let key = s.insert(99);
    let e = vec![(0, 10), (key, 99), (3, 13)];
    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_insert_2() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(3);
    s.optimize();
    let key = s.insert(99);
    let e = vec![(0, 10), (key, 99), (4, 14)];
    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_2time() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14), (5, 15)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    s.remove(4);
    s.optimize();

    let e = vec![(0, 10), (3, 13), (5, 15)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_2part() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(4);
    s.optimize();
    let e = vec![(0, 10), (3, 13)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_drain() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(3);
    s.optimize();

    let e = vec![(0, 10), (4, 14)];
    let a: Vec<_> = s.drain().collect();
    assert_eq!(a, e);
}

#[test]
fn reserve() {
    let mut s: SlabMap<u32> = SlabMap::new();
    s.reserve(10);
    assert!(s.capacity() >= 10);
}

#[test]
fn reserve_exact() {
    let mut s: SlabMap<u32> = SlabMap::new();
    s.reserve_exact(10);
    assert!(s.capacity() == 10);
}


/// A fast HashMap-like collection that automatically determines the key.
#[derive_ex(Clone(bound(T)), Default(bound()))]
pub struct SlabMap<T> {
    entries: Vec<Entry<T>>,
    next_vacant_idx: usize,
    len: usize,
    non_optimized_count: usize,
}
const INVALID_INDEX: usize = usize::MAX;

#[derive(Clone, Debug)]
enum Entry<T> {
    Occupied(T),
    VacantHead { vacant_body_len: usize },
    VacantTail { next_vacant_idx: usize },
}

impl<T> SlabMap<T> {
    /// Constructs a new, empty `SlabMap<T>`.
    /// The SlabMap will not allocate until elements are pushed onto it.
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_vacant_idx: INVALID_INDEX,
            len: 0,
            non_optimized_count: 0,
        }
    }

    /// Constructs a new, empty `SlabMap<T>` with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            next_vacant_idx: INVALID_INDEX,
            len: 0,
            non_optimized_count: 0,
        }
    }

    /// Constructs as new `SlabMap<T>` from keys and values with at least the specified capacity.
    pub fn from_iter_with_capacity(
        iter: impl IntoIterator<Item = (usize, T)>,
        capacity: usize,
    ) -> Self {
        let mut this = Self::with_capacity(capacity);
        for (key, value) in iter {
            this.set(key, value);
        }
        this.rebuild_vacants();
        this
    }
    pub(crate) fn set(&mut self, key: usize, value: T) {
        if key >= self.entries.len() {
            self.entries.resize_with(key + 1, || Entry::VacantTail {
                next_vacant_idx: INVALID_INDEX,
            });
        }
        self.entries[key] = Entry::Occupied(value);
    }

    /// Returns the number of elements the SlabMap can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Reserves capacity for at least additional more elements to be inserted in the given `SlabMap<T>`.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.    
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(self.entries_additional(additional))
    }

    /// Try to reserve capacity for at least additional more elements to be inserted in the given `SlabMap<T>`.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.entries
            .try_reserve(self.entries_additional(additional))
    }

    /// Reserves the minimum capacity for exactly additional more elements to be inserted in the given `SlabMap<T>`.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.    
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.try_reserve_exact(additional).unwrap();
    }

    /// Try to reserve the minimum capacity for exactly additional more elements to be inserted in the given `SlabMap<T>`.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.entries
            .try_reserve_exact(self.entries_additional(additional))
    }

    #[inline]
    fn entries_additional(&self, additional: usize) -> usize {
        additional.saturating_sub(self.entries.len() - self.len)
    }

    /// Returns the number of elements in the SlabMap.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// assert_eq!(s.len(), 0);
    ///
    /// let key1 = s.insert(10);
    /// let key2 = s.insert(15);
    ///
    /// assert_eq!(s.len(), 2);
    ///
    /// s.remove(key1);
    /// assert_eq!(s.len(), 1);
    ///
    /// s.remove(key2);
    /// assert_eq!(s.len(), 0);
    /// ```    
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the SlabMap contains no elements.
    ///    
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// assert_eq!(s.is_empty(), true);
    ///
    /// let key = s.insert("a");
    /// assert_eq!(s.is_empty(), false);
    ///
    /// s.remove(key);
    /// assert_eq!(s.is_empty(), true);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert(100);
    ///
    /// assert_eq!(s.get(key), Some(&100));
    /// assert_eq!(s.get(key + 1), None);
    /// ```
    #[inline]
    pub fn get(&self, key: usize) -> Option<&T> {
        if let Entry::Occupied(value) = self.entries.get(key)? {
            Some(value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if let Entry::Occupied(value) = self.entries.get_mut(key)? {
            Some(value)
        } else {
            None
        }
    }

    /// Returns true if the SlabMap contains a value for the specified key.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert(100);
    ///
    /// assert_eq!(s.contains_key(key), true);
    /// assert_eq!(s.contains_key(key + 1), false);
    /// ```
    #[inline]
    pub fn contains_key(&self, key: usize) -> bool {
        self.get(key).is_some()
    }

    /// Inserts a value into the SlabMap.
    ///
    /// Returns the key associated with the value.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key_abc = s.insert("abc");
    /// let key_xyz = s.insert("xyz");
    ///
    /// assert_eq!(s[key_abc], "abc");
    /// assert_eq!(s[key_xyz], "xyz");
    /// ```
    pub fn insert(&mut self, value: T) -> usize {
        self.insert_raw(|_| value)
    }

    /// Inserts a value given by `f` into the SlabMap. The key to be associated with the value is passed to `f`.
    ///
    /// Returns the key associated with the value.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert_with_key(|key| format!("my key is {}", key));
    ///
    /// assert_eq!(s[key], format!("my key is {}", key));
    /// ```
    pub fn insert_with_key(&mut self, f: impl FnOnce(usize) -> T) -> usize {
        self.insert_raw(f)
    }

    #[inline]
    pub fn insert_raw(&mut self, f: impl FnOnce(usize) -> T) -> usize {
        let idx;
        if self.next_vacant_idx < self.entries.len() {
            idx = self.next_vacant_idx;
            self.next_vacant_idx = match self.entries[idx] {
                Entry::VacantHead { vacant_body_len } => {
                    if vacant_body_len > 0 {
                        self.entries[idx + 1] = Entry::VacantHead {
                            vacant_body_len: vacant_body_len - 1,
                        };
                    }
                    idx + 1
                }
                Entry::VacantTail { next_vacant_idx } => next_vacant_idx,
                Entry::Occupied(_) => unreachable!(),
            };
            self.entries[idx] = Entry::Occupied(f(idx));
            self.non_optimized_count = self.non_optimized_count.saturating_sub(1);
        } else {
            idx = self.entries.len();
            self.entries.push(Entry::Occupied(f(idx)));
        }
        self.len += 1;
        idx
    }

    /// Removes a key from the SlabMap, returning the value at the key if the key was previously in the SlabMap.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert("a");
    /// assert_eq!(s.remove(key), Some("a"));
    /// assert_eq!(s.remove(key), None);
    /// ```
    pub fn remove(&mut self, key: usize) -> Option<T> {
        let is_last = key + 1 == self.entries.len();
        let e = self.entries.get_mut(key)?;
        if !matches!(e, Entry::Occupied(..)) {
            return None;
        }
        self.len -= 1;
        let e = if is_last {
            self.entries.pop().unwrap()
        } else {
            let e = replace(
                e,
                Entry::VacantTail {
                    next_vacant_idx: self.next_vacant_idx,
                },
            );
            self.next_vacant_idx = key;
            self.non_optimized_count += 1;
            e
        };
        if self.is_empty() {
            self.clear();
        }
        if let Entry::Occupied(value) = e {
            Some(value)
        } else {
            unreachable!()
        }
    }

    /// Clears the SlabMap, removing all values and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// s.insert(1);
    /// s.insert(2);
    ///
    /// s.clear();
    ///
    /// assert_eq!(s.is_empty(), true);
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.len = 0;
        self.next_vacant_idx = INVALID_INDEX;
        self.non_optimized_count = 0;
    }

    /// Clears the SlabMap, returning all values as an iterator and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let k0 = s.insert(10);
    /// let k1 = s.insert(20);
    ///
    /// let d: Vec<_> = s.drain().collect();
    /// let mut e = vec![(k0, 10), (k1, 20)];
    /// e.sort();
    ///
    /// assert_eq!(s.is_empty(), true);
    /// assert_eq!(d, e);
    /// ```
    pub fn drain(&mut self) -> Drain<T> {
        let len = self.len;
        self.len = 0;
        self.next_vacant_idx = INVALID_INDEX;
        self.non_optimized_count = 0;
        Drain {
            iter: self.entries.drain(..).enumerate(),
            len,
        }
    }

    /// Retains only the elements specified by the predicate and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// s.insert(10);
    /// s.insert(15);
    /// s.insert(20);
    /// s.insert(25);
    ///
    /// s.retain(|_idx, value| *value % 2 == 0);
    ///
    /// let value: Vec<_> = s.values().cloned().collect();
    /// assert_eq!(value, vec![10, 20]);
    /// ```
    pub fn retain(&mut self, f: impl FnMut(usize, &mut T) -> bool) {
        self.rebuild_vacants_with(f)
    }
    pub(crate) fn rebuild_vacants(&mut self) {
        self.rebuild_vacants_with(|_, _| true);
    }
    fn rebuild_vacants_with(&mut self, mut f: impl FnMut(usize, &mut T) -> bool) {
        let mut idx = 0;
        let mut vacant_head_idx = 0;
        let mut prev_vacant_tail_idx = None;
        let mut len = 0;
        self.next_vacant_idx = INVALID_INDEX;
        while let Some(e) = self.entries.get_mut(idx) {
            match e {
                Entry::VacantTail { .. } => {
                    idx += 1;
                }
                Entry::VacantHead { vacant_body_len } => {
                    idx += *vacant_body_len + 2;
                }
                Entry::Occupied(value) => {
                    if f(idx, value) {
                        self.set_vacants(vacant_head_idx, idx, &mut prev_vacant_tail_idx);
                        idx += 1;
                        len += 1;
                        vacant_head_idx = idx;
                    } else {
                        self.entries[idx] = Entry::VacantTail {
                            next_vacant_idx: INVALID_INDEX,
                        };
                        idx += 1;
                    }
                }
            }
        }
        self.entries.truncate(vacant_head_idx);
        self.non_optimized_count = 0;
        self.len = len;
    }
    fn set_vacants(
        &mut self,
        vacant_head_idx: usize,
        vacant_end_idx: usize,
        prev_vacant_tail_idx: &mut Option<usize>,
    ) {
        if vacant_head_idx >= vacant_end_idx {
            return;
        }
        if self.next_vacant_idx == INVALID_INDEX {
            self.next_vacant_idx = vacant_head_idx;
        }
        if vacant_head_idx + 2 <= vacant_end_idx {
            self.entries[vacant_head_idx] = Entry::VacantHead {
                vacant_body_len: vacant_end_idx - (vacant_head_idx + 2),
            };
        }
        self.entries[vacant_end_idx - 1] = Entry::VacantTail {
            next_vacant_idx: INVALID_INDEX,
        };
        if let Some(prev_vacant_tail_idx) = *prev_vacant_tail_idx {
            self.entries[prev_vacant_tail_idx] = Entry::VacantTail {
                next_vacant_idx: vacant_head_idx,
            };
        }
        *prev_vacant_tail_idx = Some(vacant_end_idx - 1);
    }

    /// Optimizing the free space for speeding up iterations.
    ///
    /// If the free space has already been optimized, this method does nothing and completes with O(1).
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    /// use std::time::Instant;
    ///
    /// let mut s = SlabMap::new();
    /// const COUNT: usize = 1000000;
    /// for i in 0..COUNT {
    ///     s.insert(i);
    /// }
    /// let keys: Vec<_> = s.keys().take(COUNT - 1).collect();
    /// for key in keys {
    ///     s.remove(key);
    /// }
    ///
    /// s.optimize(); // if comment out this line, `s.values().sum()` to be slow.
    ///
    /// let begin = Instant::now();
    /// let sum: usize = s.values().sum();
    /// println!("sum : {}", sum);
    /// println!("duration : {} ms", (Instant::now() - begin).as_millis());
    /// ```
    pub fn optimize(&mut self) {
        if !self.is_optimized() {
            self.rebuild_vacants();
        }
    }

    #[inline]
    fn is_optimized(&self) -> bool {
        self.non_optimized_count == 0
    }

    /// Gets an iterator over the entries of the SlabMap, sorted by key.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.entries.iter().enumerate(),
            len: self.len,
        }
    }

    /// Gets a mutable iterator over the entries of the slab, sorted by key.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter: self.entries.iter_mut().enumerate(),
            len: self.len,
        }
    }

    /// Gets an iterator over the keys of the SlabMap, in sorted order.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn keys(&self) -> Keys<T> {
        Keys(self.iter())
    }

    /// Gets an iterator over the values of the SlabMap.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn values(&self) -> Values<T> {
        Values(self.iter())
    }

    /// Gets a mutable iterator over the values of the SlabMap.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<T> {
        ValuesMut(self.iter_mut())
    }
}
impl<T: Debug> Debug for SlabMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> std::ops::Index<usize> for SlabMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("out of index.")
    }
}
impl<T> std::ops::IndexMut<usize> for SlabMap<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("out of index.")
    }
}

impl<T> FromIterator<(usize, T)> for SlabMap<T> {
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(iter: I) -> Self {
        Self::from_iter_with_capacity(iter, 0)
    }
}

impl<T> IntoIterator for SlabMap<T> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.entries.into_iter().enumerate(),
            len: self.len,
        }
    }
}

impl<'a, T> IntoIterator for &'a SlabMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut SlabMap<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An owning iterator over the values of a [`SlabMap`].
///
/// This struct is created by the [`into_iter`](SlabMap::into_iter).
pub struct IntoIter<T> {
    iter: Enumerate<std::vec::IntoIter<Entry<T>>>,
    len: usize,
}
impl<T> Iterator for IntoIter<T> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e.1 {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some((e.0, value));
                }
                Entry::VacantHead { vacant_body_len } => self.iter.nth(vacant_body_len + 1),
                Entry::VacantTail { .. } => self.iter.next(),
            }
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<T> FusedIterator for IntoIter<T> {}
impl<T> ExactSizeIterator for IntoIter<T> {}

/// A draining iterator for [`SlabMap`].
///
/// This struct is created by the [`drain`](SlabMap::drain).
pub struct Drain<'a, T> {
    iter: Enumerate<std::vec::Drain<'a, Entry<T>>>,
    len: usize,
}
impl<'a, T> Iterator for Drain<'a, T> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (mut key, mut value) = self.iter.next()?;
        loop {
            (key, value) = match value {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some((key, value));
                }
                Entry::VacantHead { vacant_body_len } => self.iter.nth(vacant_body_len + 1)?,
                Entry::VacantTail { .. } => self.iter.next()?,
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for Drain<'a, T> {}
impl<'a, T> ExactSizeIterator for Drain<'a, T> {}

/// An iterator over the entries of a [`SlabMap`].
///
/// This struct is created by the [`iter`](SlabMap::iter).
pub struct Iter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, Entry<T>>>,
    len: usize,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (mut key, mut value) = self.iter.next()?;
        loop {
            (key, value) = match value {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some((key, value));
                }
                Entry::VacantHead { vacant_body_len } => self.iter.nth(*vacant_body_len + 1)?,
                Entry::VacantTail { .. } => self.iter.next()?,
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for Iter<'a, T> {}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

/// A mutable iterator over the entries of a [`SlabMap`].
///
/// This struct is created by the [`iter_mut`](SlabMap::iter_mut).
pub struct IterMut<'a, T> {
    iter: std::iter::Enumerate<std::slice::IterMut<'a, Entry<T>>>,
    len: usize,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (mut key, mut value) = self.iter.next()?;
        loop {
            (key, value) = match value {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some((key, value));
                }
                Entry::VacantHead { vacant_body_len } => self.iter.nth(*vacant_body_len + 1)?,
                Entry::VacantTail { .. } => self.iter.next()?,
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for IterMut<'a, T> {}
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

/// An iterator over the keys of a [`SlabMap`].
///
/// This struct is created by the [`keys`](SlabMap::keys).
pub struct Keys<'a, T>(Iter<'a, T>);
impl<'a, T> Iterator for Keys<'a, T> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for Keys<'a, T> {}
impl<'a, T> ExactSizeIterator for Keys<'a, T> {}

/// An iterator over the values of a [`SlabMap`]`.
///
/// This struct is created by the [`values`](SlabMap::values).
pub struct Values<'a, T>(Iter<'a, T>);
impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for Values<'a, T> {}
impl<'a, T> ExactSizeIterator for Values<'a, T> {}

/// A mutable iterator over the values of a [`SlabMap`].
///
/// This struct is created by the [`values_mut`](SlabMap::values_mut).
pub struct ValuesMut<'a, T>(IterMut<'a, T>);
impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for ValuesMut<'a, T> {}
impl<'a, T> ExactSizeIterator for ValuesMut<'a, T> {}

pub mod small_slab_map//! A variant of [`SlabMap`] that avoids heap allocation when the number of elements is small.

use std::{
    array::{self, from_fn},
    collections::TryReserveError,
    fmt::Debug,
    iter::{self, FusedIterator},
    mem,
    result::Result,
    slice,
};

use derive_ex::derive_ex;

use crate::SlabMap;

#[cfg(test)]
mod testsuse std::time::Instant;

use crate::SmallSlabMap;

#[test]
fn test_new() {
    let s = SmallSlabMap::<u32, 1>::new();
    assert_eq!(s.len(), 0);
}

#[test]
fn test_with_capacity() {
    for cap in 0..100 {
        let s = SmallSlabMap::<u32, 1>::with_capacity(cap);
        assert!(s.capacity() >= cap);
    }
}

#[test]
fn test_retain() {
    let mut s = SmallSlabMap::<_, 1>::new();
    s.insert(10);
    s.insert(15);
    s.insert(20);
    s.insert(25);

    s.retain(|_idx, x| *x % 2 == 0);

    let value: Vec<_> = s.values().cloned().collect();
    assert_eq!(value, vec![10, 20]);
    assert_eq!(s.len(), 2);
}

#[test]
fn test_len() {
    let mut s = SmallSlabMap::<_, 1>::new();
    assert_eq!(s.len(), 0);

    let key1 = s.insert(10);
    let key2 = s.insert(15);

    assert_eq!(s.len(), 2);

    s.remove(key1);
    assert_eq!(s.len(), 1);

    s.remove(key2);
    assert_eq!(s.len(), 0);
}

#[test]
fn test_is_empty() {
    let mut s = SmallSlabMap::<_, 1>::new();
    assert!(s.is_empty());

    let key = s.insert("a");
    assert!(!s.is_empty());

    s.remove(key);
    assert!(s.is_empty());
}

#[test]
fn test_get() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let key = s.insert(100);

    assert_eq!(s.get(key), Some(&100));
    assert_eq!(s.get(key + 1), None);
}

#[test]
fn test_contains_key() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let key = s.insert(100);

    assert!(s.contains_key(key));
    assert!(!s.contains_key(key + 1));
}

#[test]
fn test_insert() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let key_abc = s.insert("abc");
    let key_xyz = s.insert("xyz");

    assert_eq!(s[key_abc], "abc");
    assert_eq!(s[key_xyz], "xyz");
}

#[test]
fn test_insert_with_key() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let key = s.insert_with_key(|key| format!("my key is {}", key));

    assert_eq!(s[key], format!("my key is {}", key));
}

#[test]
fn test_remove() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let key = s.insert("a");
    assert_eq!(s.remove(key), Some("a"));
    assert_eq!(s.remove(key), None);
}

#[test]
fn test_clear() {
    let mut s = SmallSlabMap::<_, 1>::new();
    s.insert(1);
    s.insert(2);

    s.clear();

    assert!(s.is_empty());
}

#[test]
fn test_drain() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let k0 = s.insert(10);
    let k1 = s.insert(20);

    let d: Vec<_> = s.drain().collect();
    let mut e = vec![(k0, 10), (k1, 20)];
    e.sort();

    assert!(s.is_empty());
    assert_eq!(d, e);
}

#[test]
fn test_optimize() {
    let mut s = SmallSlabMap::<_, 1>::new();
    const COUNT: usize = 1000000;
    for i in 0..COUNT {
        s.insert(i);
    }
    let keys: Vec<_> = s.keys().take(COUNT - 1).collect();
    for key in keys {
        s.remove(key);
    }

    s.optimize(); // if comment out this line, `s.values().sum()` to be slow.

    let begin = Instant::now();
    let sum: usize = s.values().sum();
    println!("sum : {}", sum);
    println!("duration : {} ms", (Instant::now() - begin).as_millis());
}

#[test]
fn insert_remove_capacity() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let mut keys = Vec::new();
    for _ in 0..10 {
        s.insert(11);
    }
    for _ in 0..100 {
        keys.push(s.insert(10));
    }
    let capacity = s.capacity();
    for _ in 0..1000 {
        for key in keys.drain(..) {
            s.remove(key);
        }
        for _ in 0..100 {
            keys.push(s.insert(10));
        }
    }
    assert_eq!(capacity, s.capacity());
}

#[test]
fn insert_remove_capacity_all() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let mut keys = Vec::new();
    for _ in 0..100 {
        keys.push(s.insert(10));
    }
    let capacity = s.capacity();
    for _ in 0..1000 {
        for key in keys.drain(..) {
            s.remove(key);
        }
        for _ in 0..100 {
            keys.push(s.insert(10));
        }
    }
    assert_eq!(capacity, s.capacity());
}

#[test]
fn into_iter() {
    let mut s = SmallSlabMap::<_, 1>::new();
    let k0 = s.insert(0);
    let k1 = s.insert(1);
    let k2 = s.insert(2);
    s.remove(k1);

    let a: Vec<_> = s.into_iter().collect();
    let mut e = vec![(k0, 0), (k2, 2)];
    e.sort();

    assert_eq!(a, e);
}

#[test]
fn clone_from() {
    let mut s0 = SmallSlabMap::<_, 1>::new();
    let mut s1 = SmallSlabMap::<_, 1>::new();
    for _ in 0..10 {
        s0.insert(0);
    }
    for _ in 0..1000 {
        s1.insert(0);
    }
    let cap_old = s1.capacity();
    s1.clone_from(&s0);
    let cap_new = s1.capacity();
    assert_eq!(cap_old, cap_new);
}

#[test]
fn from_iter() {
    let s: SmallSlabMap<usize, 1> = [(5, 1), (0, 3)].into_iter().collect();
    assert_eq!(s.len(), 2, "len");
    assert_eq!(s[5], 1);
    assert_eq!(s[0], 3);
}

#[test]
fn merge_vacant() {
    let mut s: SmallSlabMap<_, 1> = [(0, 10), (1, 11), (2, 12), (3, 13)].into_iter().collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    let e = vec![(0, 10), (3, 13)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_insert() {
    let mut s: SmallSlabMap<_, 1> = [(0, 10), (1, 11), (2, 12), (3, 13)].into_iter().collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    let key = s.insert(99);
    let e = vec![(0, 10), (key, 99), (3, 13)];
    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_insert_2() {
    let mut s: SmallSlabMap<_, 1> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(3);
    s.optimize();
    let key = s.insert(99);
    let e = vec![(0, 10), (key, 99), (4, 14)];
    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_2time() {
    let mut s: SmallSlabMap<_, 1> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14), (5, 15)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    s.remove(4);
    s.optimize();

    let e = vec![(0, 10), (3, 13), (5, 15)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_2part() {
    let mut s: SmallSlabMap<_, 1> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(4);
    s.optimize();
    let e = vec![(0, 10), (3, 13)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_drain() {
    let mut s: SmallSlabMap<_, 1> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(3);
    s.optimize();

    let e = vec![(0, 10), (4, 14)];
    let a: Vec<_> = s.drain().collect();
    assert_eq!(a, e);
}

#[test]
fn reserve() {
    let mut s: SmallSlabMap<u32, 1> = SmallSlabMap::new();
    s.reserve(10);
    assert!(s.capacity() >= 10);
}

#[test]
fn reserve_exact() {
    let mut s: SmallSlabMap<u32, 1> = SmallSlabMap::new();
    s.reserve_exact(10);
    assert!(s.capacity() == 10);
}


#[derive(Clone)]
enum Data<T, const N: usize> {
    Inline { len: u8, items: [Option<T>; N] },
    Heap(SlabMap<T>),
}

/// A variant of [`SlabMap`] that avoids heap allocation when the number of elements is small.
///
/// If the number of elements is less than or equal to the generic parameter `N`,
/// heap allocation is not performed and data is stored in an inline array.
///
/// It is recommended that `N` be equal to or less than 16.
/// Larger values may result in inefficient operation.
///
/// # Examples
///
/// ```
/// use slabmap::SmallSlabMap;
///
/// let mut s = SmallSlabMap::<_, 4>::new();
/// let key_a = s.insert("aaa");
/// let key_b = s.insert("bbb");
///
/// assert_eq!(s[key_a], "aaa");
/// assert_eq!(s[key_b], "bbb");
///
/// for (key, value) in &s {
///     println!("{} -> {}", key, value);
/// }
///
/// assert_eq!(s.remove(key_a), Some("aaa"));
/// assert_eq!(s.remove(key_a), None);
/// ```
#[derive_ex(Default(bound()))]
#[default(Self::new())]
pub struct SmallSlabMap<T, const N: usize>(Option<Data<T, N>>);

impl<T, const N: usize> SmallSlabMap<T, N> {
    const INLINE_CAPACITY: usize = {
        let value = N;
        let value_max = u8::MAX as usize;
        if value <= value_max {
            value
        } else {
            value_max
        }
    };
    /// Constructs a new, empty `SmallSlabMap<T, N>`.
    /// The SmallSlabMap will not allocate until elements are pushed onto it.
    #[inline]
    pub const fn new() -> Self {
        Self(None)
    }

    /// Constructs a new, empty `SmallSlabMap<T, N>` with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= Self::INLINE_CAPACITY {
            Self::new()
        } else {
            Self(Some(Data::Heap(SlabMap::with_capacity(capacity))))
        }
    }

    /// Constructs as new `SmallSlabMap<T>` from keys and values with at least the specified capacity.
    pub fn from_iter_with_capacity(
        iter: impl IntoIterator<Item = (usize, T)>,
        capacity: usize,
    ) -> Self {
        let mut this = Self::with_capacity(capacity);
        for (key, value) in iter {
            this.set(key, value);
        }
        this.rebuild_vacants();
        this
    }
    fn set(&mut self, key: usize, value: T) {
        if key >= Self::INLINE_CAPACITY {
            self.as_heap();
        }
        match self.as_data() {
            Data::Inline { len, items } => {
                if items[key].is_none() {
                    *len += 1;
                }
                items[key] = Some(value);
            }
            Data::Heap(m) => m.set(key, value),
        }
    }
    fn rebuild_vacants(&mut self) {
        match self.as_data() {
            Data::Inline { len, items } => {
                *len = items.iter().filter(|x| x.is_some()).count() as u8
            }
            Data::Heap(m) => m.rebuild_vacants(),
        }
    }

    /// Returns the number of elements the SmallSlabMap can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        match &self.0 {
            None | Some(Data::Inline { .. }) => Self::INLINE_CAPACITY,
            Some(Data::Heap(m)) => m.capacity(),
        }
    }

    /// Reserves capacity for at least additional more elements to be inserted in the given `SmallSlabMap<T, N>`.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.    
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional).unwrap()
    }

    /// Try to reserve capacity for at least additional more elements to be inserted in the given `SmallSlabMap<T, N>`.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if !self.is_heap() && self.len() + additional <= Self::INLINE_CAPACITY {
            Ok(())
        } else {
            self.as_heap().try_reserve(additional)
        }
    }

    /// Reserves the minimum capacity for exactly additional more elements to be inserted in the given `SmallSlabMap<T, N>`.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.    
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.try_reserve_exact(additional).unwrap()
    }

    /// Try to reserve the minimum capacity for exactly additional more elements to be inserted in the given `SmallSlabMap<T, N>`.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if !self.is_heap() && self.len() + additional <= Self::INLINE_CAPACITY {
            Ok(())
        } else {
            self.as_heap().try_reserve_exact(additional)
        }
    }

    /// Returns the number of elements in the SmallSlabMap.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// assert_eq!(s.len(), 0);
    ///
    /// let key1 = s.insert(10);
    /// let key2 = s.insert(15);
    ///
    /// assert_eq!(s.len(), 2);
    ///
    /// s.remove(key1);
    /// assert_eq!(s.len(), 1);
    ///
    /// s.remove(key2);
    /// assert_eq!(s.len(), 0);
    /// ```    
    #[inline]
    pub fn len(&self) -> usize {
        match &self.0 {
            None => 0,
            Some(Data::Inline { len, .. }) => *len as usize,
            Some(Data::Heap(m)) => m.len(),
        }
    }

    /// Returns true if the SmallSlabMap contains no elements.
    ///    
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// assert_eq!(s.is_empty(), true);
    ///
    /// let key = s.insert("a");
    /// assert_eq!(s.is_empty(), false);
    ///
    /// s.remove(key);
    /// assert_eq!(s.is_empty(), true);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// let key = s.insert(100);
    ///
    /// assert_eq!(s.get(key), Some(&100));
    /// assert_eq!(s.get(key + 1), None);
    /// ```
    #[inline]
    pub fn get(&self, key: usize) -> Option<&T> {
        match self.0.as_ref()? {
            Data::Inline { items, .. } => items.get(key)?.as_ref(),
            Data::Heap(m) => m.get(key),
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        match self.as_data() {
            Data::Inline { items, .. } => items.get_mut(key)?.as_mut(),
            Data::Heap(m) => m.get_mut(key),
        }
    }

    /// Returns true if the SmallSlabMap contains a value for the specified key.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// let key = s.insert(100);
    ///
    /// assert_eq!(s.contains_key(key), true);
    /// assert_eq!(s.contains_key(key + 1), false);
    /// ```
    #[inline]
    pub fn contains_key(&self, key: usize) -> bool {
        self.get(key).is_some()
    }

    /// Inserts a value into the SmallSlabMap.
    ///
    /// Returns the key associated with the value.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// let key_abc = s.insert("abc");
    /// let key_xyz = s.insert("xyz");
    ///
    /// assert_eq!(s[key_abc], "abc");
    /// assert_eq!(s[key_xyz], "xyz");
    /// ```
    pub fn insert(&mut self, value: T) -> usize {
        self.insert_with_key(|_| value)
    }

    /// Inserts a value given by `f` into the SmallSlabMap. The key to be associated with the value is passed to `f`.
    ///
    /// Returns the key associated with the value.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// let key = s.insert_with_key(|key| format!("my key is {}", key));
    ///
    /// assert_eq!(s[key], format!("my key is {}", key));
    /// ```
    #[inline]
    pub fn insert_with_key(&mut self, f: impl FnOnce(usize) -> T) -> usize {
        self.reserve(1);
        match self.as_data() {
            Data::Inline { len, items } => {
                let index = items.iter().position(|x| x.is_none()).unwrap();
                items[index] = Some(f(index));
                *len += 1;
                index
            }
            Data::Heap(m) => m.insert_with_key(f),
        }
    }

    /// Removes a key from the SmallSlabMap, returning the value at the key if the key was previously in the SmallSlabMap.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// let key = s.insert("a");
    /// assert_eq!(s.remove(key), Some("a"));
    /// assert_eq!(s.remove(key), None);
    /// ```
    pub fn remove(&mut self, key: usize) -> Option<T> {
        match self.as_data() {
            Data::Inline { items, len } => {
                let ret = items.get_mut(key)?.take();
                if ret.is_some() {
                    *len -= 1;
                }
                ret
            }
            Data::Heap(m) => m.remove(key),
        }
    }

    /// Clears the SmallSlabMap, removing all values and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// s.insert(1);
    /// s.insert(2);
    ///
    /// s.clear();
    ///
    /// assert_eq!(s.is_empty(), true);
    /// ```
    pub fn clear(&mut self) {
        match &mut self.as_data() {
            Data::Inline { len, items } => {
                *len = 0;
                *items = from_fn(|_| None);
            }
            Data::Heap(m) => m.clear(),
        }
    }

    /// Clears the SmallSlabMap, returning all values as an iterator and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// let k0 = s.insert(10);
    /// let k1 = s.insert(20);
    ///
    /// let d: Vec<_> = s.drain().collect();
    /// let mut e = vec![(k0, 10), (k1, 20)];
    /// e.sort();
    ///
    /// assert_eq!(s.is_empty(), true);
    /// assert_eq!(d, e);
    /// ```
    pub fn drain(&mut self) -> Drain<T, N> {
        match self.as_data() {
            Data::Inline { len, items } => {
                let len = mem::take(len);
                let items = mem::replace(items, from_fn(|_| None));
                return Drain(RawDrain::Inline {
                    iter: items.into_iter().enumerate(),
                    len: len as usize,
                });
            }
            Data::Heap(m) => Drain(RawDrain::Heap(m.drain())),
        }
    }

    /// Retains only the elements specified by the predicate and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SmallSlabMap;
    ///
    /// let mut s = SmallSlabMap::<_, 4>::new();
    /// s.insert(10);
    /// s.insert(15);
    /// s.insert(20);
    /// s.insert(25);
    ///
    /// s.retain(|_idx, value| *value % 2 == 0);
    ///
    /// let value: Vec<_> = s.values().cloned().collect();
    /// assert_eq!(value, vec![10, 20]);
    /// ```
    pub fn retain(&mut self, mut f: impl FnMut(usize, &mut T) -> bool) {
        match self.as_data() {
            Data::Inline { items, len } => {
                let mut len_new = 0;
                for item in items {
                    if let Some(value) = item {
                        if f(len_new, value) {
                            len_new += 1;
                        } else {
                            *item = None;
                        }
                    }
                }
                *len = len_new as u8;
            }
            Data::Heap(m) => m.retain(f),
        }
    }

    /// Optimizing the free space for speeding up iterations.
    ///
    /// If the free space has already been optimized, this method does nothing and completes with O(1).
    pub fn optimize(&mut self) {
        match &mut self.0 {
            None | Some(Data::Inline { .. }) => {}
            Some(Data::Heap(m)) => m.optimize(),
        }
    }

    /// Gets an iterator over the entries of the SmallSlabMap, sorted by key.
    ///
    /// If you make a large number of [`remove`](SmallSlabMap::remove) calls, [`optimize`](SmallSlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn iter(&self) -> Iter<T, N> {
        self.into_iter()
    }

    /// Gets a mutable iterator over the entries of the SmallSlabMap, sorted by key.
    ///
    /// If you make a large number of [`remove`](SmallSlabMap::remove) calls, [`optimize`](SmallSlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T, N> {
        self.into_iter()
    }

    /// Gets an iterator over the keys of the SmallSlabMap, in sorted order.
    ///
    /// If you make a large number of [`remove`](SmallSlabMap::remove) calls, [`optimize`](SmallSlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn keys(&self) -> Keys<T, N> {
        Keys(self.iter())
    }

    /// Gets an iterator over the values of the SmallSlabMap.
    ///
    /// If you make a large number of [`remove`](SmallSlabMap::remove) calls, [`optimize`](SmallSlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn values(&self) -> Values<T, N> {
        Values(self.iter())
    }

    /// Gets a mutable iterator over the values of the SmallSlabMap.
    ///
    /// If you make a large number of [`remove`](SmallSlabMap::remove) calls, [`optimize`](SmallSlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<T, N> {
        ValuesMut(self.iter_mut())
    }

    fn is_heap(&self) -> bool {
        matches!(self.0, Some(Data::Heap(_)))
    }
    fn as_data(&mut self) -> &mut Data<T, N> {
        if self.0.is_none() {
            self.0 = Some(Data::Inline {
                len: 0,
                items: from_fn(|_| None),
            });
        }
        self.0.as_mut().unwrap()
    }
    fn as_heap(&mut self) -> &mut SlabMap<T> {
        if !self.is_heap() {
            self.0 = Some(Data::Heap(
                mem::take(self).into_iter().collect::<SlabMap<T>>(),
            ));
        }
        if let Some(Data::Heap(m)) = &mut self.0 {
            m
        } else {
            unreachable!()
        }
    }
}

impl<T: Debug, const N: usize> Debug for SmallSlabMap<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T, const N: usize> Clone for SmallSlabMap<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.clear();
        self.reserve(source.keys().map(|x| x + 1).max().unwrap_or(0));
        for (key, value) in source {
            self.set(key, value.clone());
        }
        self.rebuild_vacants();
    }
}

impl<T, const N: usize> std::ops::Index<usize> for SmallSlabMap<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("out of index.")
    }
}
impl<T, const N: usize> std::ops::IndexMut<usize> for SmallSlabMap<T, N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("out of index.")
    }
}

impl<T, const N: usize> FromIterator<(usize, T)> for SmallSlabMap<T, N> {
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(iter: I) -> Self {
        Self::from_iter_with_capacity(iter, 0)
    }
}

enum RawIntoIter<T, const N: usize> {
    Inline {
        iter: iter::Enumerate<array::IntoIter<Option<T>, N>>,
        len: usize,
    },
    Heap(crate::slab_map::IntoIter<T>),
}

/// An owning iterator over the values of a [`SmallSlabMap`].
///
/// This struct is created by the [`into_iter`](SmallSlabMap::into_iter).
pub struct IntoIter<T, const N: usize>(RawIntoIter<T, N>);

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            RawIntoIter::Inline { iter, len } => {
                if *len == 0 {
                    return None;
                }
                *len -= 1;
                loop {
                    if let (key, Some(value)) = iter.next().unwrap() {
                        return Some((key, value));
                    }
                }
            }
            RawIntoIter::Heap(iter) => iter.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.len()
    }
}
impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    fn len(&self) -> usize {
        match &self.0 {
            RawIntoIter::Inline { len, .. } => *len,
            RawIntoIter::Heap(iter) => iter.len(),
        }
    }
}

impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T, const N: usize> IntoIterator for SmallSlabMap<T, N> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T, N>;
    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            None => IntoIter(RawIntoIter::Inline {
                iter: from_fn(|_| None).into_iter().enumerate(),
                len: 0,
            }),
            Some(Data::Inline { len, items }) => IntoIter(RawIntoIter::Inline {
                iter: items.into_iter().enumerate(),
                len: len as usize,
            }),
            Some(Data::Heap(m)) => IntoIter(RawIntoIter::Heap(m.into_iter())),
        }
    }
}

enum RawDrain<'a, T, const N: usize> {
    Inline {
        iter: iter::Enumerate<array::IntoIter<Option<T>, N>>,
        len: usize,
    },
    Heap(crate::slab_map::Drain<'a, T>),
}

/// A draining iterator for [`SmallSlabMap`].
///
/// This struct is created by the [`drain`](SmallSlabMap::drain).
pub struct Drain<'a, T, const N: usize>(RawDrain<'a, T, N>);

impl<'a, T, const N: usize> Iterator for Drain<'a, T, N> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            RawDrain::Inline { iter, len } => {
                if *len == 0 {
                    return None;
                }
                *len -= 1;
                loop {
                    let (key, value) = iter.next().unwrap();
                    if let Some(value) = value {
                        return Some((key, value));
                    }
                }
            }
            RawDrain::Heap(iter) => iter.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.len()
    }
}
impl<'a, T, const N: usize> ExactSizeIterator for Drain<'a, T, N> {
    fn len(&self) -> usize {
        match &self.0 {
            RawDrain::Inline { len, .. } => *len,
            RawDrain::Heap(iter) => iter.len(),
        }
    }
}
impl<'a, T, const N: usize> FusedIterator for Drain<'a, T, N> {}

enum RawIter<'a, T, const N: usize> {
    Inline {
        iter: iter::Enumerate<slice::Iter<'a, Option<T>>>,
        len: usize,
    },
    Heap(crate::slab_map::Iter<'a, T>),
}

/// An iterator over the entries of a [`SmallSlabMap`].
///
/// This struct is created by the [`iter`](SmallSlabMap::iter).
pub struct Iter<'a, T, const N: usize>(RawIter<'a, T, N>);

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            RawIter::Inline { iter, len } => {
                if *len == 0 {
                    return None;
                }
                *len -= 1;
                loop {
                    if let (key, Some(value)) = iter.next().unwrap() {
                        return Some((key, value));
                    }
                }
            }
            RawIter::Heap(iter) => iter.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.len()
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for Iter<'a, T, N> {
    fn len(&self) -> usize {
        match &self.0 {
            RawIter::Inline { len, .. } => *len,
            RawIter::Heap(iter) => iter.len(),
        }
    }
}
impl<'a, T, const N: usize> FusedIterator for Iter<'a, T, N> {}

impl<'a, T, const N: usize> IntoIterator for &'a SmallSlabMap<T, N> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T, N>;
    fn into_iter(self) -> Self::IntoIter {
        match &self.0 {
            None => Iter(RawIter::Inline {
                iter: [].iter().enumerate(),
                len: 0,
            }),
            Some(Data::Inline { len, items }) => Iter(RawIter::Inline {
                iter: items.iter().enumerate(),
                len: *len as usize,
            }),
            Some(Data::Heap(m)) => Iter(RawIter::Heap(m.iter())),
        }
    }
}

enum RawIterMut<'a, T, const N: usize> {
    Inline {
        iter: iter::Enumerate<slice::IterMut<'a, Option<T>>>,
        len: usize,
    },
    Heap(crate::slab_map::IterMut<'a, T>),
}

/// A mutable iterator over the entries of a [`SmallSlabMap`].
///
/// This struct is created by the [`iter_mut`](SmallSlabMap::iter_mut).
pub struct IterMut<'a, T, const N: usize>(RawIterMut<'a, T, N>);

impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            RawIterMut::Inline { iter, len } => {
                if *len == 0 {
                    return None;
                }
                *len -= 1;
                loop {
                    if let (key, Some(value)) = iter.next().unwrap() {
                        return Some((key, value));
                    }
                }
            }
            RawIterMut::Heap(iter) => iter.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.len()
    }
}

impl<'a, T, const N: usize> ExactSizeIterator for IterMut<'a, T, N> {
    fn len(&self) -> usize {
        match &self.0 {
            RawIterMut::Inline { len, .. } => *len,
            RawIterMut::Heap(iter) => iter.len(),
        }
    }
}
impl<'a, T, const N: usize> FusedIterator for IterMut<'a, T, N> {}

impl<'a, T, const N: usize> IntoIterator for &'a mut SmallSlabMap<T, N> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T, N>;
    fn into_iter(self) -> Self::IntoIter {
        match &mut self.0 {
            None => IterMut(RawIterMut::Inline {
                iter: [].iter_mut().enumerate(),
                len: 0,
            }),
            Some(Data::Inline { len, items }) => IterMut(RawIterMut::Inline {
                iter: items.iter_mut().enumerate(),
                len: *len as usize,
            }),
            Some(Data::Heap(m)) => IterMut(RawIterMut::Heap(m.iter_mut())),
        }
    }
}

/// An iterator over the keys of a [`SmallSlabMap`].
///
/// This struct is created by the [`keys`](SmallSlabMap::keys).
pub struct Keys<'a, T, const N: usize>(Iter<'a, T, N>);

impl<'a, T, const N: usize> Iterator for Keys<'a, T, N> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next()?.0)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize {
        self.0.count()
    }
}
impl<'a, T, const N: usize> ExactSizeIterator for Keys<'a, T, N> {}
impl<'a, T, const N: usize> FusedIterator for Keys<'a, T, N> {}

/// An iterator over the values of a [`SmallSlabMap`].
///
/// This struct is created by the [`values`](SmallSlabMap::values).
pub struct Values<'a, T, const N: usize>(Iter<'a, T, N>);
impl<'a, T, const N: usize> Iterator for Values<'a, T, N> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next()?.1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize {
        self.0.count()
    }
}
impl<'a, T, const N: usize> ExactSizeIterator for Values<'a, T, N> {}
impl<'a, T, const N: usize> FusedIterator for Values<'a, T, N> {}

/// A mutable iterator over the values of a [`SmallSlabMap`].
///
/// This struct is created by the [`values_mut`](SmallSlabMap::values_mut).
pub struct ValuesMut<'a, T, const N: usize>(IterMut<'a, T, N>);
impl<'a, T, const N: usize> Iterator for ValuesMut<'a, T, N> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next()?.1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize {
        self.0.count()
    }
}
impl<'a, T, const N: usize> ExactSizeIterator for ValuesMut<'a, T, N> {}
impl<'a, T, const N: usize> FusedIterator for ValuesMut<'a, T, N> {}


#[doc(inline)]
pub use slab_map::SlabMap;

#[doc(inline)]
pub use small_slab_map::SmallSlabMap;

