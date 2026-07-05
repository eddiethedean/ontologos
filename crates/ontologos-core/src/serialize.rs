use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::axiom::{Axiom, DataLiteral};
use crate::dl::DlStore;
use crate::entity::{EntityId, EntityKind};
use crate::error::{Error, Result};
use crate::limits::Limits;
use crate::ontology::Ontology;
use crate::parse_meta::{ParseMeta, ParseMetaSummary};
use crate::swrl::SwrlRule;

const FORMAT_VERSION: u32 = 4;
const MIN_READ_FORMAT_VERSION: u32 = 2;

fn skip_parse_meta(meta: &Option<ParseMetaSummary>) -> bool {
    meta.as_ref().is_none_or(ParseMetaSummary::omit_from_json)
}

/// JSON snapshot format for ontology round-trip (format version 4).
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OntologySnapshot {
    format_version: u32,
    entities: Vec<SnapshotEntity>,
    axioms: Vec<SnapshotAxiom>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dl: Option<DlStore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    swrl_rules: Option<Vec<SwrlRule>>,
    #[serde(default, skip_serializing_if = "skip_parse_meta")]
    parse_meta: Option<ParseMetaSummary>,
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
    /// Serialize the ontology to a JSON string (format version 4).
    pub fn to_json(&self) -> Result<String> {
        let inferred = self
            .axioms()
            .iter()
            .filter(|(id, _)| self.axioms().is_inferred(*id))
            .count();
        if inferred > 0 {
            tracing::warn!(
                inferred_axiom_count = inferred,
                "JSON export omits inferred axioms; round-trip will not preserve materialized axioms"
            );
        }
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

        Self::reject_duplicate_top_level_keys(json)?;

        let snapshot: OntologySnapshot =
            serde_json::from_str(json).map_err(|e| Error::Serialization(e.to_string()))?;
        if snapshot.format_version == 1 {
            return Err(Error::Serialization(
                "format_version 1 is not supported for untrusted input; use format_version 2 or later"
                    .into(),
            ));
        }
        Self::from_snapshot(snapshot, limits)
    }

    fn reject_duplicate_top_level_keys(json: &str) -> Result<()> {
        let keys = Self::top_level_json_keys(json)?;
        let mut seen = std::collections::HashSet::new();
        for key in keys {
            if !seen.insert(key.clone()) {
                return Err(Error::Serialization(format!(
                    "duplicate top-level key {key:?} in JSON snapshot"
                )));
            }
        }
        Ok(())
    }

    /// Extract top-level object keys from a JSON object (depth 0 only).
    fn top_level_json_keys(json: &str) -> Result<Vec<String>> {
        let trimmed = json.trim_start();
        if !trimmed.starts_with('{') {
            return Err(Error::Serialization(
                "JSON snapshot must be a top-level object".into(),
            ));
        }
        let mut keys = Vec::new();
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;
        let bytes = trimmed.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if in_string {
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            match b {
                b'"' if depth == 1 => {
                    let start = i + 1;
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' {
                            i += 2;
                            continue;
                        }
                        if bytes[i] == b'"' {
                            let key = std::str::from_utf8(&bytes[start..i])
                                .map_err(|e| Error::Serialization(e.to_string()))?;
                            keys.push(key.to_owned());
                            break;
                        }
                        i += 1;
                    }
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                b'"' => in_string = true,
                _ => {}
            }
            i += 1;
        }
        Ok(keys)
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
            .iter_asserted()
            .map(|(_, axiom)| axiom_to_snapshot(axiom, self))
            .collect::<Result<Vec<_>>>()?;
        let dl = if self.dl().axiom_count() > 0
            || self.dl().ce_count() > 0
            || self.dl().de_count() > 0
        {
            Some(self.dl().clone())
        } else {
            None
        };
        let swrl_rules = if self.swrl_rules().is_empty() {
            None
        } else {
            Some(self.swrl_rules().to_vec())
        };
        let parse_meta = self
            .parse_meta()
            .map(ParseMetaSummary::from)
            .filter(|meta| !meta.omit_from_json());
        Ok(OntologySnapshot {
            format_version: FORMAT_VERSION,
            entities,
            axioms,
            dl,
            swrl_rules,
            parse_meta,
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
        let axiom_count = snapshot.axioms.len();

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

        if let Some(dl) = snapshot.dl {
            let dl_axioms = dl.axiom_count();
            let total = axiom_count.saturating_add(dl_axioms);
            if total > limits.max_axioms {
                return Err(Error::ResourceLimit(format!(
                    "combined axiom count exceeds maximum of {}",
                    limits.max_axioms
                )));
            }
            if dl.ce_count() > limits.max_entities || dl.de_count() > limits.max_entities {
                return Err(Error::ResourceLimit(format!(
                    "DL expression count exceeds maximum of {}",
                    limits.max_entities
                )));
            }
            *Arc::make_mut(&mut ontology.dl) = dl;
        }
        if let Some(rules) = snapshot.swrl_rules {
            if rules.len() > limits.max_swrl_rules {
                return Err(Error::ResourceLimit(format!(
                    "SWRL rule count exceeds maximum of {}",
                    limits.max_swrl_rules
                )));
            }
            ontology.swrl_rules = rules;
        }
        if let Some(summary) = snapshot.parse_meta {
            ontology.set_parse_meta(ParseMeta {
                warnings: summary.warnings,
                mapped_axiom_count: summary.mapped_axiom_count,
                skipped_axiom_count: summary.skipped_axiom_count,
                logical_axiom_count: summary.logical_axiom_count,
                ..ParseMeta::default()
            });
        }

        ontology.clear_dirty();
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
        let mut ontology = Ontology::builder()
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
        assert!(json.contains("\"format_version\": 4"));
        ontology.clear_dirty();
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

    #[test]
    fn round_trip_json_preserves_dl_store_and_swrl_rules() {
        use crate::dl::{ClassExpr, DlAxiom};

        let mut ontology = Ontology::builder()
            .class("http://example.org/A")
            .expect("class")
            .class("http://example.org/B")
            .expect("class")
            .build()
            .expect("build");
        let a = ontology.lookup_entity("http://example.org/A").expect("A");
        let b = ontology.lookup_entity("http://example.org/B").expect("B");
        let ce_a = ontology.dl_mut().intern_ce(ClassExpr::Atomic(a));
        let ce_b = ontology.dl_mut().intern_ce(ClassExpr::Atomic(b));
        ontology.dl_mut().push_axiom(DlAxiom::SubClassOf {
            sub: ce_a,
            sup: ce_b,
        });
        ontology
            .push_swrl_rule(crate::swrl::SwrlRule {
                body: vec![crate::swrl::SwrlAtom::Class {
                    class: a,
                    arg: crate::swrl::SwrlIArg::Individual(a),
                }],
                head: vec![crate::swrl::SwrlAtom::Class {
                    class: b,
                    arg: crate::swrl::SwrlIArg::Individual(a),
                }],
            })
            .expect("swrl");

        let json = ontology.to_json().expect("to_json");
        assert!(json.contains("\"dl\""));
        assert!(json.contains("\"swrl_rules\""));
        let restored = Ontology::from_json(&json).expect("from_json");
        assert_eq!(restored.dl().axiom_count(), 1);
        assert_eq!(restored.swrl_rules().len(), 1);
        assert!(!restored.dirty().is_dirty());
    }

    #[test]
    fn to_json_omits_inferred_axioms() {
        let mut ontology = Ontology::builder()
            .class("http://example.org/A")
            .expect("class")
            .class("http://example.org/B")
            .expect("class")
            .class("http://example.org/C")
            .expect("class")
            .subclass_of("http://example.org/A", "http://example.org/B")
            .expect("sub")
            .build()
            .expect("build");
        let c = ontology.lookup_entity("http://example.org/C").expect("C");
        let b = ontology.lookup_entity("http://example.org/B").expect("B");
        ontology
            .add_inferred_axiom(crate::axiom::Axiom::SubClassOf {
                subclass: c,
                superclass: b,
            })
            .expect("inferred");
        assert_eq!(ontology.axiom_count(), 2);
        let json = ontology.to_json().expect("to_json");
        let restored = Ontology::from_json(&json).expect("from_json");
        assert_eq!(restored.axiom_count(), 1);
        assert!(restored.lookup_entity("http://example.org/C").is_some());
    }
}
