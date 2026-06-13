use ontologos_core::{Profile, Reasoner, Taxonomy};
use ontologos_profile::{detect_profile, OwlProfile};
use ontologos_rdfs::{materialize_reasoner, MaterializationReport as RdfsReport};
use ontologos_rl::{MaterializationReport as RlReport, RlEngine};

use crate::{ElClassifier, Error};

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
        Profile::El => Ok(ClassifyOutcome::Taxonomy(
            crate::classify_with_report(reasoner)?.taxonomy,
        )),
        Profile::Rdfs => Ok(ClassifyOutcome::Rdfs(materialize_reasoner(reasoner)?)),
        Profile::Rl => Ok(ClassifyOutcome::Rl(saturate_rl(reasoner)?)),
        Profile::Alc => Ok(ClassifyOutcome::Taxonomy(
            ontologos_alc::classify(reasoner.ontology())
                .map_err(|e| Error::Profile(e.to_string()))?,
        )),
        Profile::Dl => Err(Error::UnsupportedProfile(OwlProfile::Dl)),
        Profile::Swrl => Err(Error::UnsupportedProfile(ontologos_profile::OwlProfile::Dl)),
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
    saturate_rl_unchecked(reasoner)
}

fn saturate_rl_unchecked(reasoner: &mut Reasoner) -> Result<RlReport, Error> {
    let parallelism = reasoner.config().parallelism;
    let record_traces = reasoner.config().explanations;
    Ok(RlEngine::try_new(parallelism)?
        .with_traces(record_traces)
        .saturate_reasoner(reasoner)?)
}

fn classify_auto(reasoner: &mut Reasoner) -> Result<ClassifyOutcome, Error> {
    let report = detect_profile(reasoner.ontology()).map_err(|e| Error::Profile(e.to_string()))?;
    let detected = report
        .detected
        .ok_or_else(|| Error::Profile("no profile detected".into()))?;

    match detected {
        // OWL QL TBox is a subset of EL for mapped axioms; EL completion is sound for QL corpora.
        OwlProfile::El | OwlProfile::Ql => {
            if reasoner.config().incremental {
                Ok(ClassifyOutcome::Taxonomy(
                    crate::classify_with_report(reasoner)?.taxonomy,
                ))
            } else {
                Ok(ClassifyOutcome::Taxonomy(
                    ElClassifier::new().classify(reasoner.ontology())?,
                ))
            }
        }
        OwlProfile::Rl => Ok(ClassifyOutcome::Rl(saturate_rl_unchecked(reasoner)?)),
        OwlProfile::Dl => Err(Error::UnsupportedProfile(OwlProfile::Dl)),
    }
}

/// Resolve an explicit CLI/API profile override against auto-detection.
pub fn resolve_profile_flag(flag: ProfileFlag) -> Profile {
    match flag {
        ProfileFlag::Auto => Profile::Auto,
        ProfileFlag::El => Profile::El,
        ProfileFlag::Rl => Profile::Rl,
        ProfileFlag::Alc => Profile::Alc,
        ProfileFlag::Dl => Profile::Dl,
        ProfileFlag::Swrl => Profile::Swrl,
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
    /// OWL ALC tableau-lite.
    Alc,
    /// OWL 2 DL classification.
    Dl,
    /// DLSafe SWRL with DL.
    Swrl,
    /// RDFS materialization.
    Rdfs,
}
