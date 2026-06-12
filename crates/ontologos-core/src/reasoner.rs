use crate::error::{Error, Result};
use crate::ontology::Ontology;

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
}

impl Default for ReasonerConfig {
    fn default() -> Self {
        Self {
            incremental: false,
            explanations: false,
            parallelism: 1,
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
        })
    }
}

/// Main reasoner facade over profile-specific engines.
#[derive(Debug)]
pub struct Reasoner {
    ontology: Ontology,
    profile: Profile,
    config: ReasonerConfig,
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
        &mut self.ontology
    }

    /// Run classification over the loaded ontology.
    ///
    /// Profile engines live in separate crates. Use `ontologos_rdfs::classify_reasoner`
    /// for [`Profile::Rdfs`] and `ontologos_rl::classify_reasoner` for [`Profile::Rl`].
    /// EL/Auto return [`Error::NotImplemented`]. Calling this with [`Profile::Rdfs`]
    /// or [`Profile::Rl`] returns [`Error::Message`] with a delegate hint.
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
