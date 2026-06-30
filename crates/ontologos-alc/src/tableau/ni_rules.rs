//! HermiT nominal introduction (NI) rule engine with pruning.

use std::cell::RefCell;
use std::rc::Rc;

use super::dependency_set::{DependencySetFactory, DependencySetRef, PermanentDependencySet};
use super::extension_manager::{Node, NodeId, Tableau};
use super::tuple_table::TupleTableFullIndex;

/// Annotated equality `@atMost(n <R> <C>)(X)` (HermiT `AnnotatedEquality`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnnotatedEquality {
    /// Maximum cardinality (1 or 2 in NI tests).
    pub cardinality: u32,
    /// Object role name.
    pub role: &'static str,
    /// Filler concept name.
    pub concept: &'static str,
}

impl AnnotatedEquality {
    /// HermiT `AnnotatedEquality.create(n, role, concept)`.
    #[must_use]
    pub const fn new(cardinality: u32, role: &'static str, concept: &'static str) -> Self {
        Self {
            cardinality,
            role,
            concept,
        }
    }
}

/// Stored annotated equality awaiting NI processing.
#[derive(Debug, Clone)]
struct AnnotatedEqualityEntry {
    equality: AnnotatedEquality,
    node0: NodeId,
    node1: NodeId,
    node2: NodeId,
    dependency: Rc<PermanentDependencySet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RootLookupKey {
    root: NodeId,
    cardinality: u32,
    index: u32,
}

impl RootLookupKey {
    fn as_tuple(self) -> [NodeId; 3] {
        [
            self.root,
            NodeId(self.cardinality as usize),
            NodeId(self.index as usize),
        ]
    }
}

/// HermiT `NominalIntroductionManager`.
pub struct NominalIntroductionManager {
    tableau: Tableau,
    annotated_equalities: RefCell<Vec<AnnotatedEqualityEntry>>,
    new_root_nodes: RefCell<TupleTableFullIndex<NodeId>>,
    first_unprocessed: RefCell<usize>,
    ni_branching: RefCell<Option<NiBranchingState>>,
}

struct NiBranchingState {
    level: i32,
    root: NodeId,
    ni_target: NodeId,
    other: NodeId,
    equality: AnnotatedEquality,
    current_root: u32,
}

impl NominalIntroductionManager {
    /// Attach to a tableau (HermiT constructor).
    #[must_use]
    pub fn new(tableau: Tableau) -> Self {
        Self {
            tableau,
            annotated_equalities: RefCell::new(Vec::new()),
            new_root_nodes: RefCell::new(TupleTableFullIndex::new(4, 3)),
            first_unprocessed: RefCell::new(0),
            ni_branching: RefCell::new(None),
        }
    }

    /// Pending annotated equalities not yet processed.
    #[must_use]
    pub fn pending_annotated_equalities(&self) -> usize {
        self.annotated_equalities.borrow().len()
    }

    /// Lookup NI root `(rootNode, equality, index)` or `None`.
    #[must_use]
    pub fn root_node_for(
        &self,
        root: &Node,
        equality: AnnotatedEquality,
        index: u32,
    ) -> Option<Node> {
        let lookup = RootLookupKey {
            root: root.id(),
            cardinality: equality.cardinality,
            index,
        };
        let probe = lookup.as_tuple();
        let idx = self.new_root_nodes.borrow().get_tuple_index(&[
            probe[0],
            probe[1],
            probe[2],
            NodeId(usize::MAX),
        ]);
        if idx == -1 {
            return None;
        }
        let node_id = self
            .new_root_nodes
            .borrow()
            .tuple_table()
            .get_tuple_object(idx, 3);
        Some(self.tableau.node_by_id(node_id))
    }

    /// HermiT `addAnnotatedEquality`.
    pub fn add_annotated_equality(
        &self,
        equality: AnnotatedEquality,
        node0: &Node,
        node1: &Node,
        node2: &Node,
        dependency: Rc<PermanentDependencySet>,
    ) -> bool {
        if !node0.is_active() || !node1.is_active() || !node2.is_active() {
            return false;
        }
        let ext = self.tableau.extension_manager();
        if can_forget_annotation(node0, node1, node2) {
            return ext.merge_nodes(node0, node1, dependency);
        }
        if equality.cardinality == 1 {
            return self.apply_ni_rule(equality, node0, node1, node2, dependency);
        }
        self.annotated_equalities
            .borrow_mut()
            .push(AnnotatedEqualityEntry {
                equality,
                node0: node0.id(),
                node1: node1.id(),
                node2: node2.id(),
                dependency,
            });
        true
    }

    /// HermiT `processAnnotatedEqualities`.
    pub fn process_annotated_equalities(&self) -> bool {
        let mut changed = false;
        let mut idx = *self.first_unprocessed.borrow();
        let entries = self.annotated_equalities.borrow().clone();
        while idx < entries.len() {
            let entry = entries[idx].clone();
            idx += 1;
            *self.first_unprocessed.borrow_mut() = idx;
            let ext = self.tableau.extension_manager();
            let n0 = ext.node(entry.node0);
            let n1 = ext.node(entry.node1);
            let n2 = ext.node(entry.node2);
            if self.apply_ni_rule(entry.equality, &n0, &n1, &n2, entry.dependency) {
                changed = true;
            }
        }
        changed
    }

    /// HermiT package-visible NI application.
    pub fn apply_ni_rule(
        &self,
        equality: AnnotatedEquality,
        n0: &Node,
        n1: &Node,
        n2: &Node,
        dependencies: Rc<PermanentDependencySet>,
    ) -> bool {
        if n0.is_pruned() || n1.is_pruned() || n2.is_pruned() {
            return false;
        }
        let ext = self.tableau.extension_manager();
        let node0 = ext.canonical(n0);
        let node1 = ext.canonical(n1);
        let node2 = ext.canonical(n2);
        if can_forget_annotation(&node0, &node1, &node2) {
            return ext.merge_nodes(&node0, &node1, dependencies);
        }
        let (ni_target, other) = if !node0.is_root_node() && !node2.is_parent_of(&node0) {
            (node0, node1)
        } else {
            (node1, node0)
        };
        let mut dependency_set = dependencies;
        if equality.cardinality > 1 {
            let level = self.tableau.push_branching_point_from_tableau();
            let factory = DependencySetFactory::new();
            dependency_set =
                factory.add_branching_point(&DependencySetRef::Permanent(dependency_set), level);
            *self.ni_branching.borrow_mut() = Some(NiBranchingState {
                level,
                root: node2.id(),
                ni_target: ni_target.id(),
                other: other.id(),
                equality,
                current_root: 1,
            });
        }
        let new_root = self.get_ni_root_for(dependency_set.clone(), &node2, equality, 1);
        let new_root = if !new_root.is_active() {
            ext.canonical(&new_root)
        } else {
            new_root
        };
        ext.merge_nodes(&ni_target, &new_root, dependency_set.clone());
        if !other.is_pruned() {
            let other = ext.canonical(&other);
            ext.merge_nodes(&other, &new_root, dependency_set);
        }
        true
    }

    fn get_ni_root_for(
        &self,
        dependency: Rc<PermanentDependencySet>,
        root_node: &Node,
        equality: AnnotatedEquality,
        number: u32,
    ) -> Node {
        let lookup = [
            root_node.id(),
            NodeId(equality.cardinality as usize),
            NodeId(number as usize),
            NodeId(usize::MAX),
        ];
        let tuple_index = self.new_root_nodes.borrow().get_tuple_index(&lookup);
        if tuple_index != -1 {
            let node_id = self
                .new_root_nodes
                .borrow()
                .tuple_table()
                .get_tuple_object(tuple_index, 3);
            return self.tableau.node_by_id(node_id);
        }
        let new_root = self.tableau.create_new_ni_node(dependency);
        let full = [
            root_node.id(),
            NodeId(equality.cardinality as usize),
            NodeId(number as usize),
            new_root.id(),
        ];
        let mut index = self.new_root_nodes.borrow_mut();
        let tentative = index.tuple_table().first_free_tuple_index();
        let idx = index.add_tuple(&full, tentative);
        if idx == tentative {
            index.tuple_table_mut().add_tuple(&full);
        }
        new_root
    }

    /// Start next NI choice after clash.
    pub fn start_next_ni_choice(&self, clash_dependency: Rc<PermanentDependencySet>) -> bool {
        let Some(mut state) = self.ni_branching.borrow_mut().take() else {
            return false;
        };
        state.current_root += 1;
        if state.current_root > state.equality.cardinality {
            return false;
        }
        let factory = DependencySetFactory::new();
        let mut dependency_set = clash_dependency;
        if state.current_root == state.equality.cardinality {
            dependency_set = factory
                .remove_branching_point(&DependencySetRef::Permanent(dependency_set), state.level);
        }
        let root = self.tableau.node_by_id(state.root);
        let ni_target = self.tableau.node_by_id(state.ni_target);
        let other = self.tableau.node_by_id(state.other);
        let new_root = self.get_ni_root_for(
            dependency_set.clone(),
            &root,
            state.equality,
            state.current_root,
        );
        let ext = self.tableau.extension_manager();
        let new_root = if !new_root.is_active() {
            ext.canonical(&new_root)
        } else {
            new_root
        };
        ext.merge_nodes(&ni_target, &new_root, dependency_set.clone());
        if !other.is_pruned() {
            let other = ext.canonical(&other);
            ext.merge_nodes(&other, &new_root, dependency_set);
        }
        *self.ni_branching.borrow_mut() = Some(state);
        true
    }
}

/// HermiT `canForgetAnnotation`.
#[must_use]
pub fn can_forget_annotation(node0: &Node, node1: &Node, node2: &Node) -> bool {
    node0.is_root_node()
        || node1.is_root_node()
        || !node2.is_root_node()
        || (node2.is_parent_of(node0) && node2.is_parent_of(node1))
}
