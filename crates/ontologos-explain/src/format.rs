use ontologos_core::{Axiom, EntityId, Ontology};

use crate::{NodeId, ProofGraph};

/// Render a proof graph as human-readable text.
pub fn render_text(ontology: &Ontology, graph: &ProofGraph) -> String {
    let mut lines = Vec::new();
    for (idx, node) in graph.nodes.iter().enumerate() {
        let premise_labels: Vec<String> = node
            .premises
            .iter()
            .map(|p| format_node_ref(ontology, graph, *p))
            .collect();
        let conclusion = format_conclusion(ontology, node);
        if node.rule == "asserted" {
            lines.push(format!("[{idx}] asserted => {conclusion}"));
        } else {
            lines.push(format!(
                "[{idx}] {}({}) => {conclusion}",
                node.rule,
                premise_labels.join(", ")
            ));
        }
    }
    lines.join("\n")
}

fn format_node_ref(ontology: &Ontology, graph: &ProofGraph, id: NodeId) -> String {
    let node = &graph.nodes[id.0 as usize];
    format_conclusion(ontology, node)
}

fn format_conclusion(ontology: &Ontology, node: &crate::ProofNode) -> String {
    if let Some(id) = node.conclusion_axiom {
        return format_axiom(ontology, id);
    }
    if let Some((sub, sup)) = node.conclusion_sub {
        return format!(
            "{} ⊑ {}",
            format_entity(ontology, sub),
            format_entity(ontology, sup)
        );
    }
    if let Some((class, property, filler)) = node.conclusion_existential {
        return format!(
            "{} ⊑ ∃{}.{}",
            format_entity(ontology, class),
            format_entity(ontology, property),
            format_entity(ontology, filler)
        );
    }
    if let Some((sub, sup)) = node.conclusion_subproperty {
        return format!(
            "{} ⊑ {}",
            format_entity(ontology, sub),
            format_entity(ontology, sup)
        );
    }
    "?".to_string()
}

fn format_axiom(ontology: &Ontology, id: ontologos_core::AxiomId) -> String {
    match ontology.axiom(id) {
        Ok(axiom) => format_axiom_shape(ontology, axiom),
        Err(_) => format!("axiom#{}", id.0),
    }
}

fn format_axiom_shape(ontology: &Ontology, axiom: &Axiom) -> String {
    match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => format!(
            "{} ⊑ {}",
            format_entity(ontology, *subclass),
            format_entity(ontology, *superclass)
        ),
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => format!(
            "{} ⊑ {}",
            format_entity(ontology, *sub_property),
            format_entity(ontology, *super_property)
        ),
        _ => format!("{axiom:?}"),
    }
}

fn format_entity(ontology: &Ontology, id: EntityId) -> String {
    match ontology.entity(id) {
        Ok(record) => ontology
            .resolve_iri(record.iri)
            .map(str::to_string)
            .unwrap_or_else(|_| format!("entity#{}", id.0)),
        Err(_) => format!("entity#{}", id.0),
    }
}
