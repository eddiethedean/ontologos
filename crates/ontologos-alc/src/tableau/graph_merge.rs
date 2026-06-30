//! Description graph tuple merging (HermiT `GraphTest.testGraphMerging`).

use std::rc::Rc;

use super::dependency_set::PermanentDependencySet;
use super::description_graph::DescriptionGraphId;
use super::extension_manager::{DlObject, DlPredicate, ExtensionView, Node, NodeId, Tableau};

/// Merge description-graph extension tuples that share nodes (HermiT saturation).
pub fn saturate_graph_merges(tableau: &Tableau) -> bool {
    let ext = tableau.extension_manager();
    let mut tuples: Vec<(DescriptionGraphId, NodeId, NodeId, NodeId)> = Vec::new();
    {
        let table = ext.quaternary_extension_table();
        let mut retrieval = table.create_retrieval(&[true, false, false, false], ExtensionView::Total);
        retrieval.open();
        while !retrieval.after_last() {
            if let (
                DlObject::Predicate(DlPredicate::DescriptionGraph(g)),
                DlObject::Node(a),
                DlObject::Node(b),
                DlObject::Node(c),
            ) = (
                &retrieval.tuple_buffer()[0],
                &retrieval.tuple_buffer()[1],
                &retrieval.tuple_buffer()[2],
                &retrieval.tuple_buffer()[3],
            ) {
                tuples.push((*g, *a, *b, *c));
            }
            retrieval.next();
        }
    }
    if tuples.len() < 2 {
        return false;
    }
    let empty = tableau.empty_dependency_set();
    let mut merged_any = false;
    for i in 0..tuples.len() {
        for j in (i + 1)..tuples.len() {
            if tuples[i].0 != tuples[j].0 {
                continue;
            }
            if connects(&tuples[i], &tuples[j]) {
                merged_any |= merge_pair(tableau, tuples[i].1, tuples[j].1, empty.clone());
                merged_any |= merge_pair(tableau, tuples[i].2, tuples[j].2, empty.clone());
                merged_any |= merge_pair(tableau, tuples[i].3, tuples[j].3, empty.clone());
            }
        }
    }
    merged_any
}

fn connects(
    left: &(DescriptionGraphId, NodeId, NodeId, NodeId),
    right: &(DescriptionGraphId, NodeId, NodeId, NodeId),
) -> bool {
    let (_, a1, b1, c1) = left;
    let (_, a2, b2, c2) = right;
    a1 == a2 || a1 == b2 || a1 == c2 || b1 == a2 || b1 == b2 || b1 == c2 || c1 == a2 || c1 == b2 || c1 == c2
}

fn merge_pair(
    tableau: &Tableau,
    a: NodeId,
    b: NodeId,
    dependency: Rc<PermanentDependencySet>,
) -> bool {
    if a == b {
        return false;
    }
    let ext = tableau.extension_manager();
    let na = ext.node(a);
    let nb = ext.node(b);
    if na.is_active() && nb.is_active() {
        ext.merge_nodes(&na, &nb, dependency)
    } else {
        false
    }
}

/// HermiT `GraphTest.testGraphMerging` merge expectations after saturation.
pub fn assert_graph_merging_canonicals(
    tableau: &Tableau,
    n1: &Node,
    n2: &Node,
    n3: &Node,
    n4: &Node,
    n5: &Node,
    n6: &Node,
    n7: &Node,
) {
    let ext = tableau.extension_manager();
    let c = |n: &Node| ext.canonical(n);
    assert_eq!(c(n1).id(), n1.id());
    assert_eq!(c(n7).id(), c(n2).id());
    assert_eq!(c(n6).id(), c(n3).id());
    assert_eq!(c(n1).id(), c(n4).id());
    assert_eq!(c(n7).id(), c(n5).id());
    assert_eq!(c(n6).id(), n6.id());
    assert_eq!(c(n7).id(), n7.id());
}
