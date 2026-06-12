use std::collections::{HashSet, VecDeque};

use ontologos_core::{
    Axiom, EntityId, InferenceTrace, Ontology, Profile, TraceConclusion, TracePremise, TraceStep,
};

use crate::build::build_proof_graph;
use crate::{Error, ProofGraph, Result};

/// Explain why `sub ⊑ sup` holds under the given profile.
pub fn explain_subsumption(
    ontology: &Ontology,
    sub: EntityId,
    sup: EntityId,
    profile: Profile,
    trace: &InferenceTrace,
) -> Result<ProofGraph> {
    let subgraph = minimal_subsumption_trace(ontology, trace, sub, sup, profile)?;
    build_proof_graph(ontology, &subgraph)
}

/// Explain why `class` is unsatisfiable under the given profile.
pub fn explain_unsatisfiable(
    ontology: &Ontology,
    class: EntityId,
    bottom: EntityId,
    profile: Profile,
    trace: &InferenceTrace,
) -> Result<ProofGraph> {
    let subgraph = minimal_subsumption_trace(ontology, trace, class, bottom, profile)?;
    build_proof_graph(ontology, &subgraph)
}

fn minimal_subsumption_trace(
    ontology: &Ontology,
    trace: &InferenceTrace,
    sub: EntityId,
    sup: EntityId,
    profile: Profile,
) -> Result<InferenceTrace> {
    let step_idx = trace
        .steps
        .iter()
        .position(|s| conclusion_matches_subsumption(ontology, &s.conclusion, sub, sup))
        .ok_or_else(|| {
            Error::Core(ontologos_core::Error::Message(format!(
                "no inference step concludes {sub:?} ⊑ {sup:?}"
            )))
        })?;

    if matches!(profile, Profile::El | Profile::Auto) {
        Ok(InferenceTrace {
            steps: hst_prune(trace, step_idx),
        })
    } else {
        Ok(InferenceTrace {
            steps: backward_closure(trace, step_idx),
        })
    }
}

/// Hit-set style pruning: prefer smaller premise sets (EL first).
fn hst_prune(trace: &InferenceTrace, target_idx: usize) -> Vec<TraceStep> {
    let mut needed = HashSet::new();
    let mut queue = VecDeque::from([target_idx]);

    while let Some(idx) = queue.pop_front() {
        if !needed.insert(idx) {
            continue;
        }
        let step = &trace.steps[idx];
        if step.premises.is_empty() {
            continue;
        }
        // Heuristic: include only the first resolvable premise branch at each step.
        if let Some(premise_idx) = find_premise_step(trace, &step.premises[0]) {
            queue.push_back(premise_idx);
        }
        for premise in step.premises.iter().skip(1) {
            if let Some(premise_idx) = find_premise_step(trace, premise) {
                queue.push_back(premise_idx);
            }
        }
    }

    let mut ordered: Vec<usize> = needed.into_iter().collect();
    ordered.sort_unstable();
    ordered
        .into_iter()
        .map(|i| trace.steps[i].clone())
        .collect()
}

fn backward_closure(trace: &InferenceTrace, target_idx: usize) -> Vec<TraceStep> {
    let mut needed = HashSet::new();
    let mut queue = VecDeque::from([target_idx]);

    while let Some(idx) = queue.pop_front() {
        if !needed.insert(idx) {
            continue;
        }
        for premise in &trace.steps[idx].premises {
            if let Some(premise_idx) = find_premise_step(trace, premise) {
                queue.push_back(premise_idx);
            }
        }
    }

    let mut ordered: Vec<usize> = needed.into_iter().collect();
    ordered.sort_unstable();
    ordered
        .into_iter()
        .map(|i| trace.steps[i].clone())
        .collect()
}

fn conclusion_matches_subsumption(
    ontology: &Ontology,
    conclusion: &TraceConclusion,
    sub: EntityId,
    sup: EntityId,
) -> bool {
    match conclusion {
        TraceConclusion::SubClassOf { sub: s, sup: p } => *s == sub && *p == sup,
        TraceConclusion::Axiom { id } => matches!(
            ontology.axiom(*id),
            Ok(Axiom::SubClassOf {
                subclass,
                superclass,
            }) if subclass == &sub && superclass == &sup
        ),
        _ => false,
    }
}

fn find_premise_step(trace: &InferenceTrace, premise: &TracePremise) -> Option<usize> {
    trace
        .steps
        .iter()
        .position(|step| match (&step.conclusion, premise) {
            (
                TraceConclusion::SubClassOf { sub, sup },
                TracePremise::SubClassOf {
                    sub: psub,
                    sup: psup,
                },
            ) => sub == psub && sup == psup,
            (
                TraceConclusion::Existential {
                    class,
                    property,
                    filler,
                },
                TracePremise::Existential {
                    class: pclass,
                    property: pprop,
                    filler: pfiller,
                },
            ) => class == pclass && property == pprop && filler == pfiller,
            (
                TraceConclusion::SubObjectPropertyOf { sub, sup },
                TracePremise::SubObjectPropertyOf {
                    sub: psub,
                    sup: psup,
                },
            ) => sub == psub && sup == psup,
            (TraceConclusion::Axiom { id }, TracePremise::Axiom { id: pid }) => id == pid,
            _ => false,
        })
}

/// Find bottom (⊥) class used in a subsumption trace for `class`.
pub fn find_bottom_subsumption(
    ontology: &Ontology,
    class: EntityId,
    trace: &InferenceTrace,
) -> Option<EntityId> {
    trace.steps.iter().find_map(|step| {
        let TraceConclusion::SubClassOf { sub, sup } = step.conclusion else {
            return None;
        };
        if sub != class {
            return None;
        }
        let record = ontology.entity(sup).ok()?;
        if record.kind != ontologos_core::EntityKind::Class {
            return None;
        }
        let iri = ontology.resolve_iri(record.iri).ok()?;
        if iri.contains("Nothing") || iri.ends_with("#Bottom") || iri.ends_with("/Nothing") {
            Some(sup)
        } else {
            None
        }
    })
}

/// Locate a subsumption conclusion for two classes in a trace.
pub fn find_subsumption_step(trace: &InferenceTrace, sub: EntityId, sup: EntityId) -> bool {
    trace.steps.iter().any(|step| {
        matches!(
            step.conclusion,
            TraceConclusion::SubClassOf { sub: s, sup: p } if s == sub && p == sup
        )
    })
}

/// Locate an axiom subsumption in trace via ontology scan.
pub fn subsumption_from_axioms(ontology: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::SubClassOf {
                subclass,
                superclass,
            } if *subclass == sub && *superclass == sup
        )
    })
}
