use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::entity::{EntityId, EntityKind, EntityRegistry};
use crate::error::{Error, Result};
use crate::limits::Limits;

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
    /// EL pattern: `SubClassOf(C, ObjectSomeValuesFrom(p, D))` with named class `D`.
    SubClassOfExistential {
        /// The subclass.
        subclass: EntityId,
        /// The object property.
        property: EntityId,
        /// The filler class.
        filler: EntityId,
    },
    /// Symmetric object property declaration.
    SymmetricObjectProperty(EntityId),
    /// Reflexive object property declaration.
    ReflexiveObjectProperty(EntityId),
    /// Functional object property declaration.
    FunctionalObjectProperty(EntityId),
    /// Inverse-functional object property declaration.
    InverseFunctionalObjectProperty(EntityId),
    /// Irreflexive object property declaration.
    IrreflexiveObjectProperty(EntityId),
    /// Asymmetric object property declaration.
    AsymmetricObjectProperty(EntityId),
    /// Named object property equivalence.
    EquivalentObjectProperties(Vec<EntityId>),
    /// `ClassAssertion` / `rdf:type` for a named individual.
    ClassAssertion {
        /// The individual.
        individual: EntityId,
        /// The asserted class.
        class: EntityId,
    },
    /// `ObjectPropertyAssertion` between named individuals.
    ObjectPropertyAssertion {
        /// Subject individual.
        subject: EntityId,
        /// Object property.
        property: EntityId,
        /// Object individual.
        object: EntityId,
    },
    /// `owl:sameAs` cluster.
    SameIndividual(Vec<EntityId>),
    /// `owl:differentFrom` group.
    DifferentIndividuals(Vec<EntityId>),
}

impl Axiom {
    /// Validate entity references and kinds for this axiom using default [`Limits`](crate::Limits).
    pub fn validate(&self, registry: &EntityRegistry) -> Result<()> {
        self.validate_with_limits(registry, Limits::default())
    }

    /// Validate entity references, kinds, and resource limits for this axiom.
    pub fn validate_with_limits(&self, registry: &EntityRegistry, limits: Limits) -> Result<()> {
        let max_class_operands = limits.max_class_operands;
        match self {
            Self::SubClassOf {
                subclass,
                superclass,
            } => {
                require_kind(registry, *subclass, EntityKind::Class, "subclass")?;
                require_kind(registry, *superclass, EntityKind::Class, "superclass")?;
            }
            Self::EquivalentClasses(classes)
            | Self::DisjointClasses(classes)
            | Self::EquivalentObjectProperties(classes) => {
                if classes.len() < 2 {
                    return Err(Error::InvalidAxiom(
                        "equivalent/disjoint operands require at least two members".into(),
                    ));
                }
                if classes.len() > max_class_operands {
                    return Err(Error::InvalidAxiom(format!(
                        "equivalent/disjoint operands exceed maximum operand count of {max_class_operands}"
                    )));
                }
                if classes.iter().copied().collect::<HashSet<_>>().len() < 2 {
                    return Err(Error::InvalidAxiom(
                        "equivalent/disjoint operands require at least two distinct members".into(),
                    ));
                }
                let kind = match self {
                    Self::EquivalentObjectProperties(_) => EntityKind::ObjectProperty,
                    _ => EntityKind::Class,
                };
                let role = if kind == EntityKind::ObjectProperty {
                    "property operand"
                } else {
                    "class operand"
                };
                for id in classes {
                    require_kind(registry, *id, kind, role)?;
                }
            }
            Self::SameIndividual(individuals) | Self::DifferentIndividuals(individuals) => {
                if individuals.len() < 2 {
                    return Err(Error::InvalidAxiom(
                        "same/different individuals require at least two operands".into(),
                    ));
                }
                if individuals.len() > max_class_operands {
                    return Err(Error::InvalidAxiom(format!(
                        "same/different individuals exceed maximum operand count of {max_class_operands}"
                    )));
                }
                if individuals.iter().copied().collect::<HashSet<_>>().len() < 2 {
                    return Err(Error::InvalidAxiom(
                        "same/different individuals require at least two distinct operands".into(),
                    ));
                }
                for id in individuals {
                    require_kind(registry, *id, EntityKind::Individual, "individual operand")?;
                }
            }
            Self::ClassAssertion { individual, class } => {
                require_kind(registry, *individual, EntityKind::Individual, "individual")?;
                require_kind(registry, *class, EntityKind::Class, "class")?;
            }
            Self::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                require_kind(registry, *subject, EntityKind::Individual, "subject")?;
                require_kind(registry, *property, EntityKind::ObjectProperty, "property")?;
                require_kind(registry, *object, EntityKind::Individual, "object")?;
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
                if left == right {
                    return Err(Error::InvalidAxiom(
                        "property cannot be inverse of itself".into(),
                    ));
                }
                require_kind(registry, *left, EntityKind::ObjectProperty, "left property")?;
                require_kind(
                    registry,
                    *right,
                    EntityKind::ObjectProperty,
                    "right property",
                )?;
            }
            Self::TransitiveObjectProperty(property)
            | Self::SymmetricObjectProperty(property)
            | Self::ReflexiveObjectProperty(property)
            | Self::FunctionalObjectProperty(property)
            | Self::InverseFunctionalObjectProperty(property)
            | Self::IrreflexiveObjectProperty(property)
            | Self::AsymmetricObjectProperty(property) => {
                require_kind(registry, *property, EntityKind::ObjectProperty, "property")?;
            }
            Self::SubClassOfExistential {
                subclass,
                property,
                filler,
            } => {
                require_kind(registry, *subclass, EntityKind::Class, "subclass")?;
                require_kind(registry, *property, EntityKind::ObjectProperty, "property")?;
                require_kind(registry, *filler, EntityKind::Class, "filler")?;
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
            Self::SubClassOfExistential { .. } => "SubClassOfExistential",
            Self::SymmetricObjectProperty(_) => "SymmetricObjectProperty",
            Self::ReflexiveObjectProperty(_) => "ReflexiveObjectProperty",
            Self::FunctionalObjectProperty(_) => "FunctionalObjectProperty",
            Self::InverseFunctionalObjectProperty(_) => "InverseFunctionalObjectProperty",
            Self::IrreflexiveObjectProperty(_) => "IrreflexiveObjectProperty",
            Self::AsymmetricObjectProperty(_) => "AsymmetricObjectProperty",
            Self::EquivalentObjectProperties(_) => "EquivalentObjectProperties",
            Self::ClassAssertion { .. } => "ClassAssertion",
            Self::ObjectPropertyAssertion { .. } => "ObjectPropertyAssertion",
            Self::SameIndividual(_) => "SameIndividual",
            Self::DifferentIndividuals(_) => "DifferentIndividuals",
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
    if !record.kind.satisfies(expected) {
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

    struct Fixture {
        pool: InternPool,
        registry: EntityRegistry,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                pool: InternPool::new(),
                registry: EntityRegistry::new(),
            }
        }

        fn class(&mut self, iri: &str) -> EntityId {
            let iri_id = self.pool.intern(iri).expect("intern");
            self.registry
                .get_or_register(iri_id, iri, EntityKind::Class)
                .expect("register")
        }

        fn object_property(&mut self, iri: &str) -> EntityId {
            let iri_id = self.pool.intern(iri).expect("intern");
            self.registry
                .get_or_register(iri_id, iri, EntityKind::ObjectProperty)
                .expect("register")
        }
    }

    #[test]
    fn validates_subclass_of() {
        let mut fx = Fixture::new();
        let a = fx.class("http://ex.org/A");
        let b = fx.class("http://ex.org/B");
        Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        }
        .validate(&fx.registry)
        .expect("valid");
    }

    #[test]
    fn rejects_subclass_of_wrong_kind() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/p");
        let err = Axiom::SubClassOf {
            subclass: prop,
            superclass: prop,
        }
        .validate(&fx.registry)
        .expect_err("wrong kind");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn validates_equivalent_classes() {
        let mut fx = Fixture::new();
        let a = fx.class("http://ex.org/A");
        let b = fx.class("http://ex.org/B");
        Axiom::EquivalentClasses(vec![a, b])
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn rejects_equivalent_classes_too_few() {
        let mut fx = Fixture::new();
        let a = fx.class("http://ex.org/A");
        let err = Axiom::EquivalentClasses(vec![a])
            .validate(&fx.registry)
            .expect_err("too few");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn validates_disjoint_classes() {
        let mut fx = Fixture::new();
        let a = fx.class("http://ex.org/A");
        let b = fx.class("http://ex.org/B");
        Axiom::DisjointClasses(vec![a, b])
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn validates_object_property_domain() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/p");
        let domain = fx.class("http://ex.org/D");
        Axiom::ObjectPropertyDomain {
            property: prop,
            domain,
        }
        .validate(&fx.registry)
        .expect("valid");
    }

    #[test]
    fn rejects_domain_with_class_as_property() {
        let mut fx = Fixture::new();
        let not_prop = fx.class("http://ex.org/C");
        let domain = fx.class("http://ex.org/D");
        let err = Axiom::ObjectPropertyDomain {
            property: not_prop,
            domain,
        }
        .validate(&fx.registry)
        .expect_err("wrong kind");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn validates_object_property_range() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/p");
        let range = fx.class("http://ex.org/R");
        Axiom::ObjectPropertyRange {
            property: prop,
            range,
        }
        .validate(&fx.registry)
        .expect("valid");
    }

    #[test]
    fn validates_sub_object_property_of() {
        let mut fx = Fixture::new();
        let sub = fx.object_property("http://ex.org/sub");
        let sup = fx.object_property("http://ex.org/super");
        Axiom::SubObjectPropertyOf {
            sub_property: sub,
            super_property: sup,
        }
        .validate(&fx.registry)
        .expect("valid");
    }

    #[test]
    fn validates_inverse_object_properties() {
        let mut fx = Fixture::new();
        let left = fx.object_property("http://ex.org/left");
        let right = fx.object_property("http://ex.org/right");
        Axiom::InverseObjectProperties { left, right }
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn validates_transitive_object_property() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/trans");
        Axiom::TransitiveObjectProperty(prop)
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn rejects_equivalent_classes_with_duplicate_operands() {
        let mut fx = Fixture::new();
        let a = fx.class("http://ex.org/A");
        let err = Axiom::EquivalentClasses(vec![a, a])
            .validate(&fx.registry)
            .expect_err("dup");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn rejects_inverse_to_self() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/p");
        let err = Axiom::InverseObjectProperties {
            left: prop,
            right: prop,
        }
        .validate(&fx.registry)
        .expect_err("self");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn rejects_unknown_entity_reference() {
        let fx = Fixture::new();
        let err = Axiom::SubClassOf {
            subclass: EntityId(0),
            superclass: EntityId(1),
        }
        .validate(&fx.registry)
        .expect_err("unknown");
        assert!(matches!(err, Error::UnknownEntity(_)));
    }

    #[test]
    fn validates_subclass_of_existential() {
        let mut fx = Fixture::new();
        let subclass = fx.class("http://ex.org/C");
        let property = fx.object_property("http://ex.org/hasPart");
        let filler = fx.class("http://ex.org/B");
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        }
        .validate(&fx.registry)
        .expect("valid");
    }

    #[test]
    fn rejects_subclass_of_existential_wrong_kinds() {
        let mut fx = Fixture::new();
        let subclass = fx.class("http://ex.org/C");
        let property = fx.class("http://ex.org/not-a-property");
        let filler = fx.class("http://ex.org/B");
        let err = Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        }
        .validate(&fx.registry)
        .expect_err("wrong kind");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }

    #[test]
    fn validates_symmetric_object_property() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/symmetric");
        Axiom::SymmetricObjectProperty(prop)
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn validates_reflexive_object_property() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/reflexive");
        Axiom::ReflexiveObjectProperty(prop)
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn validates_functional_object_property() {
        let mut fx = Fixture::new();
        let prop = fx.object_property("http://ex.org/functional");
        Axiom::FunctionalObjectProperty(prop)
            .validate(&fx.registry)
            .expect("valid");
    }

    #[test]
    fn rejects_rl_property_axiom_with_class_entity() {
        let mut fx = Fixture::new();
        let not_prop = fx.class("http://ex.org/C");
        let err = Axiom::SymmetricObjectProperty(not_prop)
            .validate(&fx.registry)
            .expect_err("wrong kind");
        assert!(matches!(err, Error::InvalidAxiom(_)));
    }
}
