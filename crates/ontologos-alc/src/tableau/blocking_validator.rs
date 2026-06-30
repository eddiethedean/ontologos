//! HermiT validated blocking checker + validator (internal test port).
#![allow(dead_code)]

use std::collections::HashSet;

use super::dl_clause_eval::DlAtom;
use super::extension_manager::{ExtensionManagerRef, Node, NodeId};
use super::ni_rules::AnnotatedEquality;

/// DL clause indexed for blocking validation.
#[derive(Debug, Clone)]
pub struct DlClauseInfo {
    /// Head annotated equalities.
    pub head_equalities: Vec<AnnotatedEquality>,
    /// Body concept requirements on X.
    pub x_concepts: Vec<&'static str>,
    /// Body role requirements X→Y.
    pub x_to_y_roles: Vec<&'static str>,
    /// Body concept requirements on Y nodes.
    pub y_concepts: Vec<&'static str>,
}

/// HermiT `BlockingValidator`.
pub struct BlockingValidator {
    clauses: Vec<DlClauseInfo>,
}

impl BlockingValidator {
    /// Build validator from DL clauses.
    #[must_use]
    pub fn new(clauses: Vec<DlClauseInfo>) -> Self {
        Self { clauses }
    }

    /// HermiT `isBlockValid`.
    #[must_use]
    pub fn is_block_valid(&self, ext: &ExtensionManagerRef, blocked: &Node) -> bool {
        let Some(blocker) = blocked.blocker(ext) else {
            return true;
        };
        let parent = blocked.parent_node(ext);
        if !parent.is_parent_checked() {
            self.check_constraints_for_nonblocked_x(ext, &parent);
            parent.set_parent_checked(true);
        }
        if blocked.block_violates_parent_constraints() {
            return false;
        }
        self.satisfies_constraints_for_blocked_x(ext, blocked, &blocker)
    }

    fn check_constraints_for_nonblocked_x(&self, ext: &ExtensionManagerRef, node: &Node) {
        for child in ext.tree_children(node) {
            child.set_block_violates_parent(false);
        }
        for clause in &self.clauses {
            self.check_clause_for_nonblocked_x(ext, clause, node);
        }
    }

    fn check_clause_for_nonblocked_x(
        &self,
        ext: &ExtensionManagerRef,
        clause: &DlClauseInfo,
        x: &Node,
    ) {
        for x_concept in &clause.x_concepts {
            if !ext.has_concept(x_concept, x) {
                return;
            }
        }
        let y_candidates = self.y_witnesses(ext, clause, x);
        if y_candidates.len() < 2 {
            return;
        }
        for eq in &clause.head_equalities {
            if !self.annotated_equality_satisfied(ext, *eq, &y_candidates) {
                for child in ext.blocked_successors(x) {
                    child.set_block_violates_parent(true);
                }
            }
        }
    }

    fn satisfies_constraints_for_blocked_x(
        &self,
        ext: &ExtensionManagerRef,
        blocked: &Node,
        blocker: &Node,
    ) -> bool {
        for clause in &self.clauses {
            if !self.satisfies_clause_for_blocked_x(ext, clause, blocked, blocker) {
                return false;
            }
        }
        true
    }

    fn satisfies_clause_for_blocked_x(
        &self,
        ext: &ExtensionManagerRef,
        clause: &DlClauseInfo,
        blocked: &Node,
        blocker: &Node,
    ) -> bool {
        for x_concept in &clause.x_concepts {
            if !ext.has_concept(x_concept, blocker) {
                return true;
            }
        }
        let parent = blocked.parent_node(ext);
        if !clause
            .y_concepts
            .iter()
            .all(|concept| ext.has_concept(concept, &parent))
        {
            return true;
        }
        for eq in &clause.head_equalities {
            let witnesses = self.collect_witnesses_for_blocked(ext, *eq, blocker, blocked);
            if witnesses.len() >= eq.cardinality as usize {
                return true;
            }
        }
        false
    }

    fn y_witnesses(&self, ext: &ExtensionManagerRef, clause: &DlClauseInfo, x: &Node) -> Vec<Node> {
        let mut out = Vec::new();
        for role in &clause.x_to_y_roles {
            for succ in ext.role_successors(role, x) {
                if clause.y_concepts.iter().all(|c| ext.has_concept(c, &succ)) {
                    out.push(succ);
                }
            }
        }
        out
    }

    fn annotated_equality_satisfied(
        &self,
        ext: &ExtensionManagerRef,
        eq: AnnotatedEquality,
        witnesses: &[Node],
    ) -> bool {
        let matching = witnesses
            .iter()
            .filter(|w| ext.has_concept(eq.concept, w))
            .count();
        matching <= eq.cardinality as usize
    }

    fn collect_witnesses_for_blocked(
        &self,
        ext: &ExtensionManagerRef,
        eq: AnnotatedEquality,
        blocker: &Node,
        blocked: &Node,
    ) -> Vec<NodeId> {
        let mut set = HashSet::new();
        for succ in ext.role_successors(eq.role, blocker) {
            if ext.has_concept(eq.concept, &succ) {
                set.insert(succ.id());
            }
        }
        let _ = blocked;
        set.into_iter().collect()
    }
}

/// HermiT `AnywhereValidatedBlocking` direct blocking computation.
pub struct BlockingStrategy {
    #[allow(dead_code)]
    has_inverses: bool,
}

impl BlockingStrategy {
    /// Create validated anywhere blocking.
    #[must_use]
    pub fn new(has_inverses: bool) -> Self {
        Self { has_inverses }
    }

    /// HermiT `computeBlocking`.
    pub fn compute_blocking(&self, ext: &ExtensionManagerRef) {
        for node in ext.active_nodes() {
            node.clear_blocking_state(ext);
        }
        let tree_nodes: Vec<Node> = ext.active_tree_nodes();
        for node in tree_nodes {
            if !self.can_be_blocked(&node, ext) {
                continue;
            }
            if let Some(blocker) = self.find_blocker(&node, ext) {
                node.set_directly_blocked(blocker.id(), ext);
            }
        }
    }

    fn can_be_blocked(&self, node: &Node, _ext: &ExtensionManagerRef) -> bool {
        node.is_tree_node() && node.parent_id().is_some()
    }

    fn find_blocker(&self, blocked: &Node, ext: &ExtensionManagerRef) -> Option<Node> {
        let blocked_label = ext.atomic_concept_label(blocked);
        for candidate in ext.active_tree_nodes() {
            if candidate.id() == blocked.id() {
                continue;
            }
            if !self.can_be_blocker(&candidate, ext) {
                continue;
            }
            if candidate.is_blocked(ext) {
                continue;
            }
            if ext.atomic_concept_label(&candidate) == blocked_label {
                return Some(candidate);
            }
        }
        None
    }

    fn can_be_blocker(&self, node: &Node, _ext: &ExtensionManagerRef) -> bool {
        node.is_tree_node()
    }
}

/// Clauses for `BlockingValidatorTest.testOneInvalidBlock`.
#[must_use]
pub fn blocking_test_one_invalid_block_clauses() -> Vec<DlClauseInfo> {
    vec![DlClauseInfo {
        head_equalities: vec![AnnotatedEquality::new(1, "R", "D")],
        x_concepts: vec!["C"],
        x_to_y_roles: vec!["R"],
        y_concepts: vec!["D"],
    }]
}

/// Clauses for `BlockingValidatorTest.testInvalidBlockWithAnnotatedEqualities`.
#[must_use]
pub fn blocking_test_annotated_equalities_clauses() -> Vec<DlClauseInfo> {
    vec![DlClauseInfo {
        head_equalities: vec![AnnotatedEquality::new(1, "R", "C")],
        x_concepts: vec!["B"],
        x_to_y_roles: vec!["R"],
        y_concepts: vec!["C"],
    }]
}

/// Build clause info from generic DL atoms (test helper).
#[must_use]
#[allow(dead_code)]
pub fn clause_info_from_atoms(head: &[DlAtom], body: &[DlAtom]) -> DlClauseInfo {
    let mut x_concepts = Vec::new();
    let mut x_to_y_roles = Vec::new();
    let mut y_concepts = Vec::new();
    for atom in body {
        match atom {
            DlAtom::Concept(c, super::dl_clause_eval::VarSlot::X) => x_concepts.push(*c),
            DlAtom::Role(
                r,
                super::dl_clause_eval::VarSlot::X,
                super::dl_clause_eval::VarSlot::Y,
            ) => {
                x_to_y_roles.push(*r);
            }
            DlAtom::Concept(c, super::dl_clause_eval::VarSlot::Y) => y_concepts.push(*c),
            _ => {}
        }
    }
    let _ = head;
    DlClauseInfo {
        head_equalities: Vec::new(),
        x_concepts,
        x_to_y_roles,
        y_concepts,
    }
}

/// Role reference with optional inverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleRef {
    /// Atomic role.
    Atomic(&'static str),
    /// Inverse of atomic role.
    Inverse(&'static str),
}

/// At-least concept for blocking tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct AtLeastConcept {
    /// Minimum cardinality.
    pub n: u32,
    /// Role (possibly inverse).
    pub role: RoleRef,
    /// Filler concept.
    pub concept: &'static str,
}

/// Known at-least concepts used in blocking tests.
pub mod blocking_concepts {
    use super::RoleRef;
    use crate::tableau::extension_manager::DlPredicate;

    fn at_least(n: u32, role: RoleRef, concept: &'static str) -> DlPredicate {
        DlPredicate::AtLeastConcept {
            n,
            role,
            filler: Box::new(DlPredicate::AtomicConcept(concept)),
        }
    }

    /// ∃≥2 R.A
    #[must_use]
    pub fn at_least_2_r_a() -> DlPredicate {
        at_least(2, RoleRef::Atomic("R"), "A")
    }

    /// ∃≥2 R⁻.B
    #[must_use]
    pub fn at_least_2_inv_r_b() -> DlPredicate {
        at_least(2, RoleRef::Inverse("R"), "B")
    }

    /// ∃≥1 R⁻.E
    #[must_use]
    pub fn at_least_1_inv_r_e() -> DlPredicate {
        at_least(1, RoleRef::Inverse("R"), "E")
    }

    /// ∃≥1 S.A
    #[must_use]
    pub fn at_least_1_s_a() -> DlPredicate {
        at_least(1, RoleRef::Atomic("S"), "A")
    }

    /// ∃≥1 S.B
    #[must_use]
    pub fn at_least_1_s_b() -> DlPredicate {
        at_least(1, RoleRef::Atomic("S"), "B")
    }

    /// ∃≥1 R.C
    #[must_use]
    pub fn at_least_1_r_c() -> DlPredicate {
        at_least(1, RoleRef::Atomic("R"), "C")
    }

    /// ∃≥1 T.D
    #[must_use]
    pub fn at_least_1_t_d() -> DlPredicate {
        at_least(1, RoleRef::Atomic("T"), "D")
    }

    /// ∃≥1 R⁻.B
    #[must_use]
    pub fn at_least_1_inv_r_b() -> DlPredicate {
        at_least(1, RoleRef::Inverse("R"), "B")
    }
}
