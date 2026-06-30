//! HermiT-style trie + hash index for tuples (`TupleIndex`).

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

const LOAD_FACTOR: f32 = 0.7;
const BUCKET_OFFSET: i32 = 1;

const TRIE_NODE_PARENT: usize = 0;
const TRIE_NODE_FIRST_CHILD: usize = 1;
const TRIE_NODE_TUPLE_INDEX: usize = 1;
const TRIE_NODE_PREVIOUS_SIBLING: usize = 2;
const TRIE_NODE_NEXT_SIBLING: usize = 3;
const TRIE_NODE_NEXT_ENTRY: usize = 4;
const TRIE_NODE_SIZE: usize = 5;
const TRIE_NODE_PAGE_SIZE: usize = 1024;

/// Index tuples by a permuted key sequence (HermiT `TupleIndex`).
pub struct TupleIndex<T: Hash + Eq + Clone> {
    indexing_sequence: Vec<usize>,
    trie_node_manager: TrieNodeManager<T>,
    root: i32,
    buckets: Vec<i32>,
    buckets_length_minus_one: i32,
    resize_threshold: i32,
    number_of_nodes: i32,
}

impl<T: Hash + Eq + Clone> TupleIndex<T> {
    /// Create an index with the given component order.
    #[must_use]
    pub fn new(indexing_sequence: &[usize]) -> Self {
        let mut index = Self {
            indexing_sequence: indexing_sequence.to_vec(),
            trie_node_manager: TrieNodeManager::new(),
            root: 0,
            buckets: Vec::new(),
            buckets_length_minus_one: 0,
            resize_threshold: 0,
            number_of_nodes: 0,
        };
        index.clear();
        index
    }

    /// Remove all tuples.
    pub fn clear(&mut self) {
        self.trie_node_manager.clear();
        self.root = self.trie_node_manager.new_trie_node();
        self.trie_node_manager.initialize_trie_node(
            self.root,
            -1,
            -1,
            -1,
            -1,
            -1,
            None,
        );
        self.buckets = vec![0; 16];
        self.buckets_length_minus_one = (self.buckets.len() as i32) - 1;
        self.resize_threshold = (self.buckets.len() as f32 * LOAD_FACTOR) as i32;
        self.number_of_nodes = 0;
    }

    /// Insert or refresh a tuple; returns the stored tuple index.
    pub fn add_tuple(&mut self, tuple: &[T], potential_tuple_index: i32) -> i32 {
        let seq = self.indexing_sequence.clone();
        let mut trie_node = self.root;
        for &position in &seq {
            let object = &tuple[position];
            trie_node = self.get_child_node_add_if_necessary(trie_node, object);
        }
        if self
            .trie_node_manager
            .get_trie_node_component(trie_node, TRIE_NODE_TUPLE_INDEX)
            == -1
        {
            self.trie_node_manager
                .set_trie_node_component(trie_node, TRIE_NODE_TUPLE_INDEX, potential_tuple_index);
            potential_tuple_index
        } else {
            self.trie_node_manager
                .get_trie_node_component(trie_node, TRIE_NODE_TUPLE_INDEX)
        }
    }

    /// Lookup tuple index, or `-1`.
    #[must_use]
    pub fn get_tuple_index(&self, tuple: &[T]) -> i32 {
        let mut trie_node = self.root;
        for &position in &self.indexing_sequence {
            let object = &tuple[position];
            trie_node = self.get_child_node(trie_node, object);
            if trie_node == -1 {
                return -1;
            }
        }
        self.trie_node_manager
            .get_trie_node_component(trie_node, TRIE_NODE_TUPLE_INDEX)
    }

    /// Remove a tuple; returns its index or `-1`.
    pub fn remove_tuple(&mut self, tuple: &[T]) -> i32 {
        let seq = self.indexing_sequence.clone();
        let mut leaf = self.root;
        for &position in &seq {
            let object = &tuple[position];
            leaf = self.get_child_node(leaf, object);
            if leaf == -1 {
                return -1;
            }
        }
        let tuple_index = self
            .trie_node_manager
            .get_trie_node_component(leaf, TRIE_NODE_TUPLE_INDEX);
        let mut trie_node = self
            .trie_node_manager
            .get_trie_node_component(leaf, TRIE_NODE_PARENT);
        self.remove_trie_node(leaf);
        while trie_node != self.root
            && self
                .trie_node_manager
                .get_trie_node_component(trie_node, TRIE_NODE_FIRST_CHILD)
                == -1
        {
            let parent = self
                .trie_node_manager
                .get_trie_node_component(trie_node, TRIE_NODE_PARENT);
            self.remove_trie_node(trie_node);
            trie_node = parent;
        }
        tuple_index
    }

    fn remove_trie_node(&mut self, trie_node: i32) {
        let object = self
            .trie_node_manager
            .get_trie_node_object(trie_node)
            .expect("node object");
        let parent = self
            .trie_node_manager
            .get_trie_node_component(trie_node, TRIE_NODE_PARENT);
        let bucket_index =
            get_index_for(hash_key(&object, parent), self.buckets_length_minus_one) as usize;
        let mut child = self.buckets[bucket_index] - BUCKET_OFFSET;
        let mut previous_child = -1;
        while child != -1 {
            let next_child = self
                .trie_node_manager
                .get_trie_node_component(child, TRIE_NODE_NEXT_ENTRY);
            if child == trie_node {
                self.number_of_nodes -= 1;
                let previous_sibling = self.trie_node_manager.get_trie_node_component(
                    trie_node,
                    TRIE_NODE_PREVIOUS_SIBLING,
                );
                let next_sibling = self
                    .trie_node_manager
                    .get_trie_node_component(trie_node, TRIE_NODE_NEXT_SIBLING);
                if previous_sibling == -1 {
                    self.trie_node_manager
                        .set_trie_node_component(parent, TRIE_NODE_FIRST_CHILD, next_sibling);
                } else {
                    self.trie_node_manager.set_trie_node_component(
                        previous_sibling,
                        TRIE_NODE_NEXT_SIBLING,
                        next_sibling,
                    );
                }
                if next_sibling != -1 {
                    self.trie_node_manager.set_trie_node_component(
                        next_sibling,
                        TRIE_NODE_PREVIOUS_SIBLING,
                        previous_sibling,
                    );
                }
                if previous_child == -1 {
                    self.buckets[bucket_index] = next_child + BUCKET_OFFSET;
                } else {
                    self.trie_node_manager.set_trie_node_component(
                        previous_child,
                        TRIE_NODE_NEXT_ENTRY,
                        next_child,
                    );
                }
                self.trie_node_manager.delete_trie_node(trie_node);
                return;
            }
            previous_child = child;
            child = next_child;
        }
        panic!("Internal error: should be able to remove the child node.");
    }

    fn get_child_node(&self, parent: i32, object: &T) -> i32 {
        let bucket_index =
            get_index_for(hash_key(&object, parent), self.buckets_length_minus_one) as usize;
        let mut child = self.buckets[bucket_index] - BUCKET_OFFSET;
        while child != -1 {
            if parent
                == self
                    .trie_node_manager
                    .get_trie_node_component(child, TRIE_NODE_PARENT)
                && Some(object) == self.trie_node_manager.get_trie_node_object(child).as_ref()
            {
                return child;
            }
            child = self
                .trie_node_manager
                .get_trie_node_component(child, TRIE_NODE_NEXT_ENTRY);
        }
        -1
    }

    fn get_child_node_add_if_necessary(&mut self, parent: i32, object: &T) -> i32 {
        let hash_code = hash_key(&object, parent);
        let bucket_index = get_index_for(hash_code, self.buckets_length_minus_one) as usize;
        let mut child = self.buckets[bucket_index] - BUCKET_OFFSET;
        while child != -1 {
            if parent
                == self
                    .trie_node_manager
                    .get_trie_node_component(child, TRIE_NODE_PARENT)
                && Some(object) == self.trie_node_manager.get_trie_node_object(child).as_ref()
            {
                return child;
            }
            child = self
                .trie_node_manager
                .get_trie_node_component(child, TRIE_NODE_NEXT_ENTRY);
        }
        if self.number_of_nodes >= self.resize_threshold {
            self.resize_buckets();
            let bucket_index = get_index_for(hash_code, self.buckets_length_minus_one) as usize;
            return self.insert_child(parent, object, bucket_index);
        }
        self.insert_child(parent, object, bucket_index)
    }

    fn insert_child(&mut self, parent: i32, object: &T, bucket_index: usize) -> i32 {
        let child = self.trie_node_manager.new_trie_node();
        let next_sibling = self
            .trie_node_manager
            .get_trie_node_component(parent, TRIE_NODE_FIRST_CHILD);
        if next_sibling != -1 {
            self.trie_node_manager.set_trie_node_component(
                next_sibling,
                TRIE_NODE_PREVIOUS_SIBLING,
                child,
            );
        }
        self.trie_node_manager
            .set_trie_node_component(parent, TRIE_NODE_FIRST_CHILD, child);
        self.trie_node_manager.initialize_trie_node(
            child,
            parent,
            -1,
            -1,
            next_sibling,
            self.buckets[bucket_index] - BUCKET_OFFSET,
            Some(object.clone()),
        );
        self.buckets[bucket_index] = child + BUCKET_OFFSET;
        self.number_of_nodes += 1;
        child
    }

    fn resize_buckets(&mut self) {
        if self.buckets.len() == 0x4000_0000 {
            self.resize_threshold = i32::MAX;
            return;
        }
        let mut new_buckets = vec![0; self.buckets.len() * 2];
        let new_len_minus_one = (new_buckets.len() as i32) - 1;
        for bucket_index in (0..=self.buckets_length_minus_one).rev() {
            let mut trie_node = self.buckets[bucket_index as usize] - BUCKET_OFFSET;
            while trie_node != -1 {
                let next = self
                    .trie_node_manager
                    .get_trie_node_component(trie_node, TRIE_NODE_NEXT_ENTRY);
                let obj = self
                    .trie_node_manager
                    .get_trie_node_object(trie_node)
                    .expect("object");
                let parent = self
                    .trie_node_manager
                    .get_trie_node_component(trie_node, TRIE_NODE_PARENT);
                let new_bucket =
                    get_index_for(hash_key(&obj, parent), new_len_minus_one) as usize;
                self.trie_node_manager.set_trie_node_component(
                    trie_node,
                    TRIE_NODE_NEXT_ENTRY,
                    new_buckets[new_bucket] - BUCKET_OFFSET,
                );
                new_buckets[new_bucket] = trie_node + BUCKET_OFFSET;
                trie_node = next;
            }
        }
        self.buckets = new_buckets;
        self.buckets_length_minus_one = new_len_minus_one;
        self.resize_threshold = (self.buckets.len() as f32 * LOAD_FACTOR) as i32;
    }
}

fn hash_key<T: Hash>(object: &T, parent: i32) -> i32 {
    let mut hasher = DefaultHasher::new();
    object.hash(&mut hasher);
    (hasher.finish() as i32).wrapping_add(parent)
}

fn get_index_for(mut hash_code: i32, table_length_minus_one: i32) -> i32 {
    hash_code = hash_code.wrapping_add(!(hash_code << 9));
    hash_code ^= (hash_code as u32 >> 14) as i32;
    hash_code = hash_code.wrapping_add(hash_code << 4);
    hash_code ^= (hash_code as u32 >> 10) as i32;
    hash_code & table_length_minus_one
}

struct TrieNodeManager<T: Clone> {
    index_pages: Vec<Option<Vec<i32>>>,
    object_pages: Vec<Option<Vec<Option<T>>>>,
    first_free_trie_node: i32,
    number_of_pages: i32,
}

impl<T: Clone> TrieNodeManager<T> {
    fn new() -> Self {
        Self {
            index_pages: Vec::new(),
            object_pages: Vec::new(),
            first_free_trie_node: 0,
            number_of_pages: 0,
        }
    }

    fn clear(&mut self) {
        self.index_pages = vec![Some(vec![0; TRIE_NODE_SIZE * TRIE_NODE_PAGE_SIZE])];
        self.object_pages = vec![Some(vec![None; TRIE_NODE_PAGE_SIZE])];
        self.number_of_pages = 1;
        self.first_free_trie_node = 0;
        self.set_trie_node_component(self.first_free_trie_node, TRIE_NODE_NEXT_SIBLING, -1);
    }

    fn get_trie_node_component(&self, trie_node: i32, component: usize) -> i32 {
        let page = trie_node as usize / TRIE_NODE_PAGE_SIZE;
        let index = (trie_node as usize % TRIE_NODE_PAGE_SIZE) * TRIE_NODE_SIZE + component;
        self.index_pages[page].as_ref().unwrap()[index]
    }

    fn set_trie_node_component(&mut self, trie_node: i32, component: usize, value: i32) {
        let page = trie_node as usize / TRIE_NODE_PAGE_SIZE;
        let index = (trie_node as usize % TRIE_NODE_PAGE_SIZE) * TRIE_NODE_SIZE + component;
        self.index_pages[page].as_mut().unwrap()[index] = value;
    }

    fn get_trie_node_object(&self, trie_node: i32) -> Option<T> {
        let page = trie_node as usize / TRIE_NODE_PAGE_SIZE;
        let index = trie_node as usize % TRIE_NODE_PAGE_SIZE;
        self.object_pages[page].as_ref().unwrap()[index].clone()
    }

    fn initialize_trie_node(
        &mut self,
        trie_node: i32,
        parent: i32,
        first_child: i32,
        previous_sibling: i32,
        next_sibling: i32,
        next_entry: i32,
        object: Option<T>,
    ) {
        let page_index = trie_node as usize / TRIE_NODE_PAGE_SIZE;
        let index_in_page = trie_node as usize % TRIE_NODE_PAGE_SIZE;
        let start = index_in_page * TRIE_NODE_SIZE;
        {
            let index_page = self.index_pages[page_index].as_mut().unwrap();
            index_page[start + TRIE_NODE_PARENT] = parent;
            index_page[start + TRIE_NODE_FIRST_CHILD] = first_child;
            index_page[start + TRIE_NODE_PREVIOUS_SIBLING] = previous_sibling;
            index_page[start + TRIE_NODE_NEXT_SIBLING] = next_sibling;
            index_page[start + TRIE_NODE_NEXT_ENTRY] = next_entry;
        }
        self.object_pages[page_index].as_mut().unwrap()[index_in_page] = object;
    }

    fn new_trie_node(&mut self) -> i32 {
        let new_trie_node = self.first_free_trie_node;
        let next_free =
            self.get_trie_node_component(self.first_free_trie_node, TRIE_NODE_NEXT_SIBLING);
        if next_free != -1 {
            self.first_free_trie_node = next_free;
        } else {
            self.first_free_trie_node += 1;
            if self.first_free_trie_node < 0 {
                panic!("TupleIndex node space exhausted");
            }
            let page_index = self.first_free_trie_node as usize / TRIE_NODE_PAGE_SIZE;
            if page_index >= self.number_of_pages as usize {
                if page_index >= self.index_pages.len() {
                    let new_len = self.index_pages.len() * 3 / 2 + 1;
                    self.index_pages.resize(new_len, None);
                    self.object_pages.resize(new_len, None);
                }
                self.index_pages[page_index] =
                    Some(vec![0; TRIE_NODE_SIZE * TRIE_NODE_PAGE_SIZE]);
                self.object_pages[page_index] = Some(vec![None; TRIE_NODE_PAGE_SIZE]);
                self.number_of_pages += 1;
            }
            self.set_trie_node_component(self.first_free_trie_node, TRIE_NODE_NEXT_SIBLING, -1);
        }
        new_trie_node
    }

    fn delete_trie_node(&mut self, trie_node: i32) {
        self.set_trie_node_component(
            trie_node,
            TRIE_NODE_NEXT_SIBLING,
            self.first_free_trie_node,
        );
        let page = trie_node as usize / TRIE_NODE_PAGE_SIZE;
        let index = trie_node as usize % TRIE_NODE_PAGE_SIZE;
        self.object_pages[page].as_mut().unwrap()[index] = None;
        self.first_free_trie_node = trie_node;
    }
}

/// Prefix iteration over stored tuples (HermiT `TupleIndexRetrieval`).
pub struct TupleIndexRetrieval<'a, T: Hash + Eq + Clone> {
    tuple_index: &'a TupleIndex<T>,
    bindings_buffer: &'a [T],
    selection_indices: &'a [usize],
    selection_indices_length: usize,
    indexing_sequence_length: usize,
    current_trie_node: i32,
}

impl<'a, T: Hash + Eq + Clone> TupleIndexRetrieval<'a, T> {
    /// Bind retrieval to prefix values in `bindings_buffer`.
    #[must_use]
    pub fn new(
        tuple_index: &'a TupleIndex<T>,
        bindings_buffer: &'a [T],
        selection_indices: &'a [usize],
    ) -> Self {
        let selection_indices_length = selection_indices.len();
        let indexing_sequence_length = tuple_index.indexing_sequence.len();
        Self {
            tuple_index,
            bindings_buffer,
            selection_indices,
            selection_indices_length,
            indexing_sequence_length,
            current_trie_node: 0,
        }
    }

    /// Position at the first matching tuple (call before iteration).
    pub fn open(&mut self) {
        self.current_trie_node = self.tuple_index.root;
        for &position in self.selection_indices {
            let object = &self.bindings_buffer[position];
            self.current_trie_node = self.tuple_index.get_child_node(self.current_trie_node, object);
            if self.current_trie_node == -1 {
                return;
            }
        }
        if self.selection_indices_length == 0
            && self
                .tuple_index
                .trie_node_manager
                .get_trie_node_component(self.tuple_index.root, TRIE_NODE_FIRST_CHILD)
                == -1
        {
            self.current_trie_node = -1;
        } else {
            for _ in self.selection_indices_length..self.indexing_sequence_length {
                self.current_trie_node = self.tuple_index.trie_node_manager.get_trie_node_component(
                    self.current_trie_node,
                    TRIE_NODE_FIRST_CHILD,
                );
            }
        }
    }

    /// Whether iteration is finished.
    #[must_use]
    pub fn after_last(&self) -> bool {
        self.current_trie_node == -1
    }

    /// Tuple index at the current position.
    #[must_use]
    pub fn get_current_tuple_index(&self) -> i32 {
        self.tuple_index
            .trie_node_manager
            .get_trie_node_component(self.current_trie_node, TRIE_NODE_TUPLE_INDEX)
    }

    /// Advance to the next matching tuple.
    pub fn next(&mut self) {
        let mut trie_node_depth = self.indexing_sequence_length;
        while trie_node_depth != self.selection_indices_length
            && self.tuple_index.trie_node_manager.get_trie_node_component(
                self.current_trie_node,
                TRIE_NODE_NEXT_SIBLING,
            ) == -1
        {
            self.current_trie_node = self.tuple_index.trie_node_manager.get_trie_node_component(
                self.current_trie_node,
                TRIE_NODE_PARENT,
            );
            trie_node_depth -= 1;
        }
        if trie_node_depth == self.selection_indices_length {
            self.current_trie_node = -1;
        } else {
            self.current_trie_node = self.tuple_index.trie_node_manager.get_trie_node_component(
                self.current_trie_node,
                TRIE_NODE_NEXT_SIBLING,
            );
            for _ in trie_node_depth..self.indexing_sequence_length {
                self.current_trie_node = self.tuple_index.trie_node_manager.get_trie_node_component(
                    self.current_trie_node,
                    TRIE_NODE_FIRST_CHILD,
                );
            }
        }
    }
}

#[cfg(test)]
mod hermit_ports {
    use super::*;

    fn assert_tuple_retrieval(index: &TupleIndex<String>, selection: &[&str], expected: &[i32]) {
        let bindings: Vec<String> = selection.iter().map(|s| (*s).to_string()).collect();
        let selection_indices: Vec<usize> = (0..bindings.len()).collect();
        let mut retrieval = TupleIndexRetrieval::new(index, &bindings, &selection_indices);
        retrieval.open();
        let mut used = vec![false; expected.len()];
        while !retrieval.after_last() {
            let tuple_index = retrieval.get_current_tuple_index();
            let found = expected.iter().enumerate().any(|(i, &exp)| {
                if tuple_index == exp && !used[i] {
                    used[i] = true;
                    true
                } else {
                    false
                }
            });
            assert!(
                found,
                "tuple index {tuple_index} not found in expected set {expected:?}"
            );
            retrieval.next();
        }
        for (i, &u) in used.iter().enumerate() {
            assert!(
                u,
                "expected tuple index {} not found in retrieval",
                expected[i]
            );
        }
    }

    fn tuple3(a: &str, b: &str, c: &str) -> [String; 3] {
        [a.to_string(), b.to_string(), c.to_string()]
    }

    #[test]
    fn hermit_tuple_index_test_1() {
        let mut index = TupleIndex::new(&[0, 1, 2]);

        assert_tuple_retrieval(&index, &[], &[]);

        index.add_tuple(&tuple3("a", "b", "c"), 1);
        assert_tuple_retrieval(&index, &["a"], &[1]);
        assert_tuple_retrieval(&index, &["a", "b"], &[1]);

        index.add_tuple(&tuple3("a", "b", "d"), 2);
        assert_tuple_retrieval(&index, &["a"], &[1, 2]);
        assert_tuple_retrieval(&index, &["a", "b"], &[1, 2]);

        index.add_tuple(&tuple3("a", "b", "c"), 3);
        assert_tuple_retrieval(&index, &["a"], &[2, 1]);
        assert_tuple_retrieval(&index, &["a", "b"], &[2, 1]);
        assert_tuple_retrieval(&index, &["a", "b", "c"], &[1]);

        index.add_tuple(&tuple3("c", "b", "d"), 4);
        assert_tuple_retrieval(&index, &[], &[2, 1, 4]);
        assert_tuple_retrieval(&index, &["a"], &[2, 1]);
        assert_tuple_retrieval(&index, &["a", "b"], &[2, 1]);
        assert_tuple_retrieval(&index, &["a", "b", "c"], &[1]);
        assert_tuple_retrieval(&index, &["f"], &[]);

        index.remove_tuple(&tuple3("a", "b", "d"));
        assert_tuple_retrieval(&index, &[], &[1, 4]);

        index.remove_tuple(&tuple3("a", "b", "c"));
        assert_tuple_retrieval(&index, &[], &[4]);

        index.remove_tuple(&tuple3("c", "b", "d"));
        assert_tuple_retrieval(&index, &[], &[]);
    }

    #[test]
    fn hermit_tuple_index_test_2() {
        let mut index = TupleIndex::new(&[0, 1, 2]);
        let mut tuples: Vec<[String; 3]> = Vec::with_capacity(10_000);
        let mut tuple_indexes = Vec::with_capacity(10_000);

        for i in 0..10_000 {
            tuples.push([
                (i % 300).to_string(),
                (i % 3000).to_string(),
                i.to_string(),
            ]);
            tuple_indexes.push(i as i32);
        }

        for (i, tuple) in tuples.iter().enumerate() {
            index.add_tuple(tuple, i as i32);
        }
        assert_tuple_retrieval(&index, &[], &tuple_indexes);

        for (i, tuple) in tuples.iter().enumerate() {
            assert_eq!(index.remove_tuple(tuple), i as i32);
        }
        assert_tuple_retrieval(&index, &[], &[]);
    }
}
