//! DL clausal normal form (post-clausification).

use ontologos_core::{CeId, EntityId, RoleExpr};

/// A clause in DL normal form after clausification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    /// `C ⊑ D`.
    Subsumption {
        /// Subclass expression id.
        sub: CeId,
        /// Superclass expression id.
        sup: CeId,
    },
    /// `C ⊓ D ⊑ E` encoded as binary steps during normalization.
    IntersectionSubsumption {
        /// Left conjunct.
        left: CeId,
        /// Right conjunct.
        right: CeId,
        /// Superclass expression id.
        sup: CeId,
    },
    /// `∃r.C ⊑ D`.
    Existential {
        /// Object property (possibly inverse).
        property: RoleExpr,
        /// Filler class expression id.
        filler: CeId,
        /// Superclass expression id.
        sup: CeId,
    },
    /// `C ⊑ ∀r.D`.
    Universal {
        /// Subclass expression id.
        sub: CeId,
        /// Object property (possibly inverse).
        property: RoleExpr,
        /// Filler class expression id.
        filler: CeId,
    },
    /// Disjointness `C ⊓ D ⊑ ⊥`.
    Disjoint {
        /// Left disjoint class expression id.
        left: CeId,
        /// Right disjoint class expression id.
        right: CeId,
    },
    /// Role inclusion `r ⊑ s` (atomic).
    RoleSubsumption {
        /// Sub-role entity id.
        sub: EntityId,
        /// Super-role entity id.
        sup: EntityId,
    },
    /// Complex role chain `r1 ∘ r2 ⊑ s`.
    RoleChain {
        /// Chain of role expressions.
        chain: Vec<RoleExpr>,
        /// Super-role entity id.
        sup: EntityId,
    },
    /// Nominal equality hint `C ⊑ {a}`.
    NominalSubsumption {
        /// Class expression id.
        sub: CeId,
        /// Named individual entity id.
        individual: EntityId,
    },
}

/// Clause set produced by clausification.
#[derive(Debug, Clone, Default)]
pub struct ClauseSet {
    clauses: Vec<Clause>,
}

impl ClauseSet {
    /// Empty clause set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a clause.
    pub fn push(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    /// All clauses in insertion order.
    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }

    /// Number of clauses.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clauses.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }
}
