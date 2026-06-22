//! ALC tableau: expansion, clash detection, blocking, taxonomy extraction.

mod block;
mod cache;
mod clash;
mod expand;

use std::collections::{HashMap, HashSet, VecDeque};

use ontologos_core::{
    Axiom, CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr,
    Taxonomy,
};

use crate::clause::Clause;
use crate::dl_ontology::DlOntology;
use crate::Error;

/// Skip pairwise entailment inference when the ontology has too many named classes.
const MAX_CLASSES_FOR_ENTAILMENT_INFER: usize = 128;

/// Facts from DL saturation to seed the initial tableau state.
#[derive(Debug, Default, Clone)]
pub struct TableauSeed {
    /// Additional subsumptions `C ⊑ D` (class expression ids).
    pub subsumptions: Vec<(CeId, CeId)>,
    /// Derived `∃r.C ⊑ D` clauses.
    pub existentials: Vec<(RoleExpr, CeId, CeId)>,
    /// Saturated atomic role subsumptions `r ⊑ s`.
    pub role_subsumptions: Vec<(EntityId, EntityId)>,
}

/// ALC tableau classifier entry point.
#[derive(Debug, Default)]
pub struct AlcClassifier;

impl AlcClassifier {
    /// Construct a tableau classifier.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify using tableau.
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy, Error> {
        classify(ontology)
    }

    /// Classify with saturation-derived seed facts.
    pub fn classify_with_seed(
        &self,
        ontology: &Ontology,
        seed: &TableauSeed,
    ) -> Result<Taxonomy, Error> {
        classify_with_seed(ontology, seed)
    }
}

/// Classify via tableau on clausified ontology.
pub fn classify(ontology: &Ontology) -> Result<Taxonomy, Error> {
    classify_with_seed(ontology, &TableauSeed::default())
}

/// Classify with optional saturation seed.
pub fn classify_with_seed(ontology: &Ontology, seed: &TableauSeed) -> Result<Taxonomy, Error> {
    let dl = DlOntology::from_ontology(ontology)?;
    run_tableau(&dl, seed)
}

/// Tableau consistency test (ABox + TBox when individuals are present).
pub fn is_consistent(ontology: &Ontology) -> Result<bool, Error> {
    is_consistent_with_seed(ontology, &TableauSeed::default())
}

/// Tableau KB consistency with saturation seed facts.
pub fn is_consistent_with_seed(ontology: &Ontology, seed: &TableauSeed) -> Result<bool, Error> {
    let dl = DlOntology::from_ontology(ontology)?;
    kb_consistent(&dl, seed)
}

/// Test whether a class expression is satisfiable in the TBox (empty ABox).
pub fn is_ce_satisfiable_with_seed(dl: &DlOntology, ce: CeId, seed: &TableauSeed) -> Result<bool, Error> {
    let mut branch = Branch::new(dl, seed);
    assert_top_tbox_axioms(&mut branch, 0);
    branch.assert(0, ce);
    run_tbox_saturation(&mut branch)
}

/// Test whether a named class is satisfiable, expanding `EquivalentClasses` definitions.
pub fn is_named_class_satisfiable_with_seed(
    dl: &DlOntology,
    class: EntityId,
    seed: &TableauSeed,
) -> Result<bool, Error> {
    let ce = dl
        .core()
        .dl()
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == class => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message(format!("missing CE for class {:?}", class.0)))?;
    let test_ce = equivalent_definition_ce(dl, class).unwrap_or(ce);
    is_ce_satisfiable_with_seed(dl, test_ce, seed)
}

fn assert_top_tbox_axioms(branch: &mut Branch<'_>, world: usize) {
    let Some(top) = branch.dl.core().dl().expressions().find_map(|(id, e)| match e {
        ClassExpr::Top => Some(id),
        _ => None,
    }) else {
        return;
    };
    branch.assert(world, top);
}

/// Drive tableau expansion for class-satisfiability tests (TBox + optional seed).
fn run_tbox_saturation(branch: &mut Branch<'_>) -> Result<bool, Error> {
    expand::materialize_existential_successors(branch);
    if branch.clash {
        return Ok(false);
    }
    expand::saturate_composed_edges(branch);
    for (from, _, to) in branch.edges.clone() {
        expand::apply_universal_on_edge(branch, from, to);
    }
    clash::detect_clash(branch);
    if branch.clash {
        return Ok(false);
    }
    let ok = branch.expand()?;
    expand::saturate_composed_edges(branch);
    for world in 0..branch.worlds.len() {
        expand::recheck_cardinality_on_world(branch, world);
        if branch.clash {
            return Ok(false);
        }
    }
    Ok(ok && !branch.clash)
}

fn kb_consistent(dl: &DlOntology, seed: &TableauSeed) -> Result<bool, Error> {
    let mut branch = Branch::new(dl, seed);
    let mut worlds: HashMap<EntityId, usize> = HashMap::new();

    for axiom in dl.core().dl().axioms() {
        apply_kb_axiom(&mut branch, &mut worlds, dl, axiom, false);
    }
    for (_, axiom) in dl.core().axioms().iter() {
        apply_graph_axiom(&mut branch, &mut worlds, dl, axiom, false);
    }

    assert_thing_on_named_individuals(&mut branch, &worlds, dl);

    apply_reflexive_loops(&mut branch, &worlds, &reflexive_object_properties(dl));

    expand::materialize_existential_successors(&mut branch);
    if branch.clash {
        return Ok(false);
    }

    if worlds.is_empty() {
        let top = dl
            .core()
            .dl()
            .expressions()
            .find_map(|(id, e)| match e {
                ClassExpr::Top => Some(id),
                _ => None,
            })
            .ok_or_else(|| Error::Message("missing ⊤".into()))?;
        branch.assert(0, top);
    }

    apply_has_key_merges(&mut branch, &mut worlds, dl);

    apply_same_individual_axioms(&mut branch, &mut worlds, dl);
    apply_different_individual_axioms(&mut branch, &mut worlds, dl);
    if branch.clash {
        return Ok(false);
    }

    expand::saturate_composed_edges(&mut branch);
    expand::reapply_universal_restrictions(&mut branch);
    if branch.clash {
        return Ok(false);
    }
    for (from, _, to) in branch.edges.clone() {
        expand::apply_universal_on_edge(&mut branch, from, to);
    }

    if negative_object_property_assertions_clash(&branch) {
        return Ok(false);
    }

    expand::materialize_top_object_property_loops(&mut branch, &worlds);
    expand::materialize_has_self_from_loops(&mut branch);
    clash::detect_clash(&mut branch);
    if branch.clash {
        return Ok(false);
    }

    let ok = branch.expand()?;
    expand::saturate_composed_edges(&mut branch);
    expand::reapply_universal_restrictions(&mut branch);
    if branch.clash {
        return Ok(false);
    }
    if negative_object_property_assertions_clash(&branch) {
        return Ok(false);
    }
    for world in 0..branch.worlds.len() {
        expand::recheck_cardinality_on_world(&mut branch, world);
        if branch.clash {
            return Ok(false);
        }
    }
    // Drive B ⊑ ∃R.B unraveling for nominal cardinality clashes (NI-rule pattern).
    if !branch.clash && !branch.named_worlds.is_empty() {
        for _ in 0..256 {
            let before_edges = branch.edges.len();
            let before_labels: usize = branch.worlds.iter().map(|w| w.labels.len()).sum();
            expand::materialize_existential_successors(&mut branch);
            expand::saturate_composed_edges(&mut branch);
            expand::reapply_universal_restrictions(&mut branch);
            expand::drive_atomic_existential_subsumptions(&mut branch);
            expand::propagate_structural_existential_subsumptions(&mut branch);
            clash::check_existential_bottom_subsumptions(&mut branch);
            if branch.clash {
                return Ok(false);
            }
            for world in 0..branch.worlds.len() {
                expand::recheck_cardinality_on_world(&mut branch, world);
                if branch.clash {
                    return Ok(false);
                }
            }
            let after_labels: usize = branch.worlds.iter().map(|w| w.labels.len()).sum();
            if branch.clash
                || (branch.edges.len() == before_edges && after_labels == before_labels)
            {
                break;
            }
        }
    }
    clash::check_existential_bottom_subsumptions(&mut branch);
    if branch.clash {
        return Ok(false);
    }
    Ok(ok && !branch.clash)
}

pub(crate) fn assert_thing_axioms_on_world(branch: &mut Branch<'_>, world: usize) {
    if let Some(top) = branch.top_ce {
        branch.assert(world, top);
    }
    for &sup in &branch.thing_restrictions.clone() {
        branch.assert(world, sup);
    }
}

fn assert_thing_on_named_individuals(
    branch: &mut Branch<'_>,
    worlds: &HashMap<EntityId, usize>,
    _dl: &DlOntology,
) {
    for &world in worlds.values() {
        assert_thing_axioms_on_world(branch, world);
    }
}

fn reflexive_object_properties(dl: &DlOntology) -> Vec<EntityId> {
    dl.core()
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| {
            if let Axiom::ReflexiveObjectProperty(property) = axiom {
                Some(*property)
            } else {
                None
            }
        })
        .collect()
}

fn apply_reflexive_loops(
    branch: &mut Branch<'_>,
    worlds: &HashMap<EntityId, usize>,
    roles: &[EntityId],
) {
    if roles.is_empty() {
        return;
    }
    for &world_idx in worlds.values() {
        for &role in roles {
            expand::add_role_edge(branch, world_idx, RoleExpr::Atomic(role), world_idx);
        }
    }
}

fn negative_object_property_assertions_clash(branch: &Branch<'_>) -> bool {
    for axiom in branch.dl.core().dl().axioms() {
        let DlAxiom::NegativeObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        if negative_role_assertion_entailed(branch, *subject, RoleExpr::Atomic(*property), *object)
        {
            return true;
        }
    }
    false
}

fn negative_role_assertion_entailed(
    branch: &Branch<'_>,
    subject: EntityId,
    property: RoleExpr,
    object: EntityId,
) -> bool {
    let Some(&from) = branch.named_worlds.get(&subject) else {
        return false;
    };
    let Some(&to) = branch.named_worlds.get(&object) else {
        return false;
    };
    let inv = expand::inverse_role(&property);
    branch.edges.iter().any(|(edge_from, role, edge_to)| {
        (*edge_from == from && *edge_to == to && expand::role_subsumes(branch, &property, role))
            || (*edge_from == to && *edge_to == from && expand::role_subsumes(branch, &inv, role))
    })
}

fn apply_kb_axiom(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    _dl: &DlOntology,
    axiom: &DlAxiom,
    equalities_only: bool,
) {
    match axiom {
        DlAxiom::ClassAssertion { individual, class } if !equalities_only => {
            let w = ensure_individual_world(branch, worlds, *individual);
            branch.assert(w, *class);
        }
        DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } if !equalities_only => {
            add_property_edge(branch, worlds, *subject, property.clone(), *object);
        }
        DlAxiom::SameIndividual(ids) if equalities_only && ids.len() >= 2 => {
            merge_individuals(branch, worlds, ids);
        }
        DlAxiom::DifferentIndividuals(ids) if equalities_only && ids.len() >= 2 => {
            mark_different_individuals(branch, worlds, ids);
        }
        _ => {}
    }
}

fn apply_graph_axiom(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    dl: &DlOntology,
    axiom: &Axiom,
    equalities_only: bool,
) {
    match axiom {
        Axiom::ClassAssertion { individual, class } if !equalities_only => {
            if let Some(ce) = atomic_ce_id(dl, *class) {
                let w = ensure_individual_world(branch, worlds, *individual);
                branch.assert(w, ce);
            }
        }
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } if !equalities_only => {
            add_property_edge(
                branch,
                worlds,
                *subject,
                RoleExpr::Atomic(*property),
                *object,
            );
        }
        Axiom::SameIndividual(ids) if equalities_only && ids.len() >= 2 => {
            merge_individuals(branch, worlds, ids);
        }
        Axiom::DifferentIndividuals(ids) if equalities_only && ids.len() >= 2 => {
            mark_different_individuals(branch, worlds, ids);
        }
        _ => {}
    }
}

fn add_property_edge(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    subject: EntityId,
    property: RoleExpr,
    object: EntityId,
) {
    let from = ensure_individual_world(branch, worlds, subject);
    let to = ensure_individual_world(branch, worlds, object);
    expand::add_role_edge(branch, from, property, to);
    expand::saturate_composed_edges(branch);
}

fn apply_same_individual_axioms(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    dl: &DlOntology,
) {
    for axiom in dl.core().dl().axioms() {
        if let DlAxiom::SameIndividual(ids) = axiom {
            merge_individuals(branch, worlds, ids);
        }
    }
    for (_, axiom) in dl.core().axioms().iter() {
        if let Axiom::SameIndividual(ids) = axiom {
            merge_individuals(branch, worlds, ids);
        }
    }
}

fn apply_different_individual_axioms(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    dl: &DlOntology,
) {
    for axiom in dl.core().dl().axioms() {
        if let DlAxiom::DifferentIndividuals(ids) = axiom {
            mark_different_individuals(branch, worlds, ids);
        }
    }
    for (_, axiom) in dl.core().axioms().iter() {
        if let Axiom::DifferentIndividuals(ids) = axiom {
            mark_different_individuals(branch, worlds, ids);
        }
    }
}

fn different_pair(left: EntityId, right: EntityId) -> (EntityId, EntityId) {
    if left.0 <= right.0 {
        (left, right)
    } else {
        (right, left)
    }
}

fn merge_individuals(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    ids: &[EntityId],
) {
    if ids.len() < 2 {
        return;
    }
    for i in 0..ids.len() {
        for &other in &ids[i + 1..] {
            if branch
                .different_pairs
                .contains(&different_pair(ids[i], other))
            {
                branch.clash = true;
                return;
            }
        }
    }
    let first = ensure_individual_world(branch, worlds, ids[0]);
    for &other in &ids[1..] {
        let w = ensure_individual_world(branch, worlds, other);
        if first != w {
            branch.merge_worlds(first, w);
            worlds.insert(other, first);
            branch.named_worlds.insert(other, first);
        }
    }
}

fn mark_different_individuals(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    ids: &[EntityId],
) {
    if ids.len() < 2 {
        return;
    }
    let w0 = ensure_individual_world(branch, worlds, ids[0]);
    for &other in &ids[1..] {
        branch.different_pairs.insert(different_pair(ids[0], other));
        let w1 = ensure_individual_world(branch, worlds, other);
        if w0 == w1 {
            branch.clash = true;
        }
    }
}

fn atomic_ce_id(dl: &DlOntology, entity: EntityId) -> Option<CeId> {
    dl.core().dl().expressions().find_map(|(id, e)| match e {
        ClassExpr::Atomic(a) if *a == entity => Some(id),
        _ => None,
    })
}

type KeyTuple = (Vec<Option<EntityId>>, Vec<Option<DeId>>);

fn apply_has_key_merges(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    dl: &DlOntology,
) {
    if branch.has_keys.is_empty() {
        return;
    }
    let data_values = collect_data_values(dl);
    for (key_class, object_properties, data_properties) in branch.has_keys.clone() {
        let mut groups: Vec<(KeyTuple, Vec<EntityId>)> = Vec::new();
        for (&individual, &world) in worlds.iter() {
            if branch.clash || !individual_in_key_class(branch, dl, world, key_class) {
                continue;
            }
            if data_properties
                .iter()
                .any(|prop| !data_values.contains_key(&(individual, *prop)))
            {
                continue;
            }
            let key = individual_key_tuple(
                branch,
                worlds,
                individual,
                world,
                &object_properties,
                &data_properties,
                &data_values,
            );
            if let Some(group) = groups
                .iter_mut()
                .find(|(existing, _)| key_tuples_equal(dl, existing, &key))
            {
                group.1.push(individual);
            } else {
                groups.push((key, vec![individual]));
            }
        }
        for (_, members) in groups {
            if members.len() >= 2 {
                merge_individuals(branch, worlds, &members);
            }
        }
    }
}

fn collect_data_values(dl: &DlOntology) -> HashMap<(EntityId, EntityId), DeId> {
    let mut out = HashMap::new();
    for axiom in dl.core().dl().axioms() {
        if let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        {
            out.insert((*subject, *property), *value);
        }
    }
    out
}

fn individual_key_tuple(
    branch: &Branch<'_>,
    worlds: &HashMap<EntityId, usize>,
    individual: EntityId,
    world: usize,
    object_properties: &[EntityId],
    data_properties: &[EntityId],
    data_values: &HashMap<(EntityId, EntityId), DeId>,
) -> KeyTuple {
    let mut objects = Vec::with_capacity(object_properties.len());
    for &prop in object_properties {
        let target = branch.edges.iter().find_map(|(from, role, to)| {
            if *from != world {
                return None;
            }
            match role {
                RoleExpr::Atomic(p) if *p == prop => Some(*to),
                _ => None,
            }
        });
        let object = target.and_then(|to_world| {
            worlds
                .iter()
                .find(|(_, &w)| w == to_world)
                .map(|(&id, _)| id)
        });
        objects.push(object);
    }
    let mut values = Vec::with_capacity(data_properties.len());
    for &prop in data_properties {
        values.push(data_values.get(&(individual, prop)).copied());
    }
    (objects, values)
}

fn individual_in_key_class(
    branch: &Branch<'_>,
    dl: &DlOntology,
    world: usize,
    key_class: CeId,
) -> bool {
    let labels = &branch.worlds[world].labels;
    if labels.contains(&key_class) {
        return true;
    }
    for &label in labels {
        if label_subsumes_key(branch, dl, label, key_class) {
            return true;
        }
    }
    if let Some(ClassExpr::Not(inner)) = dl.core().dl().ce(key_class).cloned() {
        for &label in labels {
            if classes_disjoint(branch, label, inner) {
                return true;
            }
        }
    }
    false
}

fn label_subsumes_key(branch: &Branch<'_>, dl: &DlOntology, sub: CeId, key: CeId) -> bool {
    if sub == key {
        return true;
    }
    let mut work = vec![sub];
    let mut seen = HashSet::from([sub]);
    while let Some(cur) = work.pop() {
        for &(left, right) in &branch.tbox_subsumptions {
            if left == cur && !seen.contains(&right) {
                if right == key {
                    return true;
                }
                seen.insert(right);
                work.push(right);
            }
        }
        if let (Some(ClassExpr::Atomic(a)), Some(ClassExpr::Atomic(b))) =
            (dl.core().dl().ce(cur), dl.core().dl().ce(key))
        {
            if a == b {
                return true;
            }
        }
    }
    false
}

fn classes_disjoint(branch: &Branch<'_>, left: CeId, right: CeId) -> bool {
    branch
        .disjoint
        .iter()
        .any(|&(a, b)| (a == left && b == right) || (a == right && b == left))
}

fn data_values_equal(dl: &DlOntology, left: DeId, right: DeId) -> bool {
    if left == right {
        return true;
    }
    match (dl.core().dl().de(left), dl.core().dl().de(right)) {
        (
            Some(DataExpr::Literal { lexical: la, .. }),
            Some(DataExpr::Literal { lexical: lb, .. }),
        ) => la == lb,
        _ => false,
    }
}

fn key_tuples_equal(dl: &DlOntology, a: &KeyTuple, b: &KeyTuple) -> bool {
    a.0 == b.0
        && a.1.len() == b.1.len()
        && a.1.iter().zip(&b.1).all(|(x, y)| match (x, y) {
            (Some(left), Some(right)) => data_values_equal(dl, *left, *right),
            (None, None) => true,
            _ => false,
        })
}

fn ensure_individual_world(
    branch: &mut Branch<'_>,
    worlds: &mut HashMap<EntityId, usize>,
    id: EntityId,
) -> usize {
    if let Some(&w) = worlds.get(&id) {
        return w;
    }
    let w = branch.worlds.len();
    branch.worlds.push(World::default());
    assert_thing_axioms_on_world(branch, w);
    let nom = branch
        .dl
        .core()
        .dl()
        .expressions()
        .find_map(|(ce, e)| match e {
            ClassExpr::OneOf(v) if v == &[id] => Some(ce),
            _ => None,
        });
    if let Some(ce) = nom {
        branch.assert(w, ce);
    }
    worlds.insert(id, w);
    branch.named_worlds.insert(id, w);
    w
}

fn run_tableau(dl: &DlOntology, seed: &TableauSeed) -> Result<Taxonomy, Error> {
    let mut subsumptions = Vec::new();
    for clause in dl.clauses().clauses() {
        if let Clause::Subsumption { sub, sup } = clause {
            if let (Some(a), Some(b)) = (atomic_entity(dl, *sub), atomic_entity(dl, *sup)) {
                subsumptions.push((a, b));
            }
        }
    }
    for &(sub, sup) in &seed.subsumptions {
        if let (Some(a), Some(b)) = (atomic_entity(dl, sub), atomic_entity(dl, sup)) {
            subsumptions.push((a, b));
        }
    }

    let classes: Vec<EntityId> = dl
        .core()
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    let mut unsatisfiable = Vec::new();
    let class_count = classes.len();
    for class in classes {
        if !is_satisfiable(dl, class, seed)? {
            unsatisfiable.push(class);
        }
    }

    if class_count <= MAX_CLASSES_FOR_ENTAILMENT_INFER {
        subsumptions.extend(infer_named_subsumptions(dl, seed)?);
    }
    subsumptions.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    subsumptions.dedup();

    Ok(Taxonomy {
        subsumptions,
        equivalences: Vec::new(),
        unsatisfiable,
    })
}

fn is_satisfiable(dl: &DlOntology, class: EntityId, seed: &TableauSeed) -> Result<bool, Error> {
    is_named_class_satisfiable_with_seed(dl, class, seed)
}

fn infer_named_subsumptions(
    dl: &DlOntology,
    seed: &TableauSeed,
) -> Result<Vec<(EntityId, EntityId)>, Error> {
    let classes: Vec<EntityId> = dl
        .core()
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();
    let mut out = Vec::new();
    for &sub in &classes {
        for &sup in &classes {
            if sub != sup && entails(dl, sub, sup, seed)? {
                out.push((sub, sup));
            }
        }
    }
    Ok(out)
}

fn entails(
    dl: &DlOntology,
    sub: EntityId,
    sup: EntityId,
    seed: &TableauSeed,
) -> Result<bool, Error> {
    let mut working = dl.clone();
    let store = working.core().dl();
    let sub_ce = store
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == sub => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message("missing sub CE".into()))?;
    let sup_ce = store
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == sup => Some(id),
            _ => None,
        })
        .ok_or_else(|| Error::Message("missing sup CE".into()))?;
    let sup_def = equivalent_definition_ce(&working, sup).unwrap_or(sup_ce);
    let neg_target = match working.core().dl().ce(sup_def) {
        Some(
            ClassExpr::And(_)
            | ClassExpr::MinCardinality { .. }
            | ClassExpr::MaxCardinality { .. }
            | ClassExpr::ExactCardinality { .. },
        ) => sup_def,
        _ => sup_ce,
    };
    let neg_sup = crate::normalize::negate_ce(working.core_mut(), neg_target);
    let mut branch = Branch::new(&working, seed);
    branch.assert(0, sub_ce);
    branch.assert(0, neg_sup);
    Ok(!branch.expand()?)
}

fn equivalent_definition_ce(dl: &DlOntology, class: EntityId) -> Option<CeId> {
    let store = dl.core().dl();
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

/// Resolve atomic class labels to their `EquivalentClasses` definition when present.
pub(crate) fn effective_class_expression(dl: &DlOntology, ce: CeId) -> CeId {
    let store = dl.core().dl();
    match store.ce(ce) {
        Some(ClassExpr::Atomic(entity)) => equivalent_definition_ce(dl, *entity).unwrap_or(ce),
        _ => ce,
    }
}

fn saturate_role_hierarchy(role_hierarchy: &mut HashMap<EntityId, HashSet<EntityId>>) {
    let mut changed = true;
    while changed {
        changed = false;
        let pairs: Vec<(EntityId, EntityId)> = role_hierarchy
            .iter()
            .flat_map(|(&a, ss)| ss.iter().map(move |&b| (a, b)))
            .collect();
        for (a, b) in pairs {
            if let Some(bb) = role_hierarchy.get(&b).cloned() {
                for c in bb {
                    if role_hierarchy.entry(a).or_default().insert(c) {
                        changed = true;
                    }
                }
            }
        }
    }
}

fn atomic_entity(dl: &DlOntology, ce: CeId) -> Option<EntityId> {
    match dl.core().dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct World {
    labels: HashSet<CeId>,
    negated: HashSet<CeId>,
    queue: VecDeque<CeId>,
    blocked: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Branch<'a> {
    pub(crate) dl: &'a DlOntology,
    pub(crate) worlds: Vec<World>,
    pub(crate) edges: Vec<(usize, RoleExpr, usize)>,
    pub(crate) clash: bool,
    pub(crate) disjoint: Vec<(CeId, CeId)>,
    pub(crate) role_disjoint: Vec<(EntityId, EntityId)>,
    pub(crate) existentials: Vec<(RoleExpr, CeId, CeId)>,
    pub(crate) universals: Vec<(CeId, RoleExpr, CeId)>,
    pub(crate) tbox_subsumptions: Vec<(CeId, CeId)>,
    pub(crate) role_hierarchy: HashMap<EntityId, HashSet<EntityId>>,
    pub(crate) role_inverses: HashMap<EntityId, EntityId>,
    pub(crate) symmetric_roles: Vec<RoleExpr>,
    pub(crate) role_chains: Vec<(Vec<RoleExpr>, RoleExpr)>,
    pub(crate) has_keys: Vec<(CeId, Vec<EntityId>, Vec<EntityId>)>,
    pub(crate) inverse_functional: HashSet<EntityId>,
    pub(crate) top_ce: Option<CeId>,
    pub(crate) thing_restrictions: Vec<CeId>,
    pub(crate) named_worlds: HashMap<EntityId, usize>,
    pub(crate) different_pairs: HashSet<(EntityId, EntityId)>,
    pub(crate) cache: cache::UnsatCache,
    pub(crate) expansions: u32,
}

impl<'a> Branch<'a> {
    fn new(dl: &'a DlOntology, seed: &TableauSeed) -> Self {
        let mut disjoint = Vec::new();
        let mut role_disjoint = Vec::new();
        let mut existentials = seed.existentials.clone();
        let mut universals = Vec::new();
        let mut tbox_subsumptions = seed.subsumptions.clone();
        let mut role_hierarchy: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();

        let mut role_chains: Vec<(Vec<RoleExpr>, RoleExpr)> = Vec::new();
        let mut has_keys: Vec<(CeId, Vec<EntityId>, Vec<EntityId>)> = Vec::new();
        let mut role_inverses: HashMap<EntityId, EntityId> = HashMap::new();
        let mut symmetric_roles: Vec<RoleExpr> = Vec::new();
        let mut inverse_functional: HashSet<EntityId> = HashSet::new();
        for (_, axiom) in dl.core().axioms().iter() {
            if let Axiom::InverseObjectProperties { left, right } = axiom {
                role_inverses.insert(*left, *right);
                role_inverses.insert(*right, *left);
            }
            if let Axiom::SymmetricObjectProperty(prop) = axiom {
                symmetric_roles.push(RoleExpr::Atomic(*prop));
            }
            if let Axiom::InverseFunctionalObjectProperty(prop) = axiom {
                inverse_functional.insert(*prop);
            }
        }
        for axiom in dl.core().dl().axioms() {
            match axiom {
                DlAxiom::SymmetricObjectProperty(role) => {
                    symmetric_roles.push(role.clone());
                }
                DlAxiom::InverseFunctionalObjectProperty(prop) => {
                    inverse_functional.insert(*prop);
                }
                _ => {}
            }
        }

        for clause in dl.clauses().clauses() {
            match clause {
                Clause::Subsumption { sub, sup } => {
                    tbox_subsumptions.push((*sub, *sup));
                }
                Clause::Disjoint { left, right } => disjoint.push((*left, *right)),
                Clause::RoleDisjoint { left, right } => role_disjoint.push((*left, *right)),
                Clause::Existential {
                    property,
                    filler,
                    sup,
                } => existentials.push((property.clone(), *filler, *sup)),
                Clause::Universal {
                    sub,
                    property,
                    filler,
                } => universals.push((*sub, property.clone(), *filler)),
                Clause::RoleSubsumption { sub, sup } => {
                    role_hierarchy.entry(*sub).or_default().insert(*sup);
                }
                Clause::RoleChain { chain, sup } => {
                    role_chains.push((chain.clone(), sup.clone()));
                }
                Clause::HasKey {
                    class,
                    object_properties,
                    data_properties,
                } => has_keys.push((*class, object_properties.clone(), data_properties.clone())),
                Clause::NominalSubsumption { sub, individual } => {
                    if let Some(one_of) = dl.core().dl().expressions().find_map(|(id, e)| match e {
                        ClassExpr::OneOf(v) if v == &[*individual] => Some(id),
                        _ => None,
                    }) {
                        tbox_subsumptions.push((*sub, one_of));
                    }
                }
                _ => {}
            }
        }

        for &(sub, sup) in &seed.role_subsumptions {
            role_hierarchy.entry(sub).or_default().insert(sup);
        }

        saturate_role_hierarchy(&mut role_hierarchy);

        let top_ce = dl.core().dl().expressions().find_map(|(id, e)| match e {
            ClassExpr::Top => Some(id),
            _ => None,
        });
        let thing_restrictions: Vec<CeId> = dl
            .core()
            .dl()
            .axioms()
            .filter_map(|axiom| {
                let DlAxiom::SubClassOf { sub, sup } = axiom else {
                    return None;
                };
                match dl.core().dl().ce(*sub) {
                    Some(ClassExpr::Top) => Some(*sup),
                    Some(ClassExpr::Atomic(id)) => dl
                        .core()
                        .entity(*id)
                        .ok()
                        .and_then(|record| dl.core().resolve_iri(record.iri).ok())
                        .is_some_and(|iri| {
                            iri == "http://www.w3.org/2002/07/owl#Thing"
                                || iri.ends_with("#Thing")
                                || iri.ends_with("/Thing")
                        })
                        .then_some(*sup),
                    _ => None,
                }
            })
            .collect();

        Self {
            dl,
            worlds: vec![World::default()],
            edges: Vec::new(),
            clash: false,
            disjoint,
            role_disjoint,
            existentials,
            universals,
            tbox_subsumptions,
            role_hierarchy,
            role_inverses,
            symmetric_roles,
            role_chains,
            has_keys,
            inverse_functional,
            top_ce,
            thing_restrictions,
            named_worlds: HashMap::new(),
            different_pairs: HashSet::new(),
            cache: cache::UnsatCache::new(),
            expansions: 0,
        }
    }

    fn merge_worlds(&mut self, keep: usize, drop: usize) {
        if keep == drop || keep >= self.worlds.len() || drop >= self.worlds.len() {
            return;
        }
        if expand::role_disjoint_merge_blocked(self, keep, drop) {
            self.clash = true;
            return;
        }
        let labels = self.worlds[drop].labels.clone();
        let negated = self.worlds[drop].negated.clone();
        for ce in labels {
            clash::assert_label(self, keep, ce);
        }
        for ce in negated {
            clash::assert_negation(self, keep, ce);
        }
        for (from, role, to) in self.edges.clone() {
            if from == drop {
                self.edges
                    .push((keep, role, if to == drop { keep } else { to }));
            } else if to == drop {
                self.edges.push((from, role, keep));
            }
        }
        self.edges
            .retain(|(from, _, to)| *from != drop && *to != drop);
        self.worlds[drop] = World::default();
        for world_idx in self.named_worlds.values_mut() {
            if *world_idx == drop {
                *world_idx = keep;
            }
        }
        clash::check_existential_bottom_subsumptions(self);
    }

    fn assert(&mut self, world: usize, ce: CeId) {
        if let Some(ClassExpr::Not(inner)) = self.dl.core().dl().ce(ce).cloned() {
            clash::assert_negation(self, world, inner);
        } else {
            clash::assert_label(self, world, ce);
        }
    }

    fn expand(&mut self) -> Result<bool, Error> {
        loop {
            if self.clash {
                return Ok(false);
            }

            let pending = self.next_pending();
            let Some((world, ce)) = pending else {
                return Ok(true);
            };

            if block::is_blocked(self, world) {
                if block::is_budget_exhausted(self) {
                    return Err(Error::ResourceLimit(block::MAX_EXPANSIONS));
                }
                block::mark_blocked(self, world);
                continue;
            }

            if self.cache.is_unsat(&self.worlds[world].labels) {
                return Ok(false);
            }

            expand::process(self, world, ce)?;
            if negative_object_property_assertions_clash(self) {
                return Ok(false);
            }
        }
    }

    fn next_pending(&mut self) -> Option<(usize, CeId)> {
        for (idx, world) in self.worlds.iter_mut().enumerate() {
            if let Some(ce) = world.queue.pop_front() {
                return Some((idx, ce));
            }
        }
        None
    }
}
