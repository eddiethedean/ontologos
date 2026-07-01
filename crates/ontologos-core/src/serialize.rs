use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::axiom::{Axiom, DataLiteral};
use crate::entity::{EntityId, EntityKind};
use crate::error::{Error, Result};
use crate::limits::Limits;
use crate::ontology::Ontology;

const FORMAT_VERSION: u32 = 3;
const MIN_READ_FORMAT_VERSION: u32 = 2;

/// JSON snapshot format for ontology round-trip (format version 2).
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OntologySnapshot {
    format_version: u32,
    entities: Vec<SnapshotEntity>,
    axioms: Vec<SnapshotAxiom>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotEntity {
    iri: String,
    kind: EntityKind,
}

/// Axiom representation using IRI strings in JSON.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
enum SnapshotAxiom {
    SubClassOf {
        subclass: String,
        superclass: String,
    },
    EquivalentClasses(Vec<String>),
    DisjointClasses(Vec<String>),
    ObjectPropertyDomain {
        property: String,
        domain: String,
    },
    ObjectPropertyRange {
        property: String,
        range: String,
    },
    SubObjectPropertyOf {
        sub_property: String,
        super_property: String,
    },
    InverseObjectProperties {
        left: String,
        right: String,
    },
    TransitiveObjectProperty(String),
    SubClassOfExistential {
        subclass: String,
        property: String,
        filler: String,
    },
    SymmetricObjectProperty(String),
    ReflexiveObjectProperty(String),
    FunctionalObjectProperty(String),
    InverseFunctionalObjectProperty(String),
    IrreflexiveObjectProperty(String),
    AsymmetricObjectProperty(String),
    EquivalentObjectProperties(Vec<String>),
    ClassAssertion {
        individual: String,
        class: String,
    },
    ObjectPropertyAssertion {
        subject: String,
        property: String,
        object: String,
    },
    DataPropertyAssertion {
        individual: String,
        property: String,
        value: DataLiteral,
    },
    NegativeObjectPropertyAssertion {
        subject: String,
        property: String,
        object: String,
    },
    NegativeDataPropertyAssertion {
        individual: String,
        property: String,
        value: DataLiteral,
    },
    SameIndividual(Vec<String>),
    DifferentIndividuals(Vec<String>),
}

impl Ontology {
    /// Serialize the ontology to a JSON string (format version 2).
    pub fn to_json(&self) -> Result<String> {
        let snapshot = self.to_snapshot()?;
        serde_json::to_string_pretty(&snapshot).map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Deserialize an ontology from a JSON string using default [`Limits`](crate::Limits).
    ///
    /// Accepts format version 2 only. Format v1 is rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use ontologos_core::Ontology;
    ///
    /// let json = r#"{
    ///     "format_version": 2,
    ///     "entities": [
    ///         {"iri": "http://example.org/A", "kind": "Class"},
    ///         {"iri": "http://example.org/B", "kind": "Class"}
    ///     ],
    ///     "axioms": [
    ///         {"SubClassOf": {"subclass": "http://example.org/A", "superclass": "http://example.org/B"}}
    ///     ]
    /// }"#;
    /// let ontology = Ontology::from_json(json).expect("load");
    /// assert_eq!(ontology.axiom_count(), 1);
    /// ```
    pub fn from_json(json: &str) -> Result<Self> {
        Self::from_json_with_limits(json, Limits::default())
    }

    /// Deserialize an ontology with custom resource [`Limits`](crate::Limits).
    ///
    /// Prefer this over [`from_json`](Self::from_json) for untrusted input.
    pub fn from_json_with_limits(json: &str, limits: Limits) -> Result<Self> {
        if json.len() > limits.max_json_bytes {
            return Err(Error::Serialization(format!(
                "JSON input exceeds maximum size of {} bytes",
                limits.max_json_bytes
            )));
        }

        if json.contains("\"format_version\": 1") || json.contains("\"format_version\":1") {
            return Err(Error::Serialization(
                "format_version 1 is not supported for untrusted input; use format_version 2"
                    .into(),
            ));
        }

        Self::reject_duplicate_top_level_keys(json)?;

        let snapshot: OntologySnapshot =
            serde_json::from_str(json).map_err(|e| Error::Serialization(e.to_string()))?;
        Self::from_snapshot(snapshot, limits)
    }

    fn reject_duplicate_top_level_keys(json: &str) -> Result<()> {
        for key in ["format_version", "entities", "axioms"] {
            let needle = format!("\"{key}\"");
            if json.match_indices(&needle).count() > 1 {
                return Err(Error::Serialization(format!(
                    "duplicate top-level key {key:?} in JSON snapshot"
                )));
            }
        }
        Ok(())
    }

    fn to_snapshot(&self) -> Result<OntologySnapshot> {
        let entities = self
            .entities
            .iter()
            .map(|(_, record)| -> Result<SnapshotEntity> {
                Ok(SnapshotEntity {
                    iri: self.iris.resolve(record.iri)?.to_owned(),
                    kind: record.kind,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let axioms = self
            .axioms
            .iter()
            .map(|(_, axiom)| axiom_to_snapshot(axiom, self))
            .collect::<Result<Vec<_>>>()?;
        Ok(OntologySnapshot {
            format_version: FORMAT_VERSION,
            entities,
            axioms,
        })
    }

    fn from_snapshot(snapshot: OntologySnapshot, limits: Limits) -> Result<Self> {
        if snapshot.format_version < MIN_READ_FORMAT_VERSION
            || snapshot.format_version > FORMAT_VERSION
        {
            return Err(Error::Serialization(format!(
                "unsupported format_version: {}",
                snapshot.format_version
            )));
        }

        if snapshot.entities.len() > limits.max_entities {
            return Err(Error::ResourceLimit(format!(
                "entity count exceeds maximum of {}",
                limits.max_entities
            )));
        }

        if snapshot.axioms.len() > limits.max_axioms {
            return Err(Error::ResourceLimit(format!(
                "axiom count exceeds maximum of {}",
                limits.max_axioms
            )));
        }

        let mut ontology = Self::new();
        let mut seen_iris = std::collections::HashSet::new();

        for entity in snapshot.entities {
            if !seen_iris.insert(entity.iri.clone()) {
                return Err(Error::Serialization(format!(
                    "duplicate entity IRI in snapshot: {}",
                    entity.iri
                )));
            }
            crate::iri::validate_snapshot_iri_with_max_len(&entity.iri, limits.max_iri_len)?;
            let iri_id = Arc::make_mut(&mut ontology.iris)
                .intern_with_limit(&entity.iri, limits.max_iri_len)?;
            let iri_str = ontology.iris.resolve(iri_id)?;
            Arc::make_mut(&mut ontology.entities).get_or_register(iri_id, iri_str, entity.kind)?;
        }

        for axiom in snapshot.axioms {
            let axiom = snapshot_axiom_to_axiom(&axiom, &ontology)?;
            axiom.validate_with_limits(&ontology.entities, limits)?;
            ontology.add_axiom(axiom)?;
        }

        Ok(ontology)
    }
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Result<String> {
    let record = ontology.entity(id)?;
    Ok(ontology.iris.resolve(record.iri)?.to_owned())
}

fn resolve_entity(ontology: &Ontology, iri: &str) -> Result<EntityId> {
    ontology
        .try_lookup_entity(iri)?
        .ok_or_else(|| Error::InvalidAxiom(format!("unknown entity IRI in axiom: {iri}")))
}

fn axiom_to_snapshot(axiom: &Axiom, ontology: &Ontology) -> Result<SnapshotAxiom> {
    Ok(match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => SnapshotAxiom::SubClassOf {
            subclass: entity_iri(ontology, *subclass)?,
            superclass: entity_iri(ontology, *superclass)?,
        },
        Axiom::EquivalentClasses(classes) => SnapshotAxiom::EquivalentClasses(
            classes
                .iter()
                .map(|id| entity_iri(ontology, *id))
                .collect::<Result<Vec<_>>>()?,
        ),
        Axiom::DisjointClasses(classes) => SnapshotAxiom::DisjointClasses(
            classes
                .iter()
                .map(|id| entity_iri(ontology, *id))
                .collect::<Result<Vec<_>>>()?,
        ),
        Axiom::ObjectPropertyDomain { property, domain } => SnapshotAxiom::ObjectPropertyDomain {
            property: entity_iri(ontology, *property)?,
            domain: entity_iri(ontology, *domain)?,
        },
        Axiom::ObjectPropertyRange { property, range } => SnapshotAxiom::ObjectPropertyRange {
            property: entity_iri(ontology, *property)?,
            range: entity_iri(ontology, *range)?,
        },
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => SnapshotAxiom::SubObjectPropertyOf {
            sub_property: entity_iri(ontology, *sub_property)?,
            super_property: entity_iri(ontology, *super_property)?,
        },
        Axiom::InverseObjectProperties { left, right } => SnapshotAxiom::InverseObjectProperties {
            left: entity_iri(ontology, *left)?,
            right: entity_iri(ontology, *right)?,
        },
        Axiom::TransitiveObjectProperty(property) => {
            SnapshotAxiom::TransitiveObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => SnapshotAxiom::SubClassOfExistential {
            subclass: entity_iri(ontology, *subclass)?,
            property: entity_iri(ontology, *property)?,
            filler: entity_iri(ontology, *filler)?,
        },
        Axiom::SymmetricObjectProperty(property) => {
            SnapshotAxiom::SymmetricObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::ReflexiveObjectProperty(property) => {
            SnapshotAxiom::ReflexiveObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::FunctionalObjectProperty(property) => {
            SnapshotAxiom::FunctionalObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::InverseFunctionalObjectProperty(property) => {
            SnapshotAxiom::InverseFunctionalObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::IrreflexiveObjectProperty(property) => {
            SnapshotAxiom::IrreflexiveObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::AsymmetricObjectProperty(property) => {
            SnapshotAxiom::AsymmetricObjectProperty(entity_iri(ontology, *property)?)
        }
        Axiom::EquivalentObjectProperties(properties) => SnapshotAxiom::EquivalentObjectProperties(
            properties
                .iter()
                .map(|id| entity_iri(ontology, *id))
                .collect::<Result<Vec<_>>>()?,
        ),
        Axiom::ClassAssertion { individual, class } => SnapshotAxiom::ClassAssertion {
            individual: entity_iri(ontology, *individual)?,
            class: entity_iri(ontology, *class)?,
        },
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => SnapshotAxiom::ObjectPropertyAssertion {
            subject: entity_iri(ontology, *subject)?,
            property: entity_iri(ontology, *property)?,
            object: entity_iri(ontology, *object)?,
        },
        Axiom::DataPropertyAssertion {
            individual,
            property,
            value,
        } => SnapshotAxiom::DataPropertyAssertion {
            individual: entity_iri(ontology, *individual)?,
            property: entity_iri(ontology, *property)?,
            value: value.clone(),
        },
        Axiom::NegativeObjectPropertyAssertion {
            subject,
            property,
            object,
        } => SnapshotAxiom::NegativeObjectPropertyAssertion {
            subject: entity_iri(ontology, *subject)?,
            property: entity_iri(ontology, *property)?,
            object: entity_iri(ontology, *object)?,
        },
        Axiom::NegativeDataPropertyAssertion {
            individual,
            property,
            value,
        } => SnapshotAxiom::NegativeDataPropertyAssertion {
            individual: entity_iri(ontology, *individual)?,
            property: entity_iri(ontology, *property)?,
            value: value.clone(),
        },
        Axiom::SameIndividual(individuals) => SnapshotAxiom::SameIndividual(
            individuals
                .iter()
                .map(|id| entity_iri(ontology, *id))
                .collect::<Result<Vec<_>>>()?,
        ),
        Axiom::DifferentIndividuals(individuals) => SnapshotAxiom::DifferentIndividuals(
            individuals
                .iter()
                .map(|id| entity_iri(ontology, *id))
                .collect::<Result<Vec<_>>>()?,
        ),
    })
}

fn snapshot_axiom_to_axiom(snapshot: &SnapshotAxiom, ontology: &Ontology) -> Result<Axiom> {
    Ok(match snapshot {
        SnapshotAxiom::SubClassOf {
            subclass,
            superclass,
        } => Axiom::SubClassOf {
            subclass: resolve_entity(ontology, subclass)?,
            superclass: resolve_entity(ontology, superclass)?,
        },
        SnapshotAxiom::EquivalentClasses(classes) => Axiom::EquivalentClasses(
            classes
                .iter()
                .map(|iri| resolve_entity(ontology, iri))
                .collect::<Result<Vec<_>>>()?,
        ),
        SnapshotAxiom::DisjointClasses(classes) => Axiom::DisjointClasses(
            classes
                .iter()
                .map(|iri| resolve_entity(ontology, iri))
                .collect::<Result<Vec<_>>>()?,
        ),
        SnapshotAxiom::ObjectPropertyDomain { property, domain } => Axiom::ObjectPropertyDomain {
            property: resolve_entity(ontology, property)?,
            domain: resolve_entity(ontology, domain)?,
        },
        SnapshotAxiom::ObjectPropertyRange { property, range } => Axiom::ObjectPropertyRange {
            property: resolve_entity(ontology, property)?,
            range: resolve_entity(ontology, range)?,
        },
        SnapshotAxiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => Axiom::SubObjectPropertyOf {
            sub_property: resolve_entity(ontology, sub_property)?,
            super_property: resolve_entity(ontology, super_property)?,
        },
        SnapshotAxiom::InverseObjectProperties { left, right } => Axiom::InverseObjectProperties {
            left: resolve_entity(ontology, left)?,
            right: resolve_entity(ontology, right)?,
        },
        SnapshotAxiom::TransitiveObjectProperty(property) => {
            Axiom::TransitiveObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => Axiom::SubClassOfExistential {
            subclass: resolve_entity(ontology, subclass)?,
            property: resolve_entity(ontology, property)?,
            filler: resolve_entity(ontology, filler)?,
        },
        SnapshotAxiom::SymmetricObjectProperty(property) => {
            Axiom::SymmetricObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::ReflexiveObjectProperty(property) => {
            Axiom::ReflexiveObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::FunctionalObjectProperty(property) => {
            Axiom::FunctionalObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::InverseFunctionalObjectProperty(property) => {
            Axiom::InverseFunctionalObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::IrreflexiveObjectProperty(property) => {
            Axiom::IrreflexiveObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::AsymmetricObjectProperty(property) => {
            Axiom::AsymmetricObjectProperty(resolve_entity(ontology, property)?)
        }
        SnapshotAxiom::EquivalentObjectProperties(properties) => Axiom::EquivalentObjectProperties(
            properties
                .iter()
                .map(|iri| resolve_entity(ontology, iri))
                .collect::<Result<Vec<_>>>()?,
        ),
        SnapshotAxiom::ClassAssertion { individual, class } => Axiom::ClassAssertion {
            individual: resolve_entity(ontology, individual)?,
            class: resolve_entity(ontology, class)?,
        },
        SnapshotAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => Axiom::ObjectPropertyAssertion {
            subject: resolve_entity(ontology, subject)?,
            property: resolve_entity(ontology, property)?,
            object: resolve_entity(ontology, object)?,
        },
        SnapshotAxiom::DataPropertyAssertion {
            individual,
            property,
            value,
        } => Axiom::DataPropertyAssertion {
            individual: resolve_entity(ontology, individual)?,
            property: resolve_entity(ontology, property)?,
            value: value.clone(),
        },
        SnapshotAxiom::NegativeObjectPropertyAssertion {
            subject,
            property,
            object,
        } => Axiom::NegativeObjectPropertyAssertion {
            subject: resolve_entity(ontology, subject)?,
            property: resolve_entity(ontology, property)?,
            object: resolve_entity(ontology, object)?,
        },
        SnapshotAxiom::NegativeDataPropertyAssertion {
            individual,
            property,
            value,
        } => Axiom::NegativeDataPropertyAssertion {
            individual: resolve_entity(ontology, individual)?,
            property: resolve_entity(ontology, property)?,
            value: value.clone(),
        },
        SnapshotAxiom::SameIndividual(individuals) => Axiom::SameIndividual(
            individuals
                .iter()
                .map(|iri| resolve_entity(ontology, iri))
                .collect::<Result<Vec<_>>>()?,
        ),
        SnapshotAxiom::DifferentIndividuals(individuals) => Axiom::DifferentIndividuals(
            individuals
                .iter()
                .map(|iri| resolve_entity(ontology, iri))
                .collect::<Result<Vec<_>>>()?,
        ),
    })
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
        assert!(json.contains("\"format_version\": 3"));
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
    fn rejects_format_version_1() {
        let json = r#"{
            "format_version": 1,
            "iris": ["http://example.org/A"],
            "entities": [{"iri_index": 1, "kind": "Class"}],
            "axioms": []
        }"#;
        let err = Ontology::from_json(json).expect_err("v1");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_unsupported_format_version() {
        let json = r#"{"format_version":99,"entities":[],"axioms":[]}"#;
        let err = Ontology::from_json(json).expect_err("version");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_invalid_json_syntax() {
        let err = Ontology::from_json("{not json").expect_err("json");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_unknown_entity_iri_in_axiom() {
        let json = r#"{
            "format_version": 2,
            "entities": [
                {"iri": "http://example.org/A", "kind": "Class"},
                {"iri": "http://example.org/B", "kind": "Class"}
            ],
            "axioms": [
                {"SubClassOf": {"subclass": "http://example.org/A", "superclass": "http://example.org/Missing"}}
            ]
        }"#;
        let err = Ontology::from_json(json).expect_err("entity");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn rejects_duplicate_entity_iris() {
        let json = r#"{
            "format_version": 2,
            "entities": [
                {"iri": "http://example.org/A", "kind": "Class"},
                {"iri": "http://example.org/A", "kind": "Class"}
            ],
            "axioms": []
        }"#;
        let err = Ontology::from_json(json).expect_err("dup");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_unknown_snapshot_fields() {
        let json = r#"{
            "format_version": 2,
            "entitys": [],
            "entities": [],
            "axioms": []
        }"#;
        let err = Ontology::from_json(json).expect_err("typo");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_oversized_json() {
        let limits = Limits {
            max_json_bytes: 10,
            ..Limits::default()
        };
        let json = r#"{"format_version":2,"entities":[],"axioms":[]}"#;
        let err = Ontology::from_json_with_limits(json, limits).expect_err("size");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn rejects_unknown_axiom_fields() {
        let json = r#"{
            "format_version": 2,
            "entities": [
                {"iri": "http://example.org/A", "kind": "Class"},
                {"iri": "http://example.org/B", "kind": "Class"}
            ],
            "axioms": [
                {"SubClassOf": {"subclass": "http://example.org/A", "superclass": "http://example.org/B", "extra": true}}
            ]
        }"#;
        let err = Ontology::from_json(json).expect_err("unknown axiom field");
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn round_trip_subclass_of_existential() {
        let ontology = Ontology::builder()
            .class("http://example.org/C")
            .expect("class")
            .class("http://example.org/B")
            .expect("class")
            .object_property("http://example.org/hasPart")
            .expect("property")
            .build()
            .expect("build");
        let mut ontology = ontology;
        let c = ontology.lookup_entity("http://example.org/C").expect("C");
        let b = ontology.lookup_entity("http://example.org/B").expect("B");
        let has_part = ontology
            .lookup_entity("http://example.org/hasPart")
            .expect("hasPart");
        ontology
            .add_axiom(crate::axiom::Axiom::SubClassOfExistential {
                subclass: c,
                property: has_part,
                filler: b,
            })
            .expect("axiom");

        let json = ontology.to_json().expect("to_json");
        let restored = Ontology::from_json(&json).expect("from_json");
        assert_eq!(restored.axiom_count(), 1);
        assert!(restored.direct_superclasses(c).is_empty());
        assert_eq!(restored.existentials_of(c), &[(has_part, b)]);
    }

    #[test]
    fn round_trip_abox_axiom_variants() {
        let ontology = Ontology::builder()
            .individual("http://example.org/alice")
            .expect("alice")
            .individual("http://example.org/bob")
            .expect("bob")
            .class("http://example.org/Person")
            .expect("Person")
            .object_property("http://example.org/knows")
            .expect("knows")
            .class_assertion("http://example.org/alice", "http://example.org/Person")
            .expect("type")
            .object_property_assertion(
                "http://example.org/alice",
                "http://example.org/knows",
                "http://example.org/bob",
            )
            .expect("assertion")
            .same_individual(&["http://example.org/alice", "http://example.org/bob"])
            .expect("same")
            .build()
            .expect("build");

        let json = ontology.to_json().expect("to_json");
        let restored = Ontology::from_json(&json).expect("from_json");
        assert_eq!(restored.axiom_count(), 3);
        let alice = restored
            .lookup_entity("http://example.org/alice")
            .expect("alice");
        let person = restored
            .lookup_entity("http://example.org/Person")
            .expect("Person");
        assert_eq!(restored.classes_of(alice), &[person]);
    }

    #[test]
    fn round_trip_rl_property_axiom_variants() {
        let mut ontology = Ontology::builder()
            .object_property("http://example.org/symmetric")
            .expect("symmetric")
            .object_property("http://example.org/reflexive")
            .expect("reflexive")
            .object_property("http://example.org/functional")
            .expect("functional")
            .build()
            .expect("build");

        let symmetric = ontology
            .lookup_entity("http://example.org/symmetric")
            .expect("symmetric");
        let reflexive = ontology
            .lookup_entity("http://example.org/reflexive")
            .expect("reflexive");
        let functional = ontology
            .lookup_entity("http://example.org/functional")
            .expect("functional");

        ontology
            .add_axiom(crate::axiom::Axiom::SymmetricObjectProperty(symmetric))
            .expect("symmetric");
        ontology
            .add_axiom(crate::axiom::Axiom::ReflexiveObjectProperty(reflexive))
            .expect("reflexive");
        ontology
            .add_axiom(crate::axiom::Axiom::FunctionalObjectProperty(functional))
            .expect("functional");

        let json = ontology.to_json().expect("to_json");
        let restored = Ontology::from_json(&json).expect("from_json");
        assert_eq!(restored.axiom_count(), 3);
        assert_eq!(restored.index().by_kind("SymmetricObjectProperty").len(), 1);
        assert_eq!(restored.index().by_kind("ReflexiveObjectProperty").len(), 1);
        assert_eq!(
            restored.index().by_kind("FunctionalObjectProperty").len(),
            1
        );
    }
}
