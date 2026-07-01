//! OWL ALC/DL tableau classification.

#![warn(missing_docs)]

mod clause;
mod dl_ontology;
mod hyper_abox;
mod hyper_cardinality;
mod hyper_clausify;
mod hyper_nominals;
mod hyper_object;
mod hyperclause;
mod normalize;
mod object_property_classify;
mod tableau;

use ontologos_core::{Ontology, Taxonomy};
use ontologos_profile::{OwlProfile, detect_profile};
use thiserror::Error;

pub use clause::{Clause, ClauseSet};
pub use dl_ontology::DlOntology;
pub use hyper_clausify::clausify_hyper;
pub use hyperclause::{HyperClauseSet, format_hyper_clauses};
pub use normalize::clausify;
#[doc(hidden)]
pub use object_property_classify::{
    PreparedRoleSurrogateContext, augment_for_role_classification,
    classify_object_property_on_augmented, equivalent_object_property_on_augmented,
    sub_object_property_on_augmented,
};
pub use object_property_classify::{
    classify_object_property_expressions, equivalent_object_property_expressions,
    inverse_object_property_expressions, sub_object_property_expressions,
};
pub use tableau::AlcClassifier;
pub use tableau::blocking_validator::{
    BlockingStrategy, BlockingValidator, RoleRef, blocking_concepts,
    blocking_test_annotated_equalities_clauses, blocking_test_one_invalid_block_clauses,
};
pub use tableau::cache::UnsatCache;
pub use tableau::dependency_set::{
    DependencySetFactory, DependencySetRef, PermanentDependencySet, UnionDependencySet,
};
pub use tableau::description_graph::{DescriptionGraph, DescriptionGraphEdge, DescriptionGraphId};
pub use tableau::dl_clause_eval::{
    DlClauseEvaluator, derive_at_most_equalities, dl_clause_evaluation_test_clause, do_iteration,
    run_calculus,
};
pub use tableau::extension_manager::{
    BranchingPoint, DlObject, DlPredicate, ExtensionManagerRef, ExtensionTable, ExtensionView,
    Node, Tableau, test_helpers,
};
pub use tableau::graph_merge;
pub use tableau::ni_rules::{AnnotatedEquality, NominalIntroductionManager};
pub use tableau::tuple_index::{TupleIndex, TupleIndexRetrieval};
pub use tableau::tuple_table::{TupleTable, TupleTableFullIndex};
pub use tableau::{
    TableauSeed, classify as tableau_classify,
    classify_with_dl_and_seed as tableau_classify_with_dl_and_seed,
    classify_with_seed as tableau_classify_with_seed,
    classify_with_seed_options as tableau_classify_with_seed_options,
    is_ce_intersection_satisfiable_with_seed, is_ce_satisfiable_with_seed,
    is_consistent as tableau_is_consistent,
    is_consistent_with_seed as tableau_is_consistent_with_seed,
    is_named_class_satisfiable_with_cache, is_named_class_satisfiable_with_seed,
    role_expression_subsumes, structural_unsat_classes,
};

/// Result type for ALC operations.
pub type Result<T> = std::result::Result<T, Error>;

/// ALC engine errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Profile mismatch.
    #[error("ontology exceeds ALC fragment ({0:?})")]
    NonAlcProfile(OwlProfile),
    /// Core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// Parser error.
    #[error(transparent)]
    Parser(#[from] ontologos_parser::Error),
    /// Profile detection error.
    #[error(transparent)]
    Profile(#[from] ontologos_profile::Error),
    /// General message.
    #[error("{0}")]
    Message(String),
    /// Tableau expansion budget exhausted (incomplete reasoning).
    #[error("tableau expansion budget exhausted ({0} expansions)")]
    ResourceLimit(u32),
    /// Tuple index trie node space exhausted.
    #[error("tuple index node space exhausted")]
    TupleIndexExhausted,
}

/// Classify an ontology under ALC tableau semantics.
pub fn classify(ontology: &Ontology) -> Result<Taxonomy> {
    let report = detect_profile(ontology)?;
    if !matches!(
        report.detected,
        Some(OwlProfile::El | OwlProfile::Ql | OwlProfile::Dl)
    ) {
        if let Some(p) = report.detected {
            if p == OwlProfile::Rl {
                return Err(Error::NonAlcProfile(p));
            }
        }
    }
    tableau::classify(ontology)
}

/// Classify with saturation-derived seed facts.
pub fn classify_with_seed(ontology: &Ontology, seed: &TableauSeed) -> Result<Taxonomy> {
    tableau::classify_with_seed_options(ontology, seed, true)
}

/// Classify using a pre-clausified ontology (avoids duplicate clausification).
pub fn classify_with_dl_and_seed(
    dl: &DlOntology,
    seed: &TableauSeed,
    infer_pairwise_subsumptions: bool,
) -> Result<Taxonomy> {
    tableau::classify_with_dl_and_seed(dl, seed, infer_pairwise_subsumptions)
}

/// Classify with saturation seed, skipping expensive pairwise subsumption inference.
pub fn classify_with_seed_for_entailment(
    ontology: &Ontology,
    seed: &TableauSeed,
) -> Result<Taxonomy> {
    tableau::classify_with_seed_options(ontology, seed, false)
}

/// Tableau consistency check.
pub fn is_consistent(ontology: &Ontology) -> Result<bool> {
    tableau::is_consistent(ontology)
}

/// Classify via [`ontologos_core::Reasoner`] when profile is ALC-compatible.
pub fn classify_reasoner(reasoner: &ontologos_core::Reasoner) -> Result<Taxonomy> {
    classify(reasoner.ontology())
}
