//! HermiT `TupleTable` + `TupleTableFullIndex` for extension-table tuple storage.

use std::hash::{Hash, Hasher};

const PAGE_SIZE: usize = 512;
const BUCKET_OFFSET: i32 = 1;
const LOAD_FACTOR: f32 = 0.75;

const ENTRY_SIZE: usize = 3;
const ENTRY_NEXT: usize = 0;
const ENTRY_HASH_CODE: usize = 1;
const ENTRY_TUPLE_INDEX: usize = 2;
const ENTRY_PAGE_SIZE: usize = 512;

/// Paged tuple storage (HermiT `TupleTable`).
pub struct TupleTable<T: Clone + Eq> {
    arity: usize,
    pages: Vec<Option<Page<T>>>,
    number_of_pages: usize,
    tuple_capacity: usize,
    first_free_tuple_index: i32,
}

impl<T: Clone + Eq> TupleTable<T> {
    /// Create a table with fixed tuple arity.
    #[must_use]
    pub fn new(arity: usize) -> Self {
        let mut table = Self {
            arity,
            pages: Vec::new(),
            number_of_pages: 0,
            tuple_capacity: 0,
            first_free_tuple_index: 0,
        };
        table.clear();
        table
    }

    /// Next tuple index that would be assigned by [`Self::add_tuple`].
    #[must_use]
    pub fn first_free_tuple_index(&self) -> i32 {
        self.first_free_tuple_index
    }

    /// Append a tuple; returns its index.
    pub fn add_tuple(&mut self, tuple: &[T]) -> i32 {
        assert_eq!(tuple.len(), self.arity);
        let new_tuple_index = self.first_free_tuple_index;
        if new_tuple_index as usize == self.tuple_capacity {
            if self.number_of_pages == self.pages.len() {
                let new_len = self.number_of_pages * 3 / 2 + 1;
                self.pages.resize_with(new_len, || None);
            }
            self.pages[self.number_of_pages] = Some(Page::new(self.arity));
            self.number_of_pages += 1;
            self.tuple_capacity += PAGE_SIZE;
        }
        let page = new_tuple_index as usize / PAGE_SIZE;
        let offset = (new_tuple_index as usize % PAGE_SIZE) * self.arity;
        self.pages[page]
            .as_mut()
            .expect("page")
            .store_tuple(offset, tuple);
        self.first_free_tuple_index += 1;
        new_tuple_index
    }

    /// Compare the first `compare_length` components of `tuple` with stored tuple `tuple_index`.
    #[must_use]
    pub fn tuple_equals(&self, tuple: &[T], tuple_index: i32, compare_length: usize) -> bool {
        let page = tuple_index as usize / PAGE_SIZE;
        let offset = (tuple_index as usize % PAGE_SIZE) * self.arity;
        self.pages[page]
            .as_ref()
            .expect("page")
            .tuple_equals(tuple, offset, compare_length)
    }

    /// Compare via permuted positions (HermiT overload).
    #[must_use]
    pub fn tuple_equals_positions(
        &self,
        tuple: &[T],
        position_indexes: &[usize],
        tuple_index: i32,
        compare_length: usize,
    ) -> bool {
        let page = tuple_index as usize / PAGE_SIZE;
        let offset = (tuple_index as usize % PAGE_SIZE) * self.arity;
        self.pages[page]
            .as_ref()
            .expect("page")
            .tuple_equals_positions(tuple, position_indexes, offset, compare_length)
    }

    /// Component `object_index` of stored tuple `tuple_index`.
    #[must_use]
    pub fn get_tuple_object(&self, tuple_index: i32, object_index: usize) -> T {
        assert!(object_index < self.arity);
        let page = tuple_index as usize / PAGE_SIZE;
        let idx = (tuple_index as usize % PAGE_SIZE) * self.arity + object_index;
        self.pages[page].as_ref().expect("page").objects[idx]
            .clone()
            .expect("tuple object")
    }

    /// Copy all components of `tuple_index` into `buffer`.
    pub fn retrieve_tuple(&self, buffer: &mut [T], tuple_index: i32) {
        assert_eq!(buffer.len(), self.arity);
        for (i, slot) in buffer.iter_mut().enumerate() {
            *slot = self.get_tuple_object(tuple_index, i);
        }
    }

    /// Drop tuples at indices `>= new_first_free` (HermiT backtrack truncate).
    pub fn truncate(&mut self, new_first_free: i32) {
        assert!(new_first_free <= self.first_free_tuple_index);
        self.first_free_tuple_index = new_first_free;
    }

    fn clear(&mut self) {
        self.pages = (0..10).map(|_| Some(Page::new(self.arity))).collect();
        self.number_of_pages = 1;
        self.tuple_capacity = self.number_of_pages * PAGE_SIZE;
        self.first_free_tuple_index = 0;
    }
}

struct Page<T: Clone + Eq> {
    #[allow(dead_code)]
    arity: usize,
    objects: Vec<Option<T>>,
}

impl<T: Clone + Eq> Page<T> {
    fn new(arity: usize) -> Self {
        Self {
            arity,
            objects: vec![None; arity * PAGE_SIZE],
        }
    }

    fn store_tuple(&mut self, tuple_start_index: usize, tuple: &[T]) {
        for (i, value) in tuple.iter().enumerate() {
            self.objects[tuple_start_index + i] = Some(value.clone());
        }
    }

    fn tuple_equals(&self, tuple: &[T], tuple_start_index: usize, compare_length: usize) -> bool {
        for source_index in (0..compare_length).rev() {
            let stored = &self.objects[tuple_start_index + source_index];
            if stored.as_ref() != Some(&tuple[source_index]) {
                return false;
            }
        }
        true
    }

    fn tuple_equals_positions(
        &self,
        tuple: &[T],
        position_indexes: &[usize],
        tuple_start_index: usize,
        compare_length: usize,
    ) -> bool {
        for source_index in (0..compare_length).rev() {
            let stored = &self.objects[tuple_start_index + source_index];
            if stored.as_ref() != Some(&tuple[position_indexes[source_index]]) {
                return false;
            }
        }
        true
    }
}

/// Hash-index over the first `indexed_arity` components of tuples in a [`TupleTable`].
pub struct TupleTableFullIndex<T: Hash + Eq + Clone> {
    tuple_table: TupleTable<T>,
    indexed_arity: usize,
    entry_manager: EntryManager,
    buckets: Vec<i32>,
    resize_threshold: i32,
    number_of_tuples: i32,
}

impl<T: Hash + Eq + Clone> TupleTableFullIndex<T> {
    /// Create an index over `indexed_arity` leading components.
    #[must_use]
    pub fn new(arity: usize, indexed_arity: usize) -> Self {
        Self {
            tuple_table: TupleTable::new(arity),
            indexed_arity,
            entry_manager: EntryManager::new(),
            buckets: Vec::new(),
            resize_threshold: 0,
            number_of_tuples: 0,
        }
        .cleared()
    }

    fn cleared(mut self) -> Self {
        self.clear();
        self
    }

    /// Remove all index entries (tuple table is unchanged).
    pub fn clear(&mut self) {
        self.buckets = vec![0; 16];
        self.resize_threshold = (self.buckets.len() as f32 * LOAD_FACTOR) as i32;
        self.entry_manager.clear();
        self.number_of_tuples = 0;
    }

    /// Insert or lookup; returns the stored tuple index.
    pub fn add_tuple(&mut self, tuple: &[T], tentative_tuple_index: i32) -> i32 {
        let hash_code = tuple_hash(tuple, self.indexed_arity);
        let entry_index = bucket_index(hash_code, self.buckets.len());
        let mut entry = self.buckets[entry_index] - BUCKET_OFFSET;
        while entry != -1 {
            if hash_code
                == self
                    .entry_manager
                    .get_entry_component(entry, ENTRY_HASH_CODE)
            {
                let tuple_index = self
                    .entry_manager
                    .get_entry_component(entry, ENTRY_TUPLE_INDEX);
                if self
                    .tuple_table
                    .tuple_equals(tuple, tuple_index, self.indexed_arity)
                {
                    return tuple_index;
                }
            }
            entry = self.entry_manager.get_entry_component(entry, ENTRY_NEXT);
        }
        entry = self.entry_manager.new_entry();
        self.entry_manager.set_entry_component(
            entry,
            ENTRY_NEXT,
            self.buckets[entry_index] - BUCKET_OFFSET,
        );
        self.entry_manager
            .set_entry_component(entry, ENTRY_HASH_CODE, hash_code);
        self.entry_manager
            .set_entry_component(entry, ENTRY_TUPLE_INDEX, tentative_tuple_index);
        self.buckets[entry_index] = entry + BUCKET_OFFSET;
        self.number_of_tuples += 1;
        if self.number_of_tuples >= self.resize_threshold {
            self.resize_buckets();
        }
        tentative_tuple_index
    }

    /// Lookup tuple index, or `-1`.
    #[must_use]
    pub fn get_tuple_index(&self, tuple: &[T]) -> i32 {
        let hash_code = tuple_hash(tuple, self.indexed_arity);
        let entry_index = bucket_index(hash_code, self.buckets.len());
        let mut entry = self.buckets[entry_index] - BUCKET_OFFSET;
        while entry != -1 {
            if hash_code
                == self
                    .entry_manager
                    .get_entry_component(entry, ENTRY_HASH_CODE)
            {
                let tuple_index = self
                    .entry_manager
                    .get_entry_component(entry, ENTRY_TUPLE_INDEX);
                if self
                    .tuple_table
                    .tuple_equals(tuple, tuple_index, self.indexed_arity)
                {
                    return tuple_index;
                }
            }
            entry = self.entry_manager.get_entry_component(entry, ENTRY_NEXT);
        }
        -1
    }

    /// Remove index entry for `tuple_index`.
    pub fn remove_tuple(&mut self, tuple_index: i32) -> bool {
        let mut hash_code = 0i32;
        for i in 0..self.indexed_arity {
            hash_code = hash_code.wrapping_add(component_hash(
                &self.tuple_table.get_tuple_object(tuple_index, i),
            ));
        }
        let mut last_entry = -1;
        let entry_index = bucket_index(hash_code, self.buckets.len());
        let mut entry = self.buckets[entry_index] - BUCKET_OFFSET;
        while entry != -1 {
            let next_entry = self.entry_manager.get_entry_component(entry, ENTRY_NEXT);
            if hash_code
                == self
                    .entry_manager
                    .get_entry_component(entry, ENTRY_HASH_CODE)
                && tuple_index
                    == self
                        .entry_manager
                        .get_entry_component(entry, ENTRY_TUPLE_INDEX)
            {
                if last_entry == -1 {
                    self.buckets[entry_index] = next_entry + BUCKET_OFFSET;
                } else {
                    self.entry_manager
                        .set_entry_component(last_entry, ENTRY_NEXT, next_entry);
                }
                return true;
            }
            last_entry = entry;
            entry = next_entry;
        }
        false
    }

    /// Underlying tuple table.
    #[must_use]
    pub fn tuple_table(&self) -> &TupleTable<T> {
        &self.tuple_table
    }

    /// Mutable underlying tuple table.
    pub fn tuple_table_mut(&mut self) -> &mut TupleTable<T> {
        &mut self.tuple_table
    }

    fn resize_buckets(&mut self) {
        let mut new_buckets = vec![0; self.buckets.len() * 2];
        for bucket in (0..self.buckets.len()).rev() {
            let mut entry = self.buckets[bucket] - BUCKET_OFFSET;
            while entry != -1 {
                let next_entry = self.entry_manager.get_entry_component(entry, ENTRY_NEXT);
                let hash_code = self
                    .entry_manager
                    .get_entry_component(entry, ENTRY_HASH_CODE);
                let new_bucket_index = bucket_index(hash_code, new_buckets.len());
                self.entry_manager.set_entry_component(
                    entry,
                    ENTRY_NEXT,
                    new_buckets[new_bucket_index] - BUCKET_OFFSET,
                );
                new_buckets[new_bucket_index] = entry + BUCKET_OFFSET;
                entry = next_entry;
            }
        }
        self.buckets = new_buckets;
        self.resize_threshold = (self.buckets.len() as f32 * LOAD_FACTOR) as i32;
    }
}

fn component_hash<T: Hash>(value: &T) -> i32 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish() as i32
}

fn tuple_hash<T: Hash>(tuple: &[T], indexed_arity: usize) -> i32 {
    let mut hash_code = 0i32;
    for value in &tuple[..indexed_arity] {
        hash_code = hash_code.wrapping_add(component_hash(value));
    }
    hash_code
}

fn bucket_index(hash_code: i32, buckets_length: usize) -> usize {
    (hash_code as usize) & (buckets_length - 1)
}

struct EntryManager {
    entries: Vec<i32>,
    first_free_entry: i32,
}

impl EntryManager {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            first_free_entry: 0,
        }
    }

    fn clear(&mut self) {
        self.entries = vec![0; ENTRY_SIZE * ENTRY_PAGE_SIZE];
        self.first_free_entry = 0;
        self.entries[ENTRY_NEXT] = -1;
    }

    fn get_entry_component(&self, entry: i32, component: usize) -> i32 {
        self.entries[entry as usize + component]
    }

    fn set_entry_component(&mut self, entry: i32, component: usize, value: i32) {
        self.entries[entry as usize + component] = value;
    }

    fn new_entry(&mut self) -> i32 {
        let result = self.first_free_entry;
        let next_free = self.entries[self.first_free_entry as usize + ENTRY_NEXT];
        if next_free == -1 {
            self.first_free_entry += ENTRY_SIZE as i32;
            if self.first_free_entry as usize >= self.entries.len() {
                let mut new_entries = vec![0; self.entries.len() + ENTRY_SIZE * ENTRY_PAGE_SIZE];
                new_entries[..self.entries.len()].copy_from_slice(&self.entries);
                self.entries = new_entries;
            }
            self.entries[self.first_free_entry as usize + ENTRY_NEXT] = -1;
        } else {
            self.first_free_entry = next_free;
        }
        result
    }
}

#[cfg(test)]
mod hermit_ports {
    use super::*;

    struct IndexHarness {
        index: TupleTableFullIndex<String>,
    }

    impl IndexHarness {
        fn new(arity: usize, indexed_arity: usize) -> Self {
            Self {
                index: TupleTableFullIndex::new(arity, indexed_arity),
            }
        }

        fn add(&mut self, tuple: &[&str]) -> i32 {
            let owned: Vec<String> = tuple.iter().map(|s| (*s).to_string()).collect();
            let tentative = self.index.tuple_table().first_free_tuple_index();
            let result = self.index.add_tuple(&owned, tentative);
            if result == tentative {
                self.index.tuple_table_mut().add_tuple(&owned);
            }
            result
        }

        fn get(&self, a: &str, b: &str) -> i32 {
            self.index.get_tuple_index(&[a.to_string(), b.to_string()])
        }
    }

    #[test]
    fn hermit_tuple_table_full_index_test_1() {
        let mut h = IndexHarness::new(2, 2);

        assert_eq!(h.add(&["a", "b"]), 0);
        assert_eq!(h.add(&["b", "c"]), 1);
        assert_eq!(h.add(&["c", "d"]), 2);
        assert_eq!(h.add(&["a", "b"]), 0);

        assert_eq!(h.get("a", "b"), 0);
        assert_eq!(h.get("b", "c"), 1);
        assert_eq!(h.get("c", "d"), 2);

        assert!(h.index.remove_tuple(1));
        assert_eq!(h.get("a", "b"), 0);
        assert_eq!(h.get("b", "c"), -1);
        assert_eq!(h.get("c", "d"), 2);

        assert_eq!(h.add(&["e", "f"]), 3);
        assert_eq!(h.get("a", "b"), 0);
        assert_eq!(h.get("b", "c"), -1);
        assert_eq!(h.get("c", "d"), 2);
        assert_eq!(h.get("e", "f"), 3);

        assert_eq!(h.add(&["g", "h"]), 4);
        assert_eq!(h.get("a", "b"), 0);
        assert_eq!(h.get("b", "c"), -1);
        assert_eq!(h.get("c", "d"), 2);
        assert_eq!(h.get("e", "f"), 3);
        assert_eq!(h.get("g", "h"), 4);
    }

    #[test]
    fn hermit_tuple_table_full_index_test_2() {
        let mut h = IndexHarness::new(2, 2);
        let tuples: Vec<[String; 2]> = (0..40_000)
            .map(|index| [format!("a{index}"), format!("b{index}")])
            .collect();

        for (tuple_index, tuple) in tuples.iter().enumerate() {
            assert_eq!(
                h.index.add_tuple(tuple, tuple_index as i32),
                tuple_index as i32
            );
            h.index.tuple_table_mut().add_tuple(tuple);
        }

        for (tuple_index, tuple) in tuples.iter().enumerate() {
            assert_eq!(h.index.get_tuple_index(tuple), tuple_index as i32);
        }

        assert_eq!(h.get("e", "f"), -1);
    }
}
