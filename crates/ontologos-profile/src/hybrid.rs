//! MORe-style hybrid module routing over EL / RL / DL fragments.

use std::collections::HashSet;

use ontologos_core::axiom_signature;
use ontologos_core::{Axiom, AxiomId, EntityId, EntityKind, Ontology, Taxonomy};

use crate::rules::el_classification_forbidden_in;
use crate::scanner::axiom_constructs;
use crate::{detect_profile, OwlProfile, Result};

/// One classified module in a hybrid ontology.
#[derive(Debug, Clone)]
pub struct ClassifiedModule {
    /// Detected OWL profile for this module.
    pub profile: OwlProfile,
    /// Entity signature (class IRIs) in the module.
    pub signature: Vec<String>,
    /// Axioms belonging to this module.
    pub axiom_ids: Vec<AxiomId>,
}

/// Hybrid classification report (per-module routing).
#[derive(Debug, Clone, Default)]
pub struct HybridReport {
    /// Modules extracted from the ontology.
    pub modules: Vec<ClassifiedModule>,
    /// Merged taxonomy (when classification succeeds).
    pub taxonomy: Option<Taxonomy>,
}

/// Extract class IRIs referenced by axioms.
#[must_use]
pub fn signature_for_axioms(ontology: &Ontology, axiom_ids: &[AxiomId]) -> Vec<String> {
    let mut classes = HashSet::new();
    for &id in axiom_ids {
        let Ok(axiom) = ontology.axioms().get(id) else {
            continue;
        };
        for entity in axiom_signature(axiom) {
            let Ok(record) = ontology.entity(entity) else {
                continue;
            };
            if record.kind != EntityKind::Class {
                continue;
            }
            if let Ok(iri) = ontology.resolve_iri(record.iri) {
                classes.insert(iri.to_owned());
            }
        }
    }
    let mut out: Vec<_> = classes.into_iter().collect();
    out.sort();
    out
}

/// Extract ⊥-module style signature: all class entities in the ontology.
#[must_use]
pub fn extract_signature(ontology: &Ontology) -> Vec<String> {
    ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .filter_map(|(_, r)| ontology.resolve_iri(r.iri).ok().map(str::to_owned))
        .collect()
}

fn axiom_requires_dl(axiom: &Axiom) -> bool {
    !el_classification_forbidden_in(&axiom_constructs(axiom)).is_empty()
}

fn class_entities_in_signature(ontology: &Ontology, sig: &HashSet<EntityId>) -> HashSet<EntityId> {
    sig.iter()
        .copied()
        .filter(|&e| {
            ontology
                .entity(e)
                .ok()
                .is_some_and(|r| r.kind == EntityKind::Class)
        })
        .collect()
}

/// Partition axiom ids into EL-safe and DL-residue buckets with dependency closure.
pub fn partition_axioms(ontology: &Ontology) -> (Vec<AxiomId>, Vec<AxiomId>) {
    let mut dl_ids = Vec::new();
    let mut el_ids = Vec::new();
    for (id, axiom) in ontology.axioms().iter() {
        if axiom_requires_dl(axiom) {
            dl_ids.push(id);
        } else {
            el_ids.push(id);
        }
    }

    if dl_ids.is_empty() || el_ids.is_empty() {
        return (el_ids, dl_ids);
    }

    let mut dl_classes: HashSet<EntityId> = HashSet::new();
    for &id in &dl_ids {
        if let Ok(axiom) = ontology.axioms().get(id) {
            dl_classes.extend(class_entities_in_signature(
                ontology,
                &axiom_signature(axiom),
            ));
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        let mut still_el = Vec::new();
        for id in el_ids {
            let Ok(axiom) = ontology.axioms().get(id) else {
                continue;
            };
            let sig = axiom_signature(axiom);
            if class_entities_in_signature(ontology, &sig)
                .iter()
                .any(|c| dl_classes.contains(c))
            {
                dl_ids.push(id);
                dl_classes.extend(class_entities_in_signature(ontology, &sig));
                changed = true;
            } else {
                still_el.push(id);
            }
        }
        el_ids = still_el;
    }

    (el_ids, dl_ids)
}

/// Build a sub-ontology containing only the given axioms (entity ids preserved).
pub fn subontology_with_axioms(ontology: &Ontology, axiom_ids: &[AxiomId]) -> Result<Ontology> {
    let mut out = Ontology::new();
    for (_, record) in ontology.entities().iter() {
        let iri = ontology
            .resolve_iri(record.iri)
            .map_err(|e| crate::Error::Message(e.to_string()))?;
        out.entity_id(iri, record.kind)
            .map_err(|e| crate::Error::Message(e.to_string()))?;
    }
    for &id in axiom_ids {
        let axiom = ontology
            .axioms()
            .get(id)
            .map_err(|e| crate::Error::Message(e.to_string()))?
            .clone();
        out.add_axiom(axiom)
            .map_err(|e| crate::Error::Message(e.to_string()))?;
    }
    Ok(out)
}

/// Partition ontology into EL / RL / DL modules (signature + detected profile).
pub fn classify_hybrid(ontology: &Ontology) -> Result<HybridReport> {
    let _report = detect_profile(ontology)?;
    let (el_ids, dl_ids) = partition_axioms(ontology);

    let modules = if dl_ids.is_empty() {
        vec![ClassifiedModule {
            profile: OwlProfile::El,
            signature: signature_for_axioms(ontology, &el_ids),
            axiom_ids: el_ids,
        }]
    } else if el_ids.is_empty() {
        vec![ClassifiedModule {
            profile: OwlProfile::Dl,
            signature: signature_for_axioms(ontology, &dl_ids),
            axiom_ids: dl_ids,
        }]
    } else {
        vec![
            ClassifiedModule {
                profile: OwlProfile::El,
                signature: signature_for_axioms(ontology, &el_ids),
                axiom_ids: el_ids,
            },
            ClassifiedModule {
                profile: OwlProfile::Dl,
                signature: signature_for_axioms(ontology, &dl_ids),
                axiom_ids: dl_ids,
            },
        ]
    };

    Ok(HybridReport {
        modules,
        taxonomy: None,
    })
}

/// Merge module taxonomies (called from `ontologos-dl` after per-engine classify).
#[must_use]
pub fn merge_taxonomies(mut parts: Vec<Taxonomy>) -> Taxonomy {
    let mut subsumptions = Vec::new();
    let mut equivalences = Vec::new();
    let mut unsatisfiable = Vec::new();
    for t in &mut parts {
        subsumptions.append(&mut t.subsumptions);
        equivalences.append(&mut t.equivalences);
        unsatisfiable.append(&mut t.unsatisfiable);
    }
    subsumptions.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    subsumptions.dedup();
    equivalences.sort_by_cached_key(|cluster| cluster.iter().map(|id| id.0).min().unwrap_or(0));
    equivalences.dedup();
    unsatisfiable.sort_unstable_by_key(|id| id.0);
    unsatisfiable.dedup();
    Taxonomy {
        subsumptions,
        equivalences,
        unsatisfiable,
    }
}

/// Route module to engine name for conformance / CLI.
#[must_use]
pub fn engine_for_profile(profile: OwlProfile) -> &'static str {
    match profile {
        OwlProfile::El | OwlProfile::Ql => "el",
        OwlProfile::Rl => "rl",
        OwlProfile::Dl => "dl",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::Axiom;

    #[test]
    fn partitions_el_and_dl_axioms() {
        let mut ontology = Ontology::builder()
            .class("http://ex.org/A")
            .unwrap()
            .class("http://ex.org/B")
            .unwrap()
            .class("http://ex.org/C")
            .unwrap()
            .subclass_of("http://ex.org/A", "http://ex.org/B")
            .unwrap()
            .build()
            .unwrap();
        let a = ontology.lookup_entity("http://ex.org/A").unwrap();
        let c = ontology.lookup_entity("http://ex.org/C").unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: c,
            })
            .unwrap();

        let report = classify_hybrid(&ontology).expect("hybrid");
        assert_eq!(report.modules.len(), 1);
    }
}
