//! Datatype ABox/TBox consistency via [`LiteralIndex`].

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, Ontology};

use super::{canonical_plain_literal, LiteralIndex, LiteralValue, literals_equal};

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
        if let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        {
            individual_restrictions
                .entry(*subject)
                .or_default()
                .push((*property, DataRestriction::HasValue(*value)));
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

    let disjoint_pairs = disjoint_data_property_pairs(store);

    for (individual, restrictions) in &individual_restrictions {
        if !restrictions_satisfiable(ontology, &idx, restrictions) {
            return false;
        }
        if !negative_assertions_consistent(ontology, &idx, store, *individual, restrictions) {
            return false;
        }
        if !disjoint_assertions_consistent(ontology, *individual, restrictions, &disjoint_pairs) {
            return false;
        }
    }

    true
}

fn disjoint_data_property_pairs(store: &ontologos_core::DlStore) -> Vec<(EntityId, EntityId)> {
    let mut pairs = Vec::new();
    for axiom in store.axioms() {
        if let DlAxiom::DisjointDataProperties(props) = axiom {
            for w in props.windows(2) {
                pairs.push((w[0], w[1]));
            }
            if props.len() > 2 {
                for i in 0..props.len() {
                    for j in (i + 1)..props.len() {
                        pairs.push((props[i], props[j]));
                    }
                }
            }
        }
    }
    pairs
}

fn negative_assertions_consistent(
    ontology: &Ontology,
    idx: &LiteralIndex,
    store: &ontologos_core::DlStore,
    individual: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
) -> bool {
    for axiom in store.axioms() {
        let DlAxiom::NegativeDataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        if *subject != individual {
            continue;
        }
        let Some(lit) = literal_from_de(ontology, value) else {
            continue;
        };
        let prop_restrictions: Vec<_> = restrictions
            .iter()
            .filter(|(p, _)| p == property)
            .collect();
        if prop_restrictions.is_empty() {
            continue;
        }
        if property_requires_literal(ontology, idx, &prop_restrictions, &lit) {
            return false;
        }
    }
    true
}

fn property_requires_literal(
    ontology: &Ontology,
    idx: &LiteralIndex,
    group: &[&(EntityId, DataRestriction)],
    lit: &LiteralValue,
) -> bool {
    let mut all_ranges: Vec<DeId> = Vec::new();
    let mut some_ranges: Vec<DeId> = Vec::new();
    let mut min_card: u32 = 0;
    let mut fixed_values: Vec<DeId> = Vec::new();

    for (_, r) in group {
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
            DataRestriction::MaxCardinality(_, range) => {
                if let Some(dr) = range {
                    all_ranges.push(*dr);
                }
            }
            DataRestriction::ExactCardinality(n, range) => {
                min_card = min_card.max(*n);
                if let Some(dr) = range {
                    all_ranges.push(*dr);
                    some_ranges.push(*dr);
                }
            }
        }
    }

    for value in &fixed_values {
        if let Some(fixed) = literal_from_de(ontology, value) {
            if literals_equal_local(&fixed, lit) {
                return true;
            }
        }
    }

    if min_card > 0 {
        let mut witness_ranges = all_ranges.clone();
        witness_ranges.extend(some_ranges.clone());
        if witness_ranges.is_empty() {
            return false;
        }
        if idx.satisfies_with_ontology(lit, ontology, witness_ranges[0])
            && (witness_ranges.len() == 1
                || satisfies_all_ranges(ontology, idx, lit, &witness_ranges))
        {
            return true;
        }
    }

    if !some_ranges.is_empty() {
        return some_ranges
            .iter()
            .any(|&r| idx.satisfies_with_ontology(lit, ontology, r));
    }

    false
}

fn disjoint_assertions_consistent(
    ontology: &Ontology,
    individual: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
    disjoint_pairs: &[(EntityId, EntityId)],
) -> bool {
    let store = ontology.dl();
    let mut used_props: HashSet<EntityId> = HashSet::new();

    for axiom in store.axioms() {
        if let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            ..
        } = axiom
        {
            if *subject == individual {
                used_props.insert(*property);
            }
        }
    }

    for (prop, _) in restrictions {
        if restrictions
            .iter()
            .any(|(p, r)| p == prop && requires_nonempty_property(r))
        {
            used_props.insert(*prop);
        }
    }

    for &(a, b) in disjoint_pairs {
        if used_props.contains(&a) && used_props.contains(&b) {
            return false;
        }
    }
    true
}

fn requires_nonempty_property(r: &DataRestriction) -> bool {
    match r {
        DataRestriction::Some(_) | DataRestriction::HasValue(_) => true,
        DataRestriction::MinCardinality(n, _) => *n > 0,
        DataRestriction::ExactCardinality(n, _) => *n > 0,
        _ => false,
    }
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
        let mut distinct_fixed = HashSet::new();
        for value in &fixed_values {
            if let Some(lit) = literal_from_de(ontology, value) {
                distinct_fixed.insert(distinct_literal_key(&lit));
            }
        }
        if (distinct_fixed.len() as u32) > max {
            return false;
        }
        let mut witness_ranges = combined_all.clone();
        witness_ranges.extend(some_ranges.clone());
        if !witness_ranges.is_empty() {
            let count = distinct_values_satisfying_ranges(ontology, idx, &witness_ranges);
            if count > max {
                return false;
            }
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

fn literals_equal_local(a: &LiteralValue, b: &LiteralValue) -> bool {
    literals_equal(a, b)
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
    let mut candidates = sample_literals(ontology, idx, ranges[0]);
    for &range in &ranges[1..] {
        candidates.extend(sample_literals(ontology, idx, range));
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
    let n = parse_numeric(&lit.lexical);
    if n.is_finite() && !n.is_nan() {
        return format!("n:{n}");
    }
    canonical_plain_literal(&lit.lexical)
}

fn max_distinct_values(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> u32 {
    let store = ontology.dl();
    let Some(expr) = store.de(range) else {
        return u32::MAX;
    };
    match expr {
        DataExpr::Literal { .. } => 1,
        DataExpr::Not(inner) => {
            let mut candidates = literal_universe(ontology);
            let mut seen = HashSet::new();
            let mut count = 0_u32;
            for lit in candidates.drain(..) {
                let key = distinct_literal_key(&lit);
                if !seen.insert(key) {
                    continue;
                }
                if !idx.satisfies_with_ontology(&lit, ontology, *inner) {
                    count += 1;
                }
            }
            if count == 0 {
                u32::MAX
            } else {
                count.min(100)
            }
        }
        DataExpr::And(ops) => {
            let Some(first) = ops.first().copied() else {
                return 0;
            };
            let mut candidates = sample_literals(ontology, idx, first);
            for &op in &ops[1..] {
                candidates.extend(sample_literals(ontology, idx, op));
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
                    let key = distinct_literal_key(&LiteralValue {
                        lexical: lexical.clone(),
                        datatype: *datatype,
                    });
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

fn sample_literals(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> Vec<LiteralValue> {
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
        DataExpr::Not(inner) => {
            let mut candidates = literal_universe(ontology);
            candidates.retain(|lit| !idx.satisfies_with_ontology(lit, ontology, *inner));
            candidates
        }
        DataExpr::And(ops) => {
            let mut out = Vec::new();
            for &op in ops {
                if let Some(DataExpr::Datatype(dt)) = store.de(op) {
                    out.extend(default_witness_literals(ontology, *dt));
                } else {
                    out.extend(sample_literals(ontology, idx, op));
                }
            }
            out
        }
        DataExpr::Or(ops) => ops
            .iter()
            .flat_map(|&op| sample_literals(ontology, idx, op))
            .collect(),
        DataExpr::Facet { base, .. } => sample_literals(ontology, idx, *base),
        DataExpr::Top => literal_universe(ontology),
    }
}

fn literal_universe(ontology: &Ontology) -> Vec<LiteralValue> {
    let mut out = Vec::new();
    for dt_iri in [
        "http://www.w3.org/2001/XMLSchema#string",
        "http://www.w3.org/2001/XMLSchema#integer",
        "http://www.w3.org/2001/XMLSchema#decimal",
        "http://www.w3.org/2001/XMLSchema#float",
        "http://www.w3.org/2001/XMLSchema#double",
        "http://www.w3.org/2002/07/owl#rational",
        "http://www.w3.org/2002/07/owl#real",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString",
        "http://www.w3.org/2001/XMLSchema#dateTime",
    ] {
        if let Some(id) = ontology.lookup_entity(dt_iri) {
            out.extend(default_witness_literals(ontology, id));
        }
    }
    out
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
        "http://www.w3.org/2001/XMLSchema#integer" => &["0", "1", "2", "-1"],
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => &["0", "-1"],
        "http://www.w3.org/2001/XMLSchema#int" => &["0", "1", "2"],
        "http://www.w3.org/2001/XMLSchema#short" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#byte" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#unsignedInt" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#string" => &["a", "b", "c", "abc"],
        "http://www.w3.org/2001/XMLSchema#decimal" => &["0", "1", "1.5", "-1"],
        "http://www.w3.org/2001/XMLSchema#float" => &["0", "1", "INF", "-INF", "-0"],
        "http://www.w3.org/2001/XMLSchema#double" => &["0", "1", "INF", "-INF"],
        "http://www.w3.org/2002/07/owl#rational" => &["0", "1/2", "1"],
        "http://www.w3.org/2002/07/owl#real" => &["0", "1", "1.5"],
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString" => &["en"],
        "http://www.w3.org/2001/XMLSchema#dateTime" => {
            &["2000-01-01T00:00:00", "2000-01-01T00:00:00Z", "2000-01-01T00:00:00+05:00"]
        }
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
            | "http://www.w3.org/2002/07/owl#rational"
            | "http://www.w3.org/2002/07/owl#real"
    )
}

fn numeric_compare(a: &str, b: &str) -> i32 {
    let fa = parse_numeric(a);
    let fb = parse_numeric(b);
    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal) as i32
}

fn parse_numeric(s: &str) -> f64 {
    match s {
        "INF" | "+INF" => f64::INFINITY,
        "-INF" => f64::NEG_INFINITY,
        "NaN" => f64::NAN,
        _ => {
            let trimmed = s.strip_prefix('+').unwrap_or(s);
            if trimmed == "-0" {
                0.0
            } else if trimmed.contains('/') {
                let parts: Vec<_> = trimmed.split('/').collect();
                if parts.len() == 2 {
                    let num: f64 = parts[0].parse().unwrap_or(0.0);
                    let den: f64 = parts[1].parse().unwrap_or(1.0);
                    return if den == 0.0 { f64::NAN } else { num / den };
                }
                0.0
            } else {
                trimmed.parse().unwrap_or(0.0)
            }
        }
    }
}
