//! Consistency check outcomes for production embedders.

/// Result of an ontology consistency check.
///
/// When [`complete`](Self::complete) is `false`, [`consistent`](Self::consistent) must not be
/// used as a proof — the reasoner hit a budget or resource limit before finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsistencyResult {
    /// Whether the ontology is consistent (only meaningful when [`complete`](Self::complete)).
    pub consistent: bool,
    /// Whether the check ran to completion without budget or tableau limits.
    pub complete: bool,
}

impl ConsistencyResult {
    /// Proved consistent.
    #[must_use]
    pub const fn consistent() -> Self {
        Self {
            consistent: true,
            complete: true,
        }
    }

    /// Proved inconsistent.
    #[must_use]
    pub const fn inconsistent() -> Self {
        Self {
            consistent: false,
            complete: true,
        }
    }

    /// Check did not finish; do not treat as consistent or inconsistent.
    #[must_use]
    pub const fn incomplete() -> Self {
        Self {
            consistent: false,
            complete: false,
        }
    }

    /// Legacy bool API: error if incomplete.
    pub fn into_bool(self) -> crate::Result<bool> {
        if !self.complete {
            return Err(crate::Error::Message(
                "consistency check incomplete (budget or tableau limit); use ConsistencyResult"
                    .into(),
            ));
        }
        Ok(self.consistent)
    }
}
