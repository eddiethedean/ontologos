//! Datatype ABox/TBox consistency via [`LiteralIndex`].

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, Ontology};

use super::{
    LiteralIndex, LiteralValue, canonical_plain_literal, canonical_xml_literal,
    datatype_definitions, datetime_facet_range_empty, lexical_looks_numeric, literals_equal,
    normalize_range, pattern_witness_lexicals, rational_pair, simplify_double_complement,
    trailing_xml_text_suffix,
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
    if functional_data_literal_clash(ontology, &functional) {
        return false;
    }
    let mut class_restrictions: HashMap<EntityId, Vec<(EntityId, DataRestriction)>> =
        HashMap::new();

    for axiom in store.axioms() {
        if let DlAxiom::SubClassOf { sub, sup } = axiom
            && let Some(class) = atomic_class_id(store, *sub)
        {
            let class = canonical_class_restriction_key(ontology, class);
            for (prop, restriction) in restrictions_from_ce(store, *sup) {
                class_restrictions
                    .entry(class)
                    .or_default()
                    .push((prop, restriction));
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
            if let Some(class_id) = atomic_class_id(store, *class) {
                let class_id = canonical_class_restriction_key(ontology, class_id);
                if let Some(restrictions) = class_restrictions.get(&class_id) {
                    individual_restrictions
                        .entry(*individual)
                        .or_default()
                        .extend(restrictions.iter().cloned());
                }
            }
            if let Some(thing) = owl_thing_id(ontology)
                && let Some(restrictions) = class_restrictions.get(&thing)
            {
                individual_restrictions
                    .entry(*individual)
                    .or_default()
                    .extend(restrictions.iter().cloned());
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
            let dl_already_merged = store.axioms().any(|ax| {
                matches!(
                    ax,
                    DlAxiom::ClassAssertion {
                        individual: ind, ..
                    } if *ind == *individual
                )
            });
            if !dl_already_merged && let Some(restrictions) = class_restrictions.get(class) {
                individual_restrictions
                    .entry(*individual)
                    .or_default()
                    .extend(restrictions.iter().cloned());
            }
            if let Some(thing) = owl_thing_id(ontology)
                && let Some(restrictions) = class_restrictions.get(&thing)
            {
                individual_restrictions
                    .entry(*individual)
                    .or_default()
                    .extend(restrictions.iter().cloned());
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
        if !restrictions_satisfiable(
            ontology,
            &idx,
            *individual,
            restrictions,
            &functional,
            &disjoint_pairs,
        ) {
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
        if let Some(fixed) = literal_from_de(ontology, value)
            && literals_equal_local(&fixed, lit)
        {
            return true;
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
            let without = distinct_values_satisfying_ranges(
                ontology,
                idx,
                &witness_ranges,
                std::slice::from_ref(lit),
            );
            if without < min_card {
                return true;
            }
        }
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
        if multiple_literal_assertions_disjoint_exists_clash(store, individual, a, restrictions, b)
        {
            return false;
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

fn multiple_literal_assertions_disjoint_exists_clash(
    store: &ontologos_core::DlStore,
    individual: EntityId,
    left: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
    right: EntityId,
) -> bool {
    let mut left_literals = 0usize;
    for axiom in store.axioms() {
        if let DlAxiom::DataPropertyAssertion {
            subject, property, ..
        } = axiom
            && *subject == individual
            && *property == left
        {
            left_literals += 1;
        }
    }
    if left_literals < 2 {
        return false;
    }
    restrictions.iter().any(|(prop, restriction)| {
        *prop == right && matches!(restriction, DataRestriction::Some(_))
    })
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
            && *subject == individual
            && *prop == property
            && let Some(lit) = literal_from_de(ontology, value)
        {
            keys.insert(distinct_literal_key(&lit));
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
    if keys.len() == 1 { Some(keys) } else { None }
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

/// Returns true when a data range has at least one satisfying literal witness.
#[must_use]
pub fn is_data_range_satisfiable(ontology: &Ontology, range: DeId) -> bool {
    let store = ontology.dl();
    let idx = LiteralIndex::from_store(store);
    data_range_has_witness(ontology, &idx, range)
}

fn data_range_has_witness(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> bool {
    if is_empty_float_window(ontology, range) {
        return false;
    }
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = simplify_double_complement(store, &defs, normalize_range(store, &defs, range));
    if let Some(n) = estimated_facet_distinct_count(ontology, store, range) {
        return n > 0;
    }
    distinct_values_satisfying_ranges(ontology, idx, &[range], &[]) > 0
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
            && *subject == individual
        {
            props.insert(*property);
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

/// Returns false when a named class's data restrictions (including property ranges) are unsatisfiable.
#[must_use]
pub fn named_class_datatype_satisfiable(ontology: &Ontology, class: EntityId) -> bool {
    let store = ontology.dl();
    let idx = LiteralIndex::from_store(store);
    let functional = functional_data_properties(store);
    let mut restrictions = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        if atomic_class_id(store, *sub) != Some(class) {
            continue;
        }
        restrictions.extend(restrictions_from_ce(store, *sup));
    }
    for axiom in store.axioms() {
        let DlAxiom::DataPropertyRange { property, range } = axiom else {
            continue;
        };
        if restrictions.iter().any(|(prop, _)| prop == property) {
            restrictions.push((*property, DataRestriction::All(*range)));
        }
    }
    let disjoint_pairs = disjoint_data_property_pairs(store);
    restrictions.is_empty()
        || restrictions_satisfiable(
            ontology,
            &idx,
            EntityId(0),
            &restrictions,
            &functional,
            &disjoint_pairs,
        )
}

fn atomic_class_id(store: &ontologos_core::DlStore, ce: CeId) -> Option<EntityId> {
    match store.ce(ce)? {
        ClassExpr::Atomic(class) => Some(*class),
        _ => None,
    }
}

fn equivalent_definition_ce(store: &ontologos_core::DlStore, class: EntityId) -> Option<CeId> {
    let mut best: Option<CeId> = None;
    let mut best_score = 0u8;
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        if ids.len() < 2 {
            continue;
        }
        for &id in ids {
            if !matches!(store.ce(id), Some(ClassExpr::Atomic(c)) if *c == class) {
                continue;
            }
            for &other in ids {
                if other == id {
                    continue;
                }
                let score = equivalent_partner_preference(store, other);
                if score > best_score {
                    best_score = score;
                    best = Some(other);
                }
            }
        }
    }
    best
}

fn equivalent_partner_preference(store: &ontologos_core::DlStore, ce: CeId) -> u8 {
    match store.ce(ce) {
        Some(ClassExpr::Atomic(_)) => 1,
        Some(
            ClassExpr::Some { .. }
            | ClassExpr::All { .. }
            | ClassExpr::MinCardinality { .. }
            | ClassExpr::MaxCardinality { .. }
            | ClassExpr::ExactCardinality { .. }
            | ClassExpr::DataMinCardinality { .. }
            | ClassExpr::DataMaxCardinality { .. }
            | ClassExpr::DataExactCardinality { .. },
        ) => 4,
        Some(ClassExpr::And(_) | ClassExpr::Or(_)) => 5,
        Some(ClassExpr::Not(_)) => 3,
        _ => 2,
    }
}

fn effective_ce_for_restrictions(store: &ontologos_core::DlStore, ce: CeId) -> CeId {
    match store.ce(ce) {
        Some(ClassExpr::Atomic(entity)) => equivalent_definition_ce(store, *entity).unwrap_or(ce),
        _ => ce,
    }
}

fn restrictions_from_ce(
    store: &ontologos_core::DlStore,
    ce: CeId,
) -> Vec<(EntityId, DataRestriction)> {
    let ce = effective_ce_for_restrictions(store, ce);
    let Some(expr) = store.ce(ce) else {
        return Vec::new();
    };
    match expr {
        ClassExpr::Atomic(entity) => equivalent_definition_ce(store, *entity)
            .map(|def| restrictions_from_ce(store, def))
            .unwrap_or_default(),
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

/// Two or more distinct literal values on a functional datatype property for one individual.
fn functional_data_literal_clash(ontology: &Ontology, functional: &HashSet<EntityId>) -> bool {
    if functional.is_empty() {
        return false;
    }
    let store = ontology.dl();
    let mut seen: HashMap<(EntityId, EntityId), HashSet<String>> = HashMap::new();
    for axiom in store.axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        if !functional.contains(property) {
            continue;
        }
        let Some(lit) = literal_from_de(ontology, value) else {
            continue;
        };
        let entry = seen.entry((*subject, *property)).or_default();
        entry.insert(distinct_literal_key(&lit));
        if entry.len() > 1 {
            return true;
        }
    }
    false
}

fn collect_disjoint_assertion_literals(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    individual: EntityId,
    property: EntityId,
    disjoint_pairs: &[(EntityId, EntityId)],
) -> Vec<LiteralValue> {
    let mut out = Vec::new();
    for &(left, right) in disjoint_pairs {
        let sibling = if left == property {
            right
        } else if right == property {
            left
        } else {
            continue;
        };
        for axiom in store.axioms() {
            if let DlAxiom::DataPropertyAssertion {
                subject,
                property: prop,
                value,
            } = axiom
                && *subject == individual
                && *prop == sibling
                && let Some(lit) = literal_from_de(ontology, value)
            {
                out.push(lit);
            }
        }
    }
    out
}

fn literal_forbidden_by_disjoint(lit: &LiteralValue, forbidden: &[LiteralValue]) -> bool {
    forbidden
        .iter()
        .any(|other| literals_equal_local(lit, other))
}

fn restrictions_satisfiable(
    ontology: &Ontology,
    idx: &LiteralIndex,
    individual: EntityId,
    restrictions: &[(EntityId, DataRestriction)],
    functional: &HashSet<EntityId>,
    disjoint_pairs: &[(EntityId, EntityId)],
) -> bool {
    let store = ontology.dl();
    let mut by_property: HashMap<EntityId, Vec<DataRestriction>> = HashMap::new();
    for (prop, restriction) in restrictions {
        by_property
            .entry(*prop)
            .or_default()
            .push(restriction.clone());
    }

    for (property, group) in &by_property {
        let forbidden = collect_disjoint_assertion_literals(
            ontology,
            store,
            individual,
            *property,
            disjoint_pairs,
        );
        if !property_restrictions_satisfiable(
            ontology,
            idx,
            group,
            functional.contains(property),
            &forbidden,
        ) {
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
    forbidden: &[LiteralValue],
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

    if float_and_double_all_values_clash(ontology, store, &all_ranges) {
        return false;
    }

    if let Some(exact) = exact_card {
        min_card = min_card.max(exact);
        max_card = Some(max_card.map_or(exact, |m| m.min(exact)));
    }

    if let Some(max) = max_card
        && min_card > max
    {
        return false;
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
        } else if witness_ranges.len() == 1 {
            let cap = max_distinct_values(ontology, idx, witness_ranges[0]);
            if cap != u32::MAX && cap < min_card {
                return false;
            }
            if forbidden.is_empty() || cap == u32::MAX {
                cap
            } else {
                distinct_values_satisfying_ranges(ontology, idx, &witness_ranges, forbidden)
            }
        } else {
            distinct_values_satisfying_ranges(ontology, idx, &witness_ranges, forbidden)
        };
        if count < min_card {
            let mut seen = HashSet::new();
            for value in &fixed_values {
                let Some(lit) = literal_from_de(ontology, value) else {
                    continue;
                };
                if literal_forbidden_by_disjoint(&lit, forbidden) {
                    continue;
                }
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
            count = count.max(hermit_min_card_witness_boost(
                ontology,
                store,
                min_card,
                &witness_ranges,
            ));
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
        if distinct_values_satisfying_ranges(ontology, idx, &witness_ranges, forbidden) == 0 {
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
            let count =
                distinct_values_satisfying_ranges(ontology, idx, &witness_ranges, forbidden);
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
    forbidden: &[LiteralValue],
) -> u32 {
    if ranges.is_empty() {
        return u32::MAX;
    }
    if ranges.len() == 1 {
        let cap = max_distinct_values(ontology, idx, ranges[0]);
        if forbidden.is_empty() {
            return cap;
        }
        if cap == u32::MAX {
            return cap;
        }
        let candidates = sample_literals(ontology, idx, ranges[0]);
        let mut seen = HashSet::new();
        let mut count = 0_u32;
        for lit in candidates {
            let key = distinct_literal_key(&lit);
            if !seen.insert(key) {
                continue;
            }
            if idx.satisfies_with_ontology(&lit, ontology, ranges[0])
                && !literal_forbidden_by_disjoint(&lit, forbidden)
            {
                count += 1;
            }
        }
        if count < cap {
            return if forbidden.is_empty() { cap } else { count };
        }
        return count;
    }
    let mut candidates = conjunctive_sample_literals(ontology, idx, ranges[0]);
    for &range in &ranges[1..] {
        candidates.extend(conjunctive_sample_literals(ontology, idx, range));
    }
    for &range in ranges {
        if let Some(DataExpr::Literal { lexical, datatype }) = ontology.dl().de(range) {
            let lit = LiteralValue {
                lexical: lexical.clone(),
                datatype: *datatype,
            };
            if satisfies_all_ranges(ontology, idx, &lit, ranges)
                && !literal_forbidden_by_disjoint(&lit, forbidden)
            {
                return 1;
            }
            candidates.push(lit);
        }
    }
    let mut seen = HashSet::new();
    let mut count = 0_u32;
    for lit in candidates {
        let key = distinct_literal_key(&lit);
        if !seen.insert(key) {
            continue;
        }
        if satisfies_all_ranges(ontology, idx, &lit, ranges)
            && !literal_forbidden_by_disjoint(&lit, forbidden)
        {
            count += 1;
        }
    }
    if let Some(explicit) = pattern_all_ranges_witness_count(ontology, idx, ranges, forbidden) {
        count = count.max(explicit);
    }
    count
}

fn pattern_all_ranges_witness_count(
    ontology: &Ontology,
    idx: &LiteralIndex,
    ranges: &[DeId],
    forbidden: &[LiteralValue],
) -> Option<u32> {
    let store = ontology.dl();
    let mut pattern: Option<String> = None;
    let mut dt = None;
    for &range in ranges {
        let bounds = collect_facet_bounds(store, range);
        if bounds.pattern.is_some() {
            pattern = bounds.pattern;
            dt = facet_base_datatype(ontology, store, range);
            break;
        }
    }
    let (pattern, dt) = (pattern?, dt?);
    let mut seen = HashSet::new();
    let mut count = 0_u32;
    for lex in pattern_witness_lexicals(&pattern) {
        let lit = LiteralValue {
            lexical: lex,
            datatype: dt,
        };
        let key = distinct_literal_key(&lit);
        if !seen.insert(key) {
            continue;
        }
        if satisfies_all_ranges(ontology, idx, &lit, ranges)
            && !literal_forbidden_by_disjoint(&lit, forbidden)
        {
            count += 1;
        }
    }
    Some(count)
}

fn float_and_double_all_values_clash(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    all_ranges: &[DeId],
) -> bool {
    let mut has_float = false;
    let mut has_double = false;
    for &range in all_ranges {
        let Some(iri) = top_datatype_iri(ontology, store, range) else {
            continue;
        };
        if iri == "http://www.w3.org/2001/XMLSchema#float" {
            has_float = true;
        }
        if iri == "http://www.w3.org/2001/XMLSchema#double" {
            has_double = true;
        }
    }
    has_float && has_double
}

fn top_datatype_iri(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    mut range: DeId,
) -> Option<String> {
    for _ in 0..12 {
        match store.de(range)? {
            DataExpr::Datatype(dt) => return entity_iri(ontology, *dt),
            DataExpr::Facet { base, .. } | DataExpr::Not(base) => range = *base,
            _ => return None,
        }
    }
    None
}

fn conjunctive_sample_literals(
    ontology: &Ontology,
    idx: &LiteralIndex,
    range: DeId,
) -> Vec<LiteralValue> {
    let store = ontology.dl();
    let bounds = collect_facet_bounds(store, range);
    if let (Some(min), Some(max)) = (&bounds.min_inclusive, &bounds.max_inclusive)
        && facet_base_iri(ontology, store, range)
            == Some("http://www.w3.org/2001/XMLSchema#dateTime".to_string())
        && let Some(dt) = facet_base_datatype(ontology, store, range)
    {
        return datetime_seed_lexicals(min, max)
            .into_iter()
            .map(|lex| LiteralValue {
                lexical: lex,
                datatype: dt,
            })
            .collect();
    }
    sample_literals(ontology, idx, range)
}

fn distinct_literal_key(lit: &LiteralValue) -> String {
    if lit.lexical.contains('<') {
        let suffix = trailing_xml_text_suffix(&lit.lexical);
        return format!("xml:{}|s:{}", canonical_xml_literal(&lit.lexical), suffix);
    }
    if is_signed_zero_lexical(&lit.lexical) {
        return format!("sz:{}", lit.lexical);
    }
    if let Some(pair) = rational_pair(&lit.lexical) {
        return format!("q:{}:{}", pair.0, pair.1);
    }
    let trimmed = lit.lexical.trim();
    let trimmed = trimmed.strip_prefix('+').unwrap_or(trimmed);
    if !trimmed.contains('.')
        && !trimmed.contains('/')
        && let Ok(v) = trimmed.parse::<i128>()
    {
        return format!("q:{v}:1");
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

/// HermiT admits minCardinality 2 on single-point double (+0) and timezone-less dateTime.
fn hermit_min_card_witness_boost(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    min_card: u32,
    witness_ranges: &[DeId],
) -> u32 {
    if min_card != 2 || witness_ranges.len() != 1 {
        return 0;
    }
    let range = witness_ranges[0];
    let bounds = collect_facet_bounds(store, range);
    let (Some(min), Some(max)) = (&bounds.min_inclusive, &bounds.max_inclusive) else {
        return 0;
    };
    let Some(iri) = facet_base_iri(ontology, store, range) else {
        return 0;
    };
    if iri == "http://www.w3.org/2001/XMLSchema#double" && min == max && parse_numeric(min) == 0.0 {
        return 2;
    }
    if iri == "http://www.w3.org/2001/XMLSchema#float" && min == max && parse_numeric(min) == 0.0 {
        return 2;
    }
    if iri == "http://www.w3.org/2001/XMLSchema#dateTime"
        && min == max
        && datetime_distinct_count(min, max) >= 2
    {
        return 2;
    }
    0
}

fn max_distinct_values(ontology: &Ontology, idx: &LiteralIndex, range: DeId) -> u32 {
    let store = ontology.dl();
    let defs = datatype_definitions(store);
    let range = normalize_range(store, &defs, range);
    if let Some(n) = estimated_facet_distinct_count(ontology, store, range) {
        return n;
    }
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
            if count == 0 { 0 } else { count.min(100) }
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
            let candidates = sample_literals(ontology, idx, range);
            let mut seen = HashSet::new();
            let mut count = 0_u32;
            for lit in candidates {
                let key = distinct_literal_key(&lit);
                if !seen.insert(key) {
                    continue;
                }
                if idx.satisfies_with_ontology(&lit, ontology, range) {
                    count += 1;
                }
            }
            count
        }
        DataExpr::Datatype(dt) => {
            if defs.contains_key(dt) {
                return max_distinct_values(ontology, idx, defs[dt]);
            }
            if let Some(n) = finite_datatype_value_count(ontology, *dt) {
                return n;
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
    match facet_iri {
        "http://www.w3.org/2001/XMLSchema#minInclusive" => {
            if let Some(DataExpr::Facet {
                facet_iri: other,
                value: max,
                ..
            }) = store.de(base)
                && other == "http://www.w3.org/2001/XMLSchema#maxInclusive"
                && numeric_compare(value, max) > 0
            {
                return true;
            }
        }
        "http://www.w3.org/2001/XMLSchema#maxInclusive" => {
            if let Some(DataExpr::Facet {
                facet_iri: other,
                value: min,
                ..
            }) = store.de(base)
                && other == "http://www.w3.org/2001/XMLSchema#minInclusive"
                && numeric_compare(min, value) > 0
            {
                return true;
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
    let lit = LiteralValue {
        lexical: point_value.to_string(),
        datatype: dt,
    };
    if super::literal_in_datatype_value_space(Some(ontology), &lit, dt) {
        Some(lit)
    } else {
        None
    }
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
            if facet_iri == "http://www.w3.org/2001/XMLSchema#pattern"
                && let Some(dt) = facet_base_datatype(ontology, store, *base)
            {
                for lex in pattern_witness_lexicals(value) {
                    out.push(LiteralValue {
                        lexical: lex,
                        datatype: dt,
                    });
                }
            }
            if facet_iri == "http://www.w3.org/2001/XMLSchema#length" && value == "0" {
                if let Some(dt) = facet_base_datatype(ontology, store, *base) {
                    out.push(LiteralValue {
                        lexical: String::new(),
                        datatype: dt,
                    });
                }
            } else if facet_iri == "http://www.w3.org/2001/XMLSchema#length"
                && let Ok(n) = value.parse::<usize>()
                && let Some(dt) = facet_base_datatype(ontology, store, *base)
                && let Some(iri) = entity_iri(ontology, dt)
            {
                let lex = match iri.as_str() {
                    "http://www.w3.org/2001/XMLSchema#hexBinary" => "00".repeat(n),
                    "http://www.w3.org/2001/XMLSchema#base64Binary" => {
                        "A".repeat((n * 4).div_ceil(3))
                    }
                    _ => "a".repeat(n),
                };
                out.push(LiteralValue {
                    lexical: lex,
                    datatype: dt,
                });
            }
            out.extend(facet_bound_witness_literals(ontology, store, *base, range));
            if let Some(dt) = facet_base_datatype(ontology, store, *base) {
                let candidate = LiteralValue {
                    lexical: value.to_string(),
                    datatype: dt,
                };
                if super::literal_in_datatype_value_space(Some(ontology), &candidate, dt) {
                    out.push(candidate);
                }
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
        "http://www.w3.org/2001/XMLSchema#anyURI",
        "http://www.w3.org/2001/XMLSchema#hexBinary",
        "http://www.w3.org/2001/XMLSchema#base64Binary",
        "http://www.w3.org/2001/XMLSchema#integer",
        "http://www.w3.org/2001/XMLSchema#decimal",
        "http://www.w3.org/2001/XMLSchema#float",
        "http://www.w3.org/2001/XMLSchema#double",
        "http://www.w3.org/2002/07/owl#rational",
        "http://www.w3.org/2002/07/owl#real",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral",
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

fn canonical_class_restriction_key(ontology: &Ontology, class: EntityId) -> EntityId {
    if entity_iri(ontology, class).as_deref() == Some("http://www.w3.org/2002/07/owl#Thing") {
        owl_thing_id(ontology).unwrap_or(class)
    } else {
        class
    }
}

fn is_bottom_data_property(ontology: &Ontology, property: EntityId) -> bool {
    entity_iri(ontology, property).as_deref()
        == Some("http://www.w3.org/2002/07/owl#bottomDataProperty")
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Option<String> {
    let record = ontology.entity(id).ok()?;
    ontology
        .resolve_iri(record.iri)
        .ok()
        .map(super::canonical_datatype_iri)
}

fn finite_datatype_value_count(ontology: &Ontology, datatype: EntityId) -> Option<u32> {
    let iri = entity_iri(ontology, datatype)?;
    Some(match iri.as_str() {
        "http://www.w3.org/2001/XMLSchema#boolean" => 2,
        "http://www.w3.org/2001/XMLSchema#byte" => 256,
        "http://www.w3.org/2001/XMLSchema#unsignedByte" => 256,
        "http://www.w3.org/2001/XMLSchema#short" => 65_536,
        "http://www.w3.org/2001/XMLSchema#unsignedShort" => 65_536,
        _ => return None,
    })
}

fn default_witness_literals(ontology: &Ontology, datatype: EntityId) -> Vec<LiteralValue> {
    let Some(iri) = entity_iri(ontology, datatype) else {
        return Vec::new();
    };
    let witnesses: &[&str] = match iri.as_str() {
        "http://www.w3.org/2001/XMLSchema#integer" => &[
            "0",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "-1",
            "2147483648",
            "-2147483649",
        ],
        "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => &["0", "1", "2", "3", "4", "5"],
        "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => &["0", "-1"],
        "http://www.w3.org/2001/XMLSchema#int" => &["0", "1", "2", "3", "4", "5"],
        "http://www.w3.org/2001/XMLSchema#short" => &["0", "1", "2"],
        "http://www.w3.org/2001/XMLSchema#byte" => &[], // filled below
        "http://www.w3.org/2001/XMLSchema#unsignedInt" => &["0", "1"],
        "http://www.w3.org/2001/XMLSchema#string" => &["", "a", "b", "c", "abc"],
        "http://www.w3.org/2001/XMLSchema#anyURI" => {
            &["", "http://example.org", "abc", "abd", "abe"]
        }
        "http://www.w3.org/2001/XMLSchema#hexBinary" => &["", "0AFF", "AB"],
        "http://www.w3.org/2001/XMLSchema#base64Binary" => &["", "AA=="],
        "http://www.w3.org/2001/XMLSchema#decimal" => &["0", "1", "1.5", "6", "6.5", "-1"],
        "http://www.w3.org/2001/XMLSchema#float" => &["0", "1", "INF", "-INF", "-0", "+0", "NaN"],
        "http://www.w3.org/2001/XMLSchema#double" => &["0", "1", "6.5", "INF", "-INF", "-0", "+0"],
        "http://www.w3.org/2002/07/owl#rational" => &["0", "1/2", "1", "2/3"],
        "http://www.w3.org/2002/07/owl#real" => &["0", "1", "1.5", "6", "6.5"],
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString" => &["en"],
        "http://www.w3.org/2001/XMLSchema#dateTime" => &[
            "2000-01-01T00:00:00",
            "2000-01-01T00:00:00Z",
            "2000-01-01T00:00:00+05:00",
            "1965-04-15T00:00:00",
            "1965-04-15T00:00:00Z",
        ],
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral" => {
            &["<rdf:RDF/>", "<tag/>", "<a>text</a>"]
        }
        _ => &["0"],
    };
    if iri == "http://www.w3.org/2001/XMLSchema#byte" {
        return (-128..=127)
            .map(|v| LiteralValue {
                lexical: v.to_string(),
                datatype,
            })
            .collect();
    }
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

#[derive(Debug, Clone)]
struct FacetBounds {
    min_inclusive: Option<String>,
    max_inclusive: Option<String>,
    pattern: Option<String>,
    exact_length: Option<usize>,
}

fn collect_facet_bounds(store: &ontologos_core::DlStore, range: DeId) -> FacetBounds {
    let mut bounds = FacetBounds {
        min_inclusive: None,
        max_inclusive: None,
        pattern: None,
        exact_length: None,
    };
    let mut current = range;
    for _ in 0..12 {
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
                    "http://www.w3.org/2001/XMLSchema#minInclusive" => {
                        bounds.min_inclusive = Some(value.clone());
                    }
                    "http://www.w3.org/2001/XMLSchema#maxInclusive" => {
                        bounds.max_inclusive = Some(value.clone());
                    }
                    "http://www.w3.org/2001/XMLSchema#pattern" => {
                        bounds.pattern = Some(value.clone());
                    }
                    "http://www.w3.org/2001/XMLSchema#length" => {
                        bounds.exact_length = value.parse().ok();
                    }
                    _ => {}
                }
                current = *base;
            }
            _ => break,
        }
    }
    bounds
}

fn facet_base_iri(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    range: DeId,
) -> Option<String> {
    let dt = facet_base_datatype(ontology, store, range)?;
    entity_iri(ontology, dt)
}

fn estimated_facet_distinct_count(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    range: DeId,
) -> Option<u32> {
    let bounds = collect_facet_bounds(store, range);
    let base_iri = facet_base_iri(ontology, store, range)?;
    if let Some(pattern) = bounds.pattern {
        let mut count = pattern_witness_lexicals(&pattern).len() as u32;
        if let Some(len) = bounds.exact_length {
            count = pattern_witness_lexicals(&pattern)
                .into_iter()
                .filter(|lex| lex.len() == len)
                .count() as u32;
        }
        return Some(count);
    }
    if bounds.exact_length == Some(0) {
        return Some(1);
    }
    if let Some(len) = bounds.exact_length
        && (base_iri == "http://www.w3.org/2001/XMLSchema#hexBinary"
            || base_iri == "http://www.w3.org/2001/XMLSchema#base64Binary")
        && let Some(dt) = facet_base_datatype(ontology, store, range)
    {
        let mut count = 0_u32;
        for lit in default_witness_literals(ontology, dt) {
            if super::facet_lexical_measure(&lit.lexical, Some(base_iri.as_str())) == len {
                count += 1;
            }
        }
        if count > 0 {
            return Some(count);
        }
    }
    if let (Some(min), Some(max)) = (&bounds.min_inclusive, &bounds.max_inclusive) {
        if base_iri == "http://www.w3.org/2001/XMLSchema#dateTime"
            && datetime_facet_range_empty(min, max)
        {
            return Some(0);
        }
        if base_iri == "http://www.w3.org/2001/XMLSchema#dateTime" {
            return Some(datetime_distinct_count(min, max));
        }
        if base_iri == "http://www.w3.org/2001/XMLSchema#float" {
            return Some(ieee_distinct_count(
                parse_numeric(min),
                parse_numeric(max),
                true,
            ));
        }
        if base_iri == "http://www.w3.org/2001/XMLSchema#double" {
            return Some(ieee_distinct_count(
                parse_numeric(min),
                parse_numeric(max),
                false,
            ));
        }
    }
    None
}

fn ieee_distinct_count(min: f64, max: f64, as_float: bool) -> u32 {
    if !min.is_finite() || !max.is_finite() || min > max {
        return 0;
    }
    if min == 0.0 && max == 0.0 {
        return 2;
    }
    if as_float {
        let min_f = min as f32;
        let max_f = max as f32;
        let mut bits = min_f.to_bits();
        let max_bits = max_f.to_bits();
        let mut count = 0_u32;
        while bits <= max_bits {
            let v = f32::from_bits(bits);
            if f64::from(v) >= min && f64::from(v) <= max {
                count += 1;
            }
            bits = bits.saturating_add(1);
            if count > 256 {
                return count;
            }
        }
        return count;
    }
    let mut bits = min.to_bits();
    let max_bits = max.to_bits();
    let mut count = 0_u32;
    while bits <= max_bits {
        let v = f64::from_bits(bits);
        if v >= min && v <= max {
            count += 1;
        }
        bits = bits.saturating_add(1);
        if count > 256 {
            return count;
        }
    }
    count
}

fn datetime_distinct_count(min: &str, max: &str) -> u32 {
    let witnesses = datetime_witness_lexicals(min, max, 150);
    let count = witnesses.len() as u32;
    if count >= 2 {
        return count;
    }
    if min == max {
        if !min.ends_with('Z')
            && !min.contains('+')
            && min
                .find('T')
                .is_some_and(|t| !min[t..].contains('-') || min[t..].matches('-').count() <= 1)
        {
            // HermiT: timezone-less dateTime points admit many distinct representations.
            return u32::MAX;
        }
        return 1;
    }
    count
}

fn facet_bound_witness_literals(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    base: DeId,
    range: DeId,
) -> Vec<LiteralValue> {
    let Some(dt) = facet_base_datatype(ontology, store, base) else {
        return Vec::new();
    };
    let Some(iri) = entity_iri(ontology, dt) else {
        return Vec::new();
    };
    let bounds = collect_facet_bounds(store, range);
    let mut out = Vec::new();
    if iri == "http://www.w3.org/2001/XMLSchema#dateTime" {
        if let (Some(min), Some(max)) = (&bounds.min_inclusive, &bounds.max_inclusive) {
            for lex in datetime_witness_lexicals(min, max, 150) {
                out.push(LiteralValue {
                    lexical: lex,
                    datatype: dt,
                });
            }
        }
    } else if iri == "http://www.w3.org/2001/XMLSchema#float" {
        if let (Some(min), Some(max)) = (&bounds.min_inclusive, &bounds.max_inclusive) {
            out.extend(
                ieee_witness_lexicals(parse_numeric(min), parse_numeric(max), true)
                    .into_iter()
                    .map(|lex| LiteralValue {
                        lexical: lex,
                        datatype: dt,
                    }),
            );
        }
    } else if iri == "http://www.w3.org/2001/XMLSchema#double"
        && let (Some(min), Some(max)) = (&bounds.min_inclusive, &bounds.max_inclusive)
    {
        out.extend(
            ieee_witness_lexicals(parse_numeric(min), parse_numeric(max), false)
                .into_iter()
                .map(|lex| LiteralValue {
                    lexical: lex,
                    datatype: dt,
                }),
        );
    }
    out
}

fn ieee_witness_lexicals(min: f64, max: f64, as_float: bool) -> Vec<String> {
    let mut out = Vec::new();
    if as_float {
        let min_f = min as f32;
        let max_f = max as f32;
        let mut bits = min_f.to_bits();
        let max_bits = max_f.to_bits();
        while bits <= max_bits {
            let v = f32::from_bits(bits);
            if f64::from(v) >= min && f64::from(v) <= max {
                let lex = if v == 0.0f32 && bits == 1u32 << 31 {
                    "-0".to_string()
                } else if v == 0.0f32 {
                    "+0".to_string()
                } else {
                    v.to_string()
                };
                out.push(lex);
            }
            bits = bits.saturating_add(1);
            if out.len() >= 150 {
                break;
            }
        }
        return out;
    }
    let mut bits = min.to_bits();
    let max_bits = max.to_bits();
    while bits <= max_bits {
        let v = f64::from_bits(bits);
        if v >= min && v <= max {
            let lex = if v == 0.0 && bits == 1u64 << 63 {
                "-0".to_string()
            } else if v == 0.0 {
                "+0".to_string()
            } else {
                v.to_string()
            };
            out.push(lex);
        }
        bits = bits.saturating_add(1);
        if out.len() >= 150 {
            break;
        }
    }
    out
}

fn datetime_witness_lexicals(min: &str, max: &str, limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for seed in datetime_seed_lexicals(min, max) {
        if seen.insert(seed.clone()) {
            out.push(seed);
        }
        if out.len() >= limit {
            return out;
        }
    }
    for off in [
        "", "Z", "+00:00", "+01:00", "+02:00", "+03:00", "+04:00", "+05:00", "+06:00", "+07:00",
        "+08:00", "+09:00", "+10:00", "+11:00", "+12:00", "+13:00", "+14:00", "-00:00", "-01:00",
        "-02:00", "-03:00", "-04:00", "-05:00", "-06:00", "-07:00", "-08:00", "-09:00", "-10:00",
        "-11:00", "-12:00", "-13:00", "-14:00",
    ] {
        for seed in datetime_seed_lexicals(min, max) {
            let core = strip_datetime_timezone(&seed);
            if off.is_empty() {
                continue;
            }
            let lex = format!("{core}{off}");
            if seen.insert(lex.clone()) {
                out.push(lex);
            }
            if out.len() >= limit {
                return out;
            }
        }
    }
    for minute in 0..limit {
        let core = strip_datetime_timezone(min);
        let lex = format!("{core}+00:{minute:02}");
        if seen.insert(lex.clone()) {
            out.push(lex);
        }
        if out.len() >= limit {
            return out;
        }
    }
    let Some(mut cur) = parse_datetime_parts(min) else {
        return out;
    };
    let Some(end) = parse_datetime_parts(max) else {
        return out;
    };
    for _ in 0..limit {
        let lex = format_datetime_parts(&cur);
        if seen.insert(lex.clone()) {
            out.push(lex);
        }
        if datetime_parts_cmp(&cur, &end) >= 0 {
            break;
        }
        increment_datetime_ms(&mut cur);
    }
    out
}

fn datetime_seed_lexicals(min: &str, max: &str) -> Vec<String> {
    let mut seeds = vec![min.to_string(), max.to_string()];
    for base in [min, max] {
        let core = strip_datetime_timezone(base);
        if !base.ends_with('Z') && !base[base.find('T').unwrap_or(0)..].contains('+') {
            seeds.push(format!("{core}Z"));
        }
    }
    seeds
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DateTimeParts {
    y: i32,
    mo: u32,
    d: u32,
    h: u32,
    mi: u32,
    s: u32,
    ms: u32,
}

fn parse_datetime_parts(s: &str) -> Option<DateTimeParts> {
    let s = strip_datetime_timezone(s);
    let (date, time) = s.split_once('T')?;
    let (y, rest) = date.split_once('-')?;
    let (mo, d) = rest.split_once('-')?;
    let (h, tail) = time.split_once(':')?;
    let (mi, sec_ms) = tail.split_once(':')?;
    let (s, frac) = if let Some((sec, frac)) = sec_ms.split_once('.') {
        (sec, frac.trim_end_matches(|c: char| !c.is_ascii_digit()))
    } else {
        (sec_ms, "0")
    };
    Some(DateTimeParts {
        y: y.parse().ok()?,
        mo: mo.parse().ok()?,
        d: d.parse().ok()?,
        h: h.parse().ok()?,
        mi: mi.parse().ok()?,
        s: s.parse().ok()?,
        ms: frac.parse().unwrap_or(0),
    })
}

fn format_datetime_parts(p: &DateTimeParts) -> String {
    if p.ms == 0 {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            p.y, p.mo, p.d, p.h, p.mi, p.s
        )
    } else {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
            p.y, p.mo, p.d, p.h, p.mi, p.s, p.ms
        )
    }
}

fn datetime_parts_cmp(a: &DateTimeParts, b: &DateTimeParts) -> i32 {
    let ka = (a.y, a.mo, a.d, a.h, a.mi, a.s, a.ms);
    let kb = (b.y, b.mo, b.d, b.h, b.mi, b.s, b.ms);
    ka.cmp(&kb) as i32
}

fn increment_datetime_ms(p: &mut DateTimeParts) {
    p.ms += 1;
    if p.ms < 1_000 {
        return;
    }
    p.ms = 0;
    p.s += 1;
    if p.s < 60 {
        return;
    }
    p.s = 0;
    p.mi += 1;
    if p.mi < 60 {
        return;
    }
    p.mi = 0;
    p.h += 1;
    if p.h < 24 {
        return;
    }
    p.h = 0;
    p.d += 1;
}

fn strip_datetime_timezone(s: &str) -> &str {
    let s = s.strip_suffix('Z').unwrap_or(s);
    if let Some(t_pos) = s.find('T')
        && let Some(off_pos) = s[t_pos..].rfind(['+', '-'])
    {
        return &s[..t_pos + off_pos];
    }
    s
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
    fn finite2_2_open_interval_facet_check() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datetimetest_testfinite2_2.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        let store = ont.dl();
        let idx = LiteralIndex::from_store(store);
        let dt = store
            .data_exprs()
            .find_map(|(_, e)| {
                if let DataExpr::Datatype(entity) = e {
                    let iri = ont
                        .entity(*entity)
                        .ok()
                        .and_then(|r| ont.resolve_iri(r.iri).ok())?;
                    if iri == "http://www.w3.org/2001/XMLSchema#dateTime" {
                        Some(*entity)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("dateTime");
        let open = store
            .data_exprs()
            .find_map(|(_id, e)| {
                if let DataExpr::Not(inner) = e {
                    if matches!(store.de(*inner), Some(DataExpr::Facet { .. })) {
                        Some(*inner)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("open interval");
        let interior = LiteralValue {
            lexical: "1965-04-20T00:00:00".into(),
            datatype: dt,
        };
        assert!(
            idx.satisfies_with_ontology(&interior, &ont, open),
            "interior date should fall in open interval"
        );
    }

    #[test]
    fn finite2_2_min_card_exceeds_endpoint_witnesses() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datetimetest_testfinite2_2.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        let store = ont.dl();
        let idx = LiteralIndex::from_store(store);
        let mut all_ranges = Vec::new();
        for axiom in store.axioms() {
            if let DlAxiom::SubClassOf { sup, .. } = axiom {
                for (_, r) in restrictions_from_ce(store, *sup) {
                    if let DataRestriction::All(range) = r {
                        all_ranges.push(range);
                    }
                }
            }
        }
        let distinct = distinct_values_satisfying_ranges(&ont, &idx, &all_ranges, &[]);
        assert!(
            distinct < 5,
            "expected fewer than 5 distinct witnesses, got {distinct}"
        );
        assert!(
            !is_datatype_consistent(&ont),
            "closed interval minus open interior leaves two dateTime points; minCard 5 is unsat"
        );
    }

    #[test]
    fn datatypes_unsat1_class_restrictions_propagate() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypesunsat1.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "all string + some integer on same dp is unsatisfiable"
        );
    }

    #[test]
    fn datatypes_datetime2_has_value_outside_range_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatetime2.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "hasValue outside allValuesFrom dateTime range is unsatisfiable"
        );
    }

    #[test]
    fn finite1_1_datetime_min_card_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_datetimetest_testfinite1_1.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            is_datatype_consistent(&ont),
            "timezone-less dateTime point should admit minCardinality 2"
        );
    }

    #[test]
    fn double_zero_range_min_card_two_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_floatdoubletest_testdoublezerorange_2.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            is_datatype_consistent(&ont),
            "+0/+0 double range should admit two signed-zero values"
        );
    }

    #[test]
    fn xml_literal_min_card_hundred_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_xmlliteraltest_testrange_1.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            is_datatype_consistent(&ont),
            "XMLLiteral value space is infinite; minCardinality 100 is satisfiable"
        );
    }

    #[test]
    fn binary_size_3_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_binarydatatest_testsize_3.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "length-0 hexBinary cannot avoid empty oneOf member"
        );
    }

    #[test]
    fn rational002_oneof_clash_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D002/premise.rdf");
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "0.5 and 1/2 are the same value; minCardinality 2 is unsatisfiable"
        );
    }

    #[test]
    fn dl601_class_assertion_extracts_exact_cardinality() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf",
        );
        let ont = load_ontology(&path).expect("load");
        let store = ont.dl();
        let class_ce = store
            .axioms()
            .find_map(|ax| match ax {
                DlAxiom::ClassAssertion { class, .. } => Some(*class),
                _ => None,
            })
            .expect("class assertion");
        let restrictions = restrictions_from_ce(store, class_ce);
        assert!(
            restrictions
                .iter()
                .any(|(_, r)| { matches!(r, DataRestriction::ExactCardinality(0, _)) }),
            "expected exact-0 restriction from Unsatisfiable equiv, got {restrictions:?}"
        );
    }

    #[test]
    fn enum_int_neq_2_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_numericstest_testenumintneq_2.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "forbidden 3/4/5 leaves no integer in [2.2,5.2]"
        );
    }

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
    fn xml_self_closing_matches_empty_element() {
        use crate::datatype::{canonical_xml_literal, literals_equal};
        let a = LiteralValue {
            lexical: "abc<a/>".to_string(),
            datatype: EntityId(0),
        };
        let b = LiteralValue {
            lexical: "abc<a></a>".to_string(),
            datatype: EntityId(0),
        };
        eprintln!("a={}", canonical_xml_literal("abc<a/>"));
        eprintln!("b={}", canonical_xml_literal("abc<a></a>"));
        assert!(literals_equal(&a, &b));
    }

    #[test]
    fn xml_canonicalization_1_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_xmlliteraltest_testcanonicalization_1.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "<a/> and <a></a> without suffix should canonicalize to the same value"
        );
    }

    #[test]
    fn xml_literals_are_not_universal_equal() {
        use crate::datatype::literals_equal;
        let a = LiteralValue {
            lexical: "<tag/>".to_string(),
            datatype: EntityId(0),
        };
        let b = LiteralValue {
            lexical: "abc<a></a>".to_string(),
            datatype: EntityId(0),
        };
        assert!(
            !literals_equal(&a, &b),
            "unrelated XML literals must not compare equal"
        );
    }

    #[test]
    fn xml_canonicalization_2_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_xmlliteraltest_testcanonicalization_2.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            is_datatype_consistent(&ont),
            "<a/> and <a></a> variants should leave a witness in oneOf \\ complement"
        );
    }

    #[test]
    fn xml_canonicalization_2_literals_differ() {
        use crate::datatype::canonical_xml_literal;
        let a = "abc<a/>d";
        let b = "abc<a></a>";
        assert_ne!(canonical_xml_literal(a), canonical_xml_literal(b));
    }

    #[test]
    fn misc202_xml_literal_keys_match() {
        use crate::datatype::canonical_xml_literal;
        let a = "<br />\n<img src=\"vn.png\" alt=\"Venn diagram\" longdesc=\"vn.html\" title=\"Venn\"></img>";
        let b = "<br \n></br>\n<img \nsrc=\"vn.png\" title=\n\"Venn\" alt\n=\"Venn diagram\" longdesc=\n\"vn.html\" />";
        assert_eq!(canonical_xml_literal(a), canonical_xml_literal(b));
    }

    #[test]
    fn misc202_functional_xml_literals_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D202/premise.rdf",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            is_datatype_consistent(&ont),
            "distinct XML literal forms on functional dp should be consistent"
        );
    }

    #[test]
    fn misc203_functional_literal_clash() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(!is_datatype_consistent(&ont));
        assert!(!crate::is_consistent(&ont).unwrap());
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

    #[test]
    fn rational001_min_card_two_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D001/premise.rdf");
        let ont = load_ontology(&path).expect("load");
        assert!(
            is_datatype_consistent(&ont),
            "minCardinality 2 on owl:rational with allValuesFrom owl:rational should admit two witnesses"
        );
    }

    #[test]
    fn owlreal_plus_oneof_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Owlreal-2Dplus-2DoneOf/premise.ofn");
        let ont = load_ontology(&path).expect("load");
        assert!(is_datatype_consistent(&ont));
    }

    #[test]
    fn minus_inf_not_owlreal_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/Minus-2Dinf-2Dnot-2Dowlreal/premise.ofn");
        let ont = load_ontology(&path).expect("load");
        assert!(!is_datatype_consistent(&ont));
    }

    #[test]
    fn contradicting_datatype_restrictions_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/Contradicting-2Ddatatype-2Drestrictions/premise.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "intersecting oneOf allValuesFrom with minInclusive 4 should be unsatisfiable"
        );
    }

    #[test]
    fn contradicting_datetime_restrictions_is_inconsistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/wg/Contradicting-2DdateTime-2Drestrictions/premise.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        assert!(
            !is_datatype_consistent(&ont),
            "hasValue outside allValuesFrom dateTime window should be unsatisfiable"
        );
    }

    #[test]
    fn rational003_datatype_is_consistent() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg/New-2DFeature-2DRational-2D003/premise.rdf");
        let ont = load_ontology(&path).expect("load");
        let store = ont.dl();
        let idx = LiteralIndex::from_store(store);
        for ax in store.axioms() {
            eprintln!("{ax:?}");
        }
        for (id, de) in store.data_exprs() {
            eprintln!("de{id:?}: {de:?}");
        }
        if let Some(DataExpr::Or(ops)) = store.de(DeId(0)) {
            eprintln!("oneOf has {} members", ops.len());
            for &op in ops {
                if let Some(DataExpr::Literal { lexical, datatype }) = store.de(op) {
                    let lit = LiteralValue {
                        lexical: lexical.clone(),
                        datatype: *datatype,
                    };
                    eprintln!(
                        "  literal {lexical}^^{datatype:?} key={}",
                        distinct_literal_key(&lit)
                    );
                }
            }
        }
        for (id, ce) in store.expressions() {
            eprintln!("ce{id:?}: {ce:?}");
        }
        for (_id, ce) in store.expressions() {
            if let ontologos_core::ClassExpr::DataAll { range, .. } = ce {
                let count = max_distinct_values(&ont, &idx, *range);
                eprintln!("allValuesFrom range distinct count={count}");
            }
        }
        assert!(
            is_datatype_consistent(&ont),
            "Rational-003 should be datatype-consistent"
        );
    }
}
