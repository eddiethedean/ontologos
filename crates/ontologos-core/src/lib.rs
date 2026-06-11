//! Core data model and reasoner API for OntoLogos.
//!
//! v0.1 provides an in-memory ontology representation with interned IRIs,
//! typed entities, structured axioms, secondary indexes, and JSON v2 serialization.
//! OWL file parsing is available in v0.2 via `ontologos-parser`.
//!
//! # Quick example
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
mod entity;
mod error;
mod graph;
mod iri;
mod limits;
mod ontology;
mod reasoner;
mod serialize;

pub use axiom::{Axiom, AxiomId};
pub use entity::{EntityId, EntityKind, EntityRecord, EntityRegistry};
pub use error::{Error, Result};
pub use graph::{AxiomIndex, AxiomStore};
pub use iri::{InternPool, IriId};
pub use limits::Limits;
pub use ontology::{Ontology, OntologyBuilder};
pub use reasoner::{Profile, Reasoner, ReasonerBuilder, ReasonerConfig};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn classify_returns_not_implemented() {
        let ontology = Ontology::default();
        let reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(ontology)
            .expect("build");
        assert_eq!(reasoner.classify().unwrap_err(), Error::NotImplemented);
        assert_eq!(reasoner.ontology().entity_count(), 0);
    }

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
