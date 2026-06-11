use crate::error::{Error, Result};
use crate::ontology::Ontology;

/// OWL profile selected for reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    #[default]
    Auto,
    Rdfs,
    Rl,
    El,
}

/// Configuration options for the reasoner builder.
#[derive(Debug, Clone)]
pub struct ReasonerConfig {
    pub incremental: bool,
    pub explanations: bool,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }

    #[must_use]
    pub fn config(mut self, config: ReasonerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self, ontology: Ontology) -> Result<Reasoner> {
        Ok(Reasoner {
            ontology,
            profile: self.profile,
            config: self.config,
        })
    }
}

/// Main reasoner facade over profile-specific engines.
pub struct Reasoner {
    ontology: Ontology,
    profile: Profile,
    config: ReasonerConfig,
}

impl Reasoner {
    #[must_use]
    pub fn builder() -> ReasonerBuilder {
        ReasonerBuilder::new()
    }

    #[must_use]
    pub fn profile(&self) -> Profile {
        self.profile
    }

    #[must_use]
    pub fn config(&self) -> &ReasonerConfig {
        &self.config
    }

    #[must_use]
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }

    /// Run classification over the loaded ontology.
    pub fn classify(&self) -> Result<()> {
        let _ = &self.ontology;
        let _ = self.profile;
        let _ = &self.config;
        Err(Error::NotImplemented)
    }
}
