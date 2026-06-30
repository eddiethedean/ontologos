//! OWL 2 DL class expressions and axioms (complex CE storage).

use std::collections::HashMap;

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
    /// Data complement.
    Not(DeId),
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
    #[serde(skip, default)]
    ce_dedup: std::collections::HashMap<u64, CeId>,
    #[serde(skip, default)]
    de_dedup: std::collections::HashMap<u64, DeId>,
}

fn ce_fingerprint(expr: &ClassExpr) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    expr.hash(&mut hasher);
    hasher.finish()
}

fn de_fingerprint(expr: &DataExpr) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    expr.hash(&mut hasher);
    hasher.finish()
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
        let fp = ce_fingerprint(&expr);
        if let Some(&id) = self.ce_dedup.get(&fp) {
            if self
                .expressions
                .get(id.index() as usize)
                .is_some_and(|e| e == &expr)
            {
                return id;
            }
        }
        if let Some((i, _)) = self
            .expressions
            .iter()
            .enumerate()
            .find(|(_, e)| *e == &expr)
        {
            let id = CeId(i as u32);
            self.ce_dedup.insert(fp, id);
            return id;
        }
        let id = CeId(self.expressions.len() as u32);
        self.expressions.push(expr);
        self.ce_dedup.insert(fp, id);
        id
    }

    /// Intern a data expression.
    #[must_use]
    pub fn intern_de(&mut self, expr: DataExpr) -> DeId {
        let fp = de_fingerprint(&expr);
        if let Some(&id) = self.de_dedup.get(&fp) {
            if self
                .data_exprs
                .get(id.0 as usize)
                .is_some_and(|e| e == &expr)
            {
                return id;
            }
        }
        if let Some((i, _)) = self
            .data_exprs
            .iter()
            .enumerate()
            .find(|(_, e)| *e == &expr)
        {
            let id = DeId(i as u32);
            self.de_dedup.insert(fp, id);
            return id;
        }
        let id = DeId(self.data_exprs.len() as u32);
        self.data_exprs.push(expr);
        self.de_dedup.insert(fp, id);
        id
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

    /// Import DL axioms from `source`, remapping entity and expression ids into this store.
    pub fn import_axioms_from(
        &mut self,
        source: &DlStore,
        remap_entity: impl Fn(EntityId) -> EntityId,
    ) {
        let mut ce_map = HashMap::new();
        let mut de_map = HashMap::new();
        let mut imported = Vec::new();
        for axiom in source.axioms() {
            imported.push(remap_dl_axiom(
                axiom,
                source,
                &mut self.expressions,
                &mut self.data_exprs,
                &mut ce_map,
                &mut de_map,
                &remap_entity,
            ));
        }
        self.axioms.extend(imported);
    }
}

fn intern_ce_in(expressions: &mut Vec<ClassExpr>, expr: ClassExpr) -> CeId {
    if let Some((i, _)) = expressions.iter().enumerate().find(|(_, e)| *e == &expr) {
        return CeId(i as u32);
    }
    let id = expressions.len() as u32;
    expressions.push(expr);
    CeId(id)
}

fn intern_de_in(data_exprs: &mut Vec<DataExpr>, expr: DataExpr) -> DeId {
    if let Some((i, _)) = data_exprs.iter().enumerate().find(|(_, e)| *e == &expr) {
        return DeId(i as u32);
    }
    let id = data_exprs.len() as u32;
    data_exprs.push(expr);
    DeId(id)
}

fn remap_ce(
    source: &DlStore,
    expressions: &mut Vec<ClassExpr>,
    data_exprs: &mut Vec<DataExpr>,
    id: CeId,
    ce_map: &mut HashMap<CeId, CeId>,
    de_map: &mut HashMap<DeId, DeId>,
    remap_entity: &impl Fn(EntityId) -> EntityId,
) -> CeId {
    if let Some(&mapped) = ce_map.get(&id) {
        return mapped;
    }
    let expr = source.ce(id).expect("source ce").clone();
    let mapped_expr = remap_class_expr(
        &expr,
        source,
        expressions,
        data_exprs,
        ce_map,
        de_map,
        remap_entity,
    );
    let new_id = intern_ce_in(expressions, mapped_expr);
    ce_map.insert(id, new_id);
    new_id
}

fn remap_de(
    source: &DlStore,
    expressions: &mut Vec<ClassExpr>,
    data_exprs: &mut Vec<DataExpr>,
    id: DeId,
    ce_map: &mut HashMap<CeId, CeId>,
    de_map: &mut HashMap<DeId, DeId>,
    remap_entity: &impl Fn(EntityId) -> EntityId,
) -> DeId {
    if let Some(&mapped) = de_map.get(&id) {
        return mapped;
    }
    let expr = source.de(id).expect("source de").clone();
    let mapped_expr = remap_data_expr(
        &expr,
        source,
        expressions,
        data_exprs,
        ce_map,
        de_map,
        remap_entity,
    );
    let new_id = intern_de_in(data_exprs, mapped_expr);
    de_map.insert(id, new_id);
    new_id
}

fn remap_role(role: &RoleExpr, remap_entity: &impl Fn(EntityId) -> EntityId) -> RoleExpr {
    match role {
        RoleExpr::Atomic(id) => RoleExpr::Atomic(remap_entity(*id)),
        RoleExpr::Inverse(id) => RoleExpr::Inverse(remap_entity(*id)),
    }
}

fn remap_class_expr(
    expr: &ClassExpr,
    source: &DlStore,
    expressions: &mut Vec<ClassExpr>,
    data_exprs: &mut Vec<DataExpr>,
    ce_map: &mut HashMap<CeId, CeId>,
    de_map: &mut HashMap<DeId, DeId>,
    remap_entity: &impl Fn(EntityId) -> EntityId,
) -> ClassExpr {
    match expr {
        ClassExpr::Top | ClassExpr::Bottom => expr.clone(),
        ClassExpr::Atomic(id) => ClassExpr::Atomic(remap_entity(*id)),
        ClassExpr::Not(inner) => ClassExpr::Not(remap_ce(
            source,
            expressions,
            data_exprs,
            *inner,
            ce_map,
            de_map,
            remap_entity,
        )),
        ClassExpr::And(ops) => ClassExpr::And(
            ops.iter()
                .map(|&id| {
                    remap_ce(
                        source,
                        expressions,
                        data_exprs,
                        id,
                        ce_map,
                        de_map,
                        remap_entity,
                    )
                })
                .collect(),
        ),
        ClassExpr::Or(ops) => ClassExpr::Or(
            ops.iter()
                .map(|&id| {
                    remap_ce(
                        source,
                        expressions,
                        data_exprs,
                        id,
                        ce_map,
                        de_map,
                        remap_entity,
                    )
                })
                .collect(),
        ),
        ClassExpr::Some { property, filler } => ClassExpr::Some {
            property: remap_role(property, remap_entity),
            filler: remap_ce(
                source,
                expressions,
                data_exprs,
                *filler,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        ClassExpr::All { property, filler } => ClassExpr::All {
            property: remap_role(property, remap_entity),
            filler: remap_ce(
                source,
                expressions,
                data_exprs,
                *filler,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        ClassExpr::OneOf(ids) => ClassExpr::OneOf(ids.iter().map(|id| remap_entity(*id)).collect()),
        ClassExpr::HasValue {
            property,
            individual,
        } => ClassExpr::HasValue {
            property: remap_role(property, remap_entity),
            individual: remap_entity(*individual),
        },
        ClassExpr::HasSelf(id) => ClassExpr::HasSelf(remap_entity(*id)),
        ClassExpr::MinCardinality {
            n,
            property,
            filler,
        } => ClassExpr::MinCardinality {
            n: *n,
            property: remap_role(property, remap_entity),
            filler: filler.map(|id| {
                remap_ce(
                    source,
                    expressions,
                    data_exprs,
                    id,
                    ce_map,
                    de_map,
                    remap_entity,
                )
            }),
        },
        ClassExpr::MaxCardinality {
            n,
            property,
            filler,
        } => ClassExpr::MaxCardinality {
            n: *n,
            property: remap_role(property, remap_entity),
            filler: filler.map(|id| {
                remap_ce(
                    source,
                    expressions,
                    data_exprs,
                    id,
                    ce_map,
                    de_map,
                    remap_entity,
                )
            }),
        },
        ClassExpr::ExactCardinality {
            n,
            property,
            filler,
        } => ClassExpr::ExactCardinality {
            n: *n,
            property: remap_role(property, remap_entity),
            filler: filler.map(|id| {
                remap_ce(
                    source,
                    expressions,
                    data_exprs,
                    id,
                    ce_map,
                    de_map,
                    remap_entity,
                )
            }),
        },
        ClassExpr::DataAll { property, range } => ClassExpr::DataAll {
            property: remap_entity(*property),
            range: remap_de(
                source,
                expressions,
                data_exprs,
                *range,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        ClassExpr::DataSome { property, range } => ClassExpr::DataSome {
            property: remap_entity(*property),
            range: remap_de(
                source,
                expressions,
                data_exprs,
                *range,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        ClassExpr::DataHasValue { property, value } => ClassExpr::DataHasValue {
            property: remap_entity(*property),
            value: remap_de(
                source,
                expressions,
                data_exprs,
                *value,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        ClassExpr::DataMinCardinality { n, property, range } => ClassExpr::DataMinCardinality {
            n: *n,
            property: remap_entity(*property),
            range: range.map(|id| {
                remap_de(
                    source,
                    expressions,
                    data_exprs,
                    id,
                    ce_map,
                    de_map,
                    remap_entity,
                )
            }),
        },
        ClassExpr::DataMaxCardinality { n, property, range } => ClassExpr::DataMaxCardinality {
            n: *n,
            property: remap_entity(*property),
            range: range.map(|id| {
                remap_de(
                    source,
                    expressions,
                    data_exprs,
                    id,
                    ce_map,
                    de_map,
                    remap_entity,
                )
            }),
        },
        ClassExpr::DataExactCardinality { n, property, range } => ClassExpr::DataExactCardinality {
            n: *n,
            property: remap_entity(*property),
            range: range.map(|id| {
                remap_de(
                    source,
                    expressions,
                    data_exprs,
                    id,
                    ce_map,
                    de_map,
                    remap_entity,
                )
            }),
        },
    }
}

fn remap_data_expr(
    expr: &DataExpr,
    source: &DlStore,
    expressions: &mut Vec<ClassExpr>,
    data_exprs: &mut Vec<DataExpr>,
    ce_map: &mut HashMap<CeId, CeId>,
    de_map: &mut HashMap<DeId, DeId>,
    remap_entity: &impl Fn(EntityId) -> EntityId,
) -> DataExpr {
    match expr {
        DataExpr::Top => DataExpr::Top,
        DataExpr::Datatype(id) => DataExpr::Datatype(remap_entity(*id)),
        DataExpr::Facet {
            base,
            facet_iri,
            value,
        } => DataExpr::Facet {
            base: remap_de(
                source,
                expressions,
                data_exprs,
                *base,
                ce_map,
                de_map,
                remap_entity,
            ),
            facet_iri: facet_iri.clone(),
            value: value.clone(),
        },
        DataExpr::And(ops) => DataExpr::And(
            ops.iter()
                .map(|&id| {
                    remap_de(
                        source,
                        expressions,
                        data_exprs,
                        id,
                        ce_map,
                        de_map,
                        remap_entity,
                    )
                })
                .collect(),
        ),
        DataExpr::Or(ops) => DataExpr::Or(
            ops.iter()
                .map(|&id| {
                    remap_de(
                        source,
                        expressions,
                        data_exprs,
                        id,
                        ce_map,
                        de_map,
                        remap_entity,
                    )
                })
                .collect(),
        ),
        DataExpr::Literal { lexical, datatype } => DataExpr::Literal {
            lexical: lexical.clone(),
            datatype: remap_entity(*datatype),
        },
        DataExpr::Not(inner) => DataExpr::Not(remap_de(
            source,
            expressions,
            data_exprs,
            *inner,
            ce_map,
            de_map,
            remap_entity,
        )),
    }
}

fn remap_dl_axiom(
    axiom: &DlAxiom,
    source: &DlStore,
    expressions: &mut Vec<ClassExpr>,
    data_exprs: &mut Vec<DataExpr>,
    ce_map: &mut HashMap<CeId, CeId>,
    de_map: &mut HashMap<DeId, DeId>,
    remap_entity: &impl Fn(EntityId) -> EntityId,
) -> DlAxiom {
    match axiom {
        DlAxiom::SubClassOf { sub, sup } => DlAxiom::SubClassOf {
            sub: remap_ce(
                source,
                expressions,
                data_exprs,
                *sub,
                ce_map,
                de_map,
                remap_entity,
            ),
            sup: remap_ce(
                source,
                expressions,
                data_exprs,
                *sup,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::EquivalentClasses(ids) => DlAxiom::EquivalentClasses(
            ids.iter()
                .map(|&id| {
                    remap_ce(
                        source,
                        expressions,
                        data_exprs,
                        id,
                        ce_map,
                        de_map,
                        remap_entity,
                    )
                })
                .collect(),
        ),
        DlAxiom::DisjointClasses(ids) => DlAxiom::DisjointClasses(
            ids.iter()
                .map(|&id| {
                    remap_ce(
                        source,
                        expressions,
                        data_exprs,
                        id,
                        ce_map,
                        de_map,
                        remap_entity,
                    )
                })
                .collect(),
        ),
        DlAxiom::ObjectPropertyDomain { property, domain } => DlAxiom::ObjectPropertyDomain {
            property: remap_entity(*property),
            domain: remap_ce(
                source,
                expressions,
                data_exprs,
                *domain,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::ObjectPropertyRange { property, range } => DlAxiom::ObjectPropertyRange {
            property: remap_entity(*property),
            range: remap_ce(
                source,
                expressions,
                data_exprs,
                *range,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::SubObjectPropertyChain {
            chain,
            super_property,
        } => DlAxiom::SubObjectPropertyChain {
            chain: chain.iter().map(|r| remap_role(r, remap_entity)).collect(),
            super_property: remap_role(super_property, remap_entity),
        },
        DlAxiom::SubObjectPropertyOf { sub, sup } => DlAxiom::SubObjectPropertyOf {
            sub: remap_role(sub, remap_entity),
            sup: remap_role(sup, remap_entity),
        },
        DlAxiom::HasKey {
            class,
            object_properties,
            data_properties,
        } => DlAxiom::HasKey {
            class: remap_ce(
                source,
                expressions,
                data_exprs,
                *class,
                ce_map,
                de_map,
                remap_entity,
            ),
            object_properties: object_properties
                .iter()
                .map(|id| remap_entity(*id))
                .collect(),
            data_properties: data_properties.iter().map(|id| remap_entity(*id)).collect(),
        },
        DlAxiom::ClassAssertion { individual, class } => DlAxiom::ClassAssertion {
            individual: remap_entity(*individual),
            class: remap_ce(
                source,
                expressions,
                data_exprs,
                *class,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::DataPropertyDomain { property, domain } => DlAxiom::DataPropertyDomain {
            property: remap_entity(*property),
            domain: remap_ce(
                source,
                expressions,
                data_exprs,
                *domain,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::DataPropertyRange { property, range } => DlAxiom::DataPropertyRange {
            property: remap_entity(*property),
            range: remap_de(
                source,
                expressions,
                data_exprs,
                *range,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::SubDataPropertyOf { sub, sup } => DlAxiom::SubDataPropertyOf {
            sub: remap_entity(*sub),
            sup: remap_entity(*sup),
        },
        DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } => DlAxiom::DataPropertyAssertion {
            subject: remap_entity(*subject),
            property: remap_entity(*property),
            value: remap_de(
                source,
                expressions,
                data_exprs,
                *value,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => DlAxiom::ObjectPropertyAssertion {
            subject: remap_entity(*subject),
            property: remap_role(property, remap_entity),
            object: remap_entity(*object),
        },
        DlAxiom::NegativeObjectPropertyAssertion {
            subject,
            property,
            object,
        } => DlAxiom::NegativeObjectPropertyAssertion {
            subject: remap_entity(*subject),
            property: remap_entity(*property),
            object: remap_entity(*object),
        },
        DlAxiom::NegativeDataPropertyAssertion {
            subject,
            property,
            value,
        } => DlAxiom::NegativeDataPropertyAssertion {
            subject: remap_entity(*subject),
            property: remap_entity(*property),
            value: remap_de(
                source,
                expressions,
                data_exprs,
                *value,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::DatatypeDefinition { datatype, range } => DlAxiom::DatatypeDefinition {
            datatype: remap_entity(*datatype),
            range: remap_de(
                source,
                expressions,
                data_exprs,
                *range,
                ce_map,
                de_map,
                remap_entity,
            ),
        },
        DlAxiom::FunctionalDataProperty(id) => DlAxiom::FunctionalDataProperty(remap_entity(*id)),
        DlAxiom::EquivalentDataProperties(ids) => {
            DlAxiom::EquivalentDataProperties(ids.iter().map(|id| remap_entity(*id)).collect())
        }
        DlAxiom::DisjointDataProperties(ids) => {
            DlAxiom::DisjointDataProperties(ids.iter().map(|id| remap_entity(*id)).collect())
        }
        DlAxiom::DisjointObjectProperties(ids) => {
            DlAxiom::DisjointObjectProperties(ids.iter().map(|id| remap_entity(*id)).collect())
        }
        DlAxiom::SameIndividual(ids) => {
            DlAxiom::SameIndividual(ids.iter().map(|id| remap_entity(*id)).collect())
        }
        DlAxiom::DifferentIndividuals(ids) => {
            DlAxiom::DifferentIndividuals(ids.iter().map(|id| remap_entity(*id)).collect())
        }
        DlAxiom::TransitiveObjectProperty(role) => {
            DlAxiom::TransitiveObjectProperty(remap_role(role, remap_entity))
        }
        DlAxiom::SymmetricObjectProperty(role) => {
            DlAxiom::SymmetricObjectProperty(remap_role(role, remap_entity))
        }
        DlAxiom::SwrlRule => DlAxiom::SwrlRule,
        DlAxiom::InverseFunctionalObjectProperty(id) => {
            DlAxiom::InverseFunctionalObjectProperty(remap_entity(*id))
        }
        DlAxiom::IrreflexiveObjectProperty(id) => {
            DlAxiom::IrreflexiveObjectProperty(remap_entity(*id))
        }
    }
}
