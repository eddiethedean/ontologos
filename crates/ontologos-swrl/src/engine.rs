//! DLSafe SWRL forward chaining over asserted and inferred facts.

use std::collections::{HashMap, HashSet};

use ontologos_core::{
    Axiom, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, Ontology, RoleExpr, SwrlAtom, SwrlDArg,
    SwrlIArg, SwrlRule, Taxonomy,
};
use ontologos_dl::{LiteralIndex, LiteralValue};
use ontologos_rl::SameAsClosure;

use crate::SwrlReport;

/// Maximum forward-chaining rounds per `materialize_swrl_rules` call.
const MAX_SWRL_ITERATIONS: usize = 10_000;
/// Maximum bindings produced when matching one rule body.
const MAX_BINDINGS_PER_RULE: usize = 10_000;
/// Maximum total head applications per materialization.
const MAX_TOTAL_INFERENCES: usize = 100_000;

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
    let mut iterations = 0usize;
    while changed {
        iterations += 1;
        if iterations > MAX_SWRL_ITERATIONS {
            return Err(ontologos_core::Error::Message(format!(
                "SWRL forward chaining exceeded {MAX_SWRL_ITERATIONS} iterations"
            )));
        }
        changed = false;
        let same_as = ontologos_rl::same_as_closure(ontology);
        let taxonomy = swrl_el_taxonomy(ontology, &rules)?;
        for rule in &rules {
            let bindings = match_rule_body(ontology, &rule.body, &same_as, taxonomy.as_ref());
            if bindings.len() > MAX_BINDINGS_PER_RULE {
                return Err(ontologos_core::Error::Message(format!(
                    "SWRL rule binding explosion exceeds {MAX_BINDINGS_PER_RULE} bindings"
                )));
            }
            for binding in bindings {
                if apply_rule_head(ontology, &rule.head, &binding, &same_as)? {
                    report.inferences_added += 1;
                    if report.inferences_added > MAX_TOTAL_INFERENCES {
                        return Err(ontologos_core::Error::Message(format!(
                            "SWRL forward chaining exceeded {MAX_TOTAL_INFERENCES} inferences"
                        )));
                    }
                    changed = true;
                }
            }
        }
    }
    Ok(report)
}

fn rule_body_requires_el_taxonomy(body: &[SwrlAtom]) -> bool {
    body.iter()
        .any(|atom| matches!(atom, SwrlAtom::Class { .. }))
}

fn swrl_el_taxonomy(
    ontology: &Ontology,
    rules: &[SwrlRule],
) -> ontologos_core::Result<Option<Taxonomy>> {
    match ontologos_el::ElClassifier::new().classify_for_swrl(ontology) {
        Ok(taxonomy) => Ok(Some(taxonomy)),
        Err(ontologos_el::Error::NonElProfile { .. })
            if rules.iter().any(|rule| rule_body_requires_el_taxonomy(&rule.body)) =>
        {
            Err(ontologos_core::Error::Message(
                "SWRL forward chaining requires EL taxonomy; EL classification failed: \
                 ontology is not in OWL EL profile"
                    .into(),
            ))
        }
        Err(ontologos_el::Error::NonElProfile { .. }) => Ok(None),
        Err(e) => Err(ontologos_core::Error::Message(format!(
            "SWRL forward chaining requires EL taxonomy; EL classification failed: {e}"
        ))),
    }
}

fn match_rule_body(
    ontology: &Ontology,
    body: &[SwrlAtom],
    same_as: &SameAsClosure,
    taxonomy: Option<&Taxonomy>,
) -> Vec<RuleBinding> {
    let mut ordered: Vec<&SwrlAtom> = body.iter().collect();
    ordered.sort_by_key(|atom| atom_match_priority(atom));
    let mut bindings = vec![RuleBinding::default()];
    for atom in ordered {
        let mut next = Vec::new();
        for binding in bindings {
            next.extend(extend_binding(ontology, atom, &binding, same_as, taxonomy));
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

fn extend_binding(
    ontology: &Ontology,
    atom: &SwrlAtom,
    binding: &RuleBinding,
    same_as: &SameAsClosure,
    taxonomy: Option<&Taxonomy>,
) -> Vec<RuleBinding> {
    match atom {
        SwrlAtom::Class { class, arg } => extend_class(ontology, *class, arg, binding, taxonomy),
        SwrlAtom::ObjectProperty {
            property,
            subject,
            object,
        } => extend_object_property(ontology, *property, subject, object, binding, same_as),
        SwrlAtom::DataProperty {
            property,
            subject,
            value,
        } => extend_data_property(ontology, *property, subject, value, binding, same_as),
        SwrlAtom::DataRange { range, arg } => extend_data_range(ontology, *range, arg, binding),
        SwrlAtom::SameIndividual(a, b) => unify_same(ontology, a, b, binding, same_as),
        SwrlAtom::DifferentIndividuals(a, b) => unify_different(ontology, a, b, binding, same_as),
    }
}

fn extend_class(
    ontology: &Ontology,
    class: EntityId,
    arg: &SwrlIArg,
    binding: &RuleBinding,
    taxonomy: Option<&Taxonomy>,
) -> Vec<RuleBinding> {
    match arg {
        SwrlIArg::Individual(ind) => {
            if is_individual_typed(ontology, *ind, class, taxonomy) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
        SwrlIArg::Variable(var) => {
            if let Some(&ind) = binding.individuals.get(var) {
                return extend_class(
                    ontology,
                    class,
                    &SwrlIArg::Individual(ind),
                    binding,
                    taxonomy,
                );
            }
            individuals_of_class(ontology, class, taxonomy)
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

fn object_property_assertions(
    ontology: &Ontology,
    property: EntityId,
) -> Vec<(EntityId, EntityId)> {
    let mut out = Vec::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyAssertion {
            subject,
            property: p,
            object,
        } = axiom
            && *p == property
        {
            out.push((*subject, *object));
        }
    }
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::ObjectPropertyAssertion {
            subject,
            property: p,
            object,
        } = axiom
            && *p == RoleExpr::Atomic(property)
        {
            out.push((*subject, *object));
        }
    }
    out
}

fn extend_object_property(
    ontology: &Ontology,
    property: EntityId,
    subject: &SwrlIArg,
    object: &SwrlIArg,
    binding: &RuleBinding,
    same_as: &SameAsClosure,
) -> Vec<RuleBinding> {
    let assertions = object_property_assertions(ontology, property);

    let mut out = Vec::new();
    for (sub, obj) in assertions {
        for b in unify_args(
            ontology,
            subject,
            &SwrlIArg::Individual(sub),
            binding,
            same_as,
        ) {
            out.extend(unify_args(
                ontology,
                object,
                &SwrlIArg::Individual(obj),
                &b,
                same_as,
            ));
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
    same_as: &SameAsClosure,
) -> Vec<RuleBinding> {
    let mut out = Vec::new();
    for (sub, prop, fact) in data_property_facts(ontology) {
        if prop != property {
            continue;
        }
        for b in unify_args(
            ontology,
            subject,
            &SwrlIArg::Individual(sub),
            binding,
            same_as,
        ) {
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
    same_as: &SameAsClosure,
) -> Vec<RuleBinding> {
    let ind = unify_args(ontology, left, right, binding, same_as);
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
        && lo == hi
    {
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
    same_as: &SameAsClosure,
) -> Vec<RuleBinding> {
    match (left, right) {
        (SwrlIArg::Individual(a), SwrlIArg::Individual(b)) => {
            if same_individuals(same_as, *a, *b) {
                vec![binding.clone()]
            } else {
                vec![]
            }
        }
        (SwrlIArg::Variable(v), SwrlIArg::Individual(i))
        | (SwrlIArg::Individual(i), SwrlIArg::Variable(v)) => {
            unify_var_individual(ontology, v, *i, binding, same_as)
        }
        (SwrlIArg::Variable(a), SwrlIArg::Variable(b)) => {
            if a == b {
                return vec![binding.clone()];
            }
            let Some(&ia) = binding.individuals.get(a) else {
                if let Some(&ib) = binding.individuals.get(b) {
                    return unify_var_individual(ontology, a, ib, binding, same_as);
                }
                return vec![];
            };
            unify_var_individual(ontology, b, ia, binding, same_as)
        }
    }
}

fn unify_different(
    ontology: &Ontology,
    left: &SwrlIArg,
    right: &SwrlIArg,
    binding: &RuleBinding,
    same_as: &SameAsClosure,
) -> Vec<RuleBinding> {
    unify_args(ontology, left, right, binding, same_as)
        .into_iter()
        .filter(|b| {
            let Some(l) = resolve_iarg(left, b) else {
                return false;
            };
            let Some(r) = resolve_iarg(right, b) else {
                return false;
            };
            l != r && !same_individuals(same_as, l, r)
        })
        .collect()
}

fn unify_var_individual(
    _ontology: &Ontology,
    var: &str,
    ind: EntityId,
    binding: &RuleBinding,
    same_as: &SameAsClosure,
) -> Vec<RuleBinding> {
    if let Some(&bound) = binding.individuals.get(var) {
        if same_individuals(same_as, bound, ind) {
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
    same_as: &SameAsClosure,
) -> ontologos_core::Result<bool> {
    let mut added = false;
    for atom in head {
        if apply_head_atom(ontology, atom, binding, same_as)? {
            added = true;
        }
    }
    Ok(added)
}

fn apply_head_atom(
    ontology: &mut Ontology,
    atom: &SwrlAtom,
    binding: &RuleBinding,
    same_as: &SameAsClosure,
) -> ontologos_core::Result<bool> {
    match atom {
        SwrlAtom::Class { class, arg } => {
            let Some(ind) = resolve_iarg(arg, binding) else {
                return Ok(false);
            };
            if is_individual_typed(ontology, ind, *class, None) {
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
            if same_individuals(same_as, x, y) {
                return Ok(false);
            }
            ontology.add_axiom(Axiom::SameIndividual(vec![x, y]))?;
            Ok(true)
        }
        SwrlAtom::DifferentIndividuals(a, b) => {
            let (Some(x), Some(y)) = (resolve_iarg(a, binding), resolve_iarg(b, binding)) else {
                return Ok(false);
            };
            if x == y || same_individuals(same_as, x, y) || has_different(ontology, x, y) {
                return Ok(false);
            }
            ontology.add_axiom(Axiom::DifferentIndividuals(vec![x, y]))?;
            Ok(true)
        }
        SwrlAtom::DataProperty { .. } => Ok(false),
        SwrlAtom::DataRange { .. } => Ok(false),
    }
}

fn individuals_of_class(
    ontology: &Ontology,
    class: EntityId,
    taxonomy: Option<&Taxonomy>,
) -> Vec<EntityId> {
    let mut out = HashSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ClassAssertion {
            individual,
            class: c,
        } = axiom
            && *c == class
        {
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
            .any(|&c| is_subsumed(ontology, c, class, taxonomy))
        {
            out.insert(ind);
        }
    }
    out.into_iter().collect()
}

fn is_subsumed(
    ontology: &Ontology,
    sub: EntityId,
    sup: EntityId,
    taxonomy: Option<&Taxonomy>,
) -> bool {
    if let Some(tax) = taxonomy {
        return tax.is_subsumed(sub, sup);
    }
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

fn is_individual_typed(
    ontology: &Ontology,
    individual: EntityId,
    class: EntityId,
    taxonomy: Option<&Taxonomy>,
) -> bool {
    ontology
        .classes_of(individual)
        .iter()
        .any(|&c| is_subsumed(ontology, c, class, taxonomy))
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

fn same_individuals(same_as: &SameAsClosure, a: EntityId, b: EntityId) -> bool {
    a == b || same_as.representative(a) == same_as.representative(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::{ClassExpr, DlAxiom, Ontology, SwrlAtom, SwrlIArg, SwrlRule};

    #[test]
    fn swrl_does_not_silently_degrade_when_el_classify_fails() {
        let mut ontology = Ontology::builder()
            .class("http://example.org/A")
            .expect("class")
            .class("http://example.org/B")
            .expect("class")
            .build()
            .expect("build");
        let a = ontology.lookup_entity("http://example.org/A").expect("A");
        let b = ontology.lookup_entity("http://example.org/B").expect("B");

        // Force a non-EL construct so EL classification fails.
        let ce_a = ontology.dl_mut().intern_ce(ClassExpr::Atomic(a));
        let ce_b = ontology.dl_mut().intern_ce(ClassExpr::Atomic(b));
        let ce_not_b = ontology.dl_mut().intern_ce(ClassExpr::Not(ce_b));
        ontology.dl_mut().push_axiom(DlAxiom::SubClassOf {
            sub: ce_a,
            sup: ce_not_b,
        });

        ontology
            .push_swrl_rule(SwrlRule {
                body: vec![SwrlAtom::Class {
                    class: a,
                    arg: SwrlIArg::Individual(a),
                }],
                head: vec![SwrlAtom::Class {
                    class: b,
                    arg: SwrlIArg::Individual(a),
                }],
            })
            .expect("swrl");

        let err = materialize_swrl_rules(&mut ontology).expect_err("expected error");
        assert!(
            err.to_string().contains("EL classification failed"),
            "unexpected error: {err}"
        );
    }
}
