use serde::{Deserialize, Serialize};

/// Supported axiom types in the 1.x reasoner.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AxiomKind {
    SubClassOf,
    EquivalentClasses,
    DisjointClasses,
    ObjectPropertyDomain,
    ObjectPropertyRange,
    SubObjectPropertyOf,
    InverseObjectProperties,
    TransitiveObjectProperty,
}
