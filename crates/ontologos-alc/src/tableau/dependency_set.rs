//! HermiT-style branching-point dependency sets (persistent trie + interning).
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// Reference to a permanent or union dependency set.
#[derive(Debug, Clone)]
pub enum DependencySetRef {
    /// Interned permanent set.
    Permanent(Rc<PermanentDependencySet>),
    /// Temporary union of several sets (merged via [`DependencySetFactory::get_permanent_union`]).
    Union(Rc<UnionDependencySet>),
}

/// Temporary union of dependency sets.
#[derive(Debug)]
pub struct UnionDependencySet {
    /// Constituent sets to merge.
    pub constituents: Vec<DependencySetRef>,
}

/// Persistent dependency set: a sorted linked list of branching points (head = largest).
#[derive(Debug)]
pub struct PermanentDependencySet {
    pub(crate) rest: Option<Rc<PermanentDependencySet>>,
    pub(crate) branching_point: i32,
    #[allow(dead_code)]
    next_entry: RefCell<Option<Weak<PermanentDependencySet>>>,
}

#[allow(dead_code)]
impl PermanentDependencySet {
    /// Whether this set is empty (no branching points).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.branching_point == -1
    }

    /// Branching points from largest to smallest.
    pub fn branching_points(&self) -> impl Iterator<Item = i32> + '_ {
        DependencySetIterator {
            current: Some(self),
        }
    }

    fn contains_branching_point(&self, bp: i32) -> bool {
        self.branching_points().any(|x| x == bp)
    }

    fn maximum_branching_point(&self) -> i32 {
        if self.is_empty() {
            -1
        } else {
            self.branching_point
        }
    }
}

struct DependencySetIterator<'a> {
    current: Option<&'a PermanentDependencySet>,
}

impl Iterator for DependencySetIterator<'_> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        if node.branching_point == -1 {
            return None;
        }
        let bp = node.branching_point;
        self.current = node.rest.as_deref();
        Some(bp)
    }
}

/// Factory for interned [`PermanentDependencySet`] values (HermiT `DependencySetFactory`).
#[derive(Debug)]
pub struct DependencySetFactory {
    empty_set: Rc<PermanentDependencySet>,
    entries: RefCell<HashMap<EntryKey, Rc<PermanentDependencySet>>>,
    merge_scratch: RefCell<Vec<i32>>,
    merge_sets: RefCell<Vec<Rc<PermanentDependencySet>>>,
    unprocessed_unions: RefCell<Vec<Rc<UnionDependencySet>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EntryKey {
    rest_ptr: usize,
    branching_point: i32,
}

impl Default for DependencySetFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencySetFactory {
    /// Create a new factory with a shared empty set.
    #[must_use]
    pub fn new() -> Self {
        let empty = Rc::new(PermanentDependencySet {
            rest: None,
            branching_point: -1,
            next_entry: RefCell::new(None),
        });
        Self {
            empty_set: empty,
            entries: RefCell::new(HashMap::with_capacity(16)),
            merge_scratch: RefCell::new(Vec::new()),
            merge_sets: RefCell::new(Vec::new()),
            unprocessed_unions: RefCell::new(Vec::new()),
        }
    }

    /// Shared empty dependency set.
    #[must_use]
    pub fn empty_set(&self) -> Rc<PermanentDependencySet> {
        Rc::clone(&self.empty_set)
    }

    /// Add `branching_point` to `dependency_set` (HermiT `addBranchingPoint`).
    pub fn add_branching_point(
        &self,
        dependency_set: &DependencySetRef,
        branching_point: i32,
    ) -> Rc<PermanentDependencySet> {
        let permanent = self.get_permanent(dependency_set);
        if branching_point > permanent.branching_point {
            return self.get_dependency_set(Some(Rc::clone(&permanent)), branching_point);
        }
        if branching_point == permanent.branching_point {
            return permanent;
        }
        let mut merge = self.merge_scratch.borrow_mut();
        merge.clear();
        let mut rest = permanent;
        while branching_point < rest.branching_point {
            merge.push(rest.branching_point);
            rest = Rc::clone(
                rest.rest
                    .as_ref()
                    .expect("branching points below head must exist"),
            );
        }
        if branching_point == rest.branching_point {
            return self.get_permanent(dependency_set);
        }
        let mut result = self.get_dependency_set(Some(rest), branching_point);
        for &bp in merge.iter().rev() {
            result = self.get_dependency_set(Some(result), bp);
        }
        result
    }

    /// Remove `branching_point` from `dependency_set` if present.
    pub fn remove_branching_point(
        &self,
        dependency_set: &DependencySetRef,
        branching_point: i32,
    ) -> Rc<PermanentDependencySet> {
        let permanent = self.get_permanent(dependency_set);
        if branching_point == permanent.branching_point {
            return permanent.rest.clone().unwrap_or_else(|| self.empty_set());
        }
        if branching_point > permanent.branching_point {
            return permanent;
        }
        let mut merge = self.merge_scratch.borrow_mut();
        merge.clear();
        let mut rest = permanent;
        while branching_point < rest.branching_point {
            merge.push(rest.branching_point);
            rest = Rc::clone(
                rest.rest
                    .as_ref()
                    .expect("branching points below head must exist"),
            );
        }
        if branching_point != rest.branching_point {
            return self.get_permanent(dependency_set);
        }
        let mut result = rest.rest.clone().unwrap_or_else(|| self.empty_set());
        for &bp in merge.iter().rev() {
            result = self.get_dependency_set(Some(result), bp);
        }
        result
    }

    /// Union of two dependency sets.
    pub fn union_with(
        &self,
        set1: &DependencySetRef,
        set2: &DependencySetRef,
    ) -> Rc<PermanentDependencySet> {
        let mut permanent_set1 = self.get_permanent(set1);
        let mut permanent_set2 = self.get_permanent(set2);
        if Rc::ptr_eq(&permanent_set1, &permanent_set2) {
            return permanent_set1;
        }
        let mut merge = self.merge_scratch.borrow_mut();
        merge.clear();
        while !Rc::ptr_eq(&permanent_set1, &permanent_set2) {
            if permanent_set1.branching_point > permanent_set2.branching_point {
                merge.push(permanent_set1.branching_point);
                permanent_set1 = Rc::clone(permanent_set1.rest.as_ref().unwrap_or(&self.empty_set));
            } else if permanent_set1.branching_point < permanent_set2.branching_point {
                merge.push(permanent_set2.branching_point);
                permanent_set2 = Rc::clone(permanent_set2.rest.as_ref().unwrap_or(&self.empty_set));
            } else {
                merge.push(permanent_set1.branching_point);
                permanent_set1 = Rc::clone(permanent_set1.rest.as_ref().unwrap_or(&self.empty_set));
                permanent_set2 = Rc::clone(permanent_set2.rest.as_ref().unwrap_or(&self.empty_set));
            }
        }
        let mut result = permanent_set1;
        for &bp in merge.iter().rev() {
            result = self.get_dependency_set(Some(result), bp);
        }
        result
    }

    /// Turn a union into a permanent interned set.
    pub fn get_permanent_union(&self, union: &UnionDependencySet) -> Rc<PermanentDependencySet> {
        self.get_permanent(&DependencySetRef::Union(Rc::new(UnionDependencySet {
            constituents: union.constituents.clone(),
        })))
    }

    fn get_permanent(&self, dependency_set: &DependencySetRef) -> Rc<PermanentDependencySet> {
        match dependency_set {
            DependencySetRef::Permanent(set) => Rc::clone(set),
            DependencySetRef::Union(union) => self.materialize_union(union),
        }
    }

    fn materialize_union(&self, union: &UnionDependencySet) -> Rc<PermanentDependencySet> {
        let mut unprocessed = self.unprocessed_unions.borrow_mut();
        let mut merge_sets = self.merge_sets.borrow_mut();
        unprocessed.clear();
        merge_sets.clear();
        unprocessed.push(Rc::new(UnionDependencySet {
            constituents: union.constituents.clone(),
        }));
        while let Some(current) = unprocessed.pop() {
            for constituent in &current.constituents {
                match constituent {
                    DependencySetRef::Union(u) => unprocessed.push(Rc::clone(u)),
                    DependencySetRef::Permanent(p) => merge_sets.push(Rc::clone(p)),
                }
            }
        }
        let number_of_sets = merge_sets.len();
        if number_of_sets == 0 {
            return self.empty_set();
        }
        let mut merge = self.merge_scratch.borrow_mut();
        merge.clear();
        loop {
            let first = Rc::clone(&merge_sets[0]);
            let mut maximal = first.branching_point;
            let mut maximal_index = 0usize;
            let mut has_equals = false;
            let mut all_equal = true;
            for (index, set) in merge_sets.iter().enumerate().skip(1) {
                let bp = set.branching_point;
                if bp > maximal {
                    maximal = bp;
                    has_equals = false;
                    maximal_index = index;
                } else if bp == maximal {
                    has_equals = true;
                }
                if !Rc::ptr_eq(set, &first) {
                    all_equal = false;
                }
            }
            if all_equal {
                break;
            }
            merge.push(maximal);
            if has_equals {
                for set in merge_sets.iter_mut() {
                    if set.branching_point == maximal {
                        *set = Rc::clone(set.rest.as_ref().unwrap_or(&self.empty_set));
                    }
                }
            } else {
                let set = Rc::clone(&merge_sets[maximal_index]);
                merge_sets[maximal_index] = Rc::clone(set.rest.as_ref().unwrap_or(&self.empty_set));
            }
        }
        let mut result = Rc::clone(&merge_sets[0]);
        for &bp in merge.iter().rev() {
            result = self.get_dependency_set(Some(result), bp);
        }
        result
    }

    fn get_dependency_set(
        &self,
        rest: Option<Rc<PermanentDependencySet>>,
        branching_point: i32,
    ) -> Rc<PermanentDependencySet> {
        let rest_ptr = rest.as_ref().map(|r| Rc::as_ptr(r) as usize).unwrap_or(0);
        let key = EntryKey {
            rest_ptr,
            branching_point,
        };
        if let Some(existing) = self.entries.borrow().get(&key) {
            return Rc::clone(existing);
        }
        let new_set = Rc::new(PermanentDependencySet {
            rest,
            branching_point,
            next_entry: RefCell::new(None),
        });
        self.entries.borrow_mut().insert(key, Rc::clone(&new_set));
        new_set
    }
}

#[cfg(test)]
mod hermit_ports {
    use super::*;
    use std::rc::Rc;

    fn assert_ds_equals(set: &PermanentDependencySet, expected: &[i32]) {
        let actual: Vec<i32> = set.branching_points().collect();
        assert_eq!(actual, expected);
    }

    fn perm(set: Rc<PermanentDependencySet>) -> DependencySetRef {
        DependencySetRef::Permanent(set)
    }

    #[test]
    fn hermit_dependency_set_test_1() {
        let factory = DependencySetFactory::new();
        let mut set = factory.empty_set();
        assert_ds_equals(&set, &[]);
        assert!(set.is_empty());

        set = factory.add_branching_point(&perm(set), 32);
        assert!(!set.is_empty());
        assert_ds_equals(&set, &[32]);

        set = factory.add_branching_point(&perm(set), 0);
        assert_ds_equals(&set, &[32, 0]);

        let set2 = factory.add_branching_point(&perm(factory.empty_set()), 0);
        assert!(!Rc::ptr_eq(&set, &set2));
        let merged = factory.add_branching_point(&perm(set2.clone()), 32);
        assert!(Rc::ptr_eq(&set, &merged));

        let mid = factory.add_branching_point(&perm(set2.clone()), 15);
        let mid = factory.add_branching_point(&perm(mid), 17);
        set = factory.union_with(&perm(set), &perm(mid));
        assert_ds_equals(&set, &[32, 17, 15, 0]);

        let empty_union = factory.union_with(&perm(set.clone()), &perm(factory.empty_set()));
        assert!(Rc::ptr_eq(&set, &empty_union));
        let dup = factory.add_branching_point(&perm(empty_union.clone()), 17);
        assert!(Rc::ptr_eq(&set, &dup));

        let unchanged = factory.remove_branching_point(&perm(set.clone()), 13);
        assert!(Rc::ptr_eq(&set, &unchanged));

        set = factory.remove_branching_point(&perm(set), 17);
        assert_ds_equals(&set, &[32, 15, 0]);
        set = factory.remove_branching_point(&perm(set), 15);
        assert_ds_equals(&set, &[32, 0]);
        set = factory.remove_branching_point(&perm(set), 32);
        assert_ds_equals(&set, &[0]);
    }

    #[test]
    fn hermit_dependency_set_test_2() {
        let factory = DependencySetFactory::new();
        let mut set1 = factory.empty_set();
        set1 = factory.add_branching_point(&perm(set1), 10);
        set1 = factory.add_branching_point(&perm(set1), 3);
        set1 = factory.add_branching_point(&perm(set1), 1);
        set1 = factory.add_branching_point(&perm(set1), 14);
        assert_ds_equals(&set1, &[14, 10, 3, 1]);

        let mut set2 = factory.empty_set();
        set2 = factory.add_branching_point(&perm(set2), 15);
        set2 = factory.add_branching_point(&perm(set2), 10);
        set2 = factory.add_branching_point(&perm(set2), 1);
        set2 = factory.add_branching_point(&perm(set2), 17);
        assert_ds_equals(&set2, &[17, 15, 10, 1]);

        let set3 = factory.union_with(&perm(set1), &perm(set2));
        assert_ds_equals(&set3, &[17, 15, 14, 10, 3, 1]);
    }

    #[test]
    fn hermit_dependency_set_test_3() {
        let factory = DependencySetFactory::new();
        let mut set1 = factory.empty_set();
        set1 = factory.add_branching_point(&perm(set1), 10);
        set1 = factory.add_branching_point(&perm(set1), 3);
        set1 = factory.add_branching_point(&perm(set1), 1);
        assert_ds_equals(&set1, &[10, 3, 1]);

        let mut set2 = factory.empty_set();
        set2 = factory.add_branching_point(&perm(set2), 14);
        set2 = factory.add_branching_point(&perm(set2), 3);
        set2 = factory.add_branching_point(&perm(set2), 1);
        set2 = factory.add_branching_point(&perm(set2), 17);
        assert_ds_equals(&set2, &[17, 14, 3, 1]);

        let mut set3 = factory.empty_set();
        set3 = factory.add_branching_point(&perm(set3), 14);
        set3 = factory.add_branching_point(&perm(set3), 3);
        set3 = factory.add_branching_point(&perm(set3), 2);
        set3 = factory.add_branching_point(&perm(set3), 18);
        assert_ds_equals(&set3, &[18, 14, 3, 2]);

        let union = UnionDependencySet {
            constituents: vec![
                DependencySetRef::Permanent(set1),
                DependencySetRef::Permanent(set2),
                DependencySetRef::Permanent(set3),
            ],
        };
        let set4 = factory.get_permanent_union(&union);
        assert_ds_equals(&set4, &[18, 17, 14, 10, 3, 2, 1]);
    }
}
