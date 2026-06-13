//! Profile routing for DL classification.

use ontologos_core::{Profile, Reasoner, Taxonomy};

use crate::{classify, DlClassifier, Error};

/// DL classification report.
#[derive(Debug)]
pub struct DlReport {
    /// Extracted taxonomy.
    pub taxonomy: Taxonomy,
}

/// Classify via [`Reasoner`] when profile is DL.
pub fn classify_reasoner(reasoner: &Reasoner) -> Result<DlReport, Error> {
  if !matches!(reasoner.profile(), Profile::Dl | Profile::Auto) {
        return Err(Error::WrongProfile(reasoner.profile()));
    }
    let taxonomy = classify(reasoner.ontology())?;
    Ok(DlReport { taxonomy })
}

/// Route classification by profile flag.
pub fn classify_with_profile(reasoner: &mut Reasoner) -> Result<DlReport, Error> {
    match reasoner.profile() {
        Profile::Dl => classify_reasoner(reasoner),
        Profile::Auto => {
            let taxonomy = DlClassifier::new().classify(reasoner.ontology())?;
            Ok(DlReport { taxonomy })
        }
        other => Err(Error::WrongProfile(other)),
    }
}
