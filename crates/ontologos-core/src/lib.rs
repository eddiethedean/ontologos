//! Core data model and reasoner API for Ontologos.

mod axiom;
mod entity;
mod error;
mod ontology;
mod reasoner;

pub use axiom::AxiomKind;
pub use entity::{EntityId, EntityKind};
pub use error::{Error, Result};
pub use ontology::Ontology;
pub use reasoner::{Profile, Reasoner, ReasonerBuilder, ReasonerConfig};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ontology_loads_existing_file() {
        let path = std::env::temp_dir().join("ontologos-test.owl");
        std::fs::write(&path, b"<owl/>").expect("write");

        let ontology = Ontology::from_file(&path).expect("load");
        assert_eq!(ontology.entity_count(), 0);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn reasoner_builder_constructs() {
        let ontology = Ontology::default();
        let reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(ontology)
            .expect("build");
        assert_eq!(reasoner.profile(), Profile::El);
    }
}
