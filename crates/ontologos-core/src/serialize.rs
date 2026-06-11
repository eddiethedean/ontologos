use serde::{Deserialize, Serialize};

use crate::axiom::Axiom;
use crate::entity::EntityKind;
use crate::error::{Error, Result};
use crate::iri::IriId;
use crate::ontology::Ontology;

const FORMAT_VERSION: u32 = 1;

/// JSON snapshot format for ontology round-trip.
#[derive(Debug, Serialize, Deserialize)]
struct OntologySnapshot {
    format_version: u32,
    iris: Vec<String>,
    entities: Vec<SnapshotEntity>,
    axioms: Vec<Axiom>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotEntity {
    iri_index: u32,
    kind: EntityKind,
}

impl Ontology {
    /// Serialize the ontology to a JSON string.
    pub fn to_json(&self) -> Result<String> {
        let snapshot = self.to_snapshot()?;
        serde_json::to_string_pretty(&snapshot).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Deserialize an ontology from a JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        let snapshot: OntologySnapshot =
            serde_json::from_str(json).map_err(|e| Error::Serialization(e.to_string()))?;
        Self::from_snapshot(snapshot)
    }

    fn to_snapshot(&self) -> Result<OntologySnapshot> {
        let iris: Vec<String> = self.iris.iter().map(str::to_owned).collect();
        let entities = self
            .entities
            .iter()
            .map(|(_, record)| SnapshotEntity {
                iri_index: record.iri.index(),
                kind: record.kind,
            })
            .collect();
        let axioms = self.axioms.iter().map(|(_, axiom)| axiom.clone()).collect();
        Ok(OntologySnapshot {
            format_version: FORMAT_VERSION,
            iris,
            entities,
            axioms,
        })
    }

    fn from_snapshot(snapshot: OntologySnapshot) -> Result<Self> {
        if snapshot.format_version != FORMAT_VERSION {
            return Err(Error::Serialization(format!(
                "unsupported format_version: {}",
                snapshot.format_version
            )));
        }

        let mut ontology = Self::new();

        // Pre-intern IRIs in order so indices match.
        for iri in &snapshot.iris {
            ontology.iris.intern(iri)?;
        }

        if ontology.iris.len() != snapshot.iris.len() {
            return Err(Error::Serialization(
                "IRI table length mismatch after interning".into(),
            ));
        }

        for entity in snapshot.entities {
            let iri = IriId::from_index(entity.iri_index);
            if ontology.iris.resolve(iri).is_err() {
                return Err(Error::Serialization(format!(
                    "entity references unknown iri_index: {}",
                    entity.iri_index
                )));
            }
            ontology.entities.get_or_register(iri, entity.kind)?;
        }

        for axiom in snapshot.axioms {
            ontology.add_axiom(axiom)?;
        }

        Ok(ontology)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_json_preserves_semantics() {
        let ontology = Ontology::builder()
            .class("http://example.org/Pizza")
            .expect("class")
            .class("http://example.org/Food")
            .expect("class")
            .object_property("http://example.org/hasTopping")
            .expect("property")
            .class("http://example.org/Topping")
            .expect("class")
            .subclass_of("http://example.org/Pizza", "http://example.org/Food")
            .expect("subclass")
            .build()
            .expect("build");

        let json = ontology.to_json().expect("to_json");
        let restored = Ontology::from_json(&json).expect("from_json");
        assert_eq!(restored, ontology);

        let pizza = restored
            .lookup_entity("http://example.org/Pizza")
            .expect("pizza");
        let food = restored
            .lookup_entity("http://example.org/Food")
            .expect("food");
        assert_eq!(restored.direct_superclasses(pizza), &[food]);
    }

    #[test]
    fn rejects_unsupported_format_version() {
        let json = r#"{"format_version":99,"iris":[],"entities":[],"axioms":[]}"#;
        let err = Ontology::from_json(json).expect_err("version");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_invalid_json_syntax() {
        let err = Ontology::from_json("{not json").expect_err("json");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_unknown_iri_index() {
        let json = r#"{
            "format_version": 1,
            "iris": ["http://example.org/A"],
            "entities": [{"iri_index": 99, "kind": "Class"}],
            "axioms": []
        }"#;
        let err = Ontology::from_json(json).expect_err("iri");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_axiom_with_unknown_entity() {
        let json = r#"{
            "format_version": 1,
            "iris": ["http://example.org/A", "http://example.org/B"],
            "entities": [
                {"iri_index": 1, "kind": "Class"},
                {"iri_index": 2, "kind": "Class"}
            ],
            "axioms": [
                {"SubClassOf": {"subclass": 0, "superclass": 99}}
            ]
        }"#;
        let err = Ontology::from_json(json).expect_err("entity");
        assert!(
            matches!(err, Error::InvalidAxiom(_) | Error::UnknownEntity(_)),
            "unexpected error: {err:?}"
        );
    }
}
