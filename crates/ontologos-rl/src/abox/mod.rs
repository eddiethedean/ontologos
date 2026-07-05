//! ABox reasoning: individual typing, `sameAs` closure, consistency checks.

mod closure;
mod report;

use ontologos_core::{EntityId, Ontology};

pub use closure::{SameAsClosure, same_as_closure};
pub use report::AboxReport;

pub use crate::Error;
pub type Result<T> = crate::Result<T>;

/// Materialize ABox consequences (typing + `sameAs` closure) via RL saturation.
pub fn materialize_abox(ontology: &mut Ontology) -> Result<AboxReport> {
    let rl_report = crate::RlEngine::new(1).saturate(ontology)?;
    let closure = same_as_closure(ontology);
    Ok(AboxReport {
        same_as_clusters: closure.clusters,
        rl_inferences: rl_report.inferred_total(),
    })
}

/// Returns true when no `differentFrom` clash exists among `sameAs` clusters.
pub fn is_abox_consistent(ontology: &Ontology) -> Result<bool> {
    let mut working = ontology.clone();
    crate::RlEngine::new(1).saturate(&mut working)?;
    Ok(!detect_clash(&working))
}

/// Object individuals entailed as values of `property` on `subject` after RL ABox materialization.
pub fn object_property_values(
    ontology: &mut Ontology,
    subject: EntityId,
    property: EntityId,
) -> Result<Vec<EntityId>> {
    materialize_abox(ontology)?;
    object_property_values_materialized(ontology, subject, property, true)
}

/// Object property values after RDFS materialization (no RL inverse/symmetric expansion).
pub fn rdfs_object_property_values(
    ontology: &mut Ontology,
    subject: EntityId,
    property: EntityId,
) -> Result<Vec<EntityId>> {
    crate::rdfs::RdfsEngine::new().materialize(ontology)?;
    object_property_values_materialized(ontology, subject, property, false)
}

fn object_property_values_materialized(
    ontology: &Ontology,
    subject: EntityId,
    property: EntityId,
    rl_rich: bool,
) -> Result<Vec<EntityId>> {
    let subjects = same_as_subjects(ontology, subject);
    let mut objects = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for subj in &subjects {
        for &(prop, obj) in ontology.object_assertions_of(*subj) {
            if property_entails_assertion(ontology, prop, property) && seen.insert(obj) {
                objects.push(obj);
            }
        }
    }
    if rl_rich {
        collect_inverse_property_fillers(ontology, &subjects, property, &mut objects, &mut seen);
    }
    objects.sort_by_key(|id| id.0);
    Ok(objects)
}

fn same_as_subjects(ontology: &Ontology, subject: EntityId) -> Vec<EntityId> {
    let mut subjects = vec![subject];
    if let Some(cluster) = ontology.same_as(subject) {
        subjects.extend(cluster.iter().copied());
    }
    subjects.sort_by_key(|id| id.0);
    subjects.dedup();
    subjects
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
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::DifferentIndividuals(ids) = axiom {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = closure.representative(ids[i]);
                    let b = closure.representative(ids[j]);
                    if a == b {
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
    fn rl_rich_lookup_applies_inverse_fillers_without_materializing() {
        let mut o = Ontology::new();
        let c = o
            .entity_id("http://ex.org/c", EntityKind::Individual)
            .unwrap();
        let d = o
            .entity_id("http://ex.org/d", EntityKind::Individual)
            .unwrap();
        let p = o
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .unwrap();
        let q = o
            .entity_id("http://ex.org/q", EntityKind::ObjectProperty)
            .unwrap();
        o.add_axiom(Axiom::InverseObjectProperties { left: p, right: q })
            .unwrap();
        o.add_axiom(Axiom::ObjectPropertyAssertion {
            subject: c,
            property: p,
            object: d,
        })
        .unwrap();
        let sparse = object_property_values_materialized(&o, d, q, false).unwrap();
        assert!(sparse.is_empty());
        let rich = object_property_values_materialized(&o, d, q, true).unwrap();
        assert_eq!(rich, vec![c]);
    }
}
