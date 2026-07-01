//! Profile engine routing types (DIP boundary — no engine implementations).

/// Dispatch key for a profile-specific reasoning engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    /// OWL EL completion-based classification.
    El,
    /// RDFS materialization via reasonable.
    Rdfs,
    /// OWL RL forward-chaining saturation.
    Rl,
    /// OWL ALC tableau-lite classification.
    Alc,
    /// OWL 2 DL coupled saturation + tableau.
    Dl,
    /// DLSafe SWRL with DL classification.
    Swrl,
    /// Auto-detected hybrid ontology (multiple profile modules).
    Hybrid,
}

/// OWL 2 profile detected during Auto routing (mirrors `ontologos_profile::OwlProfile`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedProfileKind {
    /// OWL 2 EL.
    El,
    /// OWL 2 RL.
    Rl,
    /// OWL 2 QL.
    Ql,
    /// OWL 2 DL.
    Dl,
}

/// Result of profile → engine resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRoute {
    /// Engine to dispatch to.
    pub kind: EngineKind,
    /// Profile detected during Auto resolution, if applicable.
    pub detected: Option<DetectedProfileKind>,
}

impl ResolvedRoute {
    /// Build a route for an explicit profile selection.
    #[must_use]
    pub fn explicit(kind: EngineKind) -> Self {
        Self {
            kind,
            detected: None,
        }
    }

    /// Build a route from Auto detection.
    #[must_use]
    pub fn auto(kind: EngineKind, detected: DetectedProfileKind) -> Self {
        Self {
            kind,
            detected: Some(detected),
        }
    }
}

/// Whether class/property assertion entailment should use the DL tableau path.
#[must_use]
pub const fn uses_dl_entailment(kind: EngineKind) -> bool {
    matches!(
        kind,
        EngineKind::Alc | EngineKind::Dl | EngineKind::Swrl | EngineKind::Hybrid
    )
}
