/// Resource limits for OWL file parsing.
///
/// Enforced before allocating large in-memory structures. See `docs/security.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLimits {
    /// Maximum ontology file size in bytes.
    pub max_file_bytes: usize,
    /// Maximum axioms stored in the core model.
    pub max_axioms: usize,
    /// Maximum entities registered during parse.
    pub max_entities: usize,
    /// Maximum expanded RDF/XML size after entity expansion (defaults to 4× `max_file_bytes`).
    pub max_expanded_bytes: usize,
    /// Cumulative allocation budget across RDF/XML preprocessing stages.
    pub max_preprocess_bytes: usize,
    /// Maximum harvested RDF/XML assertions converted to OFN supplements per load.
    pub max_harvested_assertions: usize,
    /// When true, return an error if axioms or entities are skipped due to limits.
    pub strict: bool,
    /// When true, resolve and merge `owl:imports` for RDF/XML ontologies (default).
    pub merge_imports: bool,
    /// When true, run post-load validation on successful parses.
    pub validate_output: bool,
}

impl Default for ParseLimits {
    fn default() -> Self {
        let max_file_bytes = 64 * 1024 * 1024;
        Self {
            max_file_bytes,
            max_axioms: 10_000_000,
            max_entities: 1_000_000,
            max_expanded_bytes: max_file_bytes.saturating_mul(4),
            max_preprocess_bytes: max_file_bytes.saturating_mul(8),
            max_harvested_assertions: 100_000,
            strict: true,
            merge_imports: true,
            validate_output: true,
        }
    }
}

impl ParseLimits {
    /// Build limits with `max_expanded_bytes` derived from `max_file_bytes`.
    #[must_use]
    pub fn with_file_bytes(max_file_bytes: usize) -> Self {
        Self {
            max_file_bytes,
            max_expanded_bytes: max_file_bytes.saturating_mul(4),
            max_preprocess_bytes: max_file_bytes.saturating_mul(8),
            ..Self::default()
        }
    }

    /// Lenient limits: allow skipped axioms and incompatible declarations to warn instead of error.
    #[must_use]
    pub fn lenient() -> Self {
        Self {
            strict: false,
            validate_output: false,
            ..Self::default()
        }
    }
}

impl From<ParseLimits> for ontologos_core::Limits {
    fn from(limits: ParseLimits) -> Self {
        Self {
            max_entities: limits.max_entities,
            max_axioms: limits.max_axioms,
            ..Self::default()
        }
    }
}
