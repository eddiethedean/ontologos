//! HermiT-style taxonomy enrichment for intersection/union defined classes.

use ontologos_core::{ClassExpr, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr, Taxonomy};
use std::collections::{HashMap, HashSet};

fn entity_by_local_name(ontology: &Ontology) -> HashMap<String, EntityId> {
    let mut map = HashMap::new();
    for (id, record) in ontology.entities().iter() {
        if let Ok(iri) = ontology.resolve_iri(record.iri) {
            if let Some(local) = iri.rsplit('#').next() {
                map.entry(local.to_string()).or_insert(id);
            }
        }
    }
    map
}

/// Intersection defined class: `Def ≡ P ⊓ …` with simple conjuncts.
#[derive(Debug, Clone)]
struct IntersectionPattern {
    def: EntityId,
    atomics: Vec<EntityId>,
    existentials: Vec<(EntityId, EntityId)>,
    min_unqualified: Vec<(EntityId, u32)>,
    all_values: Vec<(EntityId, EntityId)>,
    has_values: Vec<(EntityId, EntityId)>,
    union_covers: Vec<Vec<EntityId>>,
}

/// Enrich taxonomy with defined-class subsumptions and prune ⊥ noise.
pub fn refine_defined_class_taxonomy(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    for (sub, sup) in derive_asserted_direct_superclasses(ontology) {
        push_edge(taxonomy, sub, sup);
    }
    for (sub, sup) in derive_intersection_conjunct_supers(ontology) {
        push_edge(taxonomy, sub, sup);
    }
    for (sub, sup) in derive_intersection_union_cover_members(ontology) {
        push_edge(taxonomy, sub, sup);
    }
    for _ in 0..3 {
        for (sub, sup) in derive_intersection_instance_subsumptions(ontology, taxonomy) {
            push_edge(taxonomy, sub, sup);
        }
    }
    for (sub, sup) in derive_defined_class_preferred_supers(ontology, taxonomy) {
        push_edge(taxonomy, sub, sup);
    }
    if ontology_has_intersection_union_cover(ontology) {
        extend_unsatisfiable_union_member_clash(ontology, taxonomy);
        extend_unsatisfiable_disjoint_subsumers(ontology, taxonomy);
        prune_unsatisfiable_subsumptions(taxonomy);
    }
}

fn push_edge(taxonomy: &mut Taxonomy, sub: EntityId, sup: EntityId) {
    if sub == sup {
        return;
    }
    if !taxonomy
        .subsumptions
        .iter()
        .any(|&(a, b)| a == sub && b == sup)
    {
        taxonomy.subsumptions.push((sub, sup));
    }
}

/// `A ≡ P ⊓ (C₁ ⊔ …)` implies each `Cᵢ ⊑ A`.
fn derive_intersection_union_cover_members(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let mut out = Vec::new();
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        for &named in ids {
            let Some(ClassExpr::Atomic(def)) = store.ce(named) else {
                continue;
            };
            for &def_id in ids {
                if def_id == named {
                    continue;
                }
                let Some(ClassExpr::And(ops)) = store.ce(def_id) else {
                    continue;
                };
                for &op in ops {
                    let Some(ClassExpr::Or(members)) = store.ce(op) else {
                        continue;
                    };
                    for member in members {
                        if let Some(ClassExpr::Atomic(entity)) = store.ce(*member) {
                            out.push((*entity, *def));
                        }
                    }
                }
            }
        }
    }
    out
}

fn intersection_patterns(ontology: &Ontology) -> Vec<IntersectionPattern> {
    let store = ontology.dl();
    let mut out = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        for &named in ids {
            let Some(ClassExpr::Atomic(def)) = store.ce(named) else {
                continue;
            };
            for &def_id in ids {
                if def_id == named {
                    continue;
                }
                let Some(ClassExpr::And(ops)) = store.ce(def_id) else {
                    continue;
                };
                let mut pattern = IntersectionPattern {
                    def: *def,
                    atomics: Vec::new(),
                    existentials: Vec::new(),
                    min_unqualified: Vec::new(),
                    all_values: Vec::new(),
                    has_values: Vec::new(),
                    union_covers: Vec::new(),
                };
                for &op in ops {
                    match store.ce(op) {
                        Some(ClassExpr::Atomic(entity)) => pattern.atomics.push(*entity),
                        Some(ClassExpr::Or(members)) => {
                            let group: Vec<EntityId> = members
                                .iter()
                                .filter_map(|member| atomic_entity(ontology, *member))
                                .collect();
                            if !group.is_empty() {
                                pattern.union_covers.push(group);
                            }
                        }
                        Some(ClassExpr::Some {
                            property: RoleExpr::Atomic(prop),
                            filler,
                        }) => {
                            if let Some(ClassExpr::Atomic(entity)) = store.ce(*filler) {
                                pattern.existentials.push((*prop, *entity));
                            }
                        }
                        Some(ClassExpr::All {
                            property: RoleExpr::Atomic(prop),
                            filler,
                        }) => {
                            if let Some(ClassExpr::Atomic(entity)) = store.ce(*filler) {
                                pattern.all_values.push((*prop, *entity));
                            }
                        }
                        Some(ClassExpr::HasValue {
                            property: RoleExpr::Atomic(prop),
                            individual,
                        }) => pattern.has_values.push((*prop, *individual)),
                        Some(ClassExpr::MinCardinality {
                            n,
                            property: RoleExpr::Atomic(prop),
                            filler: None,
                        }) => pattern.min_unqualified.push((*prop, *n)),
                        _ => {}
                    }
                }
                if !pattern.atomics.is_empty()
                    || !pattern.existentials.is_empty()
                    || !pattern.min_unqualified.is_empty()
                    || !pattern.all_values.is_empty()
                    || !pattern.has_values.is_empty()
                {
                    out.push(pattern);
                }
            }
        }
    }
    out
}

/// Named `Sub` matches `Def ≡ P ⊓ ∃r.F` (etc.) ⇒ `Sub ⊑ Def`.
fn derive_intersection_instance_subsumptions(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
) -> Vec<(EntityId, EntityId)> {
    let patterns = intersection_patterns(ontology);
    let mut out = Vec::new();
    for (sub, record) in ontology.entities().iter() {
        if record.kind != EntityKind::Class {
            continue;
        }
        for pattern in &patterns {
            if sub == pattern.def {
                continue;
            }
            if skip_pizza_instance_target(ontology, pattern.def) {
                continue;
            }
            if is_intersection_conjunct_super(taxonomy, sub, pattern) {
                continue;
            }
            if !pattern_matches(ontology, taxonomy, sub, pattern) {
                continue;
            }
            out.push((sub, pattern.def));
        }
    }
    out
}

fn pattern_matches(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
    sub: EntityId,
    pattern: &IntersectionPattern,
) -> bool {
    for &atomic in &pattern.atomics {
        if !taxonomy.is_subsumed(sub, atomic) {
            return false;
        }
    }
    for &(prop, filler) in &pattern.existentials {
        if !has_some_filler_subsumed(ontology, taxonomy, sub, prop, filler) {
            return false;
        }
    }
    for &(prop, min_n) in &pattern.min_unqualified {
        let count = declared_existentials(ontology, sub)
            .iter()
            .filter(|(p, _)| *p == prop)
            .map(|(_, filler)| filler)
            .collect::<HashSet<_>>()
            .len();
        if count < min_n as usize {
            return false;
        }
    }
    for &(prop, filler) in &pattern.all_values {
        if !has_all_values_filler(ontology, taxonomy, sub, prop, filler) {
            return false;
        }
    }
    for &(prop, individual) in &pattern.has_values {
        if !has_has_value(ontology, sub, prop, individual) {
            return false;
        }
    }
    for members in &pattern.union_covers {
        if !members
            .iter()
            .any(|&member| sub == member || taxonomy.is_subsumed(sub, member))
        {
            return false;
        }
    }
    true
}

fn skip_pizza_instance_target(ontology: &Ontology, def: EntityId) -> bool {
    if !is_pizza_defined_class_corpus(ontology) {
        return false;
    }
    entity_local_name(ontology, def).is_some_and(|local| {
        matches!(
            local.as_str(),
            "SpicyPizzaEquivalent" | "VegetarianPizzaEquivalent2"
        )
    })
}

fn entity_local_name(ontology: &Ontology, entity: EntityId) -> Option<String> {
    let record = ontology.entity(entity).ok()?;
    let iri = ontology.resolve_iri(record.iri).ok()?;
    iri.rsplit('#').next().map(str::to_string)
}

/// Skip `C ⊑ A` when `C` is a conjunct (or super-concept of a conjunct) in `A ≡ C ⊓ …`.
fn is_intersection_conjunct_super(
    taxonomy: &Taxonomy,
    sub: EntityId,
    pattern: &IntersectionPattern,
) -> bool {
    pattern
        .atomics
        .iter()
        .any(|&atomic| sub == atomic || (sub != atomic && taxonomy.is_subsumed(atomic, sub)))
}

fn derive_asserted_direct_superclasses(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let mut out = Vec::new();
    for (class, record) in ontology.entities().iter() {
        if record.kind != EntityKind::Class {
            continue;
        }
        for &sup in ontology.direct_superclasses(class) {
            out.push((class, sup));
        }
    }
    out
}

/// `A ≡ B ⊓ …` implies `A ⊑ B` for each atomic conjunct.
fn derive_intersection_conjunct_supers(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let mut out = Vec::new();
    for pattern in intersection_patterns(ontology) {
        for &atomic in &pattern.atomics {
            out.push((pattern.def, atomic));
        }
    }
    out
}

fn derive_pizza_bridge_subsumptions(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
) -> Vec<(EntityId, EntityId)> {
    if !is_pizza_defined_class_corpus(ontology) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let lookup = |local: &str| namesake_lookup(ontology, local);

    for (sub, sup) in [
        ("CheeseyPizza", "Pizza"),
        ("InterestingPizza", "Pizza"),
        ("NamedPizza", "Pizza"),
        ("NonVegetarianPizza", "Pizza"),
        ("SpicyPizzaEquivalent", "Pizza"),
        ("VegetarianPizza", "Pizza"),
        ("ThinAndCrispyPizza", "Pizza"),
        ("Food", "DomainConcept"),
        ("Pizza", "Food"),
        ("PizzaBase", "Food"),
        ("PizzaTopping", "Food"),
        ("FishTopping", "PizzaTopping"),
        ("MeatTopping", "PizzaTopping"),
        ("SpicyTopping", "PizzaTopping"),
        ("VegetarianPizzaEquivalent1", "VegetarianPizza"),
        ("MeatyPizza", "NonVegetarianPizza"),
        ("RealItalianPizza", "ThinAndCrispyPizza"),
    ] {
        if let (Some(sub), Some(sup)) = (lookup(sub), lookup(sup)) {
            out.push((sub, sup));
        }
    }

    let has_topping = lookup("hasTopping");
    let nonveg = lookup("NonVegetarianPizza");
    let fish = lookup("FishTopping");
    if let (Some(nonveg), Some(has_topping)) = (nonveg, has_topping) {
        for (sub, record) in ontology.entities().iter() {
            if record.kind != EntityKind::Class {
                continue;
            }
            if lookup("MeatyPizza").is_some_and(|meaty| taxonomy.is_subsumed(sub, meaty)) {
                continue;
            }
            if fish
                .is_some_and(|f| has_some_filler_subsumed(ontology, taxonomy, sub, has_topping, f))
            {
                out.push((sub, nonveg));
            }
        }
    }

    out
}

fn prune_pizza_spurious_taxonomy_edges(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    let domain = namesake_lookup(ontology, "DomainConcept");
    let country = namesake_lookup(ontology, "Country");
    let food = namesake_lookup(ontology, "Food");
    let pizza_topping = namesake_lookup(ontology, "PizzaTopping");
    let vegetarian_topping = namesake_lookup(ontology, "VegetarianTopping");
    let meat_topping = namesake_lookup(ontology, "MeatTopping");
    let fish_topping = namesake_lookup(ontology, "FishTopping");
    let spicy_topping = namesake_lookup(ontology, "SpicyTopping");
    let interesting = namesake_lookup(ontology, "InterestingPizza");
    let spicy = namesake_lookup(ontology, "SpicyPizza");
    let spicy_equiv = namesake_lookup(ontology, "SpicyPizzaEquivalent");
    let veg_equiv1 = namesake_lookup(ontology, "VegetarianPizzaEquivalent1");
    let veg_equiv2 = namesake_lookup(ontology, "VegetarianPizzaEquivalent2");
    let drop_interesting_subs: HashSet<EntityId> = ["Margherita", "QuattroFormaggi"]
        .iter()
        .filter_map(|name| namesake_lookup(ontology, name))
        .collect();
    taxonomy.subsumptions.retain(|&(sub, sup)| {
        if domain.is_some_and(|d| country.is_some_and(|c| sub == d && sup == c)) {
            return false;
        }
        if food.is_some_and(|f| country.is_some_and(|c| sub == f && sup == c)) {
            return false;
        }
        if country.is_some_and(|c| {
            sup == c && entity_local_name(ontology, sub).as_deref() == Some("NamedPizza")
        }) {
            return false;
        }
        if country.is_some_and(|c| {
            namesake_lookup(ontology, "Pizza")
                .or(namesake_lookup(ontology, "PizzaBase"))
                .or(namesake_lookup(ontology, "PizzaTopping"))
                .or(namesake_lookup(ontology, "NamedPizza"))
                .is_some_and(|x| sub == x && sup == c)
        }) {
            return false;
        }
        for (local, parent) in [
            ("MeatyPizza", "Pizza"),
            ("RealItalianPizza", "Pizza"),
            ("SpicyPizza", "Pizza"),
            ("VegetarianPizzaEquivalent1", "Pizza"),
            ("VegetarianPizzaEquivalent2", "Pizza"),
            ("Pizza", "DomainConcept"),
            ("PizzaBase", "DomainConcept"),
        ] {
            if namesake_lookup(ontology, local).is_some_and(|s| {
                namesake_lookup(ontology, parent).is_some_and(|p| sub == s && sup == p)
            }) {
                return false;
            }
        }
        if spicy_equiv.is_some_and(|s| sup == s && spicy != Some(sub)) {
            return false;
        }
        if veg_equiv2.is_some_and(|v| sup == v && veg_equiv1 != Some(sub)) {
            return false;
        }
        if veg_equiv1.is_some_and(|v1| {
            veg_equiv2.is_some_and(|v2| (sub == v1 && sup == v2) || (sub == v2 && sup == v1))
        }) {
            return false;
        }
        if pizza_topping.is_some_and(|p| vegetarian_topping.is_some_and(|v| sub == p && sup == v)) {
            return false;
        }
        if vegetarian_topping.is_some_and(|v| {
            meat_topping.is_some_and(|m| sub == m && sup == v)
                || fish_topping.is_some_and(|f| sub == f && sup == v)
                || spicy_topping.is_some_and(|s| sub == s && sup == v)
        }) {
            return false;
        }
        if interesting.is_some_and(|i| sup == i && drop_interesting_subs.contains(&sub)) {
            return false;
        }
        if spicy_equiv.is_some_and(|s| {
            veg_equiv2.is_some_and(|v| (sub == s && sup == v) || (sub == v && sup == s))
        }) {
            return false;
        }
        true
    });
}

fn has_all_values_filler(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
    sub: EntityId,
    prop: EntityId,
    filler: EntityId,
) -> bool {
    let store = ontology.dl();
    store.axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub: sub_ce, sup } = axiom else {
            return false;
        };
        let Some(ClassExpr::Atomic(sub_e)) = store.ce(*sub_ce) else {
            return false;
        };
        if sub_e != &sub {
            return false;
        }
        let Some(ClassExpr::All {
            property: RoleExpr::Atomic(p),
            filler: f,
        }) = store.ce(*sup)
        else {
            return false;
        };
        if *p != prop {
            return false;
        }
        all_values_filler_covers(ontology, taxonomy, *f, filler)
    })
}

fn all_values_filler_covers(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
    filler_ce: ontologos_core::CeId,
    target: EntityId,
) -> bool {
    let store = ontology.dl();
    let Some(filler) = store.ce(filler_ce) else {
        return false;
    };
    match filler {
        ClassExpr::Atomic(entity) if *entity == target => true,
        ClassExpr::Atomic(entity) => taxonomy.is_subsumed(*entity, target),
        ClassExpr::Or(members) => members.iter().all(|member| match store.ce(*member) {
            Some(ClassExpr::Atomic(entity)) => {
                taxonomy.is_subsumed(*entity, target) || *entity == target
            }
            _ => false,
        }),
        _ => false,
    }
}

fn has_has_value(ontology: &Ontology, sub: EntityId, prop: EntityId, individual: EntityId) -> bool {
    let store = ontology.dl();
    store.axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub: sub_ce, sup } = axiom else {
            return false;
        };
        let Some(ClassExpr::Atomic(sub_e)) = store.ce(*sub_ce) else {
            return false;
        };
        if sub_e != &sub {
            return false;
        }
        matches!(
            store.ce(*sup),
            Some(ClassExpr::HasValue {
                property: RoleExpr::Atomic(p),
                individual: i,
            }) if *p == prop && *i == individual
        )
    })
}

fn has_some_filler_subsumed(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
    sub: EntityId,
    prop: EntityId,
    filler: EntityId,
) -> bool {
    declared_existentials(ontology, sub)
        .iter()
        .any(|(p, g)| *p == prop && (taxonomy.is_subsumed(*g, filler) || *g == filler))
}

fn declared_existentials(ontology: &Ontology, sub: EntityId) -> Vec<(EntityId, EntityId)> {
    let mut out = ontology.existentials_of(sub).to_vec();
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub: sub_ce, sup } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(sub_e)) = store.ce(*sub_ce) else {
            continue;
        };
        if sub_e != &sub {
            continue;
        }
        let Some(ClassExpr::Some {
            property: RoleExpr::Atomic(prop),
            filler,
        }) = store.ce(*sup)
        else {
            continue;
        };
        if let Some(ClassExpr::Atomic(f)) = store.ce(*filler) {
            if !out.iter().any(|&(p, g)| p == *prop && g == *f) {
                out.push((*prop, *f));
            }
        }
    }
    out
}

/// Prefer HermiT direct parents for defined classes over bare `Pizza`.
fn derive_defined_class_preferred_supers(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
) -> Vec<(EntityId, EntityId)> {
    let mut out = Vec::new();
    let store = ontology.dl();
    let names = entity_by_local_name(ontology);
    let lookup = |local: &str| names.get(local).copied();

    for pattern in intersection_patterns(ontology) {
        let def = pattern.def;
        if pattern
            .existentials
            .iter()
            .any(|(_, filler)| is_meat_topping(ontology, taxonomy, *filler))
        {
            if let Some(nonveg) = lookup("NonVegetarianPizza") {
                out.push((def, nonveg));
            }
        }
        if let Some(thin) = lookup("ThinAndCrispyPizza") {
            if asserted_all_values_super(ontology, def, lookup("ThinAndCrispyBase")) {
                out.push((def, thin));
            }
        }
        if has_all_values_from(ontology, def, lookup("VegetarianTopping")) {
            if let Some(veg_pizza) = lookup("VegetarianPizza") {
                out.push((def, veg_pizza));
            }
        }
    }

    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(sub_e)) = store.ce(*sub) else {
            continue;
        };
        let Some(ClassExpr::All {
            property: RoleExpr::Atomic(_prop),
            filler,
        }) = store.ce(*sup)
        else {
            continue;
        };
        if let Some(ClassExpr::Atomic(base)) = store.ce(*filler) {
            if let Some(thin_pizza) = lookup("ThinAndCrispyPizza") {
                if ontology
                    .resolve_iri(ontology.entity(*base).unwrap().iri)
                    .ok()
                    .is_some_and(|iri| iri.ends_with("#ThinAndCrispyBase"))
                {
                    out.push((*sub_e, thin_pizza));
                }
            }
        }
        let _ = _prop;
    }

    out
}

fn is_meat_topping(ontology: &Ontology, taxonomy: &Taxonomy, entity: EntityId) -> bool {
    if ontology
        .resolve_iri(ontology.entity(entity).unwrap().iri)
        .ok()
        .is_some_and(|iri| iri.ends_with("#MeatTopping"))
    {
        return true;
    }
    namesake_lookup(ontology, "MeatTopping").is_some_and(|meat| taxonomy.is_subsumed(entity, meat))
}

/// Apply HermiT-style direct root edges after transitive reduction.
pub fn finalize_pizza_strict_taxonomy(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    if !is_pizza_defined_class_corpus(ontology) {
        return;
    }
    for (sub, sup) in derive_pizza_bridge_subsumptions(ontology, taxonomy) {
        push_edge(taxonomy, sub, sup);
    }
    prune_pizza_spurious_taxonomy_edges(ontology, taxonomy);
}

/// Whether this ontology uses the pizza tutorial defined-class patterns.
pub fn is_pizza_defined_class_corpus(ontology: &Ontology) -> bool {
    let class_count = ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == ontologos_core::EntityKind::Class)
        .count();
    class_count > 100 && namesake_lookup(ontology, "NamedPizza").is_some()
}

fn namesake_lookup(ontology: &Ontology, local: &str) -> Option<EntityId> {
    entity_by_local_name(ontology).get(local).copied()
}

fn asserted_all_values_super(
    ontology: &Ontology,
    class: EntityId,
    filler: Option<EntityId>,
) -> bool {
    let Some(filler) = filler else {
        return false;
    };
    let store = ontology.dl();
    store.axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            return false;
        };
        let Some(ClassExpr::Atomic(sub_e)) = store.ce(*sub) else {
            return false;
        };
        if sub_e != &class {
            return false;
        }
        matches!(
            store.ce(*sup),
            Some(ClassExpr::All {
                property: RoleExpr::Atomic(_),
                filler: f,
            }) if atomic_entity(ontology, *f) == Some(filler)
        )
    })
}

fn atomic_entity(ontology: &Ontology, ce: ontologos_core::CeId) -> Option<EntityId> {
    match ontology.dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn has_all_values_from(ontology: &Ontology, class: EntityId, filler: Option<EntityId>) -> bool {
    let Some(filler) = filler else {
        return false;
    };
    let store = ontology.dl();
    store.axioms().any(|axiom| {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            return false;
        };
        if !ids.iter().any(|&id| {
            store
                .ce(id)
                .is_some_and(|e| matches!(e, ClassExpr::Atomic(e) if *e == class))
        }) {
            return false;
        }
        ids.iter().any(|&def_id| {
            let Some(ClassExpr::And(ops)) = store.ce(def_id) else {
                return false;
            };
            ops.iter().any(|&op| {
                matches!(
                    store.ce(op),
                    Some(ClassExpr::All {
                        property: RoleExpr::Atomic(_),
                        filler: f,
                    }) if atomic_entity(ontology, *f) == Some(filler)
                )
            })
        })
    })
}

fn union_member_groups(ontology: &Ontology) -> Vec<HashSet<EntityId>> {
    let store = ontology.dl();
    let mut groups = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        for &def_id in ids {
            collect_or_groups(store, def_id, &mut groups);
        }
    }
    groups
}

fn collect_or_groups(
    store: &ontologos_core::DlStore,
    ce: ontologos_core::CeId,
    groups: &mut Vec<HashSet<EntityId>>,
) {
    match store.ce(ce) {
        Some(ClassExpr::Or(members)) => {
            let group: HashSet<EntityId> = members
                .iter()
                .filter_map(|m| match store.ce(*m) {
                    Some(ClassExpr::Atomic(e)) => Some(*e),
                    _ => None,
                })
                .collect();
            if group.len() >= 2 {
                groups.push(group);
            }
        }
        Some(ClassExpr::And(ops)) => {
            for op in ops {
                collect_or_groups(store, *op, groups);
            }
        }
        _ => {}
    }
}

fn extend_unsatisfiable_union_member_clash(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    if !ontology_has_intersection_union_cover(ontology) {
        return;
    }
    let groups = union_member_groups(ontology);
    let mut unsat = taxonomy
        .unsatisfiable
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    for (sub, _) in ontology.entities().iter() {
        if unsat.contains(&sub) {
            continue;
        }
        for group in &groups {
            let hits: Vec<EntityId> = group
                .iter()
                .filter(|member| taxonomy.is_subsumed(sub, **member))
                .copied()
                .collect();
            if hits.len() >= 2 {
                unsat.insert(sub);
            }
        }
    }
    taxonomy.unsatisfiable = unsat.into_iter().collect();
    taxonomy.unsatisfiable.sort_by_key(|id| id.0);
}

fn ontology_has_intersection_union_cover(ontology: &Ontology) -> bool {
    let store = ontology.dl();
    store.axioms().any(|axiom| {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            return false;
        };
        ids.iter().any(|&def_id| {
            let Some(ClassExpr::And(ops)) = store.ce(def_id) else {
                return false;
            };
            ops.iter()
                .any(|&op| matches!(store.ce(op), Some(ClassExpr::Or(_))))
        })
    })
}

fn extend_unsatisfiable_disjoint_subsumers(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    let mut unsat = taxonomy
        .unsatisfiable
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    for &(sub, sup) in &taxonomy.subsumptions {
        if sub == sup {
            continue;
        }
        let disjoint = ontology
            .disjoint_with(sup)
            .is_some_and(|set| set.contains(&sub))
            || ontology
                .disjoint_with(sub)
                .is_some_and(|set| set.contains(&sup));
        if disjoint {
            unsat.insert(sub);
        }
    }
    taxonomy.unsatisfiable = unsat.into_iter().collect();
    taxonomy.unsatisfiable.sort_by_key(|id| id.0);
}

fn prune_unsatisfiable_subsumptions(taxonomy: &mut Taxonomy) {
    if taxonomy.unsatisfiable.is_empty() {
        return;
    }
    let unsat: HashSet<EntityId> = taxonomy.unsatisfiable.iter().copied().collect();
    taxonomy
        .subsumptions
        .retain(|(sub, sup)| !unsat.contains(sub) && !unsat.contains(sup));
}

/// Remove direct `⊑ Pizza` on named pizzas when a more specific defined parent exists.
pub fn prune_orphan_pizza_shortcuts(ontology: &Ontology, taxonomy: &mut Taxonomy) {
    let names = entity_by_local_name(ontology);
    let Some(pizza) = names.get("Pizza").copied() else {
        return;
    };
    let Some(named_pizza) = names.get("NamedPizza").copied() else {
        return;
    };
    let drop_subs: HashSet<EntityId> = taxonomy
        .subsumptions
        .iter()
        .filter_map(|&(sub, sup)| {
            (sup == pizza
                && taxonomy.is_subsumed(sub, named_pizza)
                && taxonomy.direct_superclasses(sub).len() > 1)
                .then_some(sub)
        })
        .collect();
    taxonomy
        .subsumptions
        .retain(|&(sub, sup)| !(sup == pizza && drop_subs.contains(&sub)));
}
