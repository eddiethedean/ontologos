use horned_owl::model::RcStr;
use horned_owl::ontology::set::SetOntology;
use ontologos_core::{EntityKind, Ontology, Taxonomy};
use whelk::whelk::owl::translate_ontology;
use whelk::whelk::reasoner;

use crate::horned::core_to_horned;
use crate::taxonomy::{equivalence_clusters, reduce_subsumptions};
use crate::Result;

const NOTHING_IRIS: &[&str] = &[
    "http://www.w3.org/2002/07/owl#Nothing",
    "http://www.w3.org/2002/07/owl#Bottom",
];

/// Classify a horned-owl ontology with whelk and map to core `Taxonomy`.
pub fn classify_horned(ontology: &SetOntology<RcStr>, core: &Ontology) -> Result<Taxonomy> {
    let translated = translate_ontology(ontology);
    let state = reasoner::assert(&translated);
    whelk_state_to_taxonomy(core, &state)
}

/// Classify a core ontology via horned-owl + whelk.
pub fn classify_core(ontology: &Ontology) -> Result<Taxonomy> {
    let horned = core_to_horned(ontology)?;
    classify_horned(&horned, ontology)
}

fn whelk_state_to_taxonomy(
    ontology: &Ontology,
    state: &reasoner::ReasonerState,
) -> Result<Taxonomy> {
    let mut pairs = Vec::new();
    for (sub_iri, sup_iri) in state.named_subsumptions() {
        if NOTHING_IRIS.contains(&sup_iri) {
            continue;
        }
        let Some(sub) = ontology.lookup_entity(sub_iri) else {
            continue;
        };
        let Some(sup) = ontology.lookup_entity(sup_iri) else {
            continue;
        };
        if ontology.entity(sub)?.kind != EntityKind::Class
            || ontology.entity(sup)?.kind != EntityKind::Class
        {
            continue;
        }
        if sub != sup {
            pairs.push((sub, sup));
        }
    }

    let equivalences = equivalence_clusters(&pairs);
    let subsumptions = reduce_subsumptions(&pairs);

    let mut unsatisfiable = Vec::new();
    for (class, record) in ontology.entities().iter() {
        if record.kind != EntityKind::Class {
            continue;
        }
        let iri = ontology.resolve_iri(record.iri)?;
        if NOTHING_IRIS.contains(&iri) {
            continue;
        }
        for nothing in NOTHING_IRIS {
            if let Some(bot) = ontology.lookup_entity(nothing) {
                if pairs.contains(&(class, bot)) {
                    unsatisfiable.push(class);
                    break;
                }
            }
        }
    }
    unsatisfiable.sort_by_key(|id| id.0);
    unsatisfiable.dedup();

    Ok(Taxonomy {
        subsumptions,
        equivalences,
        unsatisfiable,
    })
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    #[test]
    fn classify_transitive_chain() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        let c = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: b,
                superclass: c,
            })
            .unwrap();

        let taxonomy = classify_core(&ontology).unwrap();
        assert!(taxonomy.is_subsumed(a, c));
    }
}
