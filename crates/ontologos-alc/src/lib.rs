//! OWL ALC/DL tableau classification.

#![warn(missing_docs)]

mod clause;
mod dl_ontology;
mod hyper_clausify;
mod hyperclause;
mod normalize;
mod object_property_classify;
mod tableau;

use ontologos_core::{Ontology, Taxonomy};
use ontologos_profile::{detect_profile, OwlProfile};
use thiserror::Error;

pub use clause::{Clause, ClauseSet};
pub use dl_ontology::DlOntology;
pub use hyper_clausify::clausify_hyper;
pub use hyperclause::{format_hyper_clauses, HyperClauseSet};
pub use normalize::clausify;
pub use object_property_classify::{
    classify_object_property_expressions, equivalent_object_property_expressions,
    inverse_object_property_expressions, sub_object_property_expressions,
};
#[doc(hidden)]
pub use object_property_classify::{
    augment_for_role_classification, classify_object_property_on_augmented,
    equivalent_object_property_on_augmented, PreparedRoleSurrogateContext,
    sub_object_property_on_augmented,
};
pub use tableau::cache::UnsatCache;
pub use tableau::AlcClassifier;
pub use tableau::{
    classify as tableau_classify, classify_with_seed as tableau_classify_with_seed,
    classify_with_seed_options as tableau_classify_with_seed_options,
    is_ce_intersection_satisfiable_with_seed, is_ce_satisfiable_with_seed,
    is_consistent as tableau_is_consistent,
    is_consistent_with_seed as tableau_is_consistent_with_seed,
    is_named_class_satisfiable_with_cache, is_named_class_satisfiable_with_seed,
    role_expression_subsumes, structural_unsat_classes, TableauSeed,
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
    #[error("profile detection failed: {0}")]
    Profile(String),
    /// General message.
    #[error("{0}")]
    Message(String),
    /// Tableau expansion budget exhausted (incomplete reasoning).
    #[error("tableau expansion budget exhausted ({0} expansions)")]
    ResourceLimit(u32),
}

/// Classify an ontology under ALC tableau semantics.
pub fn classify(ontology: &Ontology) -> Result<Taxonomy> {
    let report = detect_profile(ontology).map_err(|e| Error::Profile(e.to_string()))?;
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
