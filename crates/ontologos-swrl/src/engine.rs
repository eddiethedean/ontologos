//! DLSafe SWRL forward chaining over asserted and inferred facts.

use std::collections::{HashMap, HashSet};

use ontologos_core::{
    Axiom, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, Ontology, SwrlAtom, SwrlDArg, SwrlIArg,
    SwrlRule,
};
use ontologos_dl::{LiteralIndex, LiteralValue};

use crate::SwrlReport;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DataValue {
    lexical: String,
    datatype: Option<EntityId>,
}

#[derive(Debug, Clone, Default)]
struct RuleBinding {
    individuals: HashMap<String, EntityId>,
    data: HashMap<String, DataValue>,
}

/// Materialize SWRL rule consequences as core axioms until saturation.
pub fn materialize_swrl_rules(ontology: &mut Ontology) -> ontologos_core::Result<SwrlReport> {
    let rules: Vec<SwrlRule> = ontology.swrl_rules().to_vec();
    let mut report = SwrlReport {
        rules_found: rules.len(),
        ..SwrlReport::default()
    };
    if rules.is_empty() {
        return Ok(report);
    }

    let mut changed = true;
    while changed {
        changed = false;
        for rule in &rules {
            for binding in match_rule_body(ontology, &rule.body) {
                if apply_rule_head(ontology, &rule.head, &binding)? {
                    report.inferences_added += 1;
                    changed = true;
                }
            }
        }
    }
    Ok(report)
}

fn match_rule_body(ontology: &Ontology, body: &[SwrlAtom]) -> Vec<RuleBinding> {
    let mut ordered: Vec<&SwrlAtom> = body.iter().collect();
    ordered.sort_by_key(|atom| atom_match_priority(atom));
    let mut bindings = vec![RuleBinding::default()];
    for atom in ordered {
        let mut next = Vec::new();
        for binding in bindings {
            next.extend(extend_binding(ontology, atom, &binding));
        }
        bindings = next;
        if bindings.is_empty() {
            return bindings;
        }
    }
    bindings
}

fn atom_match_priority(atom: &SwrlAtom) -> u8 {
    match atom {
        SwrlAtom::Class { .. }
        | SwrlAtom::ObjectProperty { .. }
        | SwrlAtom::DataProperty { .. } => 0,
        SwrlAtom::DataRange { .. } => 1,
        SwrlAtom::SameIndividual(..) => 2,
        SwrlAtom::DifferentIndividuals(..) => 3,
    }
}

fn extend_binding(ontology: &Ontology, atom: &SwrlAtom, binding: &RuleBinding) -> Vec<RuleBinding> {
    match atom {
        SwrlAtom::Class { class, arg } => extend_class(ontology, *class, arg, binding),
        SwrlAtom::ObjectProperty {
            property,
            subject,
            object,
        } => extend_object_property(ontology, *property, subject, object, binding),
        SwrlAtom::DataProperty {
            property,
            subject,
            value,
        } => extend_data_property(ontology, *property, subject, value, binding),
        SwrlAtom::DataRange { range, arg } => extend_data_range(ontology, *range, arg, binding),
        SwrlAtom::SameIndividual(a, b) => unify_same(ontology, a, b, binding),
        SwrlAtom::DifferentIndividuals(a, b) => unify_different(ontology, a, b, binding),
    }
}

fn extend_class(
    ontology: &Ontology,
    class: EntityId,
    arg: &SwrlIArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    match arg {
        SwrlIArg::Individual(ind) => {
            if is_individual_typed(ontology, *ind, class) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
        SwrlIArg::Variable(var) => {
            if let Some(&ind) = binding.individuals.get(var) {
                return extend_class(ontology, class, &SwrlIArg::Individual(ind), binding);
            }
            individuals_of_class(ontology, class)
                .into_iter()
                .map(|ind| {
                    let mut b = binding.clone();
                    b.individuals.insert(var.clone(), ind);
                    b
                })
                .collect()
        }
    }
}

fn extend_object_property(
    ontology: &Ontology,
    property: EntityId,
    subject: &SwrlIArg,
    object: &SwrlIArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    let assertions = ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::ObjectPropertyAssertion {
                subject: s,
                property: p,
                object: o,
            } if *p == property => Some((*s, *o)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut out = Vec::new();
    for (sub, obj) in assertions {
        for b in unify_args(ontology, subject, &SwrlIArg::Individual(sub), binding) {
            out.extend(unify_args(ontology, object, &SwrlIArg::Individual(obj), &b));
        }
    }
    out
}

fn extend_data_range(
    ontology: &Ontology,
    range: DeId,
    arg: &SwrlDArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    match arg {
        SwrlDArg::Literal { lexical, datatype } => {
            let fact = DataValue {
                lexical: lexical.clone(),
                datatype: *datatype,
            };
            if data_value_satisfies_range(ontology, &fact, range) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
        SwrlDArg::Variable(var) => {
            let Some(fact) = binding.data.get(var) else {
                return vec![];
            };
            if data_value_satisfies_range(ontology, fact, range) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
    }
}

fn data_value_satisfies_range(ontology: &Ontology, fact: &DataValue, range: DeId) -> bool {
    let Some(datatype) = fact.datatype else {
        return false;
    };
    let idx = LiteralIndex::from_store(ontology.dl());
    let lit = LiteralValue {
        lexical: fact.lexical.clone(),
        datatype,
    };
    idx.satisfies_with_ontology(&lit, ontology, range)
}

fn extend_data_property(
    ontology: &Ontology,
    property: EntityId,
    subject: &SwrlIArg,
    value: &SwrlDArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    let mut out = Vec::new();
    for (sub, prop, fact) in data_property_facts(ontology) {
        if prop != property {
            continue;
        }
        for b in unify_args(ontology, subject, &SwrlIArg::Individual(sub), binding) {
            out.extend(unify_darg(&fact, value, &b));
        }
    }
    out
}

fn unify_darg(fact: &DataValue, arg: &SwrlDArg, binding: &RuleBinding) -> Vec<RuleBinding> {
    match arg {
        SwrlDArg::Literal { lexical, datatype } => {
            if lexical == &fact.lexical && datatype.is_none_or(|dt| fact.datatype == Some(dt)) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
        SwrlDArg::Variable(var) => unify_var_data(var, fact, binding),
    }
}

fn unify_var_data(var: &str, fact: &DataValue, binding: &RuleBinding) -> Vec<RuleBinding> {
    if let Some(bound) = binding.data.get(var) {
        if bound == fact {
            vec![binding.clone()]
        } else {
            vec![]
        }
    } else {
        let mut b = binding.clone();
        b.data.insert(var.to_owned(), fact.clone());
        vec![b]
    }
}

fn unify_same(
    ontology: &Ontology,
    left: &SwrlIArg,
    right: &SwrlIArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    let ind = unify_args(ontology, left, right, binding);
    if !ind.is_empty() {
        return ind;
    }
    unify_data_same(left, right, binding)
}

fn unify_data_same(left: &SwrlIArg, right: &SwrlIArg, binding: &RuleBinding) -> Vec<RuleBinding> {
    let (SwrlIArg::Variable(a), SwrlIArg::Variable(b)) = (left, right) else {
        return vec![];
    };
    if a == b {
        return vec![binding.clone()];
    }
    if let (Some(da), Some(db)) = (binding.data.get(a), binding.data.get(b)) {
        return if da == db {
            vec![binding.clone()]
        } else {
            vec![]
        };
    }
    if let Some(db) = binding.data.get(b).cloned() {
        return unify_var_data(a, &db, binding);
    }
    if let Some(da) = binding.data.get(a).cloned() {
        return unify_var_data(b, &da, binding);
    }
    vec![binding.clone()]
}

fn data_property_facts(ontology: &Ontology) -> Vec<(EntityId, EntityId, DataValue)> {
    let mut facts = Vec::new();
    for axiom in ontology.dl().axioms() {
        match axiom {
            DlAxiom::DataPropertyAssertion {
                subject,
                property,
                value,
            } => {
                if let Some(dv) = data_value_from_de(ontology, *value) {
                    facts.push((*subject, *property, dv));
                }
            }
            DlAxiom::ClassAssertion { individual, class } => {
                let Some(ce) = ontology.dl().ce(*class) else {
                    continue;
                };
                match ce {
                    ClassExpr::DataHasValue { property, value } => {
                        if let Some(dv) = data_value_from_de(ontology, *value) {
                            facts.push((*individual, *property, dv));
                        }
                    }
                    ClassExpr::DataSome { property, range } => {
                        if let Some(dv) = point_literal_from_range(ontology, *range) {
                            facts.push((*individual, *property, dv));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    facts
}

fn data_value_from_de(ontology: &Ontology, de: DeId) -> Option<DataValue> {
    match ontology.dl().de(de)? {
        DataExpr::Literal { lexical, datatype } => Some(DataValue {
            lexical: lexical.clone(),
            datatype: Some(*datatype),
        }),
        _ => point_literal_from_range(ontology, de),
    }
}

fn point_literal_from_range(ontology: &Ontology, de: DeId) -> Option<DataValue> {
    let (datatype, facets) = collect_data_facets(ontology, de);
    let min = facets
        .get("minInclusive")
        .or_else(|| facets.get("minExclusive"));
    let max = facets
        .get("maxInclusive")
        .or_else(|| facets.get("maxExclusive"));
    if let (Some(lo), Some(hi)) = (min, max)
        && lo == hi {
            return Some(DataValue {
                lexical: lo.clone(),
                datatype,
            });
        }
    min.or(max).map(|lexical| DataValue {
        lexical: lexical.clone(),
        datatype,
    })
}

fn collect_data_facets(
    ontology: &Ontology,
    de: DeId,
) -> (Option<EntityId>, HashMap<String, String>) {
    match ontology.dl().de(de) {
        Some(DataExpr::Datatype(dt)) => (Some(*dt), HashMap::new()),
        Some(DataExpr::Facet {
            base,
            facet_iri,
            value,
        }) => {
            let (datatype, mut facets) = collect_data_facets(ontology, *base);
            let local = facet_iri.rsplit(['#', '/']).next().unwrap_or(facet_iri);
            facets.insert(local.to_string(), value.clone());
            (datatype, facets)
        }
        _ => (None, HashMap::new()),
    }
}

fn unify_args(
    ontology: &Ontology,
    left: &SwrlIArg,
    right: &SwrlIArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    match (left, right) {
        (SwrlIArg::Individual(a), SwrlIArg::Individual(b)) => {
            if same_individuals(ontology, *a, *b) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
        (SwrlIArg::Variable(v), SwrlIArg::Individual(i))
        | (SwrlIArg::Individual(i), SwrlIArg::Variable(v)) => {
            unify_var_individual(ontology, v, *i, binding)
        }
        (SwrlIArg::Variable(a), SwrlIArg::Variable(b)) => {
            if a == b {
                return vec![binding.clone()];
            }
            let Some(&ia) = binding.individuals.get(a) else {
                if let Some(&ib) = binding.individuals.get(b) {
                    return unify_var_individual(ontology, a, ib, binding);
                }
                return vec![];
            };
            unify_var_individual(ontology, b, ia, binding)
        }
    }
}

fn unify_different(
    ontology: &Ontology,
    left: &SwrlIArg,
    right: &SwrlIArg,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    unify_args(ontology, left, right, binding)
        .into_iter()
        .filter(|b| {
            let Some(l) = resolve_iarg(left, b) else {
                return false;
            };
            let Some(r) = resolve_iarg(right, b) else {
                return false;
            };
            l != r && !same_individuals(ontology, l, r)
        })
        .collect()
}

fn unify_var_individual(
    ontology: &Ontology,
    var: &str,
    ind: EntityId,
    binding: &RuleBinding,
) -> Vec<RuleBinding> {
    if let Some(&bound) = binding.individuals.get(var) {
        if same_individuals(ontology, bound, ind) {
            vec![binding.clone()]
        } else {
            vec![]
        }
    } else {
        let mut b = binding.clone();
        b.individuals.insert(var.to_owned(), ind);
        vec![b]
    }
}

fn resolve_iarg(arg: &SwrlIArg, binding: &RuleBinding) -> Option<EntityId> {
    match arg {
        SwrlIArg::Individual(id) => Some(*id),
        SwrlIArg::Variable(v) => binding.individuals.get(v).copied(),
    }
}

fn apply_rule_head(
    ontology: &mut Ontology,
    head: &[SwrlAtom],
    binding: &RuleBinding,
) -> ontologos_core::Result<bool> {
    let mut added = false;
    for atom in head {
        if apply_head_atom(ontology, atom, binding)? {
            added = true;
        }
    }
    Ok(added)
}

fn apply_head_atom(
    ontology: &mut Ontology,
    atom: &SwrlAtom,
    binding: &RuleBinding,
) -> ontologos_core::Result<bool> {
    match atom {
        SwrlAtom::Class { class, arg } => {
            let Some(ind) = resolve_iarg(arg, binding) else {
                return Ok(false);
            };
            if is_individual_typed(ontology, ind, *class) {
                return Ok(false);
            }
            ontology.add_axiom(Axiom::ClassAssertion {
                individual: ind,
                class: *class,
            })?;
            let ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(*class));
            ontology.dl_mut().push_axiom(DlAxiom::ClassAssertion {
                individual: ind,
                class: ce,
            });
            Ok(true)
        }
        SwrlAtom::ObjectProperty {
            property,
            subject,
            object,
        } => {
            let (Some(sub), Some(obj)) = (
                resolve_iarg(subject, binding),
                resolve_iarg(object, binding),
            ) else {
                return Ok(false);
            };
            if has_object_assertion(ontology, sub, *property, obj) {
                return Ok(false);
            }
            ontology.add_axiom(Axiom::ObjectPropertyAssertion {
                subject: sub,
                property: *property,
                object: obj,
            })?;
            Ok(true)
        }
        SwrlAtom::SameIndividual(a, b) => {
            let (Some(x), Some(y)) = (resolve_iarg(a, binding), resolve_iarg(b, binding)) else {
                return Ok(false);
            };
            if same_individuals(ontology, x, y) {
                return Ok(false);
            }
            ontology.add_axiom(Axiom::SameIndividual(vec![x, y]))?;
            Ok(true)
        }
        SwrlAtom::DifferentIndividuals(a, b) => {
            let (Some(x), Some(y)) = (resolve_iarg(a, binding), resolve_iarg(b, binding)) else {
                return Ok(false);
            };
            if x == y || same_individuals(ontology, x, y) || has_different(ontology, x, y) {
                return Ok(false);
            }
            ontology.add_axiom(Axiom::DifferentIndividuals(vec![x, y]))?;
            Ok(true)
        }
        SwrlAtom::DataProperty { .. } => Ok(false),
        SwrlAtom::DataRange { .. } => Ok(false),
    }
}

fn individuals_of_class(ontology: &Ontology, class: EntityId) -> Vec<EntityId> {
    let mut out = HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ClassAssertion {
            individual,
            class: c,
        } = axiom
            && *c == class {
                out.insert(*individual);
            }
    }
    for ind in ontology.entities().iter().filter_map(|(id, r)| {
        if r.kind == ontologos_core::EntityKind::Individual {
            Some(id)
        } else {
            None
        }
    }) {
        if ontology
            .classes_of(ind)
            .iter()
            .any(|&c| is_subsumed(ontology, c, class))
        {
            out.insert(ind);
        }
    }
    out.into_iter().collect()
}

fn is_subsumed(ontology: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    if sub == sup {
        return true;
    }
    let mut stack = vec![sub];
    let mut seen = HashSet::from([sub]);
    while let Some(current) = stack.pop() {
        for &direct in ontology.direct_superclasses(current) {
            if direct == sup {
                return true;
            }
            if seen.insert(direct) {
                stack.push(direct);
            }
        }
    }
    false
}

fn is_individual_typed(ontology: &Ontology, individual: EntityId, class: EntityId) -> bool {
    ontology
        .classes_of(individual)
        .iter()
        .any(|&c| is_subsumed(ontology, c, class))
}

fn has_object_assertion(
    ontology: &Ontology,
    subject: EntityId,
    property: EntityId,
    object: EntityId,
) -> bool {
    ontology
        .object_assertions_of(subject)
        .iter()
        .any(|&(p, o)| p == property && o == object)
}

fn has_different(ontology: &Ontology, a: EntityId, b: EntityId) -> bool {
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::DifferentIndividuals(ids) if ids.contains(&a) && ids.contains(&b)
        )
    })
}

fn same_individuals(ontology: &Ontology, a: EntityId, b: EntityId) -> bool {
    if a == b {
        return true;
    }
    let mut clusters: Vec<HashSet<EntityId>> = Vec::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SameIndividual(ids) = axiom
            && (ids.contains(&a) || ids.contains(&b)) {
                let mut cluster: HashSet<EntityId> = ids.iter().copied().collect();
                cluster.insert(a);
                cluster.insert(b);
                if let Some(existing) = clusters.iter_mut().find(|c| !c.is_disjoint(&cluster)) {
                    existing.extend(cluster);
                } else {
                    clusters.push(cluster);
                }
            }
    }
    clusters.iter().any(|c| c.contains(&a) && c.contains(&b))
}
