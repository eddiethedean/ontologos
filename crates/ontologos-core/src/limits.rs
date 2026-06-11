/// Resource limits for ontology JSON deserialization.
///
/// Use with [`Ontology::from_json_with_limits`](crate::Ontology::from_json_with_limits)
/// when parsing untrusted input. See the [security guide](https://github.com/eddiethedean/ontologos/blob/main/docs/security.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum JSON input size in bytes.
    pub max_json_bytes: usize,
    /// Maximum number of entities in a snapshot.
    pub max_entities: usize,
    /// Maximum number of axioms in a snapshot.
    pub max_axioms: usize,
    /// Maximum length of a single IRI string.
    pub max_iri_len: usize,
    /// Maximum operands in `EquivalentClasses` / `DisjointClasses`.
    pub max_class_operands: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_json_bytes: 16 * 1024 * 1024,
            max_entities: 1_000_000,
            max_axioms: 10_000_000,
            max_iri_len: 8_192,
            max_class_operands: 10_000,
        }
    }
}

/// Maximum class operands in equivalent/disjoint axioms (shared with axiom validation).
pub const MAX_CLASS_OPERANDS: usize = 10_000;
