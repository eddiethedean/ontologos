use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::iri::IriId;

/// Stable identifier for an ontology entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Opaque entity identifier (index into the entity registry).
pub struct EntityId(pub u32);

impl EntityId {
    /// Zero-based index into the entity registry.
    #[must_use]
    pub fn index(self) -> u32 {
        self.0
    }
}

/// Kind of entity stored in the ontology registry.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    /// OWL class.
    Class,
    /// OWL named individual.
    Individual,
    /// OWL object property.
    ObjectProperty,
    /// OWL data property.
    DataProperty,
    /// OWL annotation property.
    AnnotationProperty,
    /// OWL datatype IRI.
    Datatype,
}

/// A registered ontology entity with its interned IRI and kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRecord {
    /// Interned IRI of the entity.
    pub iri: IriId,
    /// Semantic kind of the entity.
    pub kind: EntityKind,
}

/// Registry mapping interned IRIs to typed entities.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityRegistry {
    entities: Vec<EntityRecord>,
    by_iri: HashMap<IriId, EntityId>,
}

impl EntityRegistry {
    /// Create an empty entity registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of registered entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if no entities are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Look up an entity by IRI id.
    #[must_use]
    pub fn entity_by_iri(&self, iri: IriId) -> Option<EntityId> {
        self.by_iri.get(&iri).copied()
    }

    /// Look up an entity record by id.
    pub fn entity(&self, id: EntityId) -> Result<&EntityRecord> {
        self.entities
            .get(id.0 as usize)
            .ok_or(Error::UnknownEntity(id))
    }

    /// Register a new entity or return the existing id if the IRI is already registered.
    pub fn get_or_register(
        &mut self,
        iri: IriId,
        iri_str: &str,
        kind: EntityKind,
    ) -> Result<EntityId> {
        if let Some(&existing) = self.by_iri.get(&iri) {
            let record = &self.entities[existing.0 as usize];
            if record.kind != kind {
                return Err(Error::EntityKindMismatch {
                    iri: iri_str.to_owned(),
                    expected: kind,
                    found: record.kind,
                });
            }
            return Ok(existing);
        }

        let id = EntityId(
            u32::try_from(self.entities.len())
                .map_err(|_| Error::InvalidAxiom("entity registry capacity exceeded".into()))?,
        );
        self.by_iri.insert(iri, id);
        self.entities.push(EntityRecord { iri, kind });
        Ok(id)
    }

    /// Iterate over all entity records in registration order.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &EntityRecord)> {
        self.entities
            .iter()
            .enumerate()
            .map(|(i, record)| (EntityId(i as u32), record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iri::InternPool;

    #[test]
    fn register_and_lookup() {
        let mut pool = InternPool::new();
        let iri = pool.intern("http://example.org/A").expect("intern");
        let mut registry = EntityRegistry::new();
        let id = registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Class)
            .expect("register");
        assert_eq!(registry.entity_by_iri(iri), Some(id));
        assert_eq!(registry.entity(id).expect("entity").kind, EntityKind::Class);
    }

    #[test]
    fn kind_mismatch_includes_iri_string() {
        let mut pool = InternPool::new();
        let iri = pool.intern("http://example.org/A").expect("intern");
        let mut registry = EntityRegistry::new();
        registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Class)
            .expect("register");
        let err = registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Individual)
            .expect_err("mismatch");
        if let Error::EntityKindMismatch { iri, .. } = err {
            assert_eq!(iri, "http://example.org/A");
        } else {
            panic!("expected EntityKindMismatch");
        }
    }

    #[test]
    fn kind_mismatch_errors() {
        let mut pool = InternPool::new();
        let iri = pool.intern("http://example.org/A").expect("intern");
        let mut registry = EntityRegistry::new();
        registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Class)
            .expect("register");
        let err = registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Individual)
            .expect_err("mismatch");
        assert!(matches!(
            err,
            Error::EntityKindMismatch {
                expected: EntityKind::Individual,
                found: EntityKind::Class,
                ..
            }
        ));
    }

    #[test]
    fn unknown_entity_errors() {
        let registry = EntityRegistry::new();
        let err = registry.entity(EntityId(0)).expect_err("unknown");
        assert_eq!(err, Error::UnknownEntity(EntityId(0)));
    }

    #[test]
    fn reregister_same_kind_returns_stable_id() {
        let mut pool = InternPool::new();
        let iri = pool.intern("http://example.org/A").expect("intern");
        let mut registry = EntityRegistry::new();
        let first = registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Class)
            .expect("register");
        let second = registry
            .get_or_register(iri, "http://example.org/A", EntityKind::Class)
            .expect("register");
        assert_eq!(first, second);
    }
}
