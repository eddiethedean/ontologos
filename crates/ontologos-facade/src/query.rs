use ontologos_core::{Ontology, Taxonomy};

/// Navigate a classified taxonomy hierarchy (subclasses, subsumption).
///
/// For OWL QL conjunctive queries use [`ontologos_ql::answer_query`].
pub fn taxonomy_hierarchy<'a>(
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
) -> ontologos_ql::TaxonomyHierarchy<'a> {
    ontologos_ql::TaxonomyHierarchy::new(ontology, taxonomy)
}

/// Deprecated alias for [`taxonomy_hierarchy`].
#[deprecated(since = "1.1.0", note = "use taxonomy_hierarchy instead")]
pub fn query_engine<'a>(
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
) -> ontologos_ql::TaxonomyHierarchy<'a> {
    taxonomy_hierarchy(ontology, taxonomy)
}
