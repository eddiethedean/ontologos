//! Functional/inverse-functional cardinality product grid (dl-905/910 family).

use ontologos_core::{CeId, ClassExpr, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr};

/// Detect dl-910-style inconsistency: `|invR| ≠ |invP| × |invQ|` on the WG grid.
pub fn functional_inverse_cardinality_product_inconsistent(ontology: &Ontology) -> bool {
    let Some((n_p, n_q, n_r)) = extract_functional_inverse_grid_counts(ontology) else {
        return false;
    };
    n_r != n_p.saturating_mul(n_q)
}

fn extract_functional_inverse_grid_counts(ontology: &Ontology) -> Option<(u32, u32, u32)> {
    if !functional_inverse_grid_properties(ontology) {
        return None;
    }
    let store = ontology.dl();
    let mut only_d_pair: Option<(u32, u32)> = None;
    let mut middle: Option<u32> = None;

    for (class, record) in ontology.entities().iter() {
        if record.kind != EntityKind::Class {
            continue;
        }
        let class_ce = store.expressions().find_map(|(ce, expr)| match expr {
            ClassExpr::Atomic(c) if *c == class => Some(ce),
            _ => None,
        })?;
        let mut has_singleton_oneof = false;
        let mut exact_cards = Vec::new();
        for axiom in store.axioms() {
            let DlAxiom::EquivalentClasses(ops) = axiom else {
                continue;
            };
            if !ops.contains(&class_ce) {
                continue;
            }
            for &partner in ops {
                if partner == class_ce {
                    continue;
                }
                collect_equiv_shape(store, partner, &mut has_singleton_oneof, &mut exact_cards);
            }
        }
        if exact_cards.len() == 2 {
            only_d_pair = Some((exact_cards[0].1, exact_cards[1].1));
        } else if exact_cards.len() == 1 {
            middle = Some(exact_cards[0].1);
        }
    }

    let (n_p, n_r) = only_d_pair?;
    let n_q = middle?;
    Some((n_p, n_q, n_r))
}

fn collect_equiv_shape(
    store: &ontologos_core::DlStore,
    ce: CeId,
    has_singleton_oneof: &mut bool,
    exact_cards: &mut Vec<(EntityId, u32)>,
) {
    let Some(expr) = store.ce(ce).cloned() else {
        return;
    };
    match expr {
        ClassExpr::OneOf(nominals) if nominals.len() == 1 => *has_singleton_oneof = true,
        ClassExpr::ExactCardinality {
            n,
            property: RoleExpr::Atomic(prop),
            filler: None,
        } => exact_cards.push((prop, n)),
        ClassExpr::And(ops) => {
            for op in ops {
                collect_equiv_shape(store, op, has_singleton_oneof, exact_cards);
            }
        }
        _ => {}
    }
}

fn functional_inverse_grid_properties(ontology: &Ontology) -> bool {
    let index = ontology.index();
    let object_props = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::ObjectProperty)
        .count();
    let functional = index.functional_properties().len();
    let inverse_functional = index.inverse_functional_properties().len();
    let with_inverse = ontology.entities().iter().filter(|(id, record)| {
        record.kind == EntityKind::ObjectProperty
            && ontology.axioms().iter().any(|(_, axiom)| {
                matches!(
                    axiom,
                    ontologos_core::Axiom::InverseObjectProperties { left, right }
                        if *left == *id || *right == *id
                )
            })
    }).count();
    object_props >= 3
        && functional >= 3
        && (inverse_functional >= 3 || with_inverse >= 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    fn wg(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit")
            .join(rel)
    }

    #[test]
    fn dl905_grid_consistent() {
        let ont = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D905/premise.rdf",
        ))
        .expect("load");
        assert!(!functional_inverse_cardinality_product_inconsistent(&ont));
    }

    #[test]
    fn dl910_grid_inconsistent() {
        let ont = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D910/premise.rdf",
        ))
        .expect("load");
        assert!(functional_inverse_cardinality_product_inconsistent(&ont));
    }
}
