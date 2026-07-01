//! HermiT-style extension manager: nodes, merges, backtrack, clash detection.
//!
//! Minimal port of `ExtensionManager`, `MergingManager`, and `ClashManager` sufficient
//! for `MergeTest.testMergeAndBacktrack`.
#![allow(private_interfaces)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use super::blocking_validator::RoleRef;
use super::dependency_set::{DependencySetFactory, PermanentDependencySet};
use super::description_graph::DescriptionGraphId;
use super::graph_merge;
use super::ni_rules::NominalIntroductionManager;
use super::tuple_index::{TupleIndex, TupleIndexError};
use super::tuple_table::TupleTable;

/// Tableau node identifier (stable across merges).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

/// DL predicate in extension tables.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DlPredicate {
    /// Atomic concept name.
    AtomicConcept(&'static str),
    /// Negated atomic concept.
    AtomicNegationConcept(&'static str),
    /// ∃≥n R.C (role may be inverse).
    AtLeastConcept {
        /// Minimum cardinality.
        n: u32,
        /// Role reference.
        role: RoleRef,
        /// Filler concept.
        filler: Box<DlPredicate>,
    },
    /// At-most cardinality concept (internal tests).
    AtMostConcept {
        /// Maximum cardinality.
        n: u32,
        /// Role name.
        role: &'static str,
        /// Filler concept name.
        filler: &'static str,
    },
    /// Atomic object role.
    AtomicRole(&'static str),
    /// Inverse object role (internal tests).
    InverseRole(&'static str),
    /// Equality merge predicate.
    Equality,
    /// Disequality predicate.
    Inequality,
    /// Description graph (4-ary tuples).
    DescriptionGraph(DescriptionGraphId),
}

impl DlPredicate {
    fn negation_of(&self) -> Option<Self> {
        match self {
            Self::AtomicConcept(name) => Some(Self::AtomicNegationConcept(name)),
            Self::AtomicNegationConcept(name) => Some(Self::AtomicConcept(name)),
            _ => None,
        }
    }
}

/// Stored tuple component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DlObject {
    /// Predicate slot.
    Predicate(DlPredicate),
    /// Node slot.
    Node(NodeId),
}

/// HermiT `NodeType` (subset used by merge tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Named individual node.
    NiNode,
    /// Tree expansion node.
    TreeNode,
}

impl NodeType {
    fn merge_precedence(self) -> u8 {
        match self {
            Self::NiNode | Self::TreeNode => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeState {
    Active,
    Merged,
    Pruned,
}

#[derive(Debug, Default)]
struct BlockingState {
    directly_blocked: bool,
    blocked: bool,
    blocker: Option<NodeId>,
    block_violates_parent: bool,
    parent_checked: bool,
}

#[derive(Debug)]
struct NodeInner {
    id: NodeId,
    state: NodeState,
    parent: Option<NodeId>,
    node_type: NodeType,
    positive_atomic_concepts: u32,
    negated_atomic_concepts: u32,
    merged_into: Option<NodeId>,
    previous_merged_or_pruned: Option<NodeId>,
    next_tableau: Option<NodeId>,
    blocking: BlockingState,
}

/// Tableau node handle (reference equality like HermiT `Node`).
#[derive(Debug, Clone)]
pub struct Node {
    inner: Rc<RefCell<NodeInner>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for Node {}

impl Node {
    /// Stable node identifier.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.inner.borrow().id
    }

    /// Whether the node is active in the current branch.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.borrow().state == NodeState::Active
    }

    /// Whether the node was merged into another.
    #[must_use]
    pub fn is_merged(&self) -> bool {
        self.inner.borrow().state == NodeState::Merged
    }

    /// Whether the node was pruned (descendant of merged node).
    #[must_use]
    pub fn is_pruned(&self) -> bool {
        self.inner.borrow().state == NodeState::Pruned
    }

    /// HermiT `isRootNode` — NI nodes are root nodes.
    #[must_use]
    pub fn is_root_node(&self) -> bool {
        self.inner.borrow().node_type == NodeType::NiNode
    }

    /// HermiT `isTreeNode`.
    #[must_use]
    pub fn is_tree_node(&self) -> bool {
        self.inner.borrow().node_type == NodeType::TreeNode
    }

    /// Whether this node is the parent of `other`.
    #[must_use]
    pub fn is_parent_of(&self, other: &Node) -> bool {
        other.inner.borrow().parent == Some(self.id())
    }

    /// Parent node id if any.
    #[must_use]
    pub fn parent_id(&self) -> Option<NodeId> {
        self.inner.borrow().parent
    }

    /// Whether the node is directly blocked.
    #[must_use]
    pub fn is_directly_blocked(&self, ext: &ExtensionManagerRef) -> bool {
        let _ = ext;
        self.inner.borrow().blocking.directly_blocked
    }

    /// Whether the node is blocked (directly or indirectly).
    #[must_use]
    pub fn is_blocked(&self, ext: &ExtensionManagerRef) -> bool {
        let _ = ext;
        self.inner.borrow().blocking.blocked || self.inner.borrow().blocking.directly_blocked
    }

    /// Blocker node if directly blocked.
    #[must_use]
    pub fn blocker(&self, ext: &ExtensionManagerRef) -> Option<Node> {
        let _ = ext;
        self.inner.borrow().blocking.blocker.map(|id| ext.node(id))
    }

    pub(crate) fn block_violates_parent_constraints(&self) -> bool {
        self.inner.borrow().blocking.block_violates_parent
    }

    pub(crate) fn set_block_violates_parent(&self, value: bool) {
        self.inner.borrow_mut().blocking.block_violates_parent = value;
    }

    pub(crate) fn is_parent_checked(&self) -> bool {
        self.inner.borrow().blocking.parent_checked
    }

    pub(crate) fn set_parent_checked(&self, value: bool) {
        self.inner.borrow_mut().blocking.parent_checked = value;
    }

    /// Mark node as directly blocked by `blocker` (HermiT test hook).
    pub fn set_directly_blocked(&self, blocker: NodeId, ext: &ExtensionManagerRef) {
        let _ = ext;
        let mut inner = self.inner.borrow_mut();
        inner.blocking.directly_blocked = true;
        inner.blocking.blocked = true;
        inner.blocking.blocker = Some(blocker);
    }

    pub(crate) fn clear_blocking_state(&self, ext: &ExtensionManagerRef) {
        let _ = ext;
        self.inner.borrow_mut().blocking = BlockingState::default();
    }

    /// Parent node handle.
    #[must_use]
    pub fn parent_node(&self, ext: &ExtensionManagerRef) -> Node {
        self.parent_id()
            .map(|id| ext.node(id))
            .unwrap_or_else(|| ext.node(self.id()))
    }

    /// Follow merge chain to canonical representative.
    #[must_use]
    pub(crate) fn canonical_node(&self, store: &NodeStore) -> Node {
        let mut current = self.id();
        while let Some(into) = store.get(current).and_then(|n| n.merged_into) {
            current = into;
        }
        store.node(current)
    }

    fn parent(&self) -> Option<NodeId> {
        self.inner.borrow().parent
    }

    fn node_type(&self) -> NodeType {
        self.inner.borrow().node_type
    }

    fn positive_atomic_concepts(&self) -> u32 {
        self.inner.borrow().positive_atomic_concepts
    }

    fn cluster_anchor(&self) -> NodeId {
        if self.node_type() == NodeType::TreeNode {
            self.id()
        } else {
            self.parent().unwrap_or(self.id())
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct NodeStore {
    nodes: Vec<Rc<RefCell<NodeInner>>>,
}

impl NodeStore {
    fn get(&self, id: NodeId) -> Option<NodeInnerSnapshot> {
        self.nodes
            .get(id.0)
            .map(|n| NodeInnerSnapshot::from(&n.borrow()))
    }

    fn node(&self, id: NodeId) -> Node {
        Node {
            inner: self.nodes[id.0].clone(),
        }
    }

    fn push(&mut self, inner: NodeInner) -> Node {
        let id = inner.id;
        let handle = Node {
            inner: Rc::new(RefCell::new(inner)),
        };
        if self.nodes.len() <= id.0 {
            self.nodes.resize(id.0 + 1, handle.inner.clone());
        } else {
            self.nodes[id.0] = handle.inner.clone();
        }
        handle
    }

    fn with_mut(&mut self, id: NodeId, f: impl FnOnce(&mut NodeInner)) {
        f(&mut self.nodes[id.0].borrow_mut());
    }
}

#[derive(Debug, Clone)]
struct NodeInnerSnapshot {
    state: NodeState,
    parent: Option<NodeId>,
    merged_into: Option<NodeId>,
    next_tableau: Option<NodeId>,
    positive_atomic_concepts: u32,
    negated_atomic_concepts: u32,
}

impl NodeInnerSnapshot {
    fn from(inner: &NodeInner) -> Self {
        Self {
            state: inner.state,
            parent: inner.parent,
            merged_into: inner.merged_into,
            next_tableau: inner.next_tableau,
            positive_atomic_concepts: inner.positive_atomic_concepts,
            negated_atomic_concepts: inner.negated_atomic_concepts,
        }
    }
}

/// Branching point marker (HermiT `BranchingPoint`).
#[derive(Debug, Clone)]
pub struct BranchingPoint {
    level: usize,
    last_merged_or_pruned: Option<NodeId>,
}

impl BranchingPoint {
    /// Create a branching point at the tableau's current level.
    #[must_use]
    pub fn new(tableau: &Tableau) -> Self {
        Self {
            level: tableau.current_branching_point(),
            last_merged_or_pruned: tableau.last_merged_or_pruned(),
        }
    }

    /// Branching level for [`Tableau::backtrack_to`].
    #[must_use]
    pub fn level(&self) -> usize {
        self.level
    }
}

/// Extension view (HermiT `ExtensionTable.View`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionView {
    /// All tuples including delta-new.
    Total,
}

/// Iterator/retrieval over matching extension tuples.
pub struct ExtensionRetrieval<'a> {
    table: &'a ExtensionTable,
    bindings: Vec<Option<DlObject>>,
    tuple_buffer: Vec<DlObject>,
    first_tuple_index: i32,
    after_last_tuple_index: i32,
    current: i32,
}

impl ExtensionRetrieval<'_> {
    /// Whether iteration is exhausted.
    #[must_use]
    pub fn after_last(&self) -> bool {
        self.current >= self.after_last_tuple_index
    }

    /// Current tuple components.
    #[must_use]
    pub fn tuple_buffer(&self) -> &[DlObject] {
        &self.tuple_buffer
    }

    /// Whether the current tuple is marked core.
    #[must_use]
    pub fn is_core(&self) -> bool {
        self.table
            .core_flags
            .get(self.current as usize)
            .copied()
            .unwrap_or(false)
    }

    /// Bindings buffer (HermiT `getBindingsBuffer`).
    pub fn bindings_buffer(&mut self) -> &mut [Option<DlObject>] {
        &mut self.bindings
    }

    /// Start iteration with current bindings and view bounds.
    pub fn open(&mut self) {
        self.current = self.first_tuple_index;
        self.advance();
    }

    /// Advance to next matching tuple.
    pub fn next(&mut self) {
        if self.current < self.after_last_tuple_index {
            self.current += 1;
        }
        self.advance();
    }

    fn advance(&mut self) {
        while self.current < self.after_last_tuple_index {
            self.table
                .tuple_table
                .retrieve_tuple(&mut self.tuple_buffer, self.current);
            if self.matches() {
                return;
            }
            self.current += 1;
        }
    }

    fn matches(&self) -> bool {
        if !self.table.is_tuple_active(&self.tuple_buffer) {
            return false;
        }
        for (pos, binding) in self.bindings.iter().enumerate() {
            if let Some(expected) = binding {
                if &self.tuple_buffer[pos] != expected {
                    return false;
                }
            }
        }
        true
    }
}

/// Binary or ternary extension table with tuple indexes.
pub struct ExtensionTable {
    arity: usize,
    tuple_table: TupleTable<DlObject>,
    tuple_indexes: Vec<TupleIndex<DlObject>>,
    dependency_sets: Vec<Option<Rc<PermanentDependencySet>>>,
    core_flags: Vec<bool>,
    after_delta_new: i32,
    indices_by_branching_point: Vec<i32>,
    nodes: Weak<RefCell<NodeStore>>,
}

impl ExtensionTable {
    fn new(
        arity: usize,
        indexing_sequences: &[Vec<usize>],
        nodes: &Rc<RefCell<NodeStore>>,
    ) -> Self {
        Self {
            arity,
            tuple_table: TupleTable::new(arity),
            tuple_indexes: indexing_sequences
                .iter()
                .map(|seq| TupleIndex::new(seq))
                .collect(),
            dependency_sets: Vec::new(),
            core_flags: Vec::new(),
            after_delta_new: 0,
            indices_by_branching_point: vec![0; 6],
            nodes: Rc::downgrade(nodes),
        }
    }

    fn node_store(&self) -> Rc<RefCell<NodeStore>> {
        self.nodes.upgrade().expect("node store")
    }

    fn is_tuple_active(&self, tuple: &[DlObject]) -> bool {
        let nodes = self.node_store();
        let store = nodes.borrow();
        match self.arity {
            2 => match &tuple[1] {
                DlObject::Node(id) => store.get(*id).is_some_and(|n| n.state == NodeState::Active),
                _ => false,
            },
            3 => {
                let (DlObject::Node(from), DlObject::Node(to)) = (&tuple[1], &tuple[2]) else {
                    return false;
                };
                store
                    .get(*from)
                    .is_some_and(|n| n.state == NodeState::Active)
                    && store.get(*to).is_some_and(|n| n.state == NodeState::Active)
            }
            4 => tuple[1..].iter().all(|obj| {
                if let DlObject::Node(id) = obj {
                    store.get(*id).is_some_and(|n| n.state == NodeState::Active)
                } else {
                    true
                }
            }),
            _ => true,
        }
    }

    pub(crate) fn contains_tuple(&self, tuple: &[DlObject]) -> bool {
        let tuple_index = self.tuple_indexes[0].get_tuple_index(tuple);
        tuple_index != -1 && self.is_tuple_active(tuple)
    }

    pub(crate) fn dependency_for_tuple(
        &self,
        tuple: &[DlObject],
    ) -> Option<Rc<PermanentDependencySet>> {
        let tuple_index = self.tuple_indexes[0].get_tuple_index(tuple);
        if tuple_index == -1 {
            return None;
        }
        self.dependency_sets
            .get(tuple_index as usize)
            .and_then(|d| d.clone())
    }

    fn add_tuple(
        &mut self,
        tuple: &[DlObject],
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
        on_add: &mut dyn FnMut(&[DlObject], Rc<PermanentDependencySet>),
    ) -> Result<bool, TupleIndexError> {
        if !self.is_tuple_active(tuple) {
            return Ok(false);
        }
        let first_free = self.tuple_table.first_free_tuple_index();
        let add_index = self.tuple_indexes[0].add_tuple(tuple, first_free)?;
        if add_index == first_free {
            for index in &mut self.tuple_indexes[1..] {
                index.add_tuple(tuple, add_index)?;
            }
            self.tuple_table.add_tuple(tuple);
            while self.dependency_sets.len() <= add_index as usize {
                self.dependency_sets.push(None);
                self.core_flags.push(false);
            }
            self.dependency_sets[add_index as usize] = Some(dependency.clone());
            self.core_flags[add_index as usize] = is_core;
            self.after_delta_new = self.tuple_table.first_free_tuple_index();
            on_add(tuple, dependency);
            return Ok(true);
        }
        if is_core && !self.core_flags[add_index as usize] {
            self.core_flags[add_index as usize] = true;
        }
        Ok(false)
    }

    fn branching_point_pushed(&mut self, level: usize) {
        let start = level * 3;
        while self.indices_by_branching_point.len() < start + 3 {
            self.indices_by_branching_point.push(0);
        }
        self.indices_by_branching_point[start + 2] = self.after_delta_new;
    }

    fn backtrack(&mut self, level: usize) {
        let start = level * 3;
        let new_after_delta = self.indices_by_branching_point[start + 2];
        for tuple_index in (new_after_delta..self.after_delta_new).rev() {
            let mut tuple = vec![DlObject::Predicate(DlPredicate::Equality); self.arity];
            self.tuple_table.retrieve_tuple(&mut tuple, tuple_index);
            for index in self.tuple_indexes.iter_mut().rev() {
                let _ = index.remove_tuple(&tuple);
            }
        }
        self.tuple_table.truncate(new_after_delta);
        self.dependency_sets.truncate(new_after_delta as usize);
        self.core_flags.truncate(new_after_delta as usize);
        self.after_delta_new = new_after_delta;
    }

    /// Create a retrieval over bound positions.
    pub fn create_retrieval(
        &self,
        binding_pattern: &[bool],
        view: ExtensionView,
    ) -> ExtensionRetrieval<'_> {
        let bindings = vec![None; binding_pattern.len()];
        let (first, after_last) = match view {
            ExtensionView::Total => (0, self.after_delta_new),
        };
        ExtensionRetrieval {
            table: self,
            bindings,
            tuple_buffer: vec![DlObject::Predicate(DlPredicate::Equality); self.arity],
            first_tuple_index: first,
            after_last_tuple_index: after_last,
            current: first,
        }
    }
}

pub(crate) struct TableauState {
    nodes: Rc<RefCell<NodeStore>>,
    binary_table: ExtensionTable,
    ternary_table: ExtensionTable,
    quaternary_table: ExtensionTable,
    dependency_factory: DependencySetFactory,
    clash_dependency: Option<Rc<PermanentDependencySet>>,
    current_branching_point: usize,
    branching_snapshots: HashMap<usize, BranchingPoint>,
    last_merged_or_pruned: Option<NodeId>,
    last_tableau_node: Option<NodeId>,
    next_node_id: usize,
    #[allow(dead_code)]
    last_backtrack_level: Option<usize>,
    tuple_index_error: Option<TupleIndexError>,
    merge_error: Option<String>,
}

impl TableauState {
    fn new() -> Rc<RefCell<Self>> {
        let nodes = Rc::new(RefCell::new(NodeStore::default()));
        Rc::new(RefCell::new(Self {
            binary_table: ExtensionTable::new(2, &[vec![1, 0], vec![0, 1]], &nodes),
            ternary_table: ExtensionTable::new(
                3,
                &[vec![0, 1, 2], vec![1, 2, 0], vec![2, 0, 1]],
                &nodes,
            ),
            quaternary_table: ExtensionTable::new(4, &[vec![0, 1, 2, 3], vec![1, 2, 3, 0]], &nodes),
            nodes,
            dependency_factory: DependencySetFactory::new(),
            clash_dependency: None,
            current_branching_point: 0,
            branching_snapshots: HashMap::new(),
            last_merged_or_pruned: None,
            last_tableau_node: None,
            next_node_id: 0,
            last_backtrack_level: None,
            tuple_index_error: None,
            merge_error: None,
        }))
    }

    /// Take the first tableau error recorded during expansion (if any).
    #[allow(dead_code)] // reserved for DL error propagation from tableau internals
    pub(crate) fn take_tableau_error(&mut self) -> Option<crate::Error> {
        if let Some(err) = self.tuple_index_error.take() {
            return Some(err.into_alc_error());
        }
        self.merge_error.take().map(crate::Error::Message)
    }

    fn empty_set(&self) -> Rc<PermanentDependencySet> {
        self.dependency_factory.empty_set()
    }

    fn set_clash(&mut self, dependency: Rc<PermanentDependencySet>) {
        self.clash_dependency = Some(dependency);
    }

    fn clear_clash(&mut self) {
        self.clash_dependency = None;
    }

    fn contains_clash(&self) -> bool {
        self.clash_dependency.is_some()
    }

    fn adjust_concept_counts(&mut self, id: NodeId, concept: &DlPredicate, delta: i32) {
        self.nodes.borrow_mut().with_mut(id, |node| match concept {
            DlPredicate::AtomicConcept(_) => {
                node.positive_atomic_concepts =
                    (node.positive_atomic_concepts as i32 + delta).max(0) as u32;
            }
            DlPredicate::AtomicNegationConcept(_) => {
                node.negated_atomic_concepts =
                    (node.negated_atomic_concepts as i32 + delta).max(0) as u32;
            }
            _ => {}
        });
    }

    fn clash_on_add(
        &mut self,
        table: TableKind,
        tuple: &[DlObject],
        dependency: Rc<PermanentDependencySet>,
    ) {
        let DlObject::Predicate(predicate) = &tuple[0] else {
            return;
        };
        let DlObject::Node(node_id) = &tuple[1] else {
            return;
        };
        let snapshot = self.nodes.borrow().get(*node_id);

        if matches!(predicate, DlPredicate::Inequality) {
            if let (DlObject::Node(a), Some(DlObject::Node(b))) = (&tuple[1], tuple.get(2)) {
                if a == b {
                    self.set_clash(dependency);
                }
            }
            return;
        }

        let Some(snapshot) = snapshot else {
            return;
        };

        let ext = match table {
            TableKind::Binary => &self.binary_table,
            TableKind::Ternary => &self.ternary_table,
        };

        if (matches!(predicate, DlPredicate::AtomicConcept(_))
            && snapshot.negated_atomic_concepts > 0)
            || (matches!(predicate, DlPredicate::AtomicNegationConcept(_))
                && snapshot.positive_atomic_concepts > 0)
        {
            let opposite = predicate.negation_of().unwrap();
            let tuple = [DlObject::Predicate(opposite), DlObject::Node(*node_id)];
            if ext.contains_tuple(&tuple) {
                self.set_clash(dependency);
            }
        }
    }

    fn add_binary(
        &mut self,
        predicate: DlPredicate,
        node: &Node,
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
    ) -> bool {
        if matches!(
            &predicate,
            DlPredicate::AtomicConcept(_) | DlPredicate::AtomicNegationConcept(_)
        ) {
            self.adjust_concept_counts(node.id(), &predicate, 1);
        }
        let tuple = [DlObject::Predicate(predicate), DlObject::Node(node.id())];
        let added =
            match self
                .binary_table
                .add_tuple(&tuple, dependency.clone(), is_core, &mut |_, _| {})
            {
                Ok(added) => added,
                Err(err) => {
                    self.tuple_index_error = Some(err);
                    return false;
                }
            };
        if added {
            self.clash_on_add(TableKind::Binary, &tuple, dependency);
        }
        added
    }

    fn add_ternary(
        &mut self,
        predicate: DlPredicate,
        node0: &Node,
        node1: &Node,
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
    ) -> bool {
        let tuple = [
            DlObject::Predicate(predicate),
            DlObject::Node(node0.id()),
            DlObject::Node(node1.id()),
        ];
        let added =
            match self
                .ternary_table
                .add_tuple(&tuple, dependency.clone(), is_core, &mut |_, _| {})
            {
                Ok(added) => added,
                Err(err) => {
                    self.tuple_index_error = Some(err);
                    return false;
                }
            };
        if added {
            self.clash_on_add(TableKind::Ternary, &tuple, dependency);
        }
        added
    }

    fn merge_nodes(
        &mut self,
        node0: &Node,
        node1: &Node,
        dependency: Rc<PermanentDependencySet>,
    ) -> bool {
        if !node0.is_active() || !node1.is_active() || node0.id() == node1.id() {
            return false;
        }
        let (merge_from, merge_into) =
            match Self::pick_merge_direction(node0, node1, &self.nodes.borrow()) {
                Ok(direction) => direction,
                Err(msg) => {
                    self.merge_error = Some(msg);
                    return false;
                }
            };
        self.prune_descendants(merge_from);
        self.copy_unary(merge_from, merge_into);
        self.copy_ternary_first(merge_from, merge_into);
        self.copy_ternary_second(merge_from, merge_into);
        self.merge_node(merge_from, merge_into);
        let _ = dependency;
        true
    }

    fn pick_merge_direction(
        node0: &Node,
        node1: &Node,
        store: &NodeStore,
    ) -> Result<(NodeId, NodeId), String> {
        if node0.is_root_node() && !node1.is_root_node() {
            return Ok((node1.id(), node0.id()));
        }
        if node1.is_root_node() && !node0.is_root_node() {
            return Ok((node0.id(), node1.id()));
        }
        let p0 = node0.node_type().merge_precedence();
        let p1 = node1.node_type().merge_precedence();
        if p0 < p1 {
            return Ok((node1.id(), node0.id()));
        }
        if p0 > p1 {
            return Ok((node0.id(), node1.id()));
        }
        let a0 = node0.cluster_anchor();
        let a1 = node1.cluster_anchor();
        let parent0 = node0.parent();
        let parent1 = node1.parent();
        let can0_into1 =
            parent0 == parent1 || Self::is_descendant_of_at_most_three(node0.id(), a1, store);
        let can1_into0 =
            parent0 == parent1 || Self::is_descendant_of_at_most_three(node1.id(), a0, store);
        if can0_into1 && can1_into0 {
            if node0.positive_atomic_concepts() > node1.positive_atomic_concepts() {
                Ok((node1.id(), node0.id()))
            } else {
                Ok((node0.id(), node1.id()))
            }
        } else if can0_into1 {
            Ok((node0.id(), node1.id()))
        } else if can1_into0 {
            Ok((node1.id(), node0.id()))
        } else {
            Err("unsupported merge type".into())
        }
    }

    fn is_descendant_of_at_most_three(
        descendant: NodeId,
        ancestor: NodeId,
        store: &NodeStore,
    ) -> bool {
        let mut current = Some(descendant);
        for _ in 0..3 {
            let Some(id) = current else {
                return false;
            };
            let Some(snapshot) = store.get(id) else {
                return false;
            };
            if snapshot.parent == Some(ancestor) {
                return true;
            }
            current = snapshot.parent;
        }
        false
    }

    fn prune_descendants(&mut self, merge_from: NodeId) {
        let mut cursor = self
            .nodes
            .borrow()
            .get(merge_from)
            .and_then(|n| n.next_tableau);
        while let Some(id) = cursor {
            let (should_prune, next) = {
                let store = self.nodes.borrow();
                let should = store.get(id).is_some_and(|n| {
                    n.state == NodeState::Active
                        && n.parent.is_some_and(|p| {
                            store.get(p).is_some_and(|parent| {
                                parent.state != NodeState::Active || p == merge_from
                            })
                        })
                });
                let next = store.get(id).and_then(|n| n.next_tableau);
                (should, next)
            };
            if should_prune {
                self.prune_node(id);
            }
            cursor = next;
        }
    }

    fn copy_unary(&mut self, merge_from: NodeId, merge_into: NodeId) {
        let to_copy: Vec<(DlPredicate, bool)> = {
            let mut retrieval = self
                .binary_table
                .create_retrieval(&[false, true], ExtensionView::Total);
            retrieval.bindings_buffer()[1] = Some(DlObject::Node(merge_from));
            retrieval.open();
            let mut out = Vec::new();
            while !retrieval.after_last() {
                if let DlObject::Predicate(pred) = retrieval.tuple_buffer()[0].clone() {
                    if !matches!(pred, DlPredicate::DescriptionGraph(_)) {
                        out.push((pred, retrieval.is_core()));
                    }
                }
                retrieval.next();
            }
            out
        };
        let into = self.nodes.borrow().node(merge_into);
        let empty = self.empty_set();
        for (pred, is_core) in to_copy {
            self.add_binary(pred, &into, empty.clone(), is_core);
        }
    }

    fn copy_ternary_first(&mut self, merge_from: NodeId, merge_into: NodeId) {
        let to_copy: Vec<(DlPredicate, NodeId, bool)> = {
            let mut retrieval = self
                .ternary_table
                .create_retrieval(&[false, true, false], ExtensionView::Total);
            retrieval.bindings_buffer()[1] = Some(DlObject::Node(merge_from));
            retrieval.open();
            let mut out = Vec::new();
            while !retrieval.after_last() {
                let pred = match retrieval.tuple_buffer()[0].clone() {
                    DlObject::Predicate(p) => p,
                    _ => {
                        retrieval.next();
                        continue;
                    }
                };
                if matches!(pred, DlPredicate::DescriptionGraph(_)) {
                    retrieval.next();
                    continue;
                }
                let to = match retrieval.tuple_buffer()[2].clone() {
                    DlObject::Node(id) => id,
                    _ => {
                        retrieval.next();
                        continue;
                    }
                };
                out.push((pred, to, retrieval.is_core()));
                retrieval.next();
            }
            out
        };
        let into = self.nodes.borrow().node(merge_into);
        let empty = self.empty_set();
        for (pred, to, is_core) in to_copy {
            let to_node = if to == merge_from {
                into.clone()
            } else {
                self.nodes.borrow().node(to)
            };
            self.add_ternary(pred, &into, &to_node, empty.clone(), is_core);
        }
    }

    fn copy_ternary_second(&mut self, merge_from: NodeId, merge_into: NodeId) {
        let to_copy: Vec<(DlPredicate, NodeId, bool)> = {
            let mut retrieval = self
                .ternary_table
                .create_retrieval(&[false, false, true], ExtensionView::Total);
            retrieval.bindings_buffer()[2] = Some(DlObject::Node(merge_from));
            retrieval.open();
            let mut out = Vec::new();
            while !retrieval.after_last() {
                let pred = match retrieval.tuple_buffer()[0].clone() {
                    DlObject::Predicate(p) => p,
                    _ => {
                        retrieval.next();
                        continue;
                    }
                };
                if matches!(pred, DlPredicate::DescriptionGraph(_)) {
                    retrieval.next();
                    continue;
                }
                let from = match retrieval.tuple_buffer()[1].clone() {
                    DlObject::Node(id) => id,
                    _ => {
                        retrieval.next();
                        continue;
                    }
                };
                out.push((pred, from, retrieval.is_core()));
                retrieval.next();
            }
            out
        };
        let into = self.nodes.borrow().node(merge_into);
        let empty = self.empty_set();
        for (pred, from, is_core) in to_copy {
            let from_node = if from == merge_from {
                into.clone()
            } else {
                self.nodes.borrow().node(from)
            };
            self.add_ternary(pred, &from_node, &into, empty.clone(), is_core);
        }
    }

    fn merge_node(&mut self, from: NodeId, into: NodeId) {
        let prev = self.last_merged_or_pruned;
        self.nodes.borrow_mut().with_mut(from, |node| {
            node.state = NodeState::Merged;
            node.merged_into = Some(into);
            node.previous_merged_or_pruned = prev;
        });
        self.last_merged_or_pruned = Some(from);
    }

    fn prune_node(&mut self, id: NodeId) {
        let prev = self.last_merged_or_pruned;
        self.nodes.borrow_mut().with_mut(id, |node| {
            node.state = NodeState::Pruned;
            node.previous_merged_or_pruned = prev;
        });
        self.last_merged_or_pruned = Some(id);
    }

    fn backtrack_last_merged_or_pruned(&mut self) {
        let Some(id) = self.last_merged_or_pruned else {
            return;
        };
        let prev = self.nodes.borrow().nodes[id.0]
            .borrow()
            .previous_merged_or_pruned;
        self.nodes.borrow_mut().with_mut(id, |node| {
            node.state = NodeState::Active;
            node.merged_into = None;
            node.previous_merged_or_pruned = None;
        });
        self.last_merged_or_pruned = prev;
    }
}

#[derive(Clone, Copy)]
enum TableKind {
    Binary,
    Ternary,
}

/// Minimal HermiT `Tableau` for internal unit tests.
#[derive(Clone)]
pub struct Tableau {
    state: Rc<RefCell<TableauState>>,
}

impl Tableau {
    /// Empty deterministic tableau (no DL ontology clauses required for merge tests).
    #[must_use]
    pub fn new_deterministic() -> Self {
        Self {
            state: TableauState::new(),
        }
    }

    fn current_branching_point(&self) -> usize {
        self.state.borrow().current_branching_point
    }

    fn last_merged_or_pruned(&self) -> Option<NodeId> {
        self.state.borrow().last_merged_or_pruned
    }

    /// Extension manager accessor.
    #[must_use]
    pub fn extension_manager(&self) -> ExtensionManagerRef {
        ExtensionManagerRef {
            state: self.state.clone(),
        }
    }

    /// Dependency set factory.
    #[must_use]
    pub fn dependency_factory(&self) -> DependencySetFactory {
        DependencySetFactory::new()
    }

    fn create_node(&self, parent: Option<NodeId>, node_type: NodeType) -> Node {
        let mut state = self.state.borrow_mut();
        let id = NodeId(state.next_node_id);
        state.next_node_id += 1;
        let inner = NodeInner {
            id,
            state: NodeState::Active,
            parent,
            node_type,
            positive_atomic_concepts: 0,
            negated_atomic_concepts: 0,
            merged_into: None,
            previous_merged_or_pruned: None,
            next_tableau: None,
            blocking: BlockingState::default(),
        };
        if let Some(prev) = state.last_tableau_node {
            state.nodes.borrow_mut().with_mut(prev, |node| {
                node.next_tableau = Some(id);
            });
        }
        let handle = state.nodes.borrow_mut().push(inner);
        state.last_tableau_node = Some(id);
        handle
    }

    /// HermiT `createNewNINode`.
    #[must_use]
    pub fn create_new_ni_node(&self, _dependency: Rc<PermanentDependencySet>) -> Node {
        self.create_node(None, NodeType::NiNode)
    }

    /// HermiT `createNewTreeNode`.
    #[must_use]
    pub fn create_new_tree_node(
        &self,
        _dependency: Rc<PermanentDependencySet>,
        parent: &Node,
    ) -> Node {
        self.create_node(Some(parent.id()), NodeType::TreeNode)
    }

    /// Record a branching point.
    pub fn push_branching_point(&self, bp: &BranchingPoint) {
        let mut state = self.state.borrow_mut();
        state.current_branching_point = bp.level + 1;
        state.branching_snapshots.insert(bp.level, bp.clone());
        state.binary_table.branching_point_pushed(bp.level);
        state.ternary_table.branching_point_pushed(bp.level);
    }

    /// Backtrack to branching level (HermiT package-visible `backtrackTo`).
    pub fn backtrack_to(&self, level: usize) {
        let mut state = self.state.borrow_mut();
        let snapshot = state
            .branching_snapshots
            .get(&level)
            .cloned()
            .unwrap_or(BranchingPoint {
                level,
                last_merged_or_pruned: None,
            });
        state.current_branching_point = level;
        state.binary_table.backtrack(level);
        state.ternary_table.backtrack(level);
        while state.last_merged_or_pruned != snapshot.last_merged_or_pruned {
            state.backtrack_last_merged_or_pruned();
        }
        state.clear_clash();
    }

    /// Node store for canonical lookups.
    pub fn canonical_node(&self, node: &Node) -> Node {
        node.canonical_node(&self.state.borrow().nodes.borrow())
    }

    /// Assert a node's concept label (HermiT `assertLabel`).
    pub fn assert_label(&self, node: &Node, expected: &[DlPredicate]) {
        test_helpers::assert_label(&self.state.borrow(), node, expected);
    }

    /// Nominal introduction manager (HermiT `getNominalIntroductionManager`).
    #[must_use]
    pub fn ni_manager(&self) -> NominalIntroductionManager {
        NominalIntroductionManager::new(self.clone())
    }

    /// Empty dependency set.
    #[must_use]
    pub fn empty_dependency_set(&self) -> Rc<PermanentDependencySet> {
        self.state.borrow().empty_set()
    }

    /// Node handle by id.
    #[must_use]
    pub fn node_by_id(&self, id: NodeId) -> Node {
        self.state.borrow().nodes.borrow().node(id)
    }

    /// Current branching level.
    #[must_use]
    pub fn current_branching_level(&self) -> i32 {
        self.state.borrow().current_branching_point as i32
    }

    /// Push branching point from current tableau state.
    pub fn push_branching_point_from_tableau(&self) -> i32 {
        let bp = BranchingPoint::new(self);
        self.push_branching_point(&bp);
        bp.level() as i32
    }

    /// Handle clash by backtracking (returns true if backtracked).
    pub fn handle_clash_backtrack(&self) -> bool {
        if !self.extension_manager().contains_clash() {
            return false;
        }
        let level = self.state.borrow().current_branching_point;
        if level == 0 {
            return false;
        }
        self.backtrack_to(level - 1);
        let ni = self.ni_manager();
        if ni.start_next_ni_choice(
            self.state
                .borrow()
                .clash_dependency
                .clone()
                .unwrap_or_else(|| self.empty_dependency_set()),
        ) {
            self.state.borrow_mut().clear_clash();
            return true;
        }
        false
    }

    /// Saturate description-graph tuples (HermiT `runCalculus` graph fragment).
    pub fn saturate_description_graphs(&self) -> bool {
        graph_merge::saturate_graph_merges(self);
        !self.extension_manager().contains_clash()
    }
}

/// Mutable access to extension manager operations bound to a tableau.
pub struct ExtensionManagerRef {
    state: Rc<RefCell<TableauState>>,
}

impl ExtensionManagerRef {
    /// Whether a clash is present.
    #[must_use]
    pub fn contains_clash(&self) -> bool {
        self.state.borrow().contains_clash()
    }

    /// Binary extension table.
    #[must_use]
    pub fn binary_extension_table(&self) -> std::cell::Ref<'_, ExtensionTable> {
        std::cell::Ref::map(self.state.borrow(), |s| &s.binary_table)
    }

    /// Ternary extension table.
    #[must_use]
    pub fn ternary_extension_table(&self) -> std::cell::Ref<'_, ExtensionTable> {
        std::cell::Ref::map(self.state.borrow(), |s| &s.ternary_table)
    }

    /// Add concept assertion.
    pub fn add_concept_assertion(
        &self,
        concept: DlPredicate,
        node: &Node,
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
    ) {
        self.state
            .borrow_mut()
            .add_binary(concept, node, dependency, is_core);
    }

    /// Add `(predicate, node)` or role assertion helpers used in tests.
    pub fn add_assertion(
        &self,
        predicate: DlPredicate,
        node0: &Node,
        node1: Option<&Node>,
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
    ) {
        let mut state = self.state.borrow_mut();
        match (&predicate, node1) {
            (DlPredicate::Equality, Some(n1)) => {
                state.merge_nodes(node0, n1, dependency);
            }
            (DlPredicate::InverseRole(role), Some(n1)) => {
                state.add_ternary(
                    DlPredicate::AtomicRole(role),
                    n1,
                    node0,
                    dependency,
                    is_core,
                );
            }
            (_, None) => {
                state.add_binary(predicate, node0, dependency, is_core);
            }
            (_, Some(n1)) => {
                state.add_ternary(predicate, node0, n1, dependency, is_core);
            }
        }
    }

    /// HermiT `mergeNodes` exposed for NI manager.
    pub fn merge_nodes(
        &self,
        node0: &Node,
        node1: &Node,
        dependency: Rc<PermanentDependencySet>,
    ) -> bool {
        self.state
            .borrow_mut()
            .merge_nodes(node0, node1, dependency)
    }

    /// Take a fatal tableau engine error (tuple index / merge), if any.
    #[allow(dead_code)] // reserved for DL error propagation from tableau internals
    pub(crate) fn take_internal_error(&self) -> Option<crate::Error> {
        self.state.borrow_mut().take_tableau_error()
    }

    /// Node handle by id.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Node {
        self.state.borrow().nodes.borrow().node(id)
    }

    /// Canonical active representative.
    #[must_use]
    pub fn canonical(&self, node: &Node) -> Node {
        node.canonical_node(&self.state.borrow().nodes.borrow())
    }

    /// Whether a concept assertion is present on the canonical node.
    #[must_use]
    pub fn has_concept(&self, concept: &str, node: &Node) -> bool {
        let canonical_id = self.canonical(node).id();
        let state = self.state.borrow();
        let mut retrieval = state
            .binary_table
            .create_retrieval(&[false, true], ExtensionView::Total);
        retrieval.bindings_buffer()[1] = Some(DlObject::Node(canonical_id));
        retrieval.open();
        while !retrieval.after_last() {
            if let DlObject::Predicate(DlPredicate::AtomicConcept(c)) = retrieval.tuple_buffer()[0]
            {
                if c == concept {
                    return true;
                }
            }
            retrieval.next();
        }
        false
    }

    /// Whether a role assertion holds between active nodes.
    #[must_use]
    pub fn contains_assertion(&self, role: &str, from: &Node, to: &Node) -> bool {
        let from_id = self.canonical(from).id();
        let to_id = self.canonical(to).id();
        let state = self.state.borrow();
        let mut retrieval = state
            .ternary_table
            .create_retrieval(&[false, true, true], ExtensionView::Total);
        retrieval.bindings_buffer()[1] = Some(DlObject::Node(from_id));
        retrieval.bindings_buffer()[2] = Some(DlObject::Node(to_id));
        retrieval.open();
        while !retrieval.after_last() {
            if let DlObject::Predicate(DlPredicate::AtomicRole(r)) = &retrieval.tuple_buffer()[0] {
                if *r == role {
                    return true;
                }
            }
            retrieval.next();
        }
        false
    }

    /// Dependency set for a role assertion.
    #[must_use]
    pub fn get_assertion_dependency_set(
        &self,
        role: &str,
        from: &Node,
        to: &Node,
    ) -> Rc<PermanentDependencySet> {
        let from_id = self.canonical(from).id();
        let to_id = self.canonical(to).id();
        let state = self.state.borrow();
        let mut retrieval = state
            .ternary_table
            .create_retrieval(&[false, true, true], ExtensionView::Total);
        retrieval.bindings_buffer()[1] = Some(DlObject::Node(from_id));
        retrieval.bindings_buffer()[2] = Some(DlObject::Node(to_id));
        retrieval.open();
        while !retrieval.after_last() {
            if let DlObject::Predicate(DlPredicate::AtomicRole(r)) = &retrieval.tuple_buffer()[0] {
                if *r == role {
                    let tuple = retrieval.tuple_buffer();
                    return state
                        .ternary_table
                        .dependency_for_tuple(tuple)
                        .unwrap_or_else(|| state.empty_set());
                }
            }
            retrieval.next();
        }
        state.empty_set()
    }

    /// Role successors of `from` for atomic role.
    #[must_use]
    pub fn role_successors(&self, role: &str, from: &Node) -> Vec<Node> {
        let from_id = self.canonical(from).id();
        let state = self.state.borrow();
        let mut retrieval = state
            .ternary_table
            .create_retrieval(&[false, true, false], ExtensionView::Total);
        retrieval.bindings_buffer()[1] = Some(DlObject::Node(from_id));
        retrieval.open();
        let mut out = Vec::new();
        while !retrieval.after_last() {
            if let DlObject::Predicate(DlPredicate::AtomicRole(r)) = &retrieval.tuple_buffer()[0] {
                if *r == role {
                    if let DlObject::Node(to) = retrieval.tuple_buffer()[2] {
                        out.push(state.nodes.borrow().node(to));
                    }
                }
            }
            retrieval.next();
        }
        out
    }

    /// All `(from, to)` pairs for role.
    #[must_use]
    pub fn role_pairs(&self, role: &str) -> Vec<(NodeId, NodeId)> {
        let state = self.state.borrow();
        let mut retrieval = state
            .ternary_table
            .create_retrieval(&[false, false, false], ExtensionView::Total);
        retrieval.open();
        let mut out = Vec::new();
        while !retrieval.after_last() {
            if let DlObject::Predicate(DlPredicate::AtomicRole(r)) = &retrieval.tuple_buffer()[0] {
                if *r == role {
                    if let (DlObject::Node(from), DlObject::Node(to)) =
                        (&retrieval.tuple_buffer()[1], &retrieval.tuple_buffer()[2])
                    {
                        out.push((*from, *to));
                    }
                }
            }
            retrieval.next();
        }
        out
    }

    /// Add role assertion (HermiT `addRoleAssertion`).
    pub fn add_role_assertion(
        &self,
        role: &'static str,
        from: NodeId,
        to: NodeId,
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
    ) {
        let from_node = self.node(from);
        let to_node = self.node(to);
        self.add_assertion(
            DlPredicate::AtomicRole(role),
            &from_node,
            Some(&to_node),
            dependency,
            is_core,
        );
    }

    /// Active nodes in creation order.
    #[must_use]
    pub fn active_nodes(&self) -> Vec<Node> {
        let ids: Vec<NodeId> = {
            let state = self.state.borrow();
            let nodes = state.nodes.borrow();
            (0..nodes.nodes.len())
                .filter_map(|i| {
                    let id = NodeId(i);
                    let active = nodes.nodes[i].borrow().state == NodeState::Active;
                    active.then_some(id)
                })
                .collect()
        };
        ids.into_iter().map(|id| self.node(id)).collect()
    }

    /// Active tree nodes.
    #[must_use]
    pub fn active_tree_nodes(&self) -> Vec<Node> {
        self.active_nodes()
            .into_iter()
            .filter(|n| n.is_tree_node())
            .collect()
    }

    /// Direct tree children.
    #[must_use]
    pub fn tree_children(&self, parent: &Node) -> Vec<Node> {
        self.active_nodes()
            .into_iter()
            .filter(|n| n.parent_id() == Some(parent.id()))
            .collect()
    }

    /// Blocked successors of a node.
    #[must_use]
    pub fn blocked_successors(&self, parent: &Node) -> Vec<Node> {
        self.tree_children(parent)
            .into_iter()
            .filter(|n| n.is_directly_blocked(self))
            .collect()
    }

    /// Sorted atomic concept label for blocking.
    #[must_use]
    pub fn atomic_concept_label(&self, node: &Node) -> Vec<&'static str> {
        let state = self.state.borrow();
        let mut retrieval = state
            .binary_table
            .create_retrieval(&[false, true], ExtensionView::Total);
        retrieval.bindings_buffer()[1] = Some(DlObject::Node(node.id()));
        retrieval.open();
        let mut out = Vec::new();
        while !retrieval.after_last() {
            if let DlObject::Predicate(DlPredicate::AtomicConcept(c)) = retrieval.tuple_buffer()[0]
            {
                if retrieval.is_core() {
                    out.push(c);
                }
            }
            retrieval.next();
        }
        out.sort_unstable();
        out
    }

    /// Quaternary extension table (description graphs).
    #[must_use]
    pub fn quaternary_extension_table(&self) -> std::cell::Ref<'_, ExtensionTable> {
        std::cell::Ref::map(self.state.borrow(), |s| &s.quaternary_table)
    }

    /// Add a 4-tuple (graph, n0, n1, n2).
    pub fn add_quaternary_tuple(
        &self,
        graph: DescriptionGraphId,
        n0: &Node,
        n1: &Node,
        n2: &Node,
        dependency: Rc<PermanentDependencySet>,
        is_core: bool,
    ) {
        let tuple = [
            DlObject::Predicate(DlPredicate::DescriptionGraph(graph)),
            DlObject::Node(n0.id()),
            DlObject::Node(n1.id()),
            DlObject::Node(n2.id()),
        ];
        match self.state.borrow_mut().quaternary_table.add_tuple(
            &tuple,
            dependency,
            is_core,
            &mut |_, _| {},
        ) {
            Ok(_) => {}
            Err(err) => {
                self.state.borrow_mut().tuple_index_error = Some(err);
            }
        }
    }

    /// Whether a 4-tuple is present.
    #[must_use]
    pub fn contains_quaternary_tuple(
        &self,
        graph: DescriptionGraphId,
        n0: &Node,
        n1: &Node,
        n2: &Node,
    ) -> bool {
        let tuple = [
            DlObject::Predicate(DlPredicate::DescriptionGraph(graph)),
            DlObject::Node(n0.id()),
            DlObject::Node(n1.id()),
            DlObject::Node(n2.id()),
        ];
        self.state.borrow().quaternary_table.contains_tuple(&tuple)
    }
}

/// Test helpers matching HermiT `AbstractReasonerInternalsTest`.
pub mod test_helpers {
    use super::*;

    /// Build a tuple buffer for retrieval assertions.
    #[must_use]
    pub fn t(objects: Vec<DlObject>) -> Vec<DlObject> {
        objects
    }

    /// Assert extension retrieval equals expected tuples (order-independent).
    pub fn assert_retrieval(
        table: &ExtensionTable,
        search: &[Option<DlObject>],
        expected: &[Vec<DlObject>],
    ) {
        let binding_pattern: Vec<bool> = search.iter().map(|s| s.is_some()).collect();
        let mut retrieval = table.create_retrieval(&binding_pattern, ExtensionView::Total);
        for (slot, value) in search.iter().enumerate() {
            retrieval.bindings_buffer()[slot] = value.clone();
        }
        retrieval.open();
        let mut used = vec![false; expected.len()];
        while !retrieval.after_last() {
            let found = expected.iter().enumerate().any(|(i, exp)| {
                !used[i]
                    && exp.len() == retrieval.tuple_buffer().len()
                    && exp
                        .iter()
                        .zip(retrieval.tuple_buffer())
                        .all(|(a, b)| a == b)
            });
            assert!(found, "unexpected tuple {:?}", retrieval.tuple_buffer());
            for (i, exp) in expected.iter().enumerate() {
                if !used[i]
                    && exp.len() == retrieval.tuple_buffer().len()
                    && exp
                        .iter()
                        .zip(retrieval.tuple_buffer())
                        .all(|(a, b)| a == b)
                {
                    used[i] = true;
                    break;
                }
            }
            retrieval.next();
        }
        for (i, u) in used.iter().enumerate() {
            assert!(u, "expected tuple not retrieved: {:?}", expected[i]);
        }
    }

    /// Assert node label (positive/negated atomic and at-least concepts).
    pub(crate) fn assert_label(state: &TableauState, node: &Node, expected: &[DlPredicate]) {
        let mut retrieval = state
            .binary_table
            .create_retrieval(&[false, true], ExtensionView::Total);
        retrieval.bindings_buffer()[1] = Some(DlObject::Node(node.id()));
        retrieval.open();
        let mut actual = Vec::new();
        while !retrieval.after_last() {
            if let DlObject::Predicate(pred) = retrieval.tuple_buffer()[0].clone() {
                actual.push(pred);
            }
            retrieval.next();
        }
        assert_eq!(actual.len(), expected.len(), "label size mismatch");
        for exp in expected {
            assert!(actual.contains(exp), "missing {exp:?} in {actual:?}");
        }
    }
}
