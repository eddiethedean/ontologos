use serde::{Deserialize, Serialize};

use crate::entity::{EntityId, EntityKind, EntityRegistry};
use crate::error::{Error, Result};

/// Stable identifier for a stored axiom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxiomId(pub u32);

impl AxiomId {
    /// Zero-based index into the axiom store.
    #[must_use]
    pub fn index(self) -> u32 {
        self.0
    }
}

/// Supported axiom types in the 1.x reasoner.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axiom {
    /// Class subsumption: subclass ⊑ superclass.
    SubClassOf {
        /// The subsumed class.
        subclass: EntityId,
        /// The subsumer class.
        superclass: EntityId,
    },
    /// Class equivalence.
    EquivalentClasses(Vec<EntityId>),
    /// Class disjointness.
    DisjointClasses(Vec<EntityId>),
    /// Object property domain.
    ObjectPropertyDomain {
        /// The object property.
        property: EntityId,
        /// The domain class.
        domain: EntityId,
    },
    /// Object property range.
    ObjectPropertyRange {
        /// The object property.
        property: EntityId,
        /// The range class.
        range: EntityId,
    },
    /// Object property subsumption.
    SubObjectPropertyOf {
        /// The subsumed property.
        sub_property: EntityId,
        /// The subsumer property.
        super_property: EntityId,
    },
    /// Inverse object properties.
    InverseObjectProperties {
        /// One property in the inverse pair.
        left: EntityId,
        /// The other property in the inverse pair.
        right: EntityId,
    },
    /// Transitive object property declaration.
    TransitiveObjectProperty(EntityId),
}

impl Axiom {
    /// Validate entity references and kinds for this axiom.
    pub fn validate(&self, registry: &EntityRegistry) -> Result<()> {
        match self {
            Self::SubClassOf {
                subclass,
                superclass,
            } => {
                require_kind(registry, *subclass, EntityKind::Class, "subclass")?;
                require_kind(registry, *superclass, EntityKind::Class, "superclass")?;
            }
            Self::EquivalentClasses(classes) | Self::DisjointClasses(classes) => {
                if classes.len() < 2 {
                    return Err(Error::InvalidAxiom(
                        "equivalent/disjoint classes require at least two operands".into(),
                    ));
                }
                for id in classes {
                    require_kind(registry, *id, EntityKind::Class, "class operand")?;
                }
            }
            Self::ObjectPropertyDomain { property, domain } => {
                require_kind(registry, *property, EntityKind::ObjectProperty, "property")?;
                require_kind(registry, *domain, EntityKind::Class, "domain")?;
            }
            Self::ObjectPropertyRange { property, range } => {
                require_kind(registry, *property, EntityKind::ObjectProperty, "property")?;
                require_kind(registry, *range, EntityKind::Class, "range")?;
            }
            Self::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                require_kind(
                    registry,
                    *sub_property,
                    EntityKind::ObjectProperty,
                    "sub_property",
                )?;
                require_kind(
                    registry,
                    *super_property,
                    EntityKind::ObjectProperty,
                    "super_property",
                )?;
            }
            Self::InverseObjectProperties { left, right } => {
                require_kind(registry, *left, EntityKind::ObjectProperty, "left property")?;
                require_kind(
                    registry,
                    *right,
                    EntityKind::ObjectProperty,
                    "right property",
                )?;
            }
            Self::TransitiveObjectProperty(property) => {
                require_kind(registry, *property, EntityKind::ObjectProperty, "property")?;
            }
        }
        Ok(())
    }

    /// Discriminator string for axiom indexing (profile detection).
    #[must_use]
    pub fn kind_tag(&self) -> &'static str {
        match self {
            Self::SubClassOf { .. } => "SubClassOf",
            Self::EquivalentClasses(_) => "EquivalentClasses",
            Self::DisjointClasses(_) => "DisjointClasses",
            Self::ObjectPropertyDomain { .. } => "ObjectPropertyDomain",
            Self::ObjectPropertyRange { .. } => "ObjectPropertyRange",
            Self::SubObjectPropertyOf { .. } => "SubObjectPropertyOf",
            Self::InverseObjectProperties { .. } => "InverseObjectProperties",
            Self::TransitiveObjectProperty(_) => "TransitiveObjectProperty",
        }
    }
}

fn require_kind(
    registry: &EntityRegistry,
    id: EntityId,
    expected: EntityKind,
    role: &str,
) -> Result<()> {
    let record = registry.entity(id)?;
    if record.kind != expected {
        return Err(Error::InvalidAxiom(format!(
            "{role} entity {:?} must be {expected:?}, found {:?}",
            id, record.kind
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iri::InternPool;

    fn class(registry: &mut EntityRegistry, pool: &mut InternPool, iri: &str) -> EntityId {
        let iri_id = pool.intern(iri).expect("intern");
        registry
            .get_or_register(iri_id, EntityKind::Class)
            .expect("register")
    }

    #[test]
    fn validates_subclass_of() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let a = class(&mut registry, &mut pool, "http://ex.org/A");
        let b = class(&mut registry, &mut pool, "http://ex.org/B");
        let axiom = Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        };
        axiom.validate(&registry).expect("valid");
    }

    #[test]
    fn rejects_wrong_kind() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let iri = pool.intern("http://ex.org/p").expect("intern");
        let prop = registry
            .get_or_register(iri, EntityKind::ObjectProperty)
            .expect("register");
        let axiom = Axiom::SubClassOf {
            subclass: prop,
            superclass: prop,
        };
        assert!(axiom.validate(&registry).is_err());
    }
}
