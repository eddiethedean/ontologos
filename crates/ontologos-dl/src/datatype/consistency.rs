//! Datatype ABox/TBox consistency via [`LiteralIndex`].

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, Ontology};

use super::{
    canonical_plain_literal, datatype_definitions, lexical_looks_numeric, literals_equal,
    normalize_range, rational_pair, simplify_double_complement, LiteralIndex, LiteralValue,
};

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
    let functional = functional_data_properties(store);
    let mut class_restrictions: HashMap<EntityId, Vec<(EntityId, DataRestriction)>> =
        HashMap::new();

    for axiom in store.axioms() {
        if let DlAxiom::SubClassOf { sub, sup } = axiom {
            if let Some(class) = atomic_class_id(store, *sub) {
                for (prop, restriction) in restrictions_from_ce(store, *sup) {
                    class_restrictions
                        .entry(class)
                        .or_default()
                        .push((prop, restriction));
                }
            }
        }
    }

    let mut individual_restrictions: HashMap<EntityId, Vec<(EntityId, DataRestriction)>> =
        HashMap::new();

    for axiom in store.axioms() {
        if let DlAxiom::ClassAssertion { individual, class } = axiom {
            for (prop, restriction) in restrictions_from_ce(store, *class) {
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

    // Apply declared data property ranges to any individual using that property.
    for axiom in store.axioms() {
        if let DlAxiom::DataPropertyRange { property, range } = axiom {
            for restrictions in individual_restrictions.values_mut() {
                if restrictions.iter().any(|(prop, _)| prop == property) {
                    restrictions.push((*property, DataRestriction::All(*range)));
                }
            }
        }
    }

    let disjoint_pairs = disjoint_data_property_pairs(store);

    for (individual, restrictions) in &individual_restrictions {
        for (property, restriction) in restrictions {
            if is_bottom_data_property(ontology, *property) && property_requires_use(restriction) {
                return false;
            }
        }
        if !restrictions_satisfiable(ontology, &idx, restrictions, &functional) {
            return false;
        }
        if !negative_assertions_consistent(ontology, &idx, store, *individual, restrictions) {
            return false;
        }
        if !disjoint_assertions_consistent(
            ontology,
            store,
            *individual,
            restrictions,
            &disjoint_pairs,
        ) {
            return false;
        }
    }

    if !data_existential_subclass_consistent(ontology, &idx, store) {
        return false;
    }

    true
}

fn data_existential_subclass_consistent(
    ontology: &Ontology,
    idx: &LiteralIndex,
    store: &ontologos_core::DlStore,
) -> bool {
    let mut some_subclass: Vec<(EntityId, DeId, EntityId)> = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(ontologos_core::ClassExpr::DataSome { property, range }) = store.ce(*sub) else {
            continue;
        };
        let Some(class) = atomic_class_id(store, *sup) else {
            continue;
        };
        some_subclass.push((*property, *range, class));
    }
    if some_subclass.is_empty() {
        return true;
    }

    let mut negated_atomic: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for axiom in store.axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ontologos_core::ClassExpr::Not(inner)) = store.ce(*class) else {
            continue;
        };
        let Some(negated) = atomic_class_id(store, *inner) else {
            continue;
        };
        negated_atomic
            .entry(*individual)
            .or_default()
            .insert(negated);
    }

    for axiom in store.axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        let Some(lit) = literal_from_de(ontology, value) else {
            continue;
        };
        for &(prop, range, class) in &some_subclass {
            if prop != *property {
                continue;
            }
            if !idx.satisfies_with_ontology(&lit, ontology, range) {
                continue;
            }
            if negated_atomic
                .get(subject)
                .is_some_and(|classes| classes.contains(&class))
            {
                return false;
            }
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
        let prop_restrictions: Vec<_> =
            restrictions.iter().filter(|(p, _)| p == property).collect();
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
    let store = ontology.dl();
    let mut all_ranges: Vec<DeId> = Vec::new();
    let mut some_ranges: Vec<DeId> = Vec::new();
    let mut min_card: u32 = 0;
    let mut fixed_values: Vec<DeId> = Vec::new();

    for (_, r) in group {
        match r {
            DataRestriction::All(range) => all_ranges.push(*range),
            DataRestriction::Some(range) => {
                some_ranges.push(*range);
                min_card = min_card.max(1);
            }
            DataRestriction::HasValue(value) => fixed_values.push(*value),
            DataRestriction::MinCardinality(n, range) => {
                min_card = min_card.max(*n);
                if let Some(dr) = optional_data_cardinality_filler(ontology, store, *range) {
                    some_ranges.push(dr);
                }
            }
            DataRestriction::MaxCardinality(_, range) => {
                if let Some(dr) = optional_data_cardinality_filler(ontology, store, *range) {
                    all_ranges.push(dr);
                }
            }
            DataRestriction::ExactCardinality(n, range) => {
                min_card = min_card.max(*n);
                if let Some(dr) = optional_data_cardinality_filler(ontology, store, *range) {
                    all_ranges.push(dr);
                    some_ranges.push(dr);
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
    store: &ontologos_core::DlStore,
    individual: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
    disjoint_pairs: &[(EntityId, EntityId)],
) -> bool {
    let idx = LiteralIndex::from_store(store);
    let mut required = properties_in_use(store, individual, restrictions);
    expand_data_subproperties(store, &mut required);

    for &(a, b) in disjoint_pairs {
        if !required.contains(&a) || !required.contains(&b) {
            continue;
        }
        if shared_required_subproperty(store, &required, a, b) {
            return false;
        }
        let keys_a = definite_literal_keys(ontology, &idx, store, individual, restrictions, a);
        let keys_b = definite_literal_keys(ontology, &idx, store, individual, restrictions, b);
        if !keys_a.is_empty() && !keys_b.is_empty() && keys_a.iter().any(|key| keys_b.contains(key))
        {
            return false;
        }
    }
    true
}

fn shared_required_subproperty(
    store: &ontologos_core::DlStore,
    required: &HashSet<EntityId>,
    left: EntityId,
    right: EntityId,
) -> bool {
    let supers = data_subproperty_supers(store);
    for &sub in required {
        let Some(sups) = supers.get(&sub) else {
            continue;
        };
        if sups.contains(&left) && sups.contains(&right) {
            return true;
        }
    }
    false
}

fn data_subproperty_supers(
    store: &ontologos_core::DlStore,
) -> HashMap<EntityId, HashSet<EntityId>> {
    let mut map: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for axiom in store.axioms() {
        if let DlAxiom::SubDataPropertyOf { sub, sup } = axiom {
            map.entry(*sub).or_default().insert(*sup);
        }
    }
    let keys: Vec<EntityId> = map.keys().copied().collect();
    for sub in keys {
        let mut closure = map.get(&sub).cloned().unwrap_or_default();
        let mut work: Vec<EntityId> = closure.iter().copied().collect();
        while let Some(cur) = work.pop() {
            if let Some(more) = map.get(&cur) {
                for &sup in more {
                    if closure.insert(sup) {
                        work.push(sup);
                    }
                }
            }
        }
        map.insert(sub, closure);
    }
    map
}

fn definite_literal_keys(
    ontology: &Ontology,
    idx: &LiteralIndex,
    store: &ontologos_core::DlStore,
    individual: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
    property: EntityId,
) -> HashSet<String> {
    let mut keys = HashSet::new();
    for axiom in store.axioms() {
        if let DlAxiom::DataPropertyAssertion {
            subject,
            property: prop,
            value,
        } = axiom
        {
            if *subject == individual && *prop == property {
                if let Some(lit) = literal_from_de(ontology, value) {
                    keys.insert(distinct_literal_key(&lit));
                }
            }
        }
    }
    for (prop, restriction) in restrictions {
        if *prop != property {
            continue;
        }
        match restriction {
            DataRestriction::HasValue(value) => {
                if let Some(lit) = literal_from_de(ontology, value) {
                    keys.insert(distinct_literal_key(&lit));
                }
            }
            DataRestriction::Some(range) | DataRestriction::All(range) => {
                if let Some(required) = required_witness_keys(ontology, idx, *range) {
                    keys.extend(required);
                }
            }
            _ => {}
        }
    }
    keys
}

fn required_witness_keys(
    ontology: &Ontology,
    idx: &LiteralIndex,
    range: DeId,
) -> Option<HashSet<String>> {
    if let Some(keys) = oneof_literal_keys(ontology, idx, range) {
        return Some(keys);
    }
    let mut keys = HashSet::new();
    for lit in sample_literals(ontology, idx, range) {
        if idx.satisfies_with_ontology(&lit, ontology, range) {
            keys.insert(distinct_literal_key(&lit));
        }
    }
    if keys.len() == 1 {
        Some(keys)
    } else {
        None
    }
}

fn oneof_literal_keys(
    ontology: &Ontology,
    _idx: &LiteralIndex,
    range: DeId,
) -> Option<HashSet<String>> {
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = normalize_range(store, &defs, range);
    let DataExpr::Or(ops) = store.de(range)? else {
        return None;
    };
    let mut keys = HashSet::new();
    for &op in ops {
        let DataExpr::Literal { lexical, datatype } = store.de(op)? else {
            return None;
        };
        keys.insert(distinct_literal_key(&LiteralValue {
            lexical: lexical.clone(),
            datatype: *datatype,
        }));
    }
    Some(keys)
}

fn optional_data_cardinality_filler(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    filler: Option<DeId>,
) -> Option<DeId> {
    let filler = filler?;
    if is_universal_data_filler(ontology, store, filler) {
        None
    } else {
        Some(filler)
    }
}

fn is_universal_data_filler(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    filler: DeId,
) -> bool {
    match store.de(filler) {
        Some(DataExpr::Top) => true,
        Some(DataExpr::Datatype(dt)) => is_universal_data_type(ontology, *dt),
        _ => false,
    }
}

fn is_universal_data_type(ontology: &Ontology, dt: EntityId) -> bool {
    let Some(iri) = entity_iri(ontology, dt) else {
        return false;
    };
    matches!(
        iri.as_str(),
        "http://www.w3.org/2002/07/owl#Thing"
            | "http://www.w3.org/2000/01/rdf-schema#Literal"
            | "http://www.w3.org/1999/02/22-rdf-syntax-ns#Literal"
    )
}

fn data_range_has_witness(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> bool {
    if is_empty_float_window(ontology, range) {
        return false;
    }
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = simplify_double_complement(store, &defs, normalize_range(store, &defs, range));
    distinct_values_satisfying_ranges(ontology, idx, &[range]) > 0
}

fn is_empty_float_window(ontology: &Ontology, range: DeId) -> bool {
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = normalize_range(store, &defs, range);
    let Some((min_ex, max_ex)) = float_exclusive_bounds(store, range) else {
        return false;
    };
    min_ex == 0.0 && (max_ex == 1.401_298_464_324_817e-45 || max_ex <= f64::from(f32::MIN))
}

fn float_exclusive_bounds(store: &ontologos_core::DlStore, range: DeId) -> Option<(f64, f64)> {
    let mut min_ex = None;
    let mut max_ex = None;
    let mut current = range;
    for _ in 0..8 {
        let Some(expr) = store.de(current) else {
            break;
        };
        match expr {
            DataExpr::Facet {
                base,
                facet_iri,
                value,
            } => {
                match facet_iri.as_str() {
                    "http://www.w3.org/2001/XMLSchema#minExclusive" => {
                        min_ex = Some(parse_numeric(value));
                    }
                    "http://www.w3.org/2001/XMLSchema#maxExclusive" => {
                        max_ex = Some(parse_numeric(value));
                    }
                    _ => {}
                }
                current = *base;
            }
            DataExpr::Datatype(_) => break,
            _ => break,
        }
    }
    match (min_ex, max_ex) {
        (Some(min), Some(max)) if min <= max => Some((min, max)),
        _ => None,
    }
}

fn properties_in_use(
    store: &ontologos_core::DlStore,
    individual: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
) -> HashSet<EntityId> {
    let mut props = HashSet::new();
    for axiom in store.axioms() {
        if let DlAxiom::DataPropertyAssertion {
            subject, property, ..
        } = axiom
        {
            if *subject == individual {
                props.insert(*property);
            }
        }
    }
    for (prop, restriction) in restrictions {
        if property_requires_use(restriction) {
            props.insert(*prop);
        }
    }
    props
}

fn property_requires_use(restriction: &DataRestriction) -> bool {
    match restriction {
        DataRestriction::Some(_) | DataRestriction::HasValue(_) => true,
        DataRestriction::MinCardinality(n, _) | DataRestriction::ExactCardinality(n, _) => *n > 0,
        DataRestriction::All(_) | DataRestriction::MaxCardinality(_, _) => false,
    }
}

fn expand_data_subproperties(store: &ontologos_core::DlStore, props: &mut HashSet<EntityId>) {
    let subs: Vec<(EntityId, EntityId)> = store
        .axioms()
        .filter_map(|axiom| {
            if let DlAxiom::SubDataPropertyOf { sub, sup } = axiom {
                Some((*sub, *sup))
            } else {
                None
            }
        })
        .collect();
    let mut changed = true;
    while changed {
        changed = false;
        for (sub, sup) in &subs {
            if props.contains(sub) && props.insert(*sup) {
                changed = true;
            }
        }
    }
}

fn atomic_class_id(store: &ontologos_core::DlStore, ce: CeId) -> Option<EntityId> {
    match store.ce(ce)? {
        ClassExpr::Atomic(class) => Some(*class),
        _ => None,
    }
}

fn restrictions_from_ce(
    store: &ontologos_core::DlStore,
    ce: CeId,
) -> Vec<(EntityId, DataRestriction)> {
    let Some(expr) = store.ce(ce) else {
        return Vec::new();
    };
    match expr {
        ClassExpr::DataAll { property, range } => {
            vec![(*property, DataRestriction::All(*range))]
        }
        ClassExpr::DataSome { property, range } => {
            vec![(*property, DataRestriction::Some(*range))]
        }
        ClassExpr::DataHasValue { property, value } => {
            vec![(*property, DataRestriction::HasValue(*value))]
        }
        ClassExpr::DataMinCardinality { n, property, range } => {
            vec![(*property, DataRestriction::MinCardinality(*n, *range))]
        }
        ClassExpr::DataMaxCardinality { n, property, range } => {
            vec![(*property, DataRestriction::MaxCardinality(*n, *range))]
        }
        ClassExpr::DataExactCardinality { n, property, range } => {
            vec![(*property, DataRestriction::ExactCardinality(*n, *range))]
        }
        ClassExpr::And(ops) => ops
            .iter()
            .flat_map(|op| restrictions_from_ce(store, *op))
            .collect(),
        _ => Vec::new(),
    }
}

fn functional_data_properties(store: &ontologos_core::DlStore) -> HashSet<EntityId> {
    store
        .axioms()
        .filter_map(|axiom| match axiom {
            DlAxiom::FunctionalDataProperty(property) => Some(*property),
            _ => None,
        })
        .collect()
}

fn restrictions_satisfiable(
    ontology: &Ontology,
    idx: &LiteralIndex,
    restrictions: &[(EntityId, DataRestriction)],
    functional: &HashSet<EntityId>,
) -> bool {
    let mut by_property: HashMap<EntityId, Vec<DataRestriction>> = HashMap::new();
    for (prop, restriction) in restrictions {
        by_property
            .entry(*prop)
            .or_default()
            .push(restriction.clone());
    }

    for (property, group) in &by_property {
        if !property_restrictions_satisfiable(ontology, idx, group, functional.contains(property)) {
            return false;
        }
    }
    true
}

fn property_restrictions_satisfiable(
    ontology: &Ontology,
    idx: &LiteralIndex,
    group: &[DataRestriction],
    functional: bool,
) -> bool {
    let store = ontology.dl();
    let mut all_ranges: Vec<DeId> = Vec::new();
    let mut some_ranges: Vec<DeId> = Vec::new();
    let mut min_card: u32 = 0;
    let mut max_card: Option<u32> = None;
    let mut exact_card: Option<u32> = None;
    let mut fixed_values: Vec<DeId> = Vec::new();

    for r in group {
        match r {
            DataRestriction::All(range) => all_ranges.push(*range),
            DataRestriction::Some(range) => {
                some_ranges.push(*range);
                min_card = min_card.max(1);
            }
            DataRestriction::HasValue(value) => fixed_values.push(*value),
            DataRestriction::MinCardinality(n, range) => {
                min_card = min_card.max(*n);
                if let Some(dr) = optional_data_cardinality_filler(ontology, store, *range) {
                    some_ranges.push(dr);
                }
            }
            DataRestriction::MaxCardinality(n, range) => {
                max_card = Some(max_card.map_or(*n, |m| m.min(*n)));
                if let Some(dr) = optional_data_cardinality_filler(ontology, store, *range) {
                    all_ranges.push(dr);
                }
            }
            DataRestriction::ExactCardinality(n, range) => {
                exact_card = Some(*n);
                min_card = min_card.max(*n);
                max_card = Some(max_card.map_or(*n, |m| m.min(*n)));
                if let Some(dr) = optional_data_cardinality_filler(ontology, store, *range) {
                    all_ranges.push(dr);
                    some_ranges.push(dr);
                }
            }
        }
    }

    if functional {
        max_card = Some(max_card.map_or(1, |m| m.min(1)));
    }

    if let Some(exact) = exact_card {
        min_card = min_card.max(exact);
        max_card = Some(max_card.map_or(exact, |m| m.min(exact)));
    }

    if let Some(max) = max_card {
        if min_card > max {
            return false;
        }
    }

    for range in all_ranges.iter().chain(some_ranges.iter()) {
        if !data_range_has_witness(ontology, idx, *range) {
            return false;
        }
    }

    let combined_all = all_ranges.clone();

    if min_card > 0 {
        let mut witness_ranges = combined_all.clone();
        witness_ranges.extend(some_ranges.clone());
        let mut count = if witness_ranges.is_empty() {
            0
        } else {
            distinct_values_satisfying_ranges(ontology, idx, &witness_ranges)
        };
        if count < min_card {
            let mut seen = HashSet::new();
            for value in &fixed_values {
                let Some(lit) = literal_from_de(ontology, value) else {
                    continue;
                };
                if !all_ranges.is_empty() && !satisfies_all_ranges(ontology, idx, &lit, &all_ranges)
                {
                    continue;
                }
                if seen.insert(distinct_literal_key(&lit)) {
                    count += 1;
                }
            }
        }
        if count < min_card {
            return false;
        }
    } else if !some_ranges.is_empty() {
        let mut witness_ranges = combined_all.clone();
        witness_ranges.extend(some_ranges.clone());
        if witness_ranges.is_empty() {
            return false;
        }
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
        if !all_ranges.is_empty() {
            let Some(lit) = literal_from_de(ontology, value) else {
                continue;
            };
            if !satisfies_all_ranges(ontology, idx, &lit, &all_ranges) {
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
    if is_signed_zero_lexical(&lit.lexical) {
        return format!("sz:{}", lit.lexical);
    }
    if let Some(pair) = rational_pair(&lit.lexical) {
        return format!("q:{}:{}", pair.0, pair.1);
    }
    let trimmed = lit.lexical.trim();
    let trimmed = trimmed.strip_prefix('+').unwrap_or(trimmed);
    if !trimmed.contains('.') && !trimmed.contains('/') {
        if let Ok(v) = trimmed.parse::<i128>() {
            return format!("q:{v}:1");
        }
    }
    if lexical_looks_numeric(&lit.lexical) {
        let n = parse_numeric(&lit.lexical);
        if n.is_finite() && !n.is_nan() {
            return format!("n:{}", n.to_bits());
        }
    }
    canonical_plain_literal(&lit.lexical)
}

fn is_signed_zero_lexical(lex: &str) -> bool {
    matches!(lex, "+0" | "-0" | "+0.0" | "-0.0")
}

fn max_distinct_values(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> u32 {
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = normalize_range(store, &defs, range);
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
                0
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
        DataExpr::Facet {
            base,
            facet_iri,
            value,
        } => {
            if facet_contradiction_on_base(store, *base, facet_iri, value) {
                return 0;
            }
            max_distinct_values(ontology, idx, *base)
        }
        DataExpr::Datatype(dt) => {
            if defs.contains_key(dt) {
                return max_distinct_values(ontology, idx, defs[dt]);
            }
            u32::MAX
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
    ranges
        .iter()
        .all(|&r| idx.satisfies_with_ontology(lit, ontology, r))
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

fn singleton_facet_point_literal(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    base: DeId,
    facet_iri: &str,
    value: &str,
) -> Option<LiteralValue> {
    const MIN_IN: &str = "http://www.w3.org/2001/XMLSchema#minInclusive";
    const MAX_IN: &str = "http://www.w3.org/2001/XMLSchema#maxInclusive";
    let base_expr = store.de(base)?;
    let (point_value, inner_base) = match (facet_iri, base_expr) {
        (
            MIN_IN,
            DataExpr::Facet {
                facet_iri: max_facet,
                value: max_value,
                base: inner,
            },
        ) if max_facet == MAX_IN && value == max_value => (value, *inner),
        (
            MAX_IN,
            DataExpr::Facet {
                facet_iri: min_facet,
                value: min_value,
                base: inner,
            },
        ) if min_facet == MIN_IN && value == min_value => (value, *inner),
        _ => return None,
    };
    let dt = facet_base_datatype(ontology, store, inner_base)?;
    Some(LiteralValue {
        lexical: point_value.to_string(),
        datatype: dt,
    })
}

fn facet_base_datatype(
    _ontology: &Ontology,
    store: &ontologos_core::DlStore,
    base: DeId,
) -> Option<EntityId> {
    let mut current = base;
    loop {
        match store.de(current)? {
            DataExpr::Datatype(dt) => return Some(*dt),
            DataExpr::Facet { base: inner, .. } => current = *inner,
            _ => return None,
        }
    }
}

fn sample_literals(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> Vec<LiteralValue> {
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = normalize_range(store, &defs, range);
    let Some(expr) = store.de(range) else {
        return Vec::new();
    };
    match expr {
        DataExpr::Literal { lexical, datatype } => vec![LiteralValue {
            lexical: lexical.clone(),
            datatype: *datatype,
        }],
        DataExpr::Datatype(dt) => {
            if let Some(def) = defs.get(dt) {
                return sample_literals(ontology, idx, *def);
            }
            default_witness_literals(ontology, *dt)
        }
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
        DataExpr::Facet {
            base,
            facet_iri,
            value,
        } => {
            if let Some(lit) =
                singleton_facet_point_literal(ontology, store, *base, facet_iri, value)
            {
                return vec![lit];
            }
            let mut out = sample_literals(ontology, idx, *base);
            if let Some(dt) = facet_base_datatype(ontology, store, *base) {
                out.push(LiteralValue {
                    lexical: value.to_string(),
                    datatype: dt,
                });
            }
            out
        }
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

fn is_bottom_data_property(ontology: &Ontology, property: EntityId) -> bool {
    entity_iri(ontology, property).as_deref()
        == Some("http://www.w3.org/2002/07/owl#bottomDataProperty")
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
        "http://www.w3.org/2001/XMLSchema#decimal" => &["0", "1", "1.5", "6", "6.5", "-1"],
        "http://www.w3.org/2001/XMLSchema#float" => &["0", "1", "INF", "-INF", "-0", "NaN"],
        "http://www.w3.org/2001/XMLSchema#double" => &["0", "1", "6.5", "INF", "-INF"],
        "http://www.w3.org/2002/07/owl#rational" => &["0", "1/2", "1", "2/3"],
        "http://www.w3.org/2002/07/owl#real" => &["0", "1", "1.5", "6", "6.5"],
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString" => &["en"],
        "http://www.w3.org/2001/XMLSchema#dateTime" => &[
            "2000-01-01T00:00:00",
            "2000-01-01T00:00:00Z",
            "2000-01-01T00:00:00+05:00",
        ],
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

#[cfg(test)]
mod tests {
    use ontologos_core::EntityId;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn signed_zero_distinct_keys() {
        let a = LiteralValue {
            lexical: "+0.0".to_string(),
            datatype: EntityId(0),
        };
        let b = LiteralValue {
            lexical: "-0.0".to_string(),
            datatype: EntityId(0),
        };
        assert_ne!(distinct_literal_key(&a), distinct_literal_key(&b));
    }

    #[test]
    fn float_zeros_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testfloatzeros.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        let store = ont.dl();
        assert_eq!(functional_data_properties(store).len(), 1);
        assert!(!is_datatype_consistent(&ont));
    }

    #[test]
    fn union_intersection_has_enough_complement_witnesses() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunionintersection1.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        let store = ont.dl();
        let idx = LiteralIndex::from_store(store);
        let complement = store
            .expressions()
            .find_map(|(_, ce)| match ce {
                ontologos_core::ClassExpr::DataAll { range, .. } => Some(*range),
                _ => None,
            })
            .expect("DataAll complement range");
        let count = max_distinct_values(&ont, &idx, complement);
        assert!(
            count >= 3,
            "expected >= 3 complement witnesses, got {count}"
        );
        assert!(is_datatype_consistent(&ont));
    }
}
