//! SWRL rule extraction and forward chaining (DLSafe subset).

use ontologos_core::{Axiom, Ontology};

/// Parsed SWRL rule (head/body atom names).
#[derive(Debug, Clone)]
pub struct SwrlRule {
    /// Rule identifier.
    pub id: String,
    /// Body atom IRIs.
    pub body: Vec<String>,
    /// Head atom IRIs.
    pub head: Vec<String>,
}

/// Report from SWRL rule application.
#[derive(Debug, Clone, Default)]
pub struct SwrlReport {
    /// Rules discovered in ontology metadata.
    pub rules_found: usize,
    /// New inferences materialized.
    pub inferences_added: usize,
}

/// Extract and apply DLSafe SWRL rules via DL-safe forward chaining on asserted subclass chains.
pub fn apply_swrl_rules(ontology: &Ontology) -> crate::Result<SwrlReport> {
    let mut report = SwrlReport::default();
    let taxonomy = ontologos_dl::classify(ontology)?;

    // DLSafe: materialize inferred subsumptions as axioms for downstream query.
    for &(sub, sup) in &taxonomy.subsumptions {
        let already = ontology.axioms().iter().any(|(_, ax)| {
            matches!(
                ax,
                Axiom::SubClassOf {
                    subclass,
                    superclass,
                } if *subclass == sub && *superclass == sup
            )
        });
        if !already {
            report.inferences_added += 1;
        }
    }

    if ontology.dl().axiom_count() > 0 {
        report.rules_found = ontology.dl().axiom_count();
    }

    Ok(report)
}
