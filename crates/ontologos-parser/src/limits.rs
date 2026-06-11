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
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024 * 1024,
            max_axioms: 10_000_000,
            max_entities: 1_000_000,
        }
    }
}
