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
