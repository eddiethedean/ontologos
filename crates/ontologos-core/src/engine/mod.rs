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

/// Operations supported by the resolved engine route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineCapabilities {
    /// Engine can answer sub-object-property queries via DL/ALC saturation.
    pub role_query: bool,
    /// Engine uses DL tableau for class/property assertion entailment.
    pub entailment_dl: bool,
}

impl EngineCapabilities {
    /// Capabilities for a resolved [`EngineKind`].
    #[must_use]
    pub const fn for_kind(kind: EngineKind) -> Self {
        match kind {
            EngineKind::El | EngineKind::Rdfs | EngineKind::Rl => Self {
                role_query: false,
                entailment_dl: false,
            },
            EngineKind::Alc | EngineKind::Dl | EngineKind::Swrl | EngineKind::Hybrid => Self {
                role_query: true,
                entailment_dl: true,
            },
        }
    }
}

/// Result of profile → engine resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRoute {
    /// Engine to dispatch to.
    pub kind: EngineKind,
    /// Supported operations for this route.
    pub capabilities: EngineCapabilities,
    /// Profile detected during Auto resolution, if applicable.
    pub detected: Option<DetectedProfileKind>,
}

impl ResolvedRoute {
    /// Build a route for an explicit profile selection.
    #[must_use]
    pub fn explicit(kind: EngineKind) -> Self {
        Self {
            kind,
            capabilities: EngineCapabilities::for_kind(kind),
            detected: None,
        }
    }

    /// Build a route from Auto detection.
    #[must_use]
    pub fn auto(kind: EngineKind, detected: DetectedProfileKind) -> Self {
        Self {
            kind,
            capabilities: EngineCapabilities::for_kind(kind),
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

/// Whether sub-object-property queries use DL/ALC saturation.
#[must_use]
pub const fn uses_dl_role_query(kind: EngineKind) -> bool {
    uses_dl_entailment(kind)
}
