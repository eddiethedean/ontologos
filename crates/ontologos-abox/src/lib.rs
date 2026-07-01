//! ABox reasoning: individual typing, `sameAs` / `differentFrom` closure, consistency checks.

#![warn(missing_docs)]

mod closure;
mod report;

use ontologos_core::{EntityId, Ontology};
use thiserror::Error;

pub use closure::{SameAsClosure, same_as_closure};
pub use report::AboxReport;

/// Result type for ABox operations.
pub type Result<T> = std::result::Result<T, Error>;

/// ABox reasoning errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Core ontology error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// RL saturation error.
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
    /// Inconsistent ABox (clash detected).
    #[error("ABox inconsistent: {0}")]
    Inconsistent(String),
}

/// Materialize ABox consequences (typing + `sameAs` closure) via RL saturation.
pub fn materialize_abox(ontology: &mut Ontology) -> Result<AboxReport> {
    let closure = same_as_closure(ontology);
    let rl_report = ontologos_rl::RlEngine::new(1)
        .saturate(ontology)
        .map_err(Error::Rl)?;
    Ok(AboxReport {
        same_as_clusters: closure.clusters,
        rl_inferences: rl_report.inferred_total(),
    })
}

/// Returns true when no `differentFrom` clash exists among `sameAs` clusters.
pub fn is_abox_consistent(ontology: &Ontology) -> Result<bool> {
    Ok(!detect_clash(ontology))
}

/// Object individuals entailed as values of `property` on `subject` after RL ABox materialization.
pub fn object_property_values(
    ontology: &mut Ontology,
    subject: EntityId,
    property: EntityId,
) -> Result<Vec<EntityId>> {
    materialize_abox(ontology)?;
    let mut subjects = vec![subject];
    if let Some(cluster) = ontology.same_as(subject) {
        subjects.extend(cluster.iter().copied());
    }
    subjects.sort_by_key(|id| id.0);
    subjects.dedup();

    let mut objects = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for subj in &subjects {
        for &(prop, obj) in ontology.object_assertions_of(*subj) {
            if property_entails_assertion(ontology, prop, property) && seen.insert(obj) {
                objects.push(obj);
            }
        }
    }
    collect_inverse_property_fillers(ontology, &subjects, property, &mut objects, &mut seen);
    objects.sort_by_key(|id| id.0);
    Ok(objects)
}

fn collect_inverse_property_fillers(
    ontology: &Ontology,
    subjects: &[EntityId],
    property: EntityId,
    objects: &mut Vec<EntityId>,
    seen: &mut std::collections::HashSet<EntityId>,
) {
    let subject_set: std::collections::HashSet<EntityId> = subjects.iter().copied().collect();
    for (_, axiom) in ontology.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject: src,
            property: prop,
            object: dst,
        } = axiom
        else {
            continue;
        };
        if !subject_set.contains(dst) {
            continue;
        }
        let mut entailed = false;
        if let Some(inv) = ontology.inverse_of(*prop) {
            entailed = property_entails_assertion(ontology, inv, property);
        }
        if !entailed && is_symmetric_property(ontology, *prop) {
            entailed = property_entails_assertion(ontology, *prop, property);
        }
        if entailed && seen.insert(*src) {
            objects.push(*src);
        }
    }
}

fn is_symmetric_property(ontology: &Ontology, property: EntityId) -> bool {
    if ontology
        .inverse_of(property)
        .is_some_and(|inv| inv == property)
    {
        return true;
    }
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::SymmetricObjectProperty(prop) if *prop == property
        )
    })
}

fn property_entails_assertion(ontology: &Ontology, asserted: EntityId, query: EntityId) -> bool {
    if asserted == query {
        return true;
    }
    if transitive_subproperty_of(ontology, asserted, query) {
        return true;
    }
    if ontology
        .equivalents_of(query)
        .is_some_and(|set| set.contains(&asserted))
    {
        return true;
    }
    if let Some(equiv) = ontology.equivalents_of(asserted) {
        return equiv
            .iter()
            .any(|&e| e == query || transitive_subproperty_of(ontology, e, query));
    }
    false
}

fn transitive_subproperty_of(ontology: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    if sub == sup {
        return true;
    }
    let mut frontier = vec![sub];
    let mut seen = std::collections::HashSet::from([sub]);
    while let Some(prop) = frontier.pop() {
        for &super_prop in ontology.direct_superproperties(prop) {
            if super_prop == sup {
                return true;
            }
            if seen.insert(super_prop) {
                frontier.push(super_prop);
            }
        }
    }
    false
}

fn detect_clash(ontology: &Ontology) -> bool {
    let closure = same_as_closure(ontology);
    let mut rep_pairs: std::collections::HashSet<(EntityId, EntityId)> =
        std::collections::HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::DifferentIndividuals(ids) = axiom {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = closure.representative(ids[i]);
                    let b = closure.representative(ids[j]);
                    if a == b {
                        return true;
                    }
                    let key = if a.0 <= b.0 { (a, b) } else { (b, a) };
                    if !rep_pairs.insert(key) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    #[test]
    fn same_as_clusters_merge() {
        let mut o = Ontology::new();
        let a = o
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        let b = o
            .entity_id("http://ex.org/b", EntityKind::Individual)
            .unwrap();
        o.add_axiom(Axiom::SameIndividual(vec![a, b])).unwrap();
        let c = same_as_closure(&o);
        assert_eq!(c.representative(a), c.representative(b));
    }

    #[test]
    fn object_property_values_includes_subproperty_assertions() {
        let mut ontology = Ontology::builder()
            .individual("http://ex.org/a")
            .unwrap()
            .individual("http://ex.org/b")
            .unwrap()
            .object_property("http://ex.org/p")
            .unwrap()
            .object_property("http://ex.org/q")
            .unwrap()
            .subproperty_of("http://ex.org/q", "http://ex.org/p")
            .unwrap()
            .object_property_assertion("http://ex.org/a", "http://ex.org/q", "http://ex.org/b")
            .unwrap()
            .build()
            .unwrap();
        let a = ontology.lookup_entity("http://ex.org/a").unwrap();
        let p = ontology.lookup_entity("http://ex.org/p").unwrap();
        let b = ontology.lookup_entity("http://ex.org/b").unwrap();
        let values = object_property_values(&mut ontology, a, p).expect("values");
        assert_eq!(values, vec![b]);
    }

    #[test]
    fn object_property_values_includes_inverse_subproperty_assertions() {
        let mut ontology = Ontology::builder()
            .individual("http://ex.org/a")
            .unwrap()
            .individual("http://ex.org/b")
            .unwrap()
            .object_property("http://ex.org/p")
            .unwrap()
            .object_property("http://ex.org/q")
            .unwrap()
            .subproperty_of("http://ex.org/q", "http://ex.org/p")
            .unwrap()
            .object_property_assertion("http://ex.org/b", "http://ex.org/q", "http://ex.org/a")
            .unwrap()
            .build()
            .unwrap();
        let q = ontology.lookup_entity("http://ex.org/q").unwrap();
        ontology
            .add_axiom(Axiom::SymmetricObjectProperty(q))
            .expect("symmetric q");
        let a = ontology.lookup_entity("http://ex.org/a").unwrap();
        let p = ontology.lookup_entity("http://ex.org/p").unwrap();
        let b = ontology.lookup_entity("http://ex.org/b").unwrap();
        let values = object_property_values(&mut ontology, a, p).expect("values");
        assert_eq!(values, vec![b]);
    }
}
