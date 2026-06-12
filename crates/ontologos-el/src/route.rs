use ontologos_core::{Profile, Reasoner, Taxonomy};
use ontologos_profile::{detect_profile, OwlProfile};
use ontologos_rdfs::{materialize_reasoner, MaterializationReport as RdfsReport};
use ontologos_rl::{MaterializationReport as RlReport, RlEngine};

use crate::{classify_reasoner, ElClassifier, Error};

/// Result of a profile-routed classification run.
#[derive(Debug)]
pub enum ClassifyOutcome {
    /// OWL EL taxonomy from completion-based classification.
    Taxonomy(Taxonomy),
    /// RDFS materialization report.
    Rdfs(RdfsReport),
    /// OWL RL saturation report.
    Rl(RlReport),
}

/// Run classification using the reasoner's configured profile.
pub fn classify_with_profile(reasoner: &mut Reasoner) -> Result<ClassifyOutcome, Error> {
    match reasoner.profile() {
        Profile::El => Ok(ClassifyOutcome::Taxonomy(classify_reasoner(reasoner)?)),
        Profile::Rdfs => Ok(ClassifyOutcome::Rdfs(materialize_reasoner(reasoner)?)),
        Profile::Rl => Ok(ClassifyOutcome::Rl(saturate_rl(reasoner)?)),
        Profile::Auto => classify_auto(reasoner),
    }
}

fn saturate_rl(reasoner: &mut Reasoner) -> Result<RlReport, Error> {
    if reasoner.profile() != Profile::Rl {
        return Err(Error::WrongProfile {
            expected: Profile::Rl,
            actual: reasoner.profile(),
        });
    }
    let parallelism = reasoner.config().parallelism;
    Ok(RlEngine::try_new(parallelism)?.saturate(reasoner.ontology_mut())?)
}

fn saturate_rl_unchecked(reasoner: &mut Reasoner) -> Result<RlReport, Error> {
    let parallelism = reasoner.config().parallelism;
    Ok(RlEngine::try_new(parallelism)?.saturate(reasoner.ontology_mut())?)
}

fn classify_auto(reasoner: &mut Reasoner) -> Result<ClassifyOutcome, Error> {
    let report = detect_profile(reasoner.ontology()).map_err(|e| Error::Profile(e.to_string()))?;
    let detected = report
        .detected
        .ok_or_else(|| Error::Profile("no profile detected".into()))?;

    match detected {
        OwlProfile::El | OwlProfile::Ql => Ok(ClassifyOutcome::Taxonomy(
            ElClassifier::new().classify(reasoner.ontology())?,
        )),
        OwlProfile::Rl => Ok(ClassifyOutcome::Rl(saturate_rl_unchecked(reasoner)?)),
        OwlProfile::Dl => Err(Error::UnsupportedProfile(detected)),
    }
}

/// Resolve an explicit CLI/API profile override against auto-detection.
pub fn resolve_profile_flag(flag: ProfileFlag) -> Profile {
    match flag {
        ProfileFlag::Auto => Profile::Auto,
        ProfileFlag::El => Profile::El,
        ProfileFlag::Rl => Profile::Rl,
        ProfileFlag::Rdfs => Profile::Rdfs,
    }
}

/// Explicit profile selection for CLI and bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileFlag {
    /// Detect profile automatically.
    Auto,
    /// OWL EL taxonomy classification.
    El,
    /// OWL RL saturation.
    Rl,
    /// RDFS materialization.
    Rdfs,
}
