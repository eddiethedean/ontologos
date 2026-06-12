use ontologos_core::{Axiom, AxiomId, EntityId, Ontology};

use crate::report::{InferenceRecord, MaterializationReport, RlRule};
use crate::triple_index::TripleIndex;

pub(crate) struct RuleContext<'a> {
    pub ontology: &'a mut Ontology,
    pub index: &'a mut TripleIndex,
    pub report: &'a mut MaterializationReport,
    pub record_traces: bool,
    pub parallelism: usize,
    /// Axiom ids below this bound are treated as asserted for existential subsumption.
    pub asserted_axiom_count: usize,
}

pub(crate) fn infer_axiom(
    ctx: &mut RuleContext<'_>,
    rule: RlRule,
    axiom: Axiom,
    premises: Vec<AxiomId>,
) -> ontologos_core::Result<()> {
    let before = ctx.ontology.axiom_count();
    let conclusion = ctx.ontology.add_axiom(axiom.clone())?;
    if ctx.ontology.axiom_count() > before {
        ctx.index.on_axiom_added(ctx.ontology, &axiom);
        *ctx.report.inferred_by_rule.entry(rule).or_default() += 1;
        if ctx.record_traces {
            ctx.report.traces.push(InferenceRecord {
                rule,
                premises,
                conclusion,
            });
        }
    }
    Ok(())
}

pub(crate) fn expand_equivalent_classes(ontology: &Ontology, class: EntityId) -> Vec<EntityId> {
    let mut out = vec![class];
    if let Some(equiv) = ontology.equivalents_of(class) {
        out.extend(equiv.iter().copied());
    }
    out.sort_by_key(|id| id.0);
    out.dedup();
    out
}

/// True when `class_a` and `class_b` belong to disjoint classes, expanding equivalence (cls-disjoint2).
pub(crate) fn classes_are_disjoint(
    ontology: &Ontology,
    class_a: EntityId,
    class_b: EntityId,
) -> bool {
    for rep_a in expand_equivalent_classes(ontology, class_a) {
        if let Some(disjoint_set) = ontology.disjoint_with(rep_a) {
            for rep_b in expand_equivalent_classes(ontology, class_b) {
                if disjoint_set.contains(&rep_b) {
                    return true;
                }
            }
        }
    }
    false
}

/// Smallest entity id in the equivalence class of `class` (for clash deduplication).
pub(crate) fn canonical_equiv_rep(ontology: &Ontology, class: EntityId) -> EntityId {
    expand_equivalent_classes(ontology, class)
        .into_iter()
        .min_by_key(|id| id.0)
        .unwrap_or(class)
}

pub(crate) fn expand_equivalent_properties(
    ontology: &Ontology,
    property: EntityId,
) -> Vec<EntityId> {
    let mut out = vec![property];
    if let Some(equiv) = ontology.equivalent_properties_of(property) {
        out.extend(equiv.iter().copied());
    }
    out.sort_by_key(|id| id.0);
    out.dedup();
    out
}

pub(crate) fn expand_same_as(ontology: &Ontology, individual: EntityId) -> Vec<EntityId> {
    let mut out = vec![individual];
    if let Some(same) = ontology.same_as(individual) {
        out.extend(same.iter().copied());
    }
    out.sort_by_key(|id| id.0);
    out.dedup();
    out
}

pub(crate) fn transitive_superproperties(ontology: &Ontology, property: EntityId) -> Vec<EntityId> {
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<EntityId> = ontology.direct_superproperties(property).to_vec();
    let mut out = Vec::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        out.push(current);
        stack.extend_from_slice(ontology.direct_superproperties(current));
    }
    out
}

/// True when `sub` is equivalent to or a subclass of `sup`.
pub(crate) fn is_subclass_of(ontology: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    if sub == sup {
        return true;
    }
    for rep_sub in expand_equivalent_classes(ontology, sub) {
        for rep_sup in expand_equivalent_classes(ontology, sup) {
            if rep_sub == rep_sup {
                return true;
            }
            if transitive_superclasses(ontology, rep_sub).contains(&rep_sup) {
                return true;
            }
        }
    }
    false
}

pub(crate) fn transitive_superclasses(ontology: &Ontology, class: EntityId) -> Vec<EntityId> {
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<EntityId> = ontology.direct_superclasses(class).to_vec();
    let mut out = Vec::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        out.push(current);
        stack.extend_from_slice(ontology.direct_superclasses(current));
    }
    out
}

pub(crate) fn transitive_subproperties(ontology: &Ontology, property: EntityId) -> Vec<EntityId> {
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<EntityId> = ontology.direct_subproperties(property).to_vec();
    let mut out = Vec::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        out.push(current);
        stack.extend_from_slice(ontology.direct_subproperties(current));
    }
    out
}

/// Partition work across a rayon pool sized to `parallelism` when `parallelism > 1`.
pub(crate) fn map_parallel<T, R, F>(parallelism: usize, items: Vec<T>, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Sync + Send,
{
    if parallelism <= 1 || items.len() < 2 {
        return items.into_iter().map(f).collect();
    }

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        use rayon::ThreadPoolBuilder;

        let pool = ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build()
            .expect("thread pool");
        pool.install(|| items.into_par_iter().map(f).collect())
    }

    #[cfg(not(feature = "parallel"))]
    items.into_iter().map(f).collect()
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, Ontology};

    use super::classes_are_disjoint;

    const NS: &str = "http://example.org/test#";

    fn iri(local: &str) -> String {
        format!("{NS}{local}")
    }

    /// cls-disjoint2: disjoint check must expand equivalence, not rely on direct index lookup.
    #[test]
    fn classes_are_disjoint_expands_equivalent_classes() {
        let mut ontology = Ontology::builder()
            .class(&iri("A"))
            .expect("A")
            .class(&iri("B"))
            .expect("B")
            .class(&iri("D"))
            .expect("D")
            .build()
            .expect("build");

        let a = ontology.lookup_entity(&iri("A")).expect("A");
        let b = ontology.lookup_entity(&iri("B")).expect("B");
        let d = ontology.lookup_entity(&iri("D")).expect("D");
        ontology
            .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
            .expect("equiv");
        ontology
            .add_axiom(Axiom::DisjointClasses(vec![a, d]))
            .expect("disjoint");

        assert!(
            !ontology
                .disjoint_with(b)
                .is_some_and(|set| set.contains(&d)),
            "setup: direct disjoint index on B should not list D"
        );
        assert!(
            classes_are_disjoint(&ontology, b, d),
            "expected disjoint via equivalence expansion A ≡ B and A ⊥ D"
        );
    }
}
