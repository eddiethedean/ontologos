use ontologos_core::{Ontology, Taxonomy};

/// Query handle over a classified ontology (call after [`crate::classify`]).
pub fn query_engine<'a>(
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
) -> ontologos_query::QueryEngine<'a> {
    ontologos_query::QueryEngine::new(ontology, taxonomy)
}
