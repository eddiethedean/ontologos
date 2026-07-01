use crate::dirty::OntologyRevision;
use crate::error::{Error, Result};
use crate::ontology::Ontology;
use crate::taxonomy::Taxonomy;

const MAX_PARALLELISM: usize = 64;

/// OWL profile selected for reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    /// Detect the most specific supported profile automatically.
    #[default]
    Auto,
    /// RDFS reasoning only.
    Rdfs,
    /// OWL RL rule-based reasoning.
    Rl,
    /// OWL EL completion-based classification.
    El,
    /// OWL ALC tableau-lite (pre-DL bridge).
    Alc,
    /// OWL 2 DL coupled saturation + tableau.
    Dl,
    /// OWL 2 DL preview mode (gated subset checks; same engine as [`Dl`](Self::Dl)).
    DlPreview,
    /// DLSafe SWRL rules with DL.
    Swrl,
}

/// Configuration options for the reasoner builder.
#[derive(Debug, Clone)]
pub struct ReasonerConfig {
    /// Enable incremental re-classification when axioms change.
    pub incremental: bool,
    /// Record explanations for inferences.
    pub explanations: bool,
    /// Number of threads for parallel rule execution.
    pub parallelism: usize,
    /// Optional wall-clock budget (seconds) for DL consistency/classify paths.
    /// Falls back to `ONTOLOGOS_DL_BUDGET_SECS` when unset.
    pub budget_secs: Option<u64>,
}

impl Default for ReasonerConfig {
    fn default() -> Self {
        Self {
            incremental: false,
            explanations: false,
            parallelism: 1,
            budget_secs: None,
        }
    }
}

/// Builder for constructing a configured reasoner instance.
#[derive(Debug, Default)]
pub struct ReasonerBuilder {
    profile: Profile,
    config: ReasonerConfig,
}

impl ReasonerBuilder {
    /// Create a builder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the OWL profile for reasoning.
    #[must_use]
    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }

    /// Set reasoner configuration options.
    #[must_use]
    pub fn config(mut self, config: ReasonerConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a reasoner over the given ontology.
    pub fn build(self, ontology: Ontology) -> Result<Reasoner> {
        if self.config.parallelism == 0 || self.config.parallelism > MAX_PARALLELISM {
            return Err(Error::Message(format!(
                "parallelism must be in 1..={MAX_PARALLELISM}, got {}",
                self.config.parallelism
            )));
        }
        Ok(Reasoner {
            ontology,
            profile: self.profile,
            config: self.config,
            session: None,
            classify_cache: None,
        })
    }
}

/// Cached classification result keyed by ontology revision.
#[derive(Debug, Clone)]
pub struct ReasonerCache {
    revision: OntologyRevision,
    taxonomy: Taxonomy,
}

/// Main reasoner facade over profile-specific engines.
#[derive(Debug)]
pub struct Reasoner {
    ontology: Ontology,
    profile: Profile,
    config: ReasonerConfig,
    session: Option<Box<dyn crate::session::ReasonerSession>>,
    classify_cache: Option<ReasonerCache>,
}

impl Reasoner {
    /// Create a new reasoner builder.
    #[must_use]
    pub fn builder() -> ReasonerBuilder {
        ReasonerBuilder::new()
    }

    /// The configured OWL profile.
    #[must_use]
    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// The reasoner configuration.
    #[must_use]
    pub fn config(&self) -> &ReasonerConfig {
        &self.config
    }

    /// Borrow the loaded ontology.
    #[must_use]
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }

    /// Mutably borrow the loaded ontology (e.g. for RDFS materialization).
    pub fn ontology_mut(&mut self) -> &mut Ontology {
        self.invalidate_classify_cache();
        &mut self.ontology
    }

    /// Borrow incremental session state, if any.
    #[must_use]
    pub fn session(&self) -> Option<&dyn crate::session::ReasonerSession> {
        self.session.as_deref()
    }

    /// Mutably borrow incremental session state.
    pub fn session_mut(&mut self) -> Option<&mut dyn crate::session::ReasonerSession> {
        self.session.as_deref_mut()
    }

    /// Take session state out of the reasoner (for facade downcasting).
    pub fn take_session(&mut self) -> Option<Box<dyn crate::session::ReasonerSession>> {
        self.session.take()
    }

    /// Install or replace incremental session state.
    pub fn set_session(&mut self, session: Box<dyn crate::session::ReasonerSession>) {
        self.session = Some(session);
    }

    /// Clear incremental session state.
    pub fn clear_session(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.clear();
        }
        self.session = None;
    }

    /// Borrow cached taxonomy when revision matches the loaded ontology.
    #[must_use]
    pub fn cached_taxonomy(&self) -> Option<&Taxonomy> {
        let cache = self.classify_cache.as_ref()?;
        if cache.revision == self.ontology.revision() {
            Some(&cache.taxonomy)
        } else {
            None
        }
    }

    /// Store a taxonomy from the latest classification for entailment reuse.
    pub fn set_cached_taxonomy(&mut self, taxonomy: Taxonomy) {
        self.classify_cache = Some(ReasonerCache {
            revision: self.ontology.revision(),
            taxonomy,
        });
    }

    /// Drop cached classification results.
    pub fn invalidate_classify_cache(&mut self) {
        self.classify_cache = None;
    }

    /// Check ontology consistency (profile engines implement semantics).
    #[deprecated(
        since = "1.0.0",
        note = "use ontologos_facade::check_consistency or ontologos_facade::is_consistent instead"
    )]
    pub fn is_consistent(&self) -> Result<bool> {
        Err(Error::NotImplemented)
    }

    /// Run classification over the loaded ontology.
    ///
    /// Profile engines live in separate crates. Prefer [`ontologos_facade::classify`]
    /// or profile-specific helpers (`ontologos_el::classify_with_profile`,
    /// `ontologos_rdfs::classify_reasoner`, `ontologos_rl::classify_reasoner`).
    ///
    /// EL/Auto return [`Error::NotImplemented`]. [`Profile::Rdfs`] and [`Profile::Rl`]
    /// return [`Error::Message`] with a delegate hint.
    #[deprecated(
        since = "1.0.0",
        note = "use ontologos_facade::classify or profile crate helpers instead"
    )]
    pub fn classify(&mut self) -> Result<()> {
        if self.profile == Profile::Rdfs {
            return Err(Error::Message(
                "Profile::Rdfs: use ontologos_rdfs::classify_reasoner or \
                 ontologos_rdfs::materialize_reasoner; Reasoner::classify does not \
                 dispatch to profile engines"
                    .into(),
            ));
        }
        if self.profile == Profile::Rl {
            return Err(Error::Message(
                "Profile::Rl: use ontologos_rl::classify_reasoner or \
                 ontologos_rl::materialize_reasoner; Reasoner::classify does not \
                 dispatch to profile engines"
                    .into(),
            ));
        }
        Err(Error::NotImplemented)
    }
}
