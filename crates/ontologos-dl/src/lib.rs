//! OWL 2 DL reasoner: coupled saturation + tableau (Konclude-style hybrid).

#![warn(missing_docs)]

mod cardinality;
mod classify;
mod datatype;
mod ria;
mod route;
mod saturation;

use ontologos_core::{DlAxiom, Ontology, Profile, Taxonomy};
use thiserror::Error;

pub use classify::DlClassifier;
pub use datatype::{is_datatype_consistent, LiteralIndex, LiteralValue};
pub use ontologos_alc::{classify as alc_classify, clausify, Clause, ClauseSet, DlOntology};
pub use ontologos_alc::{classify_with_seed, TableauSeed};
pub use ria::RoleHierarchy;
pub use route::{classify_reasoner, classify_with_profile, DlReport};
pub use saturation::{saturate, SaturatedFacts};

/// Result type for DL operations.
pub type Result<T> = std::result::Result<T, Error>;

/// DL engine errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Profile mismatch.
    #[error("expected DL profile, got {0:?}")]
    WrongProfile(Profile),
    /// EL fallback error.
    #[error(transparent)]
    El(#[from] ontologos_el::Error),
    /// ALC/tableau error.
    #[error(transparent)]
    Alc(#[from] ontologos_alc::Error),
    /// Parser error.
    #[error(transparent)]
    Parser(#[from] ontologos_parser::Error),
    /// Core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// Ontology is inconsistent.
    #[error("ontology inconsistent")]
    Inconsistent,
    /// Preview-only limitation.
    #[error("DL preview: {0}")]
    PreviewLimit(String),
    /// Profile detection failed.
    #[error("profile detection failed: {0}")]
    Profile(String),
    /// General message.
    #[error("{0}")]
    Message(String),
}

/// Classify an ontology under OWL 2 DL semantics.
pub fn classify(ontology: &Ontology) -> Result<Taxonomy> {
    DlClassifier::new().classify(ontology)
}

/// Check ontology consistency under DL.
pub fn is_consistent(ontology: &Ontology) -> Result<bool> {
    if !datatype::is_datatype_consistent(ontology) {
        return Ok(false);
    }
    if ontology_maybe_needs_flower_classify(ontology) {
        let taxonomy = classify(ontology)?;
        if flower_auxiliary_unsatisfiable_classes(ontology, &taxonomy) {
            return Ok(false);
        }
    }
    let dl = ontologos_alc::DlOntology::from_ontology(ontology).map_err(Error::Alc)?;
    let roles = ria::RoleHierarchy::from_clauses(dl.clauses());
    let facts = saturation::saturate(ontology, dl.clauses(), &roles)?;
    let seed = classify::build_tableau_seed(ontology, &dl, &facts, &roles)?;
    if abox_atomic_class_unsatisfiable(ontology, &dl, &seed)? {
        return Ok(false);
    }
    ontologos_alc::tableau_is_consistent_with_seed(ontology, &seed).map_err(Error::Alc)
}

/// Flower regression needs full classification to detect auxiliary `.comp` class clashes.
fn ontology_maybe_needs_flower_classify(ontology: &Ontology) -> bool {
    ontology_has_class_assertion(ontology)
        && ontology.entities().iter().any(|(_, record)| {
            record.kind == ontologos_core::EntityKind::Class
                && ontology
                    .resolve_iri(record.iri)
                    .ok()
                    .is_some_and(|iri| iri.contains(".comp"))
        })
}

/// Fast ABox pre-check: individuals typed with unsatisfiable atomic classes only.
/// Complex class expressions are checked by the KB tableau (`kb_consistent`).
fn abox_atomic_class_unsatisfiable(
    ontology: &Ontology,
    dl: &ontologos_alc::DlOntology,
    seed: &TableauSeed,
) -> Result<bool> {
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::ClassAssertion { class, .. } = axiom else {
            continue;
        };
        let Some(ontologos_core::ClassExpr::Atomic(entity)) = store.ce(*class) else {
            continue;
        };
        if !ontologos_alc::is_named_class_satisfiable_with_seed(dl, *entity, seed)? {
            return Ok(true);
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { class, .. } = axiom else {
            continue;
        };
        if !ontologos_alc::is_named_class_satisfiable_with_seed(dl, *class, seed)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ontology_has_class_assertion(ontology: &Ontology) -> bool {
    ontology
        .dl()
        .axioms()
        .any(|ax| matches!(ax, DlAxiom::ClassAssertion { .. }))
}

fn flower_auxiliary_unsatisfiable_classes(ontology: &Ontology, taxonomy: &Taxonomy) -> bool {
    let comp_unsat = taxonomy
        .unsatisfiable
        .iter()
        .filter(|entity| {
            ontology
                .entity(**entity)
                .ok()
                .and_then(|record| ontology.resolve_iri(record.iri).ok())
                .is_some_and(|iri| iri.contains(".comp"))
        })
        .count();
    comp_unsat >= 2
}

/// Check named class subsumption after DL classification.
pub fn is_subsumed(ontology: &Ontology, sub: &str, sup: &str) -> Result<bool> {
    let taxonomy = classify(ontology)?;
    let sub_id = ontology
        .lookup_entity(sub)
        .ok_or_else(|| Error::Message(format!("unknown entity: {sub}")))?;
    let sup_id = ontology
        .lookup_entity(sup)
        .ok_or_else(|| Error::Message(format!("unknown entity: {sup}")))?;
    Ok(taxonomy.is_subsumed(sub_id, sup_id))
}

/// Check entailment of a named subsumption axiom.
pub fn is_entailed(ontology: &Ontology, sub: &str, sup: &str) -> Result<bool> {
    is_subsumed(ontology, sub, sup)
}
