//! Core data model and reasoner API for OntoLogos.
//!
//! v0.4 provides an in-memory ontology representation with interned IRIs,
//! typed entities, structured axioms, secondary indexes, and JSON v3 serialization
//! (v2 read supported).
//!
//! # Start here — load and reason
//!
//! **Do not use [`Ontology::from_file`] for file loading.** Use `ontologos_parser::load_ontology` and `ontologos_facade::classify` for reasoning.
//!
//! ```ignore
//! use ontologos_parser::load_ontology;
//! use ontologos_rdfs::RdfsEngine;
//!
//! let mut ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
//! let report = RdfsEngine::new().materialize(&mut ontology)?;
//! println!("inferred {}", report.inferred_total());
//! ```
//!
//! OWL RL saturation: `ontologos_rl::RlEngine::new(1)?.saturate(&mut ontology)?`
//!
//! # Builder-only (no parser)
//!
//! ```
//! use ontologos_core::{Error, Ontology};
//!
//! fn main() -> Result<(), Error> {
//!     let ontology = Ontology::builder()
//!         .class("http://example.org/Pizza")?
//!         .class("http://example.org/Food")?
//!         .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
//!         .build()?;
//!     assert_eq!(ontology.axiom_count(), 1);
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

mod axiom;
mod consistency;
mod dirty;
mod dl;
mod engine;
mod entity;
mod error;
mod graph;
mod iri;
mod limits;
mod ontology;
mod parse_meta;
mod reasoner;
mod serialize;
mod session;
mod swrl;
mod taxonomy;
mod trace;

pub use axiom::{Axiom, AxiomId, DataLiteral};
pub use consistency::ConsistencyResult;
pub use dirty::{DirtySet, OntologyRevision, axiom_signature};
pub use dl::{CeId, ClassExpr, DataExpr, DeId, DlAxiom, DlStore, RoleExpr};
pub use engine::{DetectedProfileKind, EngineKind, ResolvedRoute, uses_dl_entailment};
pub use entity::{EntityId, EntityKind, EntityRecord, EntityRegistry};
pub use error::{Error, Result};
pub use graph::{AxiomIndex, AxiomStore};
pub use iri::{
    InternPool, IriId, validate_iri, validate_snapshot_iri, validate_snapshot_iri_with_max_len,
};
pub use limits::Limits;
pub use ontology::{Ontology, OntologyBuilder};
pub use parse_meta::{OwlConstruct, ParseMeta, ParseMetaSummary};
pub use reasoner::{Profile, Reasoner, ReasonerBuilder, ReasonerConfig};
pub use session::ReasonerSession;
pub use swrl::{SwrlAtom, SwrlDArg, SwrlIArg, SwrlRule};
pub use taxonomy::Taxonomy;
pub use trace::{InferenceTrace, TraceConclusion, TracePremise, TraceStep};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn reasoner_rejects_invalid_parallelism() {
        let ontology = Ontology::default();
        let err = Reasoner::builder()
            .config(ReasonerConfig {
                parallelism: 0,
                ..ReasonerConfig::default()
            })
            .build(ontology)
            .expect_err("parallelism");
        assert!(matches!(err, Error::Message(_)));
    }
}
