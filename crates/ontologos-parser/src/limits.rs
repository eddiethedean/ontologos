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
    /// When true, return an error if axioms or entities are skipped due to limits.
    pub strict: bool,
    /// When true, resolve and merge `owl:imports` for RDF/XML ontologies (default).
    pub merge_imports: bool,
}

impl Default for ParseLimits {
    fn default() -> Self {
        let max_file_bytes = 64 * 1024 * 1024;
        Self {
            max_file_bytes,
            max_axioms: 10_000_000,
            max_entities: 1_000_000,
            max_expanded_bytes: max_file_bytes.saturating_mul(4),
            strict: false,
            merge_imports: true,
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
            ..Self::default()
        }
    }
}
