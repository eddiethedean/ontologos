//! Post-processing for RL/RDFS gaps not yet covered by `reasonable`.

use std::collections::{HashMap, HashSet, VecDeque};

use ontologos_core::{Axiom, CeId, ClassExpr, DlAxiom, EntityId, Ontology, RoleExpr};

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

/// Materialize named `P ⊑ Q` when DL stores `Inv(P) ⊑ Inv(Q)`.
pub fn apply_inverse_subproperty_materialization(ontology: &mut Ontology) -> Result<usize> {
    let pairs: Vec<(EntityId, EntityId)> = ontology
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::SubObjectPropertyOf { sub, sup } = axiom else {
                return None;
            };
            match (sub, sup) {
                (RoleExpr::Inverse(sub), RoleExpr::Inverse(sup)) => Some((*sub, *sup)),
                _ => None,
            }
        })
        .collect();

    let mut added = 0_usize;
    for (sub, sup) in pairs {
        if push_subproperty_if_missing(ontology, sub, sup)? {
            added += 1;
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

/// Materialize transitive `subDataPropertyOf` closure from DL axioms.
pub fn apply_transitive_data_subproperties(ontology: &mut Ontology) -> Result<usize> {
    let direct = data_superproperty_edges(ontology);
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
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::SubDataPropertyOf { sub, sup } = axiom else {
                return None;
            };
            Some((*sub, *sup))
        })
        .collect();

    let mut added = 0_usize;
    for (sub, sup) in all_pairs {
        if !existing.contains(&(sub, sup)) {
            ontology
                .dl_mut()
                .push_axiom(DlAxiom::SubDataPropertyOf { sub, sup });
            added += 1;
        }
    }
    Ok(added)
}

/// When `Q` has singleton domain/range matching an assertion of `P`, infer `Q ⊑ P`.
pub fn apply_domain_range_nominal_subsumption(ontology: &mut Ontology) -> Result<usize> {
    let mut assertions: Vec<(EntityId, EntityId, EntityId)> = Vec::new();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::ObjectPropertyAssertion {
            subject,
            property: RoleExpr::Atomic(prop),
            object,
        } = axiom
        {
            assertions.push((*subject, *prop, *object));
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        {
            assertions.push((*subject, *property, *object));
        }
    }

    let props: Vec<EntityId> = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    let mut added = 0_usize;
    for &q in &props {
        let Some((d, r)) = property_endpoint_singletons(ontology, q) else {
            continue;
        };
        for &(s, p, o) in &assertions {
            if s == d && o == r && p != q && push_subproperty_if_missing(ontology, q, p)? {
                added += 1;
            }
        }
    }
    Ok(added)
}
pub fn apply_equivalent_property_subproperties(ontology: &mut Ontology) -> Result<usize> {
    let mut clusters: Vec<Vec<EntityId>> = Vec::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::EquivalentObjectProperties(properties) = axiom {
            if properties.len() >= 2 {
                clusters.push(properties.clone());
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
    let mut seen = existing;
    for group in clusters {
        for i in 0..group.len() {
            for j in 0..group.len() {
                if i == j {
                    continue;
                }
                let sub = group[i];
                let sup = group[j];
                if seen.insert((sub, sup)) {
                    ontology.add_inferred_axiom(Axiom::SubObjectPropertyOf {
                        sub_property: sub,
                        super_property: sup,
                    })?;
                    added += 1;
                }
            }
        }
    }
    Ok(added)
}

/// Propagate property characteristics along `subPropertyOf` (CharPropagate).
pub fn apply_characteristic_propagation(ontology: &mut Ontology) -> Result<usize> {
    let sub_to_supers = superproperty_edges(ontology);
    let index = ontology.index();
    let mut functional: HashSet<EntityId> = index.functional_properties().iter().copied().collect();
    let mut inverse_functional: HashSet<EntityId> = index
        .inverse_functional_properties()
        .iter()
        .copied()
        .collect();
    let mut asymmetric: HashSet<EntityId> = index.asymmetric_properties().iter().copied().collect();
    let mut reflexive: HashSet<EntityId> = index.reflexive_properties().iter().copied().collect();
    let mut irreflexive: HashSet<EntityId> =
        index.irreflexive_properties().iter().copied().collect();

    let mut added = 0_usize;
    for sub in sub_to_supers.keys() {
        for sup in transitive_supers(*sub, &sub_to_supers) {
            if functional.contains(&sup) && functional.insert(*sub) {
                ontology.add_inferred_axiom(Axiom::FunctionalObjectProperty(*sub))?;
                added += 1;
            }
            if inverse_functional.contains(&sup) && inverse_functional.insert(*sub) {
                ontology.add_inferred_axiom(Axiom::InverseFunctionalObjectProperty(*sub))?;
                added += 1;
            }
            if asymmetric.contains(&sup) && asymmetric.insert(*sub) {
                ontology.add_inferred_axiom(Axiom::AsymmetricObjectProperty(*sub))?;
                added += 1;
            }
            if irreflexive.contains(&sup) && irreflexive.insert(*sub) {
                ontology.add_inferred_axiom(Axiom::IrreflexiveObjectProperty(*sub))?;
                added += 1;
            }
        }
        if reflexive.contains(sub) {
            for sup in transitive_supers(*sub, &sub_to_supers) {
                if reflexive.insert(sup) {
                    ontology.add_inferred_axiom(Axiom::ReflexiveObjectProperty(sup))?;
                    added += 1;
                }
            }
        }
    }
    Ok(added)
}

/// Infer named `SubClassOf` from existential class definitions (cls-svf1/2).
///
/// `A ⊑ ∃P.C` and `B ⊑ ∃Q.D` yields `A ⊑ B` when `P ⊑ Q` and `C ⊑ D`.
pub fn apply_existential_subclass_subsumption(ontology: &mut Ontology) -> Result<usize> {
    let defs = existential_class_definitions(ontology);
    if defs.len() < 2 {
        return Ok(0);
    }

    let sub_to_supers = superproperty_edges(ontology);
    let mut added = 0_usize;
    for i in 0..defs.len() {
        for j in 0..defs.len() {
            if i == j {
                continue;
            }
            let (sub, sub_prop, filler) = defs[i];
            let (sup, sup_prop, sup_filler) = defs[j];
            if !property_subsumed(sub_prop, sup_prop, &sub_to_supers) {
                continue;
            }
            if !class_subsumed(filler, sup_filler, ontology) {
                continue;
            }
            if push_subclass_if_missing(ontology, sub, sup)? {
                added += 1;
            }
        }
    }
    Ok(added)
}

fn existential_class_definitions(ontology: &Ontology) -> Vec<(EntityId, EntityId, EntityId)> {
    let mut out = Vec::new();
    let store = ontology.dl();

    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        for w in ids.windows(2) {
            push_existential_def(&mut out, store, w[0], w[1]);
            push_existential_def(&mut out, store, w[1], w[0]);
        }
    }

    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } = axiom
        {
            out.push((*subclass, *property, *filler));
        }
    }

    out.sort_unstable_by_key(|(c, p, f)| (c.0, p.0, f.0));
    out.dedup();
    out
}

fn push_existential_def(
    out: &mut Vec<(EntityId, EntityId, EntityId)>,
    store: &ontologos_core::DlStore,
    class: CeId,
    expr: CeId,
) {
    let (Some(ClassExpr::Atomic(class_id)), Some(ClassExpr::Some { property, filler })) =
        (store.ce(class), store.ce(expr))
    else {
        return;
    };
    let RoleExpr::Atomic(prop_id) = property else {
        return;
    };
    let Some(ClassExpr::Atomic(filler_id)) = store.ce(*filler) else {
        return;
    };
    out.push((*class_id, *prop_id, *filler_id));
}

fn class_subsumed(sub: EntityId, sup: EntityId, ontology: &Ontology) -> bool {
    if sub == sup {
        return true;
    }
    let mut queue = VecDeque::from([sub]);
    let mut seen = HashSet::from([sub]);
    while let Some(current) = queue.pop_front() {
        for &superclass in ontology.direct_superclasses(current) {
            if superclass == sup {
                return true;
            }
            if seen.insert(superclass) {
                queue.push_back(superclass);
            }
        }
    }
    false
}

fn property_subsumed(
    sub: EntityId,
    sup: EntityId,
    sub_to_supers: &HashMap<EntityId, Vec<EntityId>>,
) -> bool {
    if sub == sup {
        return true;
    }
    let mut queue = VecDeque::from([sub]);
    let mut seen = HashSet::from([sub]);
    while let Some(current) = queue.pop_front() {
        if current == sup {
            return true;
        }
        if let Some(next) = sub_to_supers.get(&current) {
            for &n in next {
                if seen.insert(n) {
                    queue.push_back(n);
                }
            }
        }
    }
    false
}

fn push_subclass_if_missing(
    ontology: &mut Ontology,
    subclass: EntityId,
    superclass: EntityId,
) -> Result<bool> {
    if subclass == superclass {
        return Ok(false);
    }
    let exists = ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::SubClassOf {
                subclass: sub,
                superclass: sup,
            } if *sub == subclass && *sup == superclass
        )
    });
    if exists {
        return Ok(false);
    }
    ontology.add_inferred_axiom(Axiom::SubClassOf {
        subclass,
        superclass,
    })?;
    Ok(true)
}

/// Run all reasonable semantic fallbacks after materialization.
pub fn apply_reasonable_fallbacks(ontology: &mut Ontology) -> Result<usize> {
    let mut total = apply_inverse_subproperty_materialization(ontology)?;
    total += apply_equivalent_property_subproperties(ontology)?;
    total += apply_transitive_subproperties(ontology)?;
    total += apply_transitive_data_subproperties(ontology)?;
    total += apply_characteristic_propagation(ontology)?;
    total += propagate_domain_range_along_subproperties(ontology)?;
    total += apply_domain_range_inheritance(ontology)?;
    total += apply_existential_subclass_subsumption(ontology)?;
    total += apply_singleton_domain_range_property_equivalence(ontology)?;
    total += apply_domain_range_nominal_subsumption(ontology)?;
    total += apply_transitive_path_property_subsumption(ontology)?;
    total += apply_functional_data_subproperty_inference(ontology)?;
    Ok(total)
}

/// When two object properties share singleton domain+range and identical assertion footprint, infer equivalence.
pub fn apply_singleton_domain_range_property_equivalence(ontology: &mut Ontology) -> Result<usize> {
    let domains = property_domains(ontology);
    let ranges = property_ranges(ontology);
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

    let mut props: Vec<EntityId> = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();
    props.sort_by_key(|id| id.0);

    let mut added = 0_usize;
    for i in 0..props.len() {
        for j in (i + 1)..props.len() {
            let p1 = props[i];
            let p2 = props[j];
            let Some(d1) = singleton_individual_for_class_domain(ontology, p1, &domains) else {
                continue;
            };
            let Some(r1) = singleton_individual_for_class_range(ontology, p1, &ranges) else {
                continue;
            };
            let Some(d2) = singleton_individual_for_class_domain(ontology, p2, &domains) else {
                continue;
            };
            let Some(r2) = singleton_individual_for_class_range(ontology, p2, &ranges) else {
                continue;
            };
            if d1 != d2 || r1 != r2 {
                continue;
            }
            let footprint1: HashSet<(EntityId, EntityId)> = assertions
                .iter()
                .filter(|(_, prop, _)| *prop == p1)
                .map(|(s, _, o)| (*s, *o))
                .collect();
            let footprint2: HashSet<(EntityId, EntityId)> = assertions
                .iter()
                .filter(|(_, prop, _)| *prop == p2)
                .map(|(s, _, o)| (*s, *o))
                .collect();
            if footprint1.is_empty() || footprint1 != footprint2 {
                continue;
            }
            if push_subproperty_if_missing(ontology, p1, p2)? {
                added += 1;
            }
            if push_subproperty_if_missing(ontology, p2, p1)? {
                added += 1;
            }
        }
    }
    Ok(added)
}

/// `P ⊑ Q` when `P` has singleton domain/range `a`,`b` and transitive `Q` connects `a` to `b`.
pub fn apply_transitive_path_property_subsumption(ontology: &mut Ontology) -> Result<usize> {
    let transitive: HashSet<EntityId> = ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::TransitiveObjectProperty(prop) => Some(*prop),
            _ => None,
        })
        .collect();
    if transitive.is_empty() {
        return Ok(0);
    }

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

    let props: Vec<EntityId> = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    let mut added = 0_usize;
    for &p in &props {
        let Some((a, b)) = property_endpoint_singletons(ontology, p) else {
            continue;
        };
        for &q in &transitive {
            if q == p || !transitive_path(&assertions, q, a, b) {
                continue;
            }
            if push_subproperty_if_missing(ontology, p, q)? {
                added += 1;
            }
        }
    }
    Ok(added)
}

fn property_endpoint_singletons(
    ontology: &Ontology,
    property: EntityId,
) -> Option<(EntityId, EntityId)> {
    let store = ontology.dl();
    let mut domain_ind = None;
    let mut range_ind = None;
    for axiom in store.axioms() {
        if let DlAxiom::ObjectPropertyDomain {
            property: prop,
            domain,
        } = axiom
        {
            if *prop == property {
                if let Some(ClassExpr::OneOf(v)) = store.ce(*domain) {
                    if v.len() == 1 {
                        domain_ind = Some(v[0]);
                    }
                } else if let Some(ClassExpr::Atomic(class)) = store.ce(*domain) {
                    domain_ind = singleton_from_equivalent_classes(ontology, *class);
                }
            }
        }
        if let DlAxiom::ObjectPropertyRange {
            property: prop,
            range,
        } = axiom
        {
            if *prop == property {
                if let Some(ClassExpr::OneOf(v)) = store.ce(*range) {
                    if v.len() == 1 {
                        range_ind = Some(v[0]);
                    }
                } else if let Some(ClassExpr::Atomic(class)) = store.ce(*range) {
                    range_ind = singleton_from_equivalent_classes(ontology, *class);
                }
            }
        }
    }
    let domains = property_domains(ontology);
    let ranges = property_ranges(ontology);
    if domain_ind.is_none() {
        domain_ind = domains
            .get(&property)
            .and_then(|class| singleton_from_equivalent_classes(ontology, *class));
    }
    if range_ind.is_none() {
        range_ind = ranges
            .get(&property)
            .and_then(|class| singleton_from_equivalent_classes(ontology, *class));
    }
    match (domain_ind, range_ind) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

/// Functional data subproperties inherit functionality from super data properties.
pub fn apply_functional_data_subproperty_inference(ontology: &mut Ontology) -> Result<usize> {
    let sub_to_supers = data_superproperty_edges(ontology);
    let functional: HashSet<EntityId> = ontology
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::FunctionalDataProperty(prop) = axiom else {
                return None;
            };
            Some(*prop)
        })
        .collect();
    let mut added = 0_usize;
    for &functional_prop in &functional {
        for (sub, supers) in &sub_to_supers {
            if supers.contains(&functional_prop)
                && ontology
                    .entity(*sub)
                    .ok()
                    .is_some_and(|r| r.kind == ontologos_core::EntityKind::DataProperty)
            {
                let exists = ontology.dl().axioms().any(|axiom| {
                    matches!(
                        axiom,
                        DlAxiom::FunctionalDataProperty(p) if *p == *sub
                    )
                });
                if !exists {
                    ontology
                        .dl_mut()
                        .push_axiom(DlAxiom::FunctionalDataProperty(*sub));
                    added += 1;
                }
            }
        }
    }
    Ok(added)
}

fn push_subproperty_if_missing(
    ontology: &mut Ontology,
    sub: EntityId,
    sup: EntityId,
) -> Result<bool> {
    if sub == sup {
        return Ok(false);
    }
    let exists = ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } if *sub_property == sub && *super_property == sup
        )
    });
    if exists {
        return Ok(false);
    }
    ontology.add_inferred_axiom(Axiom::SubObjectPropertyOf {
        sub_property: sub,
        super_property: sup,
    })?;
    Ok(true)
}

fn singleton_individual_for_class_domain(
    ontology: &Ontology,
    property: EntityId,
    domains: &HashMap<EntityId, EntityId>,
) -> Option<EntityId> {
    let domain = *domains.get(&property)?;
    singleton_individual_of_class(ontology, domain)
}

fn singleton_individual_for_class_range(
    ontology: &Ontology,
    property: EntityId,
    ranges: &HashMap<EntityId, EntityId>,
) -> Option<EntityId> {
    let range = *ranges.get(&property)?;
    singleton_individual_of_class(ontology, range)
}

fn singleton_individual_of_class(ontology: &Ontology, class: EntityId) -> Option<EntityId> {
    if let Some(ind) = singleton_from_equivalent_classes(ontology, class) {
        return Some(ind);
    }
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::ObjectPropertyDomain { domain, .. } = axiom else {
            continue;
        };
        if let Some(ClassExpr::OneOf(individuals)) = store.ce(*domain) {
            if individuals.len() == 1 {
                return Some(individuals[0]);
            }
        }
        if let Some(ClassExpr::Atomic(domain_class)) = store.ce(*domain) {
            if *domain_class == class {
                if let Some(ind) = singleton_from_equivalent_classes(ontology, class) {
                    return Some(ind);
                }
            }
        }
    }
    for axiom in store.axioms() {
        let DlAxiom::ObjectPropertyRange { range, .. } = axiom else {
            continue;
        };
        if let Some(ClassExpr::OneOf(individuals)) = store.ce(*range) {
            if individuals.len() == 1 {
                return Some(individuals[0]);
            }
        }
    }
    None
}

fn singleton_from_equivalent_classes(ontology: &Ontology, class: EntityId) -> Option<EntityId> {
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        let has_class = ids
            .iter()
            .any(|&id| matches!(store.ce(id), Some(ClassExpr::Atomic(c)) if *c == class));
        if !has_class {
            continue;
        }
        for &id in ids {
            if let Some(ClassExpr::OneOf(individuals)) = store.ce(id) {
                if individuals.len() == 1 {
                    return Some(individuals[0]);
                }
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::EquivalentClasses(classes) = axiom {
            if !classes.contains(&class) {
                continue;
            }
            for &other in classes {
                if other == class {
                    continue;
                }
                if let Some(ind) = singleton_from_equivalent_classes(ontology, other) {
                    return Some(ind);
                }
            }
        }
    }
    None
}

fn transitive_path(
    assertions: &[(EntityId, EntityId, EntityId)],
    property: EntityId,
    from: EntityId,
    to: EntityId,
) -> bool {
    if from == to {
        return true;
    }
    let mut seen = HashSet::from([from]);
    let mut queue = VecDeque::from([from]);
    while let Some(cur) = queue.pop_front() {
        for &(subj, prop, obj) in assertions {
            if subj == cur && prop == property && seen.insert(obj) {
                if obj == to {
                    return true;
                }
                queue.push_back(obj);
            }
        }
    }
    false
}

/// Detect inconsistency from property chains subsumed by `owl:bottomObjectProperty`.
#[must_use]
pub fn has_bottom_chain_violation(ontology: &Ontology) -> bool {
    let bottom = ontology.entities().iter().find_map(|(id, record)| {
        ontology
            .resolve_iri(record.iri)
            .ok()
            .filter(|iri| iri.ends_with("bottomObjectProperty"))
            .map(|_| id)
    });

    let Some(bottom_id) = bottom else {
        return false;
    };

    for axiom in ontology.dl().axioms() {
        let DlAxiom::SubObjectPropertyChain {
            chain,
            super_property,
        } = axiom
        else {
            continue;
        };
        if !role_is_bottom(super_property, bottom_id) {
            continue;
        }
        if chain_has_assertion_path(ontology, chain) {
            return true;
        }
    }
    false
}

fn role_is_bottom(role: &RoleExpr, bottom_id: EntityId) -> bool {
    matches!(role, RoleExpr::Atomic(id) if *id == bottom_id)
}

fn chain_has_assertion_path(ontology: &Ontology, chain: &[RoleExpr]) -> bool {
    if chain.is_empty() {
        return false;
    }

    let starters: Vec<EntityId> = ontology
        .entities()
        .iter()
        .filter_map(|(id, record)| {
            if record.kind == ontologos_core::EntityKind::Individual {
                Some(id)
            } else {
                None
            }
        })
        .filter(|id| !ontology.object_assertions_of(*id).is_empty())
        .collect();
    if starters.is_empty() {
        return false;
    }

    let mut current: HashSet<EntityId> = starters.into_iter().collect();
    for role in chain {
        let Some(prop) = atomic_role(role) else {
            return false;
        };
        let mut next = HashSet::new();
        for subject in &current {
            for &(property, object) in ontology.object_assertions_of(*subject) {
                if property == prop {
                    next.insert(object);
                }
            }
        }
        if next.is_empty() {
            return false;
        }
        current = next;
    }
    !current.is_empty()
}

fn atomic_role(role: &RoleExpr) -> Option<EntityId> {
    match role {
        RoleExpr::Atomic(id) => Some(*id),
        RoleExpr::Inverse(_) => None,
    }
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

fn data_superproperty_edges(ontology: &Ontology) -> HashMap<EntityId, Vec<EntityId>> {
    let mut edges = HashMap::new();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::SubDataPropertyOf { sub, sup } = axiom {
            edges.entry(*sub).or_insert_with(Vec::new).push(*sup);
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

    #[test]
    fn subsumption2_existential_property_hierarchy() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testsubsumption2.ofn",
        );
        let mut ontology = ontologos_parser::load_ontology(&path).expect("load ofn");
        let added = apply_existential_subclass_subsumption(&mut ontology).expect("postprocess");
        assert!(added >= 1, "expected A ⊑ B from ∃R.C / ∃S.C with R ⊑ S");
        let a = ontology.lookup_entity("file:/c/test.owl#A").expect("A");
        let b = ontology.lookup_entity("file:/c/test.owl#B").expect("B");
        assert!(ontology.direct_superclasses(a).contains(&b));
    }

    #[test]
    fn existential_filler_subclass_subsumption() {
        let mut ontology = Ontology::builder()
            .class(&iri("A"))
            .unwrap()
            .class(&iri("B"))
            .unwrap()
            .class(&iri("D1"))
            .unwrap()
            .class(&iri("D2"))
            .unwrap()
            .object_property(&iri("R"))
            .unwrap()
            .subclass_of(&iri("D1"), &iri("D2"))
            .unwrap()
            .build()
            .unwrap();

        let a = ontology.lookup_entity(&iri("A")).unwrap();
        let b = ontology.lookup_entity(&iri("B")).unwrap();
        let d1 = ontology.lookup_entity(&iri("D1")).unwrap();
        let r = ontology.lookup_entity(&iri("R")).unwrap();
        ontology
            .add_axiom(Axiom::SubClassOfExistential {
                subclass: a,
                property: r,
                filler: d1,
            })
            .unwrap();
        let d2 = ontology.lookup_entity(&iri("D2")).unwrap();
        ontology
            .add_axiom(Axiom::SubClassOfExistential {
                subclass: b,
                property: r,
                filler: d2,
            })
            .unwrap();

        let added = apply_existential_subclass_subsumption(&mut ontology).expect("postprocess");
        assert!(added >= 1, "expected A ⊑ B from ∃R.D1 / ∃R.D2 with D1 ⊑ D2");
        assert!(ontology.direct_superclasses(a).contains(&b));
    }

    #[test]
    fn functional_characteristic_propagates_to_subproperty() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testisfunctionalobject.ofn",
        );
        let mut ontology = ontologos_parser::load_ontology(&path).expect("load ofn");
        let added = apply_characteristic_propagation(&mut ontology).expect("postprocess");
        assert!(added >= 1, "expected functional OP to propagate to SOP");
        let sop = ontology.lookup_entity("file:/c/test.owl#SOP").expect("SOP");
        assert!(ontology.index().functional_properties().contains(&sop));
    }

    #[test]
    fn inverse_subproperty_materialization() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testobjectpropertyhierarchy.ofn",
        );
        let mut ontology = ontologos_parser::load_ontology(&path).expect("load ofn");
        let added = apply_inverse_subproperty_materialization(&mut ontology).expect("postprocess");
        assert!(added >= 2, "expected inv(r3) ⊑ inv(r1) and inv(r2) ⊑ inv(r1) to materialize");
        let r1 = ontology.lookup_entity("file:/c/test.owl#r1").expect("r1");
        let r3 = ontology.lookup_entity("file:/c/test.owl#r3").expect("r3");
        assert!(ontology.direct_superproperties(r3).contains(&r1));
    }

    #[test]
    fn asymmetric_characteristic_propagates_to_subproperty() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testisasymmetricobject.ofn",
        );
        let mut ontology = ontologos_parser::load_ontology(&path).expect("load ofn");
        let added = apply_characteristic_propagation(&mut ontology).expect("postprocess");
        assert!(added >= 1, "expected asymmetric OP to propagate to SOP1");
        let sop1 = ontology
            .lookup_entity("file:/c/test.owl#SOP1")
            .expect("SOP1");
        assert!(ontology.index().asymmetric_properties().contains(&sop1));
    }

    #[test]
    fn subsumption3_existential_equivalent_properties() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testsubsumption3.ofn",
        );
        let mut ontology = ontologos_parser::load_ontology(&path).expect("load ofn");
        apply_equivalent_property_subproperties(&mut ontology).expect("equiv props");
        let added = apply_existential_subclass_subsumption(&mut ontology).expect("postprocess");
        assert!(added >= 2, "expected mutual A/B subsumption");
        let a = ontology.lookup_entity("file:/c/test.owl#A").expect("A");
        let b = ontology.lookup_entity("file:/c/test.owl#B").expect("B");
        assert!(ontology.direct_superclasses(a).contains(&b));
        assert!(ontology.direct_superclasses(b).contains(&a));
    }

    #[test]
    fn bottom_object_property_chain_with_assertions_is_inconsistent() {
        use std::path::Path;

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testbottomobjectpropertyassertion.ofn",
        );
        let ontology = ontologos_parser::load_ontology(&path).expect("load ofn");
        assert!(
            has_bottom_chain_violation(&ontology),
            "expected bottom chain clash when r(a,b) and s(b,c) with chain r;s ⊑ bottom"
        );
    }
}
