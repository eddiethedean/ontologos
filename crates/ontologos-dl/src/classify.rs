//! DL classification: EL for EL fragment, saturation + tableau for DL.

use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::{ClassExpr, DlAxiom, EntityId, Ontology, Taxonomy};
use ontologos_el::ElClassifier;
use ontologos_profile::{
    detect_profile, el_classification_forbidden_in, merge_taxonomies, scanner::scan_constructs,
};

use crate::cardinality::derive_cardinality_subsumptions;
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
        detect_profile(ontology).map_err(|e| Error::Profile(e.to_string()))?;

        if self.preview {
            let forbidden = preview_forbidden_constructs(ontology);
            if !forbidden.is_empty() {
                return Err(Error::PreviewLimit(format!(
                    "DL preview does not support: {:?}",
                    forbidden.iter().cloned().collect::<Vec<_>>()
                )));
            }
        }

        let tab_tax = tableau_classify(ontology)?;
        let mut taxonomy = match try_el_classify(ontology)? {
            Some(el_tax) => merge_taxonomies(vec![el_tax, tab_tax]),
            None => tab_tax,
        };
        enrich_taxonomy(ontology, &mut taxonomy);
        Ok(taxonomy)
    }
}

fn preview_forbidden_constructs(
    ontology: &Ontology,
) -> std::collections::BTreeSet<ontologos_profile::OwlConstruct> {
    el_classification_forbidden_in(&scan_constructs(ontology))
}

fn try_el_classify(ontology: &Ontology) -> Result<Option<Taxonomy>, Error> {
    if !el_classification_forbidden_in(&scan_constructs(ontology)).is_empty() {
        return Ok(None);
    }
    Ok(Some(
        ElClassifier::new().classify(ontology).map_err(Error::El)?,
    ))
}

fn tableau_classify(ontology: &Ontology) -> Result<Taxonomy, Error> {
    let dl = DlOntology::from_ontology(ontology)?;
    let roles = RoleHierarchy::from_clauses(dl.clauses());
    let facts = saturate(ontology, dl.clauses(), &roles)?;
    let seed = build_tableau_seed(ontology, &dl, &facts, &roles)?;
    let mut taxonomy = ontologos_alc::classify_with_seed(ontology, &seed).map_err(Error::Alc)?;
    let derived = derive_cardinality_subsumptions(ontology);
    for (sub, sup) in derived {
        if !taxonomy
            .subsumptions
            .iter()
            .any(|&(a, b)| a == sub && b == sup)
        {
            taxonomy.subsumptions.push((sub, sup));
        }
    }
    Ok(taxonomy)
}

/// Union equivalence: `A ≡ C₁ ⊔ … ⊔ Cₙ` implies each `Cᵢ ⊑ A`.
fn derive_union_equivalence_subsumptions(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let store = ontology.dl();
    let mut out = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        if ids.len() < 2 {
            continue;
        }
        for &named in ids {
            let Some(ClassExpr::Atomic(entity)) = store.ce(named) else {
                continue;
            };
            for &def_id in ids {
                if def_id == named {
                    continue;
                }
                let Some(ClassExpr::Or(ops)) = store.ce(def_id) else {
                    continue;
                };
                for op in ops {
                    if let Some(ClassExpr::Atomic(member)) = store.ce(*op) {
                        out.push((*member, *entity));
                    }
                }
            }
        }
    }
    out
}

fn enrich_taxonomy(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    for (sub, sup) in derive_union_equivalence_subsumptions(ontology) {
        if !taxonomy
            .subsumptions
            .iter()
            .any(|&(a, b)| a == sub && b == sup)
        {
            taxonomy.subsumptions.push((sub, sup));
        }
    }
}

/// Build tableau seed from saturation (used by consistency checking).
pub fn build_tableau_seed(
    _ontology: &Ontology,
    dl: &DlOntology,
    facts: &SaturatedFacts,
    roles: &RoleHierarchy,
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
    for (sub, supers) in roles.subrole_map() {
        for &sup in supers {
            if !seed
                .role_subsumptions
                .iter()
                .any(|&(a, b)| a == *sub && b == sup)
            {
                seed.role_subsumptions.push((*sub, sup));
            }
        }
    }
    Ok(seed)
}
