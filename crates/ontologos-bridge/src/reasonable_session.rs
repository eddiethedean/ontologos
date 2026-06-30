use std::any::Any;

use ontologos_core::{Ontology, OntologyRevision, Profile, Reasoner, ReasonerSession};
use reasonable::reasoner::Reasoner as ReasonableReasoner;

use crate::{
    Error, MergeLimits, MergeReport, apply_rdfs_fallbacks, apply_reasonable_fallbacks,
    core_to_triples, core_to_triples_for_axioms, merge_triples_into_ontology_with_limits, perf,
};

/// Persistent reasonable state for incremental RL/RDFS materialization.
pub struct ReasonableSession {
    reasoner: ReasonableReasoner,
    last_revision: OntologyRevision,
    warmed: bool,
    profile: Profile,
}

impl std::fmt::Debug for ReasonableSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasonableSession")
            .field("last_revision", &self.last_revision)
            .field("warmed", &self.warmed)
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}

impl ReasonableSession {
    /// Create an empty session for the given profile.
    #[must_use]
    pub fn new_for_profile(profile: Profile) -> Self {
        Self {
            reasoner: ReasonableReasoner::new(),
            last_revision: OntologyRevision::default(),
            warmed: false,
            profile,
        }
    }

    /// Create an empty RL session.
    #[must_use]
    pub fn new() -> Self {
        Self::new_for_profile(Profile::Rl)
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
        self.profile
    }

    fn clear(&mut self) {
        *self = Self::new_for_profile(self.profile);
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

fn session_stale(session: &ReasonableSession, ontology: &Ontology) -> bool {
    session.warmed && session.last_revision != ontology.revision()
}

pub type MaterializeSessionResult =
    std::result::Result<(MaterializeOutcome, ReasonableSession), Box<(Error, ReasonableSession)>>;

/// Materialize `ontology` using batch or incremental reasonable reasoning.
pub fn materialize_with_session(
    ontology: &mut Ontology,
    mut session: ReasonableSession,
    incremental: bool,
    limits: MergeLimits,
) -> MaterializeSessionResult {
    let perf = perf::perf_enabled();
    let mut timings = perf::BridgePerfTimings::default();
    let started = perf.then(std::time::Instant::now);

    let dirty = ontology.dirty().clone();
    let stale = session_stale(&session, ontology);

    if dirty.has_removals() {
        let _ = ontology.strip_inferred_axioms();
    }

    let use_incremental =
        incremental && session.warmed && dirty.is_dirty() && !dirty.has_removals() && !stale;

    let full_rebuild =
        !incremental || !session.warmed || dirty.has_removals() || stale || !use_incremental;

    if full_rebuild {
        let triples_timer = perf.then(|| perf::PhaseTimer::start(&mut timings.core_to_triples_s));
        let triples = match core_to_triples(ontology) {
            Ok(t) => t,
            Err(e) => return Err(Box::new((e, session))),
        };
        drop(triples_timer);
        session.reasoner = ReasonableReasoner::new();
        session.reasoner.load_triples(triples);
        let reason_timer = perf.then(|| perf::PhaseTimer::start(&mut timings.reasonable_reason_s));
        session.reasoner.reason();
        drop(reason_timer);
        session.warmed = true;
    } else if use_incremental {
        let triples_timer = perf.then(|| perf::PhaseTimer::start(&mut timings.core_to_triples_s));
        let delta = match core_to_triples_for_axioms(ontology, dirty.added()) {
            Ok(d) => d,
            Err(e) => return Err(Box::new((e, session))),
        };
        drop(triples_timer);
        if delta.is_empty() {
            ontology.clear_dirty();
            session.last_revision = ontology.revision();
            return Ok((
                MaterializeOutcome {
                    merge: MergeReport::default(),
                    full_rebuild: false,
                },
                session,
            ));
        }
        session.reasoner.load_triples(delta);
        let reason_timer = perf.then(|| perf::PhaseTimer::start(&mut timings.reasonable_reason_s));
        session.reasoner.reason();
        drop(reason_timer);
    } else {
        ontology.clear_dirty();
        session.last_revision = ontology.revision();
        return Ok((
            MaterializeOutcome {
                merge: MergeReport::default(),
                full_rebuild: false,
            },
            session,
        ));
    }

    let output = session.reasoner.view_output().to_vec();
    let diagnostics = session.reasoner.diagnostics();
    let merge_timer = perf.then(|| perf::PhaseTimer::start(&mut timings.merge_s));
    let merge =
        match merge_triples_into_ontology_with_limits(ontology, &output, diagnostics, limits) {
            Ok(m) => m,
            Err(e) => return Err(Box::new((e, session))),
        };
    drop(merge_timer);

    let post_timer = perf.then(|| perf::PhaseTimer::start(&mut timings.postprocess_s));
    let post = if session.profile == Profile::Rdfs {
        apply_rdfs_fallbacks(ontology)
    } else {
        apply_reasonable_fallbacks(ontology)
    };
    drop(post_timer);
    if let Err(e) = post {
        return Err(Box::new((e, session)));
    }

    if let Some(start) = started {
        timings.total_s = start.elapsed().as_secs_f64();
    }
    if perf {
        eprintln!("{}", serde_json::to_string(&timings).unwrap_or_default());
    }

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
pub fn take_reasonable_session(reasoner: &mut Reasoner, profile: Profile) -> ReasonableSession {
    reasoner
        .take_session()
        .and_then(|mut boxed| {
            boxed
                .as_any_mut()
                .downcast_mut::<ReasonableSession>()
                .map(std::mem::take)
        })
        .filter(|s| s.profile() == profile)
        .unwrap_or_else(|| ReasonableSession::new_for_profile(profile))
}

/// Downcast helper for [`Reasoner::session_mut`].
pub fn downcast_reasonable_session(
    session: &mut dyn ReasonerSession,
) -> Option<&mut ReasonableSession> {
    session.as_any_mut().downcast_mut::<ReasonableSession>()
}
