//! DL classification: EL for EL fragment, saturation + tableau for DL.

use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::{ClassExpr, Ontology, Taxonomy};
use ontologos_el::ElClassifier;
use ontologos_profile::{detect_profile, merge_taxonomies, OwlProfile};

use crate::ria::RoleHierarchy;
use crate::saturation::{saturate, SaturatedFacts};
use crate::Error;

/// OWL 2 DL classifier facade.
#[derive(Debug, Default)]
pub struct DlClassifier {
    preview: bool,
}

impl DlClassifier {
    /// Create a DL classifier.
    #[must_use]
    pub fn new() -> Self {
        Self { preview: false }
    }

    /// Enable preview mode (subset checks).
    #[must_use]
    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }

    /// Classify the ontology (EL saturation + tableau merge for hybrid ontologies).
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy, Error> {
        let profile = detect_profile(ontology)
            .map_err(|e| Error::PreviewLimit(e.to_string()))?
            .detected;

        match profile {
            Some(OwlProfile::El) | Some(OwlProfile::Ql) => {
                let el_tax = ElClassifier::new().classify(ontology).map_err(Error::El)?;
                let tab_tax = tableau_classify(ontology)?;
                Ok(merge_taxonomies(vec![el_tax, tab_tax]))
            }
            Some(OwlProfile::Rl) | Some(OwlProfile::Dl) | None => {
                let el_tax = ElClassifier::new().classify(ontology).map_err(Error::El)?;
                let tab_tax = tableau_classify(ontology)?;
                Ok(merge_taxonomies(vec![el_tax, tab_tax]))
            }
        }
    }
}

fn tableau_classify(ontology: &Ontology) -> Result<Taxonomy, Error> {
    let dl = DlOntology::from_ontology(ontology)?;
    let roles = RoleHierarchy::from_clauses(dl.clauses());
    let facts = saturate(ontology, dl.clauses(), &roles)?;
    let seed = tableau_seed_from_facts(&dl, &facts, &roles)?;
    ontologos_alc::classify_with_seed(ontology, &seed).map_err(Error::Alc)
}

fn tableau_seed_from_facts(
    dl: &DlOntology,
    facts: &SaturatedFacts,
    _roles: &RoleHierarchy,
) -> Result<TableauSeed, Error> {
    let store = dl.core().dl();
    let mut seed = TableauSeed::default();

    for &(sub, sup) in &facts.subsumptions {
        let sub_ce = store.expressions().find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == sub => Some(id),
            _ => None,
        });
        let sup_ce = store.expressions().find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == sup => Some(id),
            _ => None,
        });
        if let (Some(sub_ce), Some(sup_ce)) = (sub_ce, sup_ce) {
            seed.subsumptions.push((sub_ce, sup_ce));
        }
    }

    seed.existentials.extend(facts.existentials.clone());
    seed.role_subsumptions = facts.role_subsumptions.clone();
    Ok(seed)
}
