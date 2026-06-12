use ontologos_core::{Axiom, AxiomId, EntityId, Ontology};

use crate::report::{InferenceRecord, MaterializationReport, RlRule};
use crate::triple_index::TripleIndex;

pub(crate) struct RuleContext<'a> {
    pub ontology: &'a mut Ontology,
    pub index: &'a mut TripleIndex,
    pub report: &'a mut MaterializationReport,
    pub record_traces: bool,
    pub parallelism: usize,
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

/// Partition work across a rayon pool when `parallelism > 1`.
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
        return items.into_par_iter().map(f).collect();
    }

    #[cfg(not(feature = "parallel"))]
    items.into_iter().map(f).collect()
}
