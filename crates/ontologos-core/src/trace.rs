//! Engine-agnostic inference trace types for explanation generation.

use serde::{Deserialize, Serialize};

use crate::axiom::AxiomId;
use crate::entity::EntityId;

/// What an inference step concludes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TraceConclusion {
    /// A new or existing ontology axiom.
    Axiom {
        /// Concluded axiom id.
        id: AxiomId,
    },
    /// EL completion: `sub ⊑ sup`.
    SubClassOf {
        /// Subclass entity.
        sub: EntityId,
        /// Superclass entity.
        sup: EntityId,
    },
    /// EL completion: `class ⊑ ∃property.filler`.
    Existential {
        /// Subject class.
        class: EntityId,
        /// Object property.
        property: EntityId,
        /// Filler class.
        filler: EntityId,
    },
    /// EL completion: `subProperty ⊑ superProperty`.
    SubObjectPropertyOf {
        /// Sub property.
        sub: EntityId,
        /// Super property.
        sup: EntityId,
    },
}

/// A premise referenced by an inference step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TracePremise {
    /// An ontology axiom used as a premise.
    Axiom {
        /// Premise axiom id.
        id: AxiomId,
    },
    /// EL graph edge `sub ⊑ sup` used as a premise.
    SubClassOf {
        /// Subclass entity.
        sub: EntityId,
        /// Superclass entity.
        sup: EntityId,
    },
    /// EL graph edge `sub ⊑ ∃property.filler` used as a premise.
    Existential {
        /// Subject class.
        class: EntityId,
        /// Object property.
        property: EntityId,
        /// Filler class.
        filler: EntityId,
    },
    /// EL graph edge `subProperty ⊑ superProperty` used as a premise.
    SubObjectPropertyOf {
        /// Sub property.
        sub: EntityId,
        /// Super property.
        sup: EntityId,
    },
}

/// A single recorded inference step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceStep {
    /// Rule name (engine-specific snake_case identifier).
    pub rule: String,
    /// Premises for this step.
    pub premises: Vec<TracePremise>,
    /// What this step concludes.
    pub conclusion: TraceConclusion,
}

/// Ordered list of inference steps from a reasoning run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceTrace {
    /// Steps in derivation order.
    pub steps: Vec<TraceStep>,
}

impl InferenceTrace {
    /// Create an empty trace.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a step.
    pub fn push(&mut self, step: TraceStep) {
        self.steps.push(step);
    }
}
