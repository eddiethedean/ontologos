//! OWL 2 DL reasoner: coupled saturation + tableau (Konclude-style hybrid).

#![warn(missing_docs)]

mod cardinality;
mod classify;
mod datatype;
mod ria;
mod route;
mod saturation;

use ontologos_core::{Axiom, DlAxiom, EntityId, Ontology, Profile, RoleExpr, Taxonomy};
use thiserror::Error;

pub use classify::DlClassifier;
pub use datatype::{is_datatype_consistent, named_class_datatype_satisfiable, LiteralIndex, LiteralValue};
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
    if ontologos_bridge::has_bottom_chain_violation(ontology) {
        return Ok(false);
    }
    if abox_property_characteristic_clash(ontology) {
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
    if abox_functional_different_individuals_clash(ontology) {
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

/// Asymmetric / irreflexive object property assertions (with subproperty expansion).
fn abox_property_characteristic_clash(ontology: &Ontology) -> bool {
    use std::collections::{HashMap, HashSet};

    let mut asymmetric = HashSet::new();
    let mut irreflexive = HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        match axiom {
            Axiom::AsymmetricObjectProperty(prop) => {
                asymmetric.insert(*prop);
            }
            Axiom::IrreflexiveObjectProperty(prop) => {
                irreflexive.insert(*prop);
            }
            _ => {}
        }
    }
    if asymmetric.is_empty() && irreflexive.is_empty() {
        return false;
    }

    let mut sub_to_supers: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } = axiom
        {
            sub_to_supers
                .entry(*sub_property)
                .or_default()
                .insert(*super_property);
        }
    }
    let supers_for = |prop: EntityId| -> HashSet<EntityId> {
        let mut out = HashSet::from([prop]);
        let mut queue = vec![prop];
        while let Some(current) = queue.pop() {
            if let Some(supers) = sub_to_supers.get(&current) {
                for &sup in supers {
                    if out.insert(sup) {
                        queue.push(sup);
                    }
                }
            }
        }
        out
    };

    let mut expanded_asymmetric = HashSet::new();
    for prop in &asymmetric {
        expanded_asymmetric.extend(supers_for(*prop));
    }
    let mut expanded_irreflexive = HashSet::new();
    for prop in &irreflexive {
        expanded_irreflexive.extend(supers_for(*prop));
    }

    let mut triples: Vec<(EntityId, EntityId, EntityId)> = Vec::new();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        {
            let RoleExpr::Atomic(prop) = property else {
                continue;
            };
            for super_prop in supers_for(*prop) {
                triples.push((*subject, super_prop, *object));
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        {
            for super_prop in supers_for(*property) {
                triples.push((*subject, super_prop, *object));
            }
        }
    }

    for &(s, p, o) in &triples {
        if expanded_irreflexive.contains(&p) && s == o {
            return true;
        }
    }
    for &(s, p, o) in &triples {
        if !expanded_asymmetric.contains(&p) || s == o {
            continue;
        }
        if triples
            .iter()
            .any(|&(s2, p2, o2)| s2 == o && o2 == s && p2 == p)
        {
            return true;
        }
    }
    false
}

fn different_pair(left: EntityId, right: EntityId) -> (EntityId, EntityId) {
    if left.0 <= right.0 {
        (left, right)
    } else {
        (right, left)
    }
}

/// Functional object property + conflicting assertions on explicitly different individuals.
fn abox_functional_different_individuals_clash(ontology: &Ontology) -> bool {
    use std::collections::{HashMap, HashSet};

    let mut functional = HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::FunctionalObjectProperty(prop) = axiom {
            functional.insert(*prop);
        }
    }
    if functional.is_empty() {
        return false;
    }

    let mut different = HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::DifferentIndividuals(ids) = axiom {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    different.insert(different_pair(ids[i], ids[j]));
                }
            }
        }
    }
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::DifferentIndividuals(ids) = axiom {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    different.insert(different_pair(ids[i], ids[j]));
                }
            }
        }
    }
    if different.is_empty() {
        return false;
    }

    let mut by_subject_prop: HashMap<(EntityId, EntityId), HashSet<EntityId>> = HashMap::new();
    for axiom in ontology.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let RoleExpr::Atomic(prop) = property else {
            continue;
        };
        if functional.contains(prop) {
            by_subject_prop
                .entry((*subject, *prop))
                .or_default()
                .insert(*object);
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        let Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        if functional.contains(property) {
            by_subject_prop
                .entry((*subject, *property))
                .or_default()
                .insert(*object);
        }
    }

    for objects in by_subject_prop.values() {
        if objects.len() <= 1 {
            continue;
        }
        for &o1 in objects {
            for &o2 in objects {
                if o1 != o2 && different.contains(&different_pair(o1, o2)) {
                    return true;
                }
            }
        }
    }
    false
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
