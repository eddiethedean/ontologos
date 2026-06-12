use std::collections::{BTreeMap, HashSet};

use ontologos_core::{AxiomId, EntityId};
use serde::Serialize;

/// OWL RL rule that produced an inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RlRule {
    /// Equivalent classes imply mutual `SubClassOf`.
    EqClassSub,
    /// Equivalent properties imply mutual `SubObjectPropertyOf`.
    EqPropSub,
    /// Property characteristics propagate along `subPropertyOf`.
    CharPropagate,
    /// Existential restriction propagates along `subPropertyOf`.
    ExistentialSubProp,
    /// Class with `∃P.D` subsumed by class with `∃Q.D` when `P ⊑ Q`.
    ExistentialSubsumption,
    /// Domain typing from property assertions.
    TypeDomain,
    /// Range typing from property assertions.
    TypeRange,
    /// Class assertion propagates along `subClassOf`.
    TypeSubclass,
    /// Class assertion propagates along class equivalence.
    TypeEquivalent,
    /// Property assertion propagates along `subPropertyOf`.
    PropSub,
    /// Property assertion via inverse.
    PropInverse,
    /// Property assertion via symmetry.
    PropSymmetric,
    /// Transitive closure on property assertions.
    PropTransitive,
    /// `sameAs` propagation on class assertions.
    SameAsClass,
    /// `sameAs` propagation on property assertions.
    SameAsProp,
    /// Reflexive property self-assertion.
    PropReflexive,
}

impl RlRule {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EqClassSub => "eq_class_sub",
            Self::EqPropSub => "eq_prop_sub",
            Self::CharPropagate => "char_propagate",
            Self::ExistentialSubProp => "existential_sub_prop",
            Self::ExistentialSubsumption => "existential_subsumption",
            Self::TypeDomain => "type_domain",
            Self::TypeRange => "type_range",
            Self::TypeSubclass => "type_subclass",
            Self::TypeEquivalent => "type_equivalent",
            Self::PropSub => "prop_sub",
            Self::PropInverse => "prop_inverse",
            Self::PropSymmetric => "prop_symmetric",
            Self::PropTransitive => "prop_transitive",
            Self::SameAsClass => "same_as_class",
            Self::SameAsProp => "same_as_prop",
            Self::PropReflexive => "prop_reflexive",
        }
    }
}

/// A single recorded inference (optional trace for explain v0.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InferenceRecord {
    pub rule: RlRule,
    pub premises: Vec<AxiomId>,
    pub conclusion: AxiomId,
}

/// Summary of RL saturation over an ontology (includes prior RDFS pass).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MaterializationReport {
    pub initial_axiom_count: usize,
    pub final_axiom_count: usize,
    pub rdfs_inferred: usize,
    pub inferred_by_rule: BTreeMap<RlRule, usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traces: Vec<InferenceRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub clashes: Vec<String>,
    /// Canonical disjoint-clash keys already reported (not serialized).
    #[serde(skip)]
    pub disjoint_clash_keys: HashSet<(EntityId, EntityId, EntityId)>,
    /// Canonical sameAs/differentFrom clash pairs already reported (not serialized).
    #[serde(skip)]
    pub same_as_clash_keys: HashSet<(EntityId, EntityId)>,
}

impl MaterializationReport {
    #[must_use]
    pub fn inferred_total(&self) -> usize {
        self.final_axiom_count
            .saturating_sub(self.initial_axiom_count)
    }
}
