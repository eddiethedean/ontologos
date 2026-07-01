//! Profile engine routing types (DIP boundary — no engine implementations).

use crate::reasoner::Profile;

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
    /// Engine can run classification / materialization.
    pub classify: bool,
    /// Engine can check ontology consistency.
    pub consistency: bool,
    /// Engine can answer sub-object-property queries.
    pub role_query: bool,
    /// Engine uses DL tableau for class/property assertion entailment.
    pub entailment_dl: bool,
}

impl EngineCapabilities {
    /// EL completion engine capabilities.
    #[must_use]
    pub const fn el() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: false,
            entailment_dl: false,
        }
    }

    /// RDFS materialization engine capabilities.
    #[must_use]
    pub const fn rdfs() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: false,
            entailment_dl: false,
        }
    }

    /// OWL RL saturation engine capabilities.
    #[must_use]
    pub const fn rl() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: false,
            entailment_dl: false,
        }
    }

    /// ALC tableau engine capabilities.
    #[must_use]
    pub const fn alc() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: true,
            entailment_dl: true,
        }
    }

    /// DL hybrid engine capabilities.
    #[must_use]
    pub const fn dl() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: true,
            entailment_dl: true,
        }
    }

    /// SWRL + DL engine capabilities.
    #[must_use]
    pub const fn swrl() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: true,
            entailment_dl: true,
        }
    }

    /// Hybrid multi-module routing capabilities.
    #[must_use]
    pub const fn hybrid() -> Self {
        Self {
            classify: true,
            consistency: true,
            role_query: true,
            entailment_dl: true,
        }
    }

    /// Capabilities for an explicit [`Profile`] selection.
    #[must_use]
    pub fn for_profile(profile: Profile) -> Self {
        match profile {
            Profile::El => Self::el(),
            Profile::Rdfs => Self::rdfs(),
            Profile::Rl => Self::rl(),
            Profile::Alc => Self::alc(),
            Profile::Dl | Profile::DlPreview => Self::dl(),
            Profile::Swrl => Self::swrl(),
            Profile::Auto => Self::hybrid(),
        }
    }

    /// Capabilities for a resolved [`EngineKind`].
    #[must_use]
    pub fn for_kind(kind: EngineKind) -> Self {
        match kind {
            EngineKind::El => Self::el(),
            EngineKind::Rdfs => Self::rdfs(),
            EngineKind::Rl => Self::rl(),
            EngineKind::Alc => Self::alc(),
            EngineKind::Dl => Self::dl(),
            EngineKind::Swrl => Self::swrl(),
            EngineKind::Hybrid => Self::hybrid(),
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
    pub fn explicit(_profile: Profile, kind: EngineKind) -> Self {
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
