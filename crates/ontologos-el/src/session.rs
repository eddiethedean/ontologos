use std::any::Any;

use ontologos_core::{Ontology, OntologyRevision, Profile, Reasoner, ReasonerSession};

use crate::graph::CompletionGraph;
use crate::partition::PartitionIndex;
use crate::trace::ElReport;
use crate::{normal_form, taxonomy_extract, ElClassifier};

const PARTITION_FALLBACK_FRACTION: f64 = 0.5;

/// Cached EL completion state for incremental classification.
pub struct ElSession {
    pub(crate) graph: CompletionGraph,
    pub(crate) partitions: PartitionIndex,
    pub(crate) last_revision: OntologyRevision,
    pub(crate) record_traces: bool,
}

impl Default for ElSession {
    fn default() -> Self {
        Self::new(CompletionGraph::default(), PartitionIndex::default(), false)
    }
}

impl std::fmt::Debug for ElSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElSession")
            .field("last_revision", &self.last_revision)
            .field("record_traces", &self.record_traces)
            .finish_non_exhaustive()
    }
}

impl ElSession {
    pub(crate) fn new(
        graph: CompletionGraph,
        partitions: PartitionIndex,
        record_traces: bool,
    ) -> Self {
        Self {
            graph,
            partitions,
            last_revision: OntologyRevision::default(),
            record_traces,
        }
    }
}

impl ReasonerSession for ElSession {
    fn profile(&self) -> Profile {
        Profile::El
    }

    fn clear(&mut self) {
        *self = Self::default();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Extract an [`ElSession`] from a core reasoner, if present.
pub fn take_el_session(reasoner: &mut Reasoner) -> Option<ElSession> {
    reasoner.take_session().and_then(|mut boxed| {
        boxed
            .as_any_mut()
            .downcast_mut::<ElSession>()
            .map(std::mem::take)
    })
}

fn session_matches(session: &ElSession, ontology: &Ontology) -> bool {
    session.last_revision == ontology.revision()
}

impl ElClassifier {
    /// Classify with incremental session when edits are small; falls back to full classify.
    pub fn classify_incremental(
        &self,
        ontology: &mut Ontology,
        session: Option<ElSession>,
        record_traces: bool,
    ) -> crate::Result<(ElReport, ElSession)> {
        normal_form::validate_el_profile(ontology)?;

        let dirty = ontology.dirty();
        if let Some(session) = session {
            if !dirty.is_dirty() && session_matches(&session, ontology) {
                let taxonomy = taxonomy_extract::extract_taxonomy(ontology, &session.graph);
                let trace = if record_traces && session.record_traces {
                    session.graph.clone().into_trace()
                } else {
                    Default::default()
                };
                let mut session = session;
                session.record_traces = record_traces;
                return Ok((ElReport { taxonomy, trace }, session));
            }

            let use_incremental =
                dirty.is_dirty() && !dirty.has_removals() && session_matches(&session, ontology);

            if use_incremental {
                let mut session = session;
                session.record_traces = record_traces;

                let sig = ontology.dirty_signatures();
                let partitions = session.partitions.partitions_for_signature(&sig);
                if session.partitions.affected_fraction(&partitions) > PARTITION_FALLBACK_FRACTION {
                    let mut graph = CompletionGraph::seed(ontology).with_traces(record_traces);
                    graph.saturate();
                    let taxonomy = taxonomy_extract::extract_taxonomy(ontology, &graph);
                    let trace = graph.clone().into_trace();
                    session.graph = graph;
                    session.partitions = PartitionIndex::build(ontology);
                    session.last_revision = ontology.revision();
                    ontology.clear_dirty();
                    return Ok((ElReport { taxonomy, trace }, session));
                }

                session.graph.overdelete_signature(&sig);
                session.graph.reseed_signature(ontology, &sig);
                session.graph.reseed_domains(ontology, &sig);
                session.graph.saturate();

                let taxonomy = taxonomy_extract::extract_taxonomy(ontology, &session.graph);
                let trace = session.graph.clone().into_trace();
                session.partitions = PartitionIndex::build(ontology);
                session.last_revision = ontology.revision();
                ontology.clear_dirty();

                return Ok((ElReport { taxonomy, trace }, session));
            }
        }

        let mut graph = CompletionGraph::seed(ontology).with_traces(record_traces);
        graph.saturate();
        let taxonomy = taxonomy_extract::extract_taxonomy(ontology, &graph);
        let trace = graph.clone().into_trace();
        let mut session = ElSession::new(graph, PartitionIndex::build(ontology), record_traces);
        session.last_revision = ontology.revision();
        ontology.clear_dirty();
        Ok((ElReport { taxonomy, trace }, session))
    }
}
