//! OWL 2 DL class expressions and axioms (complex CE storage).

use serde::{Deserialize, Serialize};

use crate::entity::EntityId;

/// Interned class expression id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CeId(pub u32);

impl CeId {
    /// Zero-based index into the class expression pool.
    #[must_use]
    pub fn index(self) -> u32 {
        self.0
    }
}

/// Interned data range / literal expression id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeId(pub u32);

/// Object property expression (atomic or inverse).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleExpr {
    /// Named object property.
    Atomic(EntityId),
    /// `ObjectInverseOf(p)`.
    Inverse(EntityId),
}

/// OWL class expression AST (structural).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClassExpr {
    /// `owl:Thing`.
    Top,
    /// `owl:Nothing`.
    Bottom,
    /// Named class.
    Atomic(EntityId),
    /// `ObjectComplementOf`.
    Not(CeId),
    /// `ObjectIntersectionOf`.
    And(Vec<CeId>),
    /// `ObjectUnionOf`.
    Or(Vec<CeId>),
    /// `ObjectSomeValuesFrom`.
    Some {
        /// Object property expression.
        property: RoleExpr,
        /// Filler class expression.
        filler: CeId,
    },
    /// `ObjectAllValuesFrom`.
    All {
        /// Object property expression.
        property: RoleExpr,
        /// Filler class expression.
        filler: CeId,
    },
    /// `ObjectOneOf` (nominals).
    OneOf(Vec<EntityId>),
    /// `ObjectHasValue`.
    HasValue {
        /// Object property expression.
        property: RoleExpr,
        /// Named individual.
        individual: EntityId,
    },
    /// `ObjectHasSelf`.
    HasSelf(EntityId),
    /// Qualified or unqualified cardinality.
    MinCardinality {
        /// Minimum cardinality bound.
        n: u32,
        /// Object property expression.
        property: RoleExpr,
        /// Optional filler class expression (qualified cardinality).
        filler: Option<CeId>,
    },
    /// `ObjectMaxCardinality`.
    MaxCardinality {
        /// Maximum cardinality bound.
        n: u32,
        /// Object property expression.
        property: RoleExpr,
        /// Optional filler class expression (qualified cardinality).
        filler: Option<CeId>,
    },
    /// `ObjectExactCardinality`.
    ExactCardinality {
        /// Exact cardinality bound.
        n: u32,
        /// Object property expression.
        property: RoleExpr,
        /// Optional filler class expression (qualified cardinality).
        filler: Option<CeId>,
    },
    /// `DataAllValuesFrom`.
    DataAll {
        /// Data property.
        property: EntityId,
        /// Range data expression.
        range: DeId,
    },
    /// `DataSomeValuesFrom`.
    DataSome {
        /// Data property.
        property: EntityId,
        /// Range data expression.
        range: DeId,
    },
    /// `DataHasValue`.
    DataHasValue {
        /// Data property.
        property: EntityId,
        /// Literal data expression.
        value: DeId,
    },
    /// `DataMinCardinality`.
    DataMinCardinality {
        /// Minimum cardinality bound.
        n: u32,
        /// Data property.
        property: EntityId,
        /// Optional range data expression.
        range: Option<DeId>,
    },
    /// `DataMaxCardinality`.
    DataMaxCardinality {
        /// Maximum cardinality bound.
        n: u32,
        /// Data property.
        property: EntityId,
        /// Optional range data expression.
        range: Option<DeId>,
    },
    /// `DataExactCardinality`.
    DataExactCardinality {
        /// Exact cardinality bound.
        n: u32,
        /// Data property.
        property: EntityId,
        /// Optional range data expression.
        range: Option<DeId>,
    },
}

/// Data range expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataExpr {
    /// Top data type.
    Top,
    /// Named datatype.
    Datatype(EntityId),
    /// Facet restriction on a datatype.
    Facet {
        /// Base datatype expression.
        base: DeId,
        /// Facet IRI (e.g. `xsd:minInclusive`).
        facet_iri: String,
        /// Facet lexical value.
        value: String,
    },
    /// Data intersection.
    And(Vec<DeId>),
    /// Data union.
    Or(Vec<DeId>),
    /// Literal value.
    Literal {
        /// Lexical form.
        lexical: String,
        /// Datatype entity.
        datatype: EntityId,
    },
}

/// DL axiom beyond flat core `Axiom` enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DlAxiom {
    /// Generalized concept inclusion `C ⊑ D`.
    SubClassOf {
        /// Subsumed class expression.
        sub: CeId,
        /// Subsumer class expression.
        sup: CeId,
    },
    /// Equivalent class expressions.
    EquivalentClasses(Vec<CeId>),
    /// Disjoint class expressions.
    DisjointClasses(Vec<CeId>),
    /// Complex domain `∃r.C` or named.
    ObjectPropertyDomain {
        /// The object property.
        property: EntityId,
        /// Domain class expression.
        domain: CeId,
    },
    /// Complex range.
    ObjectPropertyRange {
        /// The object property.
        property: EntityId,
        /// Range class expression.
        range: CeId,
    },
    /// Role chain inclusion.
    SubObjectPropertyChain {
        /// Property chain.
        chain: Vec<RoleExpr>,
        /// Super property expression.
        super_property: RoleExpr,
    },
    /// Generalized subproperty `r ⊑ s` (atomic or inverse).
    SubObjectPropertyOf {
        /// Subsumed property expression.
        sub: RoleExpr,
        /// Subsumer property expression.
        sup: RoleExpr,
    },
    /// `HasKey(C, object props, data props)`.
    HasKey {
        /// Key class expression.
        class: CeId,
        /// Object properties in the key.
        object_properties: Vec<EntityId>,
        /// Data properties in the key.
        data_properties: Vec<EntityId>,
    },
    /// Class assertion with complex CE.
    ClassAssertion {
        /// The individual.
        individual: EntityId,
        /// Asserted class expression.
        class: CeId,
    },
    /// Data property domain/range/subproperty.
    DataPropertyDomain {
        /// The data property.
        property: EntityId,
        /// Domain class expression.
        domain: CeId,
    },
    /// Data property range.
    DataPropertyRange {
        /// The data property.
        property: EntityId,
        /// Range data expression.
        range: DeId,
    },
    /// Data property subsumption.
    SubDataPropertyOf {
        /// Subsumed data property.
        sub: EntityId,
        /// Subsumer data property.
        sup: EntityId,
    },
    /// Data property assertion.
    DataPropertyAssertion {
        /// Subject individual.
        subject: EntityId,
        /// Data property.
        property: EntityId,
        /// Literal data expression.
        value: DeId,
    },
    /// Negative object property assertion.
    NegativeObjectPropertyAssertion {
        /// Subject individual.
        subject: EntityId,
        /// Object property.
        property: EntityId,
        /// Object individual.
        object: EntityId,
    },
    /// Negative data property assertion.
    NegativeDataPropertyAssertion {
        /// Subject individual.
        subject: EntityId,
        /// Data property.
        property: EntityId,
        /// Literal data expression.
        value: DeId,
    },
    /// Object property assertion (supports inverse property expressions).
    ObjectPropertyAssertion {
        /// Subject individual.
        subject: EntityId,
        /// Object property expression.
        property: RoleExpr,
        /// Object individual.
        object: EntityId,
    },
    /// Named datatype definition.
    DatatypeDefinition {
        /// Named datatype.
        datatype: EntityId,
        /// Defining data range expression.
        range: DeId,
    },
    /// Functional data property declaration.
    FunctionalDataProperty(EntityId),
    /// Equivalent data properties.
    EquivalentDataProperties(Vec<EntityId>),
    /// Disjoint data properties.
    DisjointDataProperties(Vec<EntityId>),
    /// Disjoint object properties (atomic).
    DisjointObjectProperties(Vec<EntityId>),
    /// Individual equality / inequality (including anonymous).
    SameIndividual(Vec<EntityId>),
    /// `owl:differentFrom` group.
    DifferentIndividuals(Vec<EntityId>),
    /// Transitive / symmetric on complex property expressions.
    TransitiveObjectProperty(RoleExpr),
    /// Symmetric object property declaration.
    SymmetricObjectProperty(RoleExpr),
    /// SWRL rule placeholder (parsed, execution deferred).
    SwrlRule,
    /// Inverse-functional / irreflexive declarations.
    InverseFunctionalObjectProperty(EntityId),
    /// Irreflexive object property declaration.
    IrreflexiveObjectProperty(EntityId),
}

/// Pool of class/data expressions and DL axioms attached to an ontology.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DlStore {
    expressions: Vec<ClassExpr>,
    data_exprs: Vec<DataExpr>,
    axioms: Vec<DlAxiom>,
}

impl DlStore {
    /// Empty DL store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of interned class expressions.
    #[must_use]
    pub fn ce_count(&self) -> usize {
        self.expressions.len()
    }

    /// Number of DL axioms.
    #[must_use]
    pub fn axiom_count(&self) -> usize {
        self.axioms.len()
    }

    /// Intern a class expression (structural dedup).
    #[must_use]
    pub fn intern_ce(&mut self, expr: ClassExpr) -> CeId {
        if let Some((i, _)) = self
            .expressions
            .iter()
            .enumerate()
            .find(|(_, e)| *e == &expr)
        {
            return CeId(i as u32);
        }
        let id = self.expressions.len() as u32;
        self.expressions.push(expr);
        CeId(id)
    }

    /// Intern a data expression.
    #[must_use]
    pub fn intern_de(&mut self, expr: DataExpr) -> DeId {
        if let Some((i, _)) = self
            .data_exprs
            .iter()
            .enumerate()
            .find(|(_, e)| *e == &expr)
        {
            return DeId(i as u32);
        }
        let id = self.data_exprs.len() as u32;
        self.data_exprs.push(expr);
        DeId(id)
    }

    /// Push a DL axiom.
    pub fn push_axiom(&mut self, axiom: DlAxiom) {
        self.axioms.push(axiom);
    }

    /// Borrow class expression by id.
    #[must_use]
    pub fn ce(&self, id: CeId) -> Option<&ClassExpr> {
        self.expressions.get(id.index() as usize)
    }

    /// Borrow data expression by id.
    #[must_use]
    pub fn de(&self, id: DeId) -> Option<&DataExpr> {
        self.data_exprs.get(id.0 as usize)
    }

    /// Iterate DL axioms.
    pub fn axioms(&self) -> impl Iterator<Item = &DlAxiom> {
        self.axioms.iter()
    }

    /// Number of interned data expressions.
    #[must_use]
    pub fn de_count(&self) -> usize {
        self.data_exprs.len()
    }

    /// Iterate data expressions.
    pub fn data_exprs(&self) -> impl Iterator<Item = (DeId, &DataExpr)> {
        self.data_exprs
            .iter()
            .enumerate()
            .map(|(i, e)| (DeId(i as u32), e))
    }

    /// Iterate class expressions.
    pub fn expressions(&self) -> impl Iterator<Item = (CeId, &ClassExpr)> {
        self.expressions
            .iter()
            .enumerate()
            .map(|(i, e)| (CeId(i as u32), e))
    }
}
