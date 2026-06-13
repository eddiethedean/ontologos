//! Datatype ABox/TBox consistency via [`LiteralIndex`].

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, Ontology};

use super::{LiteralIndex, LiteralValue};

#[derive(Debug, Clone)]
enum DataRestriction {
    All(DeId),
    Some(DeId),
    HasValue(DeId),
    MinCardinality(u32, Option<DeId>),
    MaxCardinality(u32, Option<DeId>),
    ExactCardinality(u32, Option<DeId>),
}

/// Returns false when datatype constraints are unsatisfiable.
#[must_use]
pub fn is_datatype_consistent(ontology: &Ontology) -> bool {
    let store = ontology.dl();
    if store.axiom_count() == 0 && store.de_count() == 0 {
        return true;
    }

    let idx = LiteralIndex::from_store(store);
    let mut class_restrictions: HashMap<EntityId, Vec<(EntityId, DataRestriction)>> =
        HashMap::new();

    for axiom in store.axioms() {
        if let DlAxiom::SubClassOf { sub, sup } = axiom {
            let Some(ClassExpr::Atomic(class)) = store.ce(*sub) else {
                continue;
            };
            if let Some((prop, restriction)) = restriction_from_ce(store, *sup) {
                class_restrictions
                    .entry(*class)
                    .or_default()
                    .push((prop, restriction));
            }
        }
    }

    let mut individual_restrictions: HashMap<EntityId, Vec<(EntityId, DataRestriction)>> =
        HashMap::new();

    for axiom in store.axioms() {
        if let DlAxiom::ClassAssertion { individual, class } = axiom {
            if let Some((prop, restriction)) = restriction_from_ce(store, *class) {
                individual_restrictions
                    .entry(*individual)
                    .or_default()
                    .push((prop, restriction));
            }
            if let Some(thing) = owl_thing_id(ontology) {
                if let Some(restrictions) = class_restrictions.get(&thing) {
                    individual_restrictions
                        .entry(*individual)
                        .or_default()
                        .extend(restrictions.iter().cloned());
                }
            }
        }
    }

    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom {
            if let Some(restrictions) = class_restrictions.get(class) {
                individual_restrictions
                    .entry(*individual)
                    .or_default()
                    .extend(restrictions.iter().cloned());
            }
            if let Some(thing) = owl_thing_id(ontology) {
                if let Some(restrictions) = class_restrictions.get(&thing) {
                    individual_restrictions
                        .entry(*individual)
                        .or_default()
                        .extend(restrictions.iter().cloned());
                }
            }
        }
    }

    for (_, restrictions) in &individual_restrictions {
        if !restrictions_satisfiable(ontology, &idx, restrictions) {
            return false;
        }
    }

    true
}

fn restriction_from_ce(
    store: &ontologos_core::DlStore,
    ce: CeId,
) -> Option<(EntityId, DataRestriction)> {
    let expr = store.ce(ce)?;
    match expr {
        ClassExpr::DataAll { property, range } => Some((*property, DataRestriction::All(*range))),
        ClassExpr::DataSome { property, range } => Some((*property, DataRestriction::Some(*range))),
        ClassExpr::DataHasValue { property, value } => {
            Some((*property, DataRestriction::HasValue(*value)))
        }
        ClassExpr::DataMinCardinality { n, property, range } => Some((
            *property,
            DataRestriction::MinCardinality(*n, *range),
        )),
        ClassExpr::DataMaxCardinality { n, property, range } => Some((
            *property,
            DataRestriction::MaxCardinality(*n, *range),
        )),
        ClassExpr::DataExactCardinality { n, property, range } => Some((
            *property,
            DataRestriction::ExactCardinality(*n, *range),
        )),
        ClassExpr::And(ops) => {
            for op in ops {
                if let Some(r) = restriction_from_ce(store, *op) {
                    return Some(r);
                }
            }
            None
        }
        _ => None,
    }
}

fn restrictions_satisfiable(
    ontology: &Ontology,
    idx: &LiteralIndex,
    restrictions: &[(EntityId, DataRestriction)],
) -> bool {
    let mut by_property: HashMap<EntityId, Vec<DataRestriction>> = HashMap::new();
    for (prop, restriction) in restrictions {
        by_property.entry(*prop).or_default().push(restriction.clone());
    }

    for group in by_property.values() {
        if !property_restrictions_satisfiable(ontology, idx, group) {
            return false;
        }
    }
    true
}

fn property_restrictions_satisfiable(
    ontology: &Ontology,
    idx: &LiteralIndex,
    group: &[DataRestriction],
) -> bool {
    let mut all_ranges: Vec<DeId> = Vec::new();
    let mut some_ranges: Vec<DeId> = Vec::new();
    let mut min_card: u32 = 0;
    let mut max_card: Option<u32> = None;
    let mut exact_card: Option<u32> = None;
    let mut fixed_values: Vec<DeId> = Vec::new();

    for r in group {
        match r {
            DataRestriction::All(range) => all_ranges.push(*range),
            DataRestriction::Some(range) => some_ranges.push(*range),
            DataRestriction::HasValue(value) => fixed_values.push(*value),
            DataRestriction::MinCardinality(n, range) => {
                min_card = min_card.max(*n);
                if let Some(dr) = range {
                    some_ranges.push(*dr);
                }
            }
            DataRestriction::MaxCardinality(n, range) => {
                max_card = Some(max_card.map_or(*n, |m| m.min(*n)));
                if let Some(dr) = range {
                    all_ranges.push(*dr);
                }
            }
            DataRestriction::ExactCardinality(n, range) => {
                exact_card = Some(*n);
                min_card = min_card.max(*n);
                max_card = Some(max_card.map_or(*n, |m| m.min(*n)));
                if let Some(dr) = range {
                    all_ranges.push(*dr);
                    some_ranges.push(*dr);
                }
            }
        }
    }

    if let Some(exact) = exact_card {
        min_card = min_card.max(exact);
        max_card = Some(max_card.map_or(exact, |m| m.min(exact)));
    }

    let mut combined_all = all_ranges;
    for value in &fixed_values {
        combined_all.push(*value);
    }

    if min_card > 0 {
        let mut witness_ranges = combined_all.clone();
        witness_ranges.extend(some_ranges.clone());
        if witness_ranges.is_empty() {
            return false;
        }
        if distinct_values_satisfying_ranges(ontology, idx, &witness_ranges) < min_card {
            return false;
        }
    }

    if !some_ranges.is_empty() && !combined_all.is_empty() {
        let mut witness_ranges = combined_all.clone();
        witness_ranges.extend(some_ranges.clone());
        if distinct_values_satisfying_ranges(ontology, idx, &witness_ranges) == 0 {
            return false;
        }
    }

    if let Some(max) = max_card {
        let mut witness_ranges = combined_all.clone();
        witness_ranges.extend(some_ranges.clone());
        if witness_ranges.is_empty() {
            return max > 0;
        }
        if distinct_values_satisfying_ranges(ontology, idx, &witness_ranges) < max {
            return false;
        }
    }

    for value in &fixed_values {
        if !combined_all.is_empty() {
            let Some(lit) = literal_from_de(ontology, value) else {
                continue;
            };
            if !satisfies_all_ranges(ontology, idx, &lit, &combined_all) {
                return false;
            }
        }
    }

    true
}

fn literal_from_de(ontology: &Ontology, value: &DeId) -> Option<LiteralValue> {
    let store = ontology.dl();
    let DataExpr::Literal { lexical, datatype } = store.de(*value)? else {
        return None;
    };
    Some(LiteralValue {
        lexical: lexical.clone(),
        datatype: *datatype,
    })
}

fn distinct_values_satisfying_ranges(
    ontology: &Ontology,
    idx: &LiteralIndex,
    ranges: &[DeId],
) -> u32 {
    if ranges.is_empty() {
        return u32::MAX;
    }
    if ranges.len() == 1 {
        return max_distinct_values(ontology, idx, ranges[0]);
    }
    let mut candidates = sample_literals(ontology, ranges[0]);
    for &range in &ranges[1..] {
        candidates.extend(sample_literals(ontology, range));
    }
    let mut seen = HashSet::new();
    let mut count = 0_u32;
    for lit in candidates {
        let key = distinct_literal_key(&lit);
        if !seen.insert(key) {
            continue;
        }
        if satisfies_all_ranges(ontology, idx, &lit, ranges) {
            count += 1;
        }
    }
    count
}

fn distinct_literal_key(lit: &LiteralValue) -> String {
    lit.lexical.clone()
}

fn max_distinct_values(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> u32 {
    let store = ontology.dl();
    let Some(expr) = store.de(range) else {
        return u32::MAX;
    };
    match expr {
        DataExpr::Literal { .. } => 1,
        DataExpr::And(ops) => {
            let Some(first) = ops.first().copied() else {
                return 0;
            };
            let mut candidates = sample_literals(ontology, first);
            for &op in &ops[1..] {
                candidates.extend(sample_literals(ontology, op));
            }
            let mut seen = HashSet::new();
            let mut count = 0_u32;
            for lit in candidates {
                let key = distinct_literal_key(&lit);
                if !seen.insert(key) {
                    continue;
                }
                if satisfies_all_ranges(ontology, idx, &lit, ops) {
                    count += 1;
                }
            }
            count
        }
        DataExpr::Or(ops) => {
            let mut seen = HashSet::new();
            let mut count = 0_u32;
            for &op in ops {
                if let Some(DataExpr::Literal { lexical, datatype }) = store.de(op) {
                    let key = (lexical.clone(), *datatype);
                    if seen.insert(key) {
                        count += 1;
                    }
                } else {
                    count += max_distinct_values(ontology, idx, op);
                }
            }
            count
        }
        DataExpr::Facet { base, facet_iri, value } => {
            if facet_contradiction_on_base(store, *base, facet_iri, value) {
                return 0;
            }
            max_distinct_values(ontology, idx, *base)
        }
        DataExpr::Datatype(dt) => {
            if primitive_datatype_is_infinite(ontology, *dt) {
                u32::MAX
            } else {
                u32::MAX
            }
        }
        DataExpr::Top => u32::MAX,
    }
}

fn satisfies_all_ranges(
    ontology: &Ontology,
    idx: &LiteralIndex,
    lit: &LiteralValue,
    ranges: &[DeId],
) -> bool {
    ranges.iter().all(|&r| idx.satisfies_with_ontology(lit, ontology, r))
}

fn facet_contradiction_on_base(
    store: &ontologos_core::DlStore,
    base: DeId,
    facet_iri: &str,
    value: &str,
) -> bool {
    let Some(DataExpr::Datatype(_)) = store.de(base) else {
        return false;
    };
    match facet_iri {
        "http://www.w3.org/2001/XMLSchema#minInclusive" => {
            if let Some(DataExpr::Facet {
                facet_iri: other,
                value: max,
                ..
            }) = store.de(base)
            {
                if other == "http://www.w3.org/2001/XMLSchema#maxInclusive"
                    && numeric_compare(value, max) > 0
                {
                    return true;
                }
            }
        }
        "http://www.w3.org/2001/XMLSchema#maxInclusive" => {
            if let Some(DataExpr::Facet {
                facet_iri: other,
                value: min,
                ..
            }) = store.de(base)
            {
                if other == "http://www.w3.org/2001/XMLSchema#minInclusive"
                    && numeric_compare(min, value) > 0
                {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

fn sample_literals(ontology: &Ontology, range: DeId) -> Vec<LiteralValue> {
    let store = ontology.dl();
    let Some(expr) = store.de(range) else {
        return Vec::new();
    };
    match expr {
        DataExpr::Literal { lexical, datatype } => vec![LiteralValue {
            lexical: lexical.clone(),
            datatype: *datatype,
        }],
        DataExpr::Datatype(dt) => default_witness_literals(ontology, *dt),
        DataExpr::And(ops) => {
            let mut out = Vec::new();
            for &op in ops {
                if let Some(DataExpr::Datatype(dt)) = store.de(op) {
                    out.extend(default_witness_literals(ontology, *dt));
                } else {
                    out.extend(sample_literals(ontology, op));
                }
            }
            out
        }
        DataExpr::Or(ops) => {
            ops.first()
                .map(|&op| sample_literals(ontology, op))
                .unwrap_or_default()
        }
        DataExpr::Facet { base, .. } => sample_literals(ontology, *base),
        DataExpr::Top => Vec::new(),
    }
}

fn owl_thing_id(ontology: &Ontology) -> Option<EntityId> {
    ontology
        .lookup_entity("http://www.w3.org/2002/07/owl#Thing")
        .or_else(|| ontology.lookup_entity("owl:Thing"))
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Option<String> {
    let record = ontology.entity(id).ok()?;
    ontology.resolve_iri(record.iri).ok().map(str::to_owned)
}

fn default_witness_literals(ontology: &Ontology, datatype: EntityId) -> Vec<LiteralValue> {
    let Some(iri) = entity_iri(ontology, datatype) else {
        return Vec::new();
    };
    let witnesses: &[&str] = match iri.as_str() {
        "http://www.w3.org/2001/XMLSchema#integer" => &["0", "1", "2"],
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => &["0", "-1"],
        "http://www.w3.org/2001/XMLSchema#string" => &["a"],
        "http://www.w3.org/2001/XMLSchema#decimal" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#float" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#double" => &["0", "1"],
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString" => &["en"],
        "http://www.w3.org/2001/XMLSchema#dateTime" => &["2000-01-01T00:00:00"],
        _ => &["0"],
    };
    witnesses
        .iter()
        .map(|lex| LiteralValue {
            lexical: (*lex).to_string(),
            datatype,
        })
        .collect()
}

fn primitive_datatype_is_infinite(ontology: &Ontology, dt: EntityId) -> bool {
    let Some(iri) = entity_iri(ontology, dt) else {
        return true;
    };
    matches!(
        iri.as_str(),
        "http://www.w3.org/2001/XMLSchema#integer"
            | "http://www.w3.org/2001/XMLSchema#decimal"
            | "http://www.w3.org/2001/XMLSchema#float"
            | "http://www.w3.org/2001/XMLSchema#double"
            | "http://www.w3.org/2001/XMLSchema#string"
            | "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"
    )
}

fn numeric_compare(a: &str, b: &str) -> i32 {
    let fa: f64 = a.parse().unwrap_or(0.0);
    let fb: f64 = b.parse().unwrap_or(0.0);
    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal) as i32
}
