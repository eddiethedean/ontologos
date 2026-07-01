use ontologos_core::{Profile, Reasoner, Taxonomy};
use ontologos_profile::{OwlProfile, detect_profile};
use ontologos_rdfs::{MaterializationReport as RdfsReport, materialize_reasoner};
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
        Profile::Alc => Ok(ClassifyOutcome::Taxonomy(ontologos_alc::classify(
            reasoner.ontology(),
        )?)),
        Profile::Dl | Profile::DlPreview => Err(Error::UnsupportedProfile(OwlProfile::Dl)),
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

pub(crate) fn classify_auto(reasoner: &mut Reasoner) -> Result<ClassifyOutcome, Error> {
    let report = detect_profile(reasoner.ontology())?;
    let detected = report
        .detected
        .ok_or_else(|| ontologos_profile::Error::Message("no profile detected".into()))?;

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
