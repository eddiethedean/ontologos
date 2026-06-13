//! DL clausal normal form (post-clausification).

use ontologos_core::{CeId, EntityId, RoleExpr};

/// A clause in DL normal form after clausification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    /// `C ⊑ D`.
    Subsumption { sub: CeId, sup: CeId },
    /// `C ⊓ D ⊑ E` encoded as binary steps during normalization.
    IntersectionSubsumption {
        left: CeId,
        right: CeId,
        sup: CeId,
    },
    /// `∃r.C ⊑ D`.
    Existential {
        property: RoleExpr,
        filler: CeId,
        sup: CeId,
    },
    /// `C ⊑ ∀r.D`.
    Universal {
        sub: CeId,
        property: RoleExpr,
        filler: CeId,
    },
    /// Disjointness `C ⊓ D ⊑ ⊥`.
    Disjoint { left: CeId, right: CeId },
    /// Role inclusion `r ⊑ s` (atomic).
    RoleSubsumption {
        sub: EntityId,
        sup: EntityId,
    },
    /// Complex role chain `r1 ∘ r2 ⊑ s`.
    RoleChain {
        chain: Vec<RoleExpr>,
        sup: EntityId,
    },
    /// Nominal equality hint `C ⊑ {a}`.
    NominalSubsumption {
        sub: CeId,
        individual: EntityId,
    },
}

/// Clause set produced by clausification.
#[derive(Debug, Clone, Default)]
pub struct ClauseSet {
    clauses: Vec<Clause>,
}

impl ClauseSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.clauses.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }
}
