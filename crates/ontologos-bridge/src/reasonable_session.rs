use std::any::Any;

use ontologos_core::{Ontology, OntologyRevision, Profile, Reasoner, ReasonerSession};
use reasonable::reasoner::Reasoner as ReasonableReasoner;

use crate::{
    core_to_triples, core_to_triples_for_axioms, merge_triples_into_ontology_with_limits,
    MergeLimits, MergeReport, Result,
};

/// Persistent reasonable state for incremental RL/RDFS materialization.
pub struct ReasonableSession {
    reasoner: ReasonableReasoner,
    last_revision: OntologyRevision,
    warmed: bool,
}

impl std::fmt::Debug for ReasonableSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasonableSession")
            .field("last_revision", &self.last_revision)
            .field("warmed", &self.warmed)
            .finish_non_exhaustive()
    }
}

impl ReasonableSession {
    /// Create an empty session.
    #[must_use]
    pub fn new() -> Self {
        Self {
            reasoner: ReasonableReasoner::new(),
            last_revision: OntologyRevision::default(),
            warmed: false,
        }
    }

    /// Whether a prior materialization warmed this session.
    #[must_use]
    pub fn is_warmed(&self) -> bool {
        self.warmed
    }

    /// Revision of the ontology after the last successful materialize.
    #[must_use]
    pub fn last_revision(&self) -> OntologyRevision {
        self.last_revision
    }
}

impl Default for ReasonableSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ReasonerSession for ReasonableSession {
    fn profile(&self) -> Profile {
        Profile::Rl
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Outcome of a materialization pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializeOutcome {
    /// Axioms inferred during this pass.
    pub merge: MergeReport,
    /// Whether a full rematerialization ran (vs incremental delta).
    pub full_rebuild: bool,
}

/// Materialize `ontology` using batch or incremental reasonable reasoning.
pub fn materialize_with_session(
    ontology: &mut Ontology,
    mut session: ReasonableSession,
    incremental: bool,
    limits: MergeLimits,
) -> Result<(MaterializeOutcome, ReasonableSession)> {
    let dirty = ontology.dirty().clone();
    let use_incremental =
        incremental && session.warmed && dirty.is_dirty() && !dirty.has_removals();

    let full_rebuild = if !incremental || !session.warmed {
        true
    } else if dirty.has_removals() {
        let triples = core_to_triples(ontology)?;
        session.reasoner.set_base_triples(triples);
        session.reasoner.reason();
        false
    } else {
        !use_incremental
    };

    if full_rebuild {
        let triples = core_to_triples(ontology)?;
        session.reasoner = ReasonableReasoner::new();
        session.reasoner.load_triples(triples);
        session.reasoner.reason();
        session.warmed = true;
    } else if use_incremental {
        let delta = core_to_triples_for_axioms(ontology, dirty.added())?;
        if delta.is_empty() {
            return Ok((
                MaterializeOutcome {
                    merge: MergeReport::default(),
                    full_rebuild: false,
                },
                session,
            ));
        }
        session.reasoner.load_triples(delta);
        session.reasoner.reason();
    }

    let output = session.reasoner.view_output().to_vec();
    let diagnostics = session.reasoner.diagnostics();
    let merge = merge_triples_into_ontology_with_limits(ontology, &output, diagnostics, limits)?;

    session.last_revision = ontology.revision();
    ontology.clear_dirty();

    Ok((
        MaterializeOutcome {
            merge,
            full_rebuild,
        },
        session,
    ))
}

/// Extract a [`ReasonableSession`] from a core reasoner, or start a fresh one.
pub fn take_reasonable_session(reasoner: &mut Reasoner) -> ReasonableSession {
    reasoner
        .take_session()
        .and_then(|mut boxed| {
            boxed
                .as_any_mut()
                .downcast_mut::<ReasonableSession>()
                .map(std::mem::take)
        })
        .unwrap_or_default()
}

/// Downcast helper for [`Reasoner::session_mut`].
pub fn downcast_reasonable_session(
    session: &mut dyn ReasonerSession,
) -> Option<&mut ReasonableSession> {
    session.as_any_mut().downcast_mut::<ReasonableSession>()
}
