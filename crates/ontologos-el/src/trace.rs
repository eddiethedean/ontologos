//! EL completion inference trace types.

use ontologos_core::{
    EntityId, InferenceTrace, Taxonomy, TraceConclusion, TracePremise, TraceStep,
};

/// EL completion rule identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElRule {
    /// Transitive subsumption propagation (forward).
    SubTransForward,
    /// Transitive subsumption propagation (backward).
    SubTransBackward,
    /// Existential propagation along filler subsumption.
    ExFillerSub,
    /// Existential propagation along subproperty.
    ExSubProp,
    /// Subproperty transitivity (forward).
    SubPropTransForward,
    /// Subproperty transitivity (backward).
    SubPropTransBackward,
    /// Existential propagation along superproperty.
    ExSuperProp,
    /// Domain propagation from `ObjectPropertyDomain`.
    PropertyDomain,
    /// Range propagation from `ObjectPropertyRange`.
    PropertyRange,
}

impl ElRule {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SubTransForward => "sub_trans_forward",
            Self::SubTransBackward => "sub_trans_backward",
            Self::ExFillerSub => "ex_filler_sub",
            Self::ExSubProp => "ex_sub_prop",
            Self::SubPropTransForward => "sub_prop_trans_forward",
            Self::SubPropTransBackward => "sub_prop_trans_backward",
            Self::ExSuperProp => "ex_super_prop",
            Self::PropertyDomain => "property_domain",
            Self::PropertyRange => "property_range",
        }
    }
}

/// EL classification report with optional inference trace.
#[derive(Debug, Clone)]
pub struct ElReport {
    /// Extracted taxonomy.
    pub taxonomy: Taxonomy,
    /// Recorded inference steps when trace recording is enabled.
    pub trace: InferenceTrace,
}

pub(crate) fn push_subsumption(
    trace: &mut InferenceTrace,
    rule: ElRule,
    premises: Vec<TracePremise>,
    sub: EntityId,
    sup: EntityId,
) {
    trace.push(TraceStep {
        rule: rule.as_str().to_string(),
        premises,
        conclusion: TraceConclusion::SubClassOf { sub, sup },
    });
}

pub(crate) fn push_subproperty(
    trace: &mut InferenceTrace,
    rule: ElRule,
    premises: Vec<TracePremise>,
    sub: EntityId,
    sup: EntityId,
) {
    trace.push(TraceStep {
        rule: rule.as_str().to_string(),
        premises,
        conclusion: TraceConclusion::SubObjectPropertyOf { sub, sup },
    });
}

pub(crate) fn push_existential(
    trace: &mut InferenceTrace,
    rule: ElRule,
    premises: Vec<TracePremise>,
    class: EntityId,
    property: EntityId,
    filler: EntityId,
) {
    trace.push(TraceStep {
        rule: rule.as_str().to_string(),
        premises,
        conclusion: TraceConclusion::Existential {
            class,
            property,
            filler,
        },
    });
}

pub(crate) fn subsumption_premise(sub: EntityId, sup: EntityId) -> TracePremise {
    TracePremise::SubClassOf { sub, sup }
}

pub(crate) fn existential_premise(
    class: EntityId,
    property: EntityId,
    filler: EntityId,
) -> TracePremise {
    TracePremise::Existential {
        class,
        property,
        filler,
    }
}

pub(crate) fn subproperty_premise(sub: EntityId, sup: EntityId) -> TracePremise {
    TracePremise::SubObjectPropertyOf { sub, sup }
}
