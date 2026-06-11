use ontologos_core::ParseMeta;

/// Parse diagnostics accumulated during mapping.
#[derive(Debug, Default)]
pub struct ParseReport {
    /// Underlying metadata attached to the ontology.
    pub meta: ParseMeta,
}

impl ParseReport {
    /// Create an empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Finalize metadata for attachment to an ontology.
    #[must_use]
    pub fn into_meta(self) -> ParseMeta {
        self.meta
    }
}
