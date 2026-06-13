//! Post-processing for RL/RDFS gaps not yet covered by `reasonable`.

use std::collections::{HashMap, HashSet, VecDeque};

use ontologos_core::{Axiom, EntityId, Ontology};

use crate::Result;

/// Apply domain/range inheritance along `subPropertyOf` (prp-dom / prp-rng fallbacks).
pub fn apply_domain_range_inheritance(ontology: &mut Ontology) -> Result<usize> {
    let sub_to_supers = superproperty_edges(ontology);
    let supers_to_subs = invert_subproperty_graph(&sub_to_supers);
    let domains = property_domains(ontology);
    let ranges = property_ranges(ontology);

    let mut added = 0_usize;
    let assertions: Vec<(EntityId, EntityId, EntityId)> = ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => Some((*subject, *property, *object)),
            _ => None,
        })
        .collect();

    for (subject, property, object) in assertions {
        for domain in inherited_domains(property, &domains, &supers_to_subs) {
            if !is_typed(ontology, subject, domain) {
                ontology.add_inferred_axiom(Axiom::ClassAssertion {
                    individual: subject,
                    class: domain,
                })?;
                added += 1;
            }
        }
        for range in inherited_ranges(property, &ranges, &supers_to_subs) {
            if !is_typed(ontology, object, range) {
                ontology.add_inferred_axiom(Axiom::ClassAssertion {
                    individual: object,
                    class: range,
                })?;
                added += 1;
            }
        }
    }

    Ok(added)
}

/// Materialize transitive `subPropertyOf` closure (RDFS 5 fallback).
pub fn apply_transitive_subproperties(ontology: &mut Ontology) -> Result<usize> {
    let direct = superproperty_edges(ontology);
    let mut all_pairs = HashSet::new();
    for &sub in direct.keys() {
        let mut reachable = HashSet::from([sub]);
        let mut queue = VecDeque::from([sub]);
        while let Some(current) = queue.pop_front() {
            if let Some(next_supers) = direct.get(&current) {
                for &sup in next_supers {
                    if reachable.insert(sup) {
                        queue.push_back(sup);
                        if sub != sup {
                            all_pairs.insert((sub, sup));
                        }
                    }
                }
            }
        }
    }

    let existing: HashSet<(EntityId, EntityId)> = ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => Some((*sub_property, *super_property)),
            _ => None,
        })
        .collect();

    let mut added = 0_usize;
    for (sub, sup) in all_pairs {
        if !existing.contains(&(sub, sup)) {
            ontology.add_inferred_axiom(Axiom::SubObjectPropertyOf {
                sub_property: sub,
                super_property: sup,
            })?;
            added += 1;
        }
    }
    Ok(added)
}

/// Run all reasonable semantic fallbacks after materialization.
pub fn apply_reasonable_fallbacks(ontology: &mut Ontology) -> Result<usize> {
    let mut total = apply_transitive_subproperties(ontology)?;
    total += propagate_domain_range_along_subproperties(ontology)?;
    total += apply_domain_range_inheritance(ontology)?;
    Ok(total)
}

/// Propagate domain/range from superproperties to subproperties (RDFS 6/8).
fn propagate_domain_range_along_subproperties(ontology: &mut Ontology) -> Result<usize> {
    let sub_to_supers = superproperty_edges(ontology);
    let mut added = 0_usize;

    let domains = property_domains(ontology);
    for sub in sub_to_supers.keys() {
        for sup in transitive_supers(*sub, &sub_to_supers) {
            if let Some(&domain) = domains.get(&sup) {
                if !has_domain_axiom(ontology, *sub, domain) {
                    ontology.add_inferred_axiom(Axiom::ObjectPropertyDomain {
                        property: *sub,
                        domain,
                    })?;
                    added += 1;
                }
            }
        }
    }

    let ranges = property_ranges(ontology);
    for sub in sub_to_supers.keys() {
        for sup in transitive_supers(*sub, &sub_to_supers) {
            if let Some(&range) = ranges.get(&sup) {
                if !has_range_axiom(ontology, *sub, range) {
                    ontology.add_inferred_axiom(Axiom::ObjectPropertyRange {
                        property: *sub,
                        range,
                    })?;
                    added += 1;
                }
            }
        }
    }

    Ok(added)
}

fn transitive_supers(
    property: EntityId,
    sub_to_supers: &HashMap<EntityId, Vec<EntityId>>,
) -> HashSet<EntityId> {
    let mut supers = HashSet::new();
    let mut queue = VecDeque::from([property]);
    while let Some(current) = queue.pop_front() {
        if let Some(direct) = sub_to_supers.get(&current) {
            for sup in direct {
                if supers.insert(*sup) {
                    queue.push_back(*sup);
                }
            }
        }
    }
    supers
}

fn has_domain_axiom(ontology: &Ontology, property: EntityId, domain: EntityId) -> bool {
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::ObjectPropertyDomain {
                property: p,
                domain: d,
            } if *p == property && *d == domain
        )
    })
}

fn has_range_axiom(ontology: &Ontology, property: EntityId, range: EntityId) -> bool {
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::ObjectPropertyRange {
                property: p,
                range: r,
            } if *p == property && *r == range
        )
    })
}

fn superproperty_edges(ontology: &Ontology) -> HashMap<EntityId, Vec<EntityId>> {
    let mut edges = HashMap::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } = axiom
        {
            edges
                .entry(*sub_property)
                .or_insert_with(Vec::new)
                .push(*super_property);
        }
    }
    edges
}

fn invert_subproperty_graph(
    sub_to_supers: &HashMap<EntityId, Vec<EntityId>>,
) -> HashMap<EntityId, Vec<EntityId>> {
    let mut supers_to_subs = HashMap::new();
    for (sub, supers) in sub_to_supers {
        for sup in supers {
            supers_to_subs
                .entry(*sup)
                .or_insert_with(Vec::new)
                .push(*sub);
        }
    }
    supers_to_subs
}

fn property_domains(ontology: &Ontology) -> HashMap<EntityId, EntityId> {
    let mut domains = HashMap::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyDomain { property, domain } = axiom {
            domains.insert(*property, *domain);
        }
    }
    domains
}

fn property_ranges(ontology: &Ontology) -> HashMap<EntityId, EntityId> {
    let mut ranges = HashMap::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyRange { property, range } = axiom {
            ranges.insert(*property, *range);
        }
    }
    ranges
}

fn subproperties_of(
    property: EntityId,
    supers_to_subs: &HashMap<EntityId, Vec<EntityId>>,
) -> HashSet<EntityId> {
    let mut subs = HashSet::new();
    let mut queue = VecDeque::from([property]);
    while let Some(current) = queue.pop_front() {
        if let Some(children) = supers_to_subs.get(&current) {
            for child in children {
                if subs.insert(*child) {
                    queue.push_back(*child);
                }
            }
        }
    }
    subs
}

fn inherited_domains(
    property: EntityId,
    domains: &HashMap<EntityId, EntityId>,
    supers_to_subs: &HashMap<EntityId, Vec<EntityId>>,
) -> HashSet<EntityId> {
    let mut out = HashSet::new();
    for sub in subproperties_of(property, supers_to_subs) {
        if let Some(&domain) = domains.get(&sub) {
            out.insert(domain);
        }
    }
    out
}

fn inherited_ranges(
    property: EntityId,
    ranges: &HashMap<EntityId, EntityId>,
    supers_to_subs: &HashMap<EntityId, Vec<EntityId>>,
) -> HashSet<EntityId> {
    let mut out = HashSet::new();
    for sub in subproperties_of(property, supers_to_subs) {
        if let Some(&range) = ranges.get(&sub) {
            out.insert(range);
        }
    }
    out
}

fn is_typed(ontology: &Ontology, individual: EntityId, class: EntityId) -> bool {
    fn subsumed(ontology: &Ontology, subclass: EntityId, superclass: EntityId) -> bool {
        if subclass == superclass {
            return true;
        }
        ontology
            .direct_superclasses(subclass)
            .iter()
            .any(|&sup| subsumed(ontology, sup, superclass))
    }
    ontology
        .classes_of(individual)
        .iter()
        .any(|&c| c == class || subsumed(ontology, c, class))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::Ontology;

    const NS: &str = "http://example.org/postprocess#";

    fn iri(local: &str) -> String {
        format!("{NS}{local}")
    }

    #[test]
    fn domain_on_subproperty_types_superproperty_assertion() {
        let mut ontology = Ontology::builder()
            .class(&iri("Person"))
            .unwrap()
            .individual(&iri("a"))
            .unwrap()
            .individual(&iri("b"))
            .unwrap()
            .object_property(&iri("P"))
            .unwrap()
            .object_property(&iri("Q"))
            .unwrap()
            .subproperty_of(&iri("Q"), &iri("P"))
            .unwrap()
            .property_domain(&iri("Q"), &iri("Person"))
            .unwrap()
            .object_property_assertion(&iri("a"), &iri("P"), &iri("b"))
            .unwrap()
            .build()
            .unwrap();

        let added = apply_domain_range_inheritance(&mut ontology).expect("postprocess");
        assert!(added >= 1, "expected at least one inferred ClassAssertion");
        assert!(is_typed(
            &ontology,
            ontology.lookup_entity(&iri("a")).unwrap(),
            ontology.lookup_entity(&iri("Person")).unwrap()
        ));
    }
}
