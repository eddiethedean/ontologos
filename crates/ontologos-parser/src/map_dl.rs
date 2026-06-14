//! Map horned-owl class expressions into [`DlStore`].

use horned_owl::model::{
    ClassExpression, DataProperty, DataRange, Individual, Literal, NamedIndividual,
    ObjectPropertyExpression, PropertyExpression, RcStr,
};
use ontologos_core::{
    CeId, ClassExpr, DataExpr, DeId, DlAxiom, EntityId, EntityKind, OwlConstruct, RoleExpr,
};

use crate::map::Mapper;

impl Mapper<'_> {
    /// Map a class expression into the DL store; returns `None` if entities fail to register.
    pub(crate) fn map_class_expression(&mut self, ce: &ClassExpression<RcStr>) -> Option<CeId> {
        let expr = self.build_class_expr(ce)?;
        Some(self.ontology.dl_mut().intern_ce(expr))
    }

    fn build_class_expr(&mut self, ce: &ClassExpression<RcStr>) -> Option<ClassExpr> {
        match ce {
            ClassExpression::Class(class) => {
                if iri_of(&class.0) == "http://www.w3.org/2002/07/owl#Nothing" {
                    Some(ClassExpr::Bottom)
                } else {
                    let id = self.register_or_warn_class(class)?;
                    Some(ClassExpr::Atomic(id))
                }
            }
            ClassExpression::ObjectIntersectionOf(ops) => {
                let ids: Vec<CeId> = ops
                    .iter()
                    .filter_map(|op| self.map_class_expression(op))
                    .collect();
                if ids.len() != ops.len() {
                    return None;
                }
                Some(ClassExpr::And(ids))
            }
            ClassExpression::ObjectUnionOf(ops) => {
                let ids: Vec<CeId> = ops
                    .iter()
                    .filter_map(|op| self.map_class_expression(op))
                    .collect();
                if ids.len() != ops.len() {
                    return None;
                }
                Some(ClassExpr::Or(ids))
            }
            ClassExpression::ObjectComplementOf(inner) => {
                let id = self.map_class_expression(inner)?;
                Some(ClassExpr::Not(id))
            }
            ClassExpression::ObjectOneOf(individuals) => {
                let ids: Vec<EntityId> = individuals
                    .iter()
                    .filter_map(|i| self.map_individual_entity(i))
                    .collect();
                if ids.len() != individuals.len() {
                    return None;
                }
                Some(ClassExpr::OneOf(ids))
            }
            ClassExpression::ObjectSomeValuesFrom { ope, bce } => {
                let property = self.map_role_expr(ope)?;
                let filler = self.map_class_expression(bce)?;
                Some(ClassExpr::Some { property, filler })
            }
            ClassExpression::ObjectAllValuesFrom { ope, bce } => {
                let property = self.map_role_expr(ope)?;
                let filler = self.map_class_expression(bce)?;
                Some(ClassExpr::All { property, filler })
            }
            ClassExpression::ObjectHasValue { ope, i } => {
                let property = self.map_role_expr(ope)?;
                let individual = self.map_individual_entity(i)?;
                Some(ClassExpr::HasValue {
                    property,
                    individual,
                })
            }
            ClassExpression::ObjectHasSelf(ope) => {
                let property = self.map_role_expr(ope)?;
                match property {
                    RoleExpr::Atomic(prop) => Some(ClassExpr::HasSelf(prop)),
                    RoleExpr::Inverse(_) => None,
                }
            }
            ClassExpression::ObjectMinCardinality { n, ope, bce } => {
                let property = self.map_role_expr(ope)?;
                let filler = self.universal_or_class_filler(bce);
                Some(ClassExpr::MinCardinality {
                    n: *n,
                    property,
                    filler,
                })
            }
            ClassExpression::ObjectMaxCardinality { n, ope, bce } => {
                let property = self.map_role_expr(ope)?;
                let filler = self.universal_or_class_filler(bce);
                Some(ClassExpr::MaxCardinality {
                    n: *n,
                    property,
                    filler,
                })
            }
            ClassExpression::ObjectExactCardinality { n, ope, bce } => {
                let property = self.map_role_expr(ope)?;
                let filler = self.universal_or_class_filler(bce);
                Some(ClassExpr::ExactCardinality {
                    n: *n,
                    property,
                    filler,
                })
            }
            ClassExpression::DataSomeValuesFrom { .. }
            | ClassExpression::DataAllValuesFrom { .. }
            | ClassExpression::DataHasValue { .. }
            | ClassExpression::DataMinCardinality { .. }
            | ClassExpression::DataMaxCardinality { .. }
            | ClassExpression::DataExactCardinality { .. } => {
                // Data restrictions become synthetic CE via datatype bridge in DL engine.
                self.map_data_restriction_as_ce(ce)
            }
        }
    }

    fn map_data_restriction_as_ce(&mut self, ce: &ClassExpression<RcStr>) -> Option<ClassExpr> {
        use ClassExpression::*;
        let range = self.map_data_range_from_ce(ce)?;
        match ce {
            DataAllValuesFrom { dp, .. } => {
                let property = self.map_data_property_id_from_dp(dp)?;
                Some(ClassExpr::DataAll { property, range })
            }
            DataSomeValuesFrom { dp, .. } => {
                let property = self.map_data_property_id_from_dp(dp)?;
                Some(ClassExpr::DataSome { property, range })
            }
            DataHasValue { dp, l } => {
                let property = self.map_data_property_id_from_dp(dp)?;
                let value = self.build_literal(l)?;
                let value = self.ontology.dl_mut().intern_de(value);
                Some(ClassExpr::DataHasValue { property, value })
            }
            DataMinCardinality { n, dp, dr } => {
                let property = self.map_data_property_id_from_dp(dp)?;
                let range = self.map_data_range(dr)?;
                Some(ClassExpr::DataMinCardinality {
                    n: *n,
                    property,
                    range: Some(range),
                })
            }
            DataMaxCardinality { n, dp, dr } => {
                let property = self.map_data_property_id_from_dp(dp)?;
                let range = self.map_data_range(dr)?;
                Some(ClassExpr::DataMaxCardinality {
                    n: *n,
                    property,
                    range: Some(range),
                })
            }
            DataExactCardinality { n, dp, dr } => {
                let property = self.map_data_property_id_from_dp(dp)?;
                let range = self.map_data_range(dr)?;
                Some(ClassExpr::DataExactCardinality {
                    n: *n,
                    property,
                    range: Some(range),
                })
            }
            _ => None,
        }
    }

    fn map_data_property_id_from_dp(
        &mut self,
        dp: &horned_owl::model::DataProperty<RcStr>,
    ) -> Option<EntityId> {
        self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty)
            .or_else(|| self.ontology.lookup_entity(iri_of(dp)))
    }

    fn map_role_expr(&mut self, ope: &ObjectPropertyExpression<RcStr>) -> Option<RoleExpr> {
        match ope {
            ObjectPropertyExpression::ObjectProperty(prop) => self
                .register_or_warn_object_property(prop)
                .or_else(|| self.ontology.lookup_entity(iri_of(prop)))
                .map(RoleExpr::Atomic),
            ObjectPropertyExpression::InverseObjectProperty(prop) => self
                .register_or_warn_object_property(prop)
                .or_else(|| self.ontology.lookup_entity(iri_of(prop)))
                .map(RoleExpr::Inverse),
        }
    }

    pub(crate) fn map_individual_entity(
        &mut self,
        individual: &Individual<RcStr>,
    ) -> Option<EntityId> {
        match individual {
            Individual::Named(NamedIndividual(iri)) => {
                self.register_or_warn_entity(iri.as_ref(), EntityKind::Individual)
            }
            Individual::Anonymous(anon) => {
                let iri = format!("file:/c/test.owl#_{}", anon.as_ref());
                self.register_or_warn_entity(&iri, EntityKind::Individual)
            }
        }
    }

    /// Push a DL axiom (stored in [`DlStore`], not core axiom count).
    pub(crate) fn push_dl_axiom(&mut self, axiom: DlAxiom) {
        self.ontology.dl_mut().push_axiom(axiom);
    }

    /// Map `SubClassOf` with complex CE into DL store.
    pub(crate) fn map_dl_subclass_of(
        &mut self,
        sub: &ClassExpression<RcStr>,
        sup: &ClassExpression<RcStr>,
    ) -> bool {
        let Some(sub_id) = self.map_class_expression(sub) else {
            return false;
        };
        let Some(sup_id) = self.map_class_expression(sup) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::SubClassOf {
            sub: sub_id,
            sup: sup_id,
        });
        if Self::class_expression_uses_datatype(sub) || Self::class_expression_uses_datatype(sup) {
            self.report
                .meta
                .note_profile_construct(OwlConstruct::Datatype);
        }
        true
    }

    fn class_expression_uses_datatype(ce: &ClassExpression<RcStr>) -> bool {
        use ClassExpression::*;
        match ce {
            DataSomeValuesFrom { .. }
            | DataAllValuesFrom { .. }
            | DataHasValue { .. }
            | DataMinCardinality { .. }
            | DataMaxCardinality { .. }
            | DataExactCardinality { .. } => true,
            ObjectIntersectionOf(ops) | ObjectUnionOf(ops) => {
                ops.iter().any(Self::class_expression_uses_datatype)
            }
            ObjectComplementOf(inner) => Self::class_expression_uses_datatype(inner),
            _ => false,
        }
    }

    pub(crate) fn map_dl_equivalent_classes(&mut self, classes: &[ClassExpression<RcStr>]) -> bool {
        let ids: Vec<CeId> = classes
            .iter()
            .filter_map(|ce| self.map_class_expression(ce))
            .collect();
        if ids.len() != classes.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::EquivalentClasses(ids));
        true
    }

    pub(crate) fn map_dl_disjoint_classes(&mut self, classes: &[ClassExpression<RcStr>]) -> bool {
        let ids: Vec<CeId> = classes
            .iter()
            .filter_map(|ce| self.map_class_expression(ce))
            .collect();
        if ids.len() != classes.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::DisjointClasses(ids));
        true
    }

    pub(crate) fn map_dl_property_domain(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        ce: &ClassExpression<RcStr>,
    ) -> bool {
        let Some(prop_id) = self.named_object_property_id(ope) else {
            return false;
        };
        let Some(domain) = self.map_class_expression(ce) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::ObjectPropertyDomain {
            property: prop_id,
            domain,
        });
        true
    }

    pub(crate) fn map_dl_property_range(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        ce: &ClassExpression<RcStr>,
    ) -> bool {
        let Some(prop_id) = self.named_object_property_id(ope) else {
            return false;
        };
        let Some(range) = self.map_class_expression(ce) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::ObjectPropertyRange {
            property: prop_id,
            range,
        });
        true
    }

    pub(crate) fn map_dl_class_assertion(
        &mut self,
        ce: &ClassExpression<RcStr>,
        individual: &Individual<RcStr>,
    ) -> bool {
        let Some(class) = self.map_class_expression(ce) else {
            return false;
        };
        let Some(individual_id) = self.map_individual_entity(individual) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::ClassAssertion {
            individual: individual_id,
            class,
        });
        true
    }

    pub(crate) fn map_dl_has_key(
        &mut self,
        ce: &ClassExpression<RcStr>,
        properties: &[PropertyExpression<RcStr>],
    ) -> bool {
        let Some(class) = self.map_class_expression(ce) else {
            return false;
        };
        let mut object_properties = Vec::new();
        let mut data_properties = Vec::new();
        for p in properties {
            match p {
                PropertyExpression::ObjectPropertyExpression(
                    ObjectPropertyExpression::ObjectProperty(prop),
                ) => {
                    let Some(id) = self.register_or_warn_object_property(prop) else {
                        return false;
                    };
                    object_properties.push(id);
                }
                PropertyExpression::DataProperty(dp) => {
                    let Some(id) =
                        self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty)
                    else {
                        return false;
                    };
                    data_properties.push(id);
                }
                _ => return false,
            }
        }
        self.push_dl_axiom(DlAxiom::HasKey {
            class,
            object_properties,
            data_properties,
        });
        true
    }

    pub(crate) fn map_dl_inverse_functional(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
    ) -> bool {
        let Some(prop_id) = self.named_object_property_id(ope) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::InverseFunctionalObjectProperty(prop_id));
        true
    }

    pub(crate) fn map_dl_irreflexive(&mut self, ope: &ObjectPropertyExpression<RcStr>) -> bool {
        let Some(prop_id) = self.named_object_property_id(ope) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::IrreflexiveObjectProperty(prop_id));
        true
    }

    pub(crate) fn map_dl_sub_property_chain(
        &mut self,
        chain: &[ObjectPropertyExpression<RcStr>],
        super_property: &ObjectPropertyExpression<RcStr>,
    ) -> bool {
        let roles: Vec<RoleExpr> = chain
            .iter()
            .filter_map(|ope| self.map_role_expr(ope))
            .collect();
        if roles.len() != chain.len() {
            return false;
        }
        let Some(super_role) = self.map_role_expr(super_property) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::SubObjectPropertyChain {
            chain: roles,
            super_property: super_role,
        });
        true
    }

    pub(crate) fn map_dl_sub_object_property_of(
        &mut self,
        sub: &ObjectPropertyExpression<RcStr>,
        sup: &ObjectPropertyExpression<RcStr>,
    ) -> bool {
        let Some(sub_role) = self.map_role_expr(sub) else {
            return false;
        };
        let Some(sup_role) = self.map_role_expr(sup) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::SubObjectPropertyOf {
            sub: sub_role,
            sup: sup_role,
        });
        true
    }

    pub(crate) fn map_dl_object_property_assertion(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        from: &Individual<RcStr>,
        to: &Individual<RcStr>,
    ) -> bool {
        let Some(property) = self.map_role_expr(ope) else {
            return false;
        };
        let Some(subject) = self.map_individual_entity(from) else {
            return false;
        };
        let Some(object) = self.map_individual_entity(to) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        });
        true
    }

    pub(crate) fn map_dl_negative_object_property_assertion(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        from: &Individual<RcStr>,
        to: &Individual<RcStr>,
    ) -> bool {
        let Some(property_id) = self.named_object_property_id(ope) else {
            return false;
        };
        let Some(subject) = self.map_individual_entity(from) else {
            return false;
        };
        let Some(object) = self.map_individual_entity(to) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::NegativeObjectPropertyAssertion {
            subject,
            property: property_id,
            object,
        });
        true
    }

    pub(crate) fn map_dl_data_property_assertion(
        &mut self,
        dp: &DataProperty<RcStr>,
        from: &Individual<RcStr>,
        to: &Literal<RcStr>,
    ) -> bool {
        let Some(property) = self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty)
        else {
            return false;
        };
        let Some(subject) = self.map_individual_entity(from) else {
            return false;
        };
        let Some(value) = self.map_data_range(&DataRange::DataOneOf(vec![to.clone()])) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        });
        true
    }

    pub(crate) fn map_dl_negative_data_property_assertion(
        &mut self,
        dp: &DataProperty<RcStr>,
        from: &Individual<RcStr>,
        to: &Literal<RcStr>,
    ) -> bool {
        let Some(property) = self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty)
        else {
            return false;
        };
        let Some(subject) = self.map_individual_entity(from) else {
            return false;
        };
        let Some(value) = self.map_data_range(&DataRange::DataOneOf(vec![to.clone()])) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::NegativeDataPropertyAssertion {
            subject,
            property,
            value,
        });
        true
    }

    pub(crate) fn map_dl_datatype_definition(
        &mut self,
        datatype: &horned_owl::model::Datatype<RcStr>,
        range: &DataRange<RcStr>,
    ) -> bool {
        let Some(dt_id) = self.register_or_warn_entity(datatype.as_ref(), EntityKind::Datatype)
        else {
            return false;
        };
        let Some(range_id) = self.map_data_range(range) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::DatatypeDefinition {
            datatype: dt_id,
            range: range_id,
        });
        true
    }

    pub(crate) fn map_dl_functional_data_property(&mut self, dp: &DataProperty<RcStr>) -> bool {
        let Some(id) = self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::FunctionalDataProperty(id));
        true
    }

    pub(crate) fn map_dl_sub_data_property(
        &mut self,
        sub: &DataProperty<RcStr>,
        sup: &DataProperty<RcStr>,
    ) -> bool {
        let Some(sub_id) = self.register_or_warn_entity(iri_of(sub), EntityKind::DataProperty)
        else {
            return false;
        };
        let Some(sup_id) = self.register_or_warn_entity(iri_of(sup), EntityKind::DataProperty)
        else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::SubDataPropertyOf {
            sub: sub_id,
            sup: sup_id,
        });
        true
    }

    pub(crate) fn map_dl_data_property_domain(
        &mut self,
        dp: &DataProperty<RcStr>,
        domain: &ClassExpression<RcStr>,
    ) -> bool {
        let Some(prop_id) = self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty)
        else {
            return false;
        };
        let Some(domain_id) = self.map_class_expression(domain) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::DataPropertyDomain {
            property: prop_id,
            domain: domain_id,
        });
        true
    }

    pub(crate) fn map_dl_data_property_range(
        &mut self,
        dp: &DataProperty<RcStr>,
        range: &DataRange<RcStr>,
    ) -> bool {
        let Some(prop_id) = self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty)
        else {
            return false;
        };
        let Some(range_id) = self.map_data_range(range) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::DataPropertyRange {
            property: prop_id,
            range: range_id,
        });
        true
    }

    pub(crate) fn map_dl_equivalent_data_properties(
        &mut self,
        properties: &[DataProperty<RcStr>],
    ) -> bool {
        let ids: Vec<EntityId> = properties
            .iter()
            .filter_map(|dp| self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty))
            .collect();
        if ids.len() != properties.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::EquivalentDataProperties(ids));
        true
    }

    pub(crate) fn map_dl_disjoint_data_properties(
        &mut self,
        properties: &[DataProperty<RcStr>],
    ) -> bool {
        let ids: Vec<EntityId> = properties
            .iter()
            .filter_map(|dp| self.register_or_warn_entity(iri_of(dp), EntityKind::DataProperty))
            .collect();
        if ids.len() != properties.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::DisjointDataProperties(ids));
        true
    }

    pub(crate) fn map_dl_same_individual(&mut self, individuals: &[Individual<RcStr>]) -> bool {
        let ids: Vec<EntityId> = individuals
            .iter()
            .filter_map(|i| self.map_individual_entity(i))
            .collect();
        if ids.len() != individuals.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::SameIndividual(ids));
        true
    }

    pub(crate) fn map_dl_different_individuals(
        &mut self,
        individuals: &[Individual<RcStr>],
    ) -> bool {
        let ids: Vec<EntityId> = individuals
            .iter()
            .filter_map(|i| self.map_individual_entity(i))
            .collect();
        if ids.len() != individuals.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::DifferentIndividuals(ids));
        true
    }

    pub(crate) fn map_dl_disjoint_object_properties(
        &mut self,
        properties: &[ObjectPropertyExpression<RcStr>],
    ) -> bool {
        let ids: Vec<EntityId> = properties
            .iter()
            .filter_map(|ope| match ope {
                ObjectPropertyExpression::ObjectProperty(prop) => {
                    self.register_or_warn_object_property(prop)
                }
                _ => None,
            })
            .collect();
        if ids.len() != properties.len() {
            return false;
        }
        self.push_dl_axiom(DlAxiom::DisjointObjectProperties(ids));
        true
    }

    pub(crate) fn map_dl_swrl_rule(&mut self) -> bool {
        self.push_dl_axiom(DlAxiom::SwrlRule);
        true
    }

    pub(crate) fn map_data_range(&mut self, dr: &DataRange<RcStr>) -> Option<DeId> {
        let expr = self.build_data_range(dr)?;
        Some(self.ontology.dl_mut().intern_de(expr))
    }

    fn build_data_range(&mut self, dr: &DataRange<RcStr>) -> Option<DataExpr> {
        use horned_owl::model::DataRange as DR;
        match dr {
            DR::Datatype(dt) => {
                let id = self.register_or_warn_entity(dt.as_ref(), EntityKind::Datatype)?;
                Some(DataExpr::Datatype(id))
            }
            DR::DataOneOf(literals) => {
                if literals.len() == 1 {
                    self.build_literal(literals.first()?)
                } else {
                    let ids: Vec<DeId> = literals
                        .iter()
                        .filter_map(|lit| {
                            let expr = self.build_literal(lit)?;
                            Some(self.ontology.dl_mut().intern_de(expr))
                        })
                        .collect();
                    if ids.len() != literals.len() {
                        return None;
                    }
                    Some(DataExpr::Or(ids))
                }
            }
            DR::DataComplementOf(inner) => {
                let base = self.map_data_range(inner)?;
                Some(DataExpr::Not(base))
            }
            DR::DataIntersectionOf(ops) => {
                let ids: Vec<DeId> = ops
                    .iter()
                    .filter_map(|op| self.map_data_range(op))
                    .collect();
                if ids.len() != ops.len() {
                    return None;
                }
                Some(DataExpr::And(ids))
            }
            DR::DataUnionOf(ops) => {
                let ids: Vec<DeId> = ops
                    .iter()
                    .filter_map(|op| self.map_data_range(op))
                    .collect();
                if ids.len() != ops.len() {
                    return None;
                }
                Some(DataExpr::Or(ids))
            }
            DR::DatatypeRestriction(dt, facets) => {
                let mut expr = self.build_data_range(&DataRange::Datatype(dt.clone()))?;
                for fr in facets {
                    let base = self.ontology.dl_mut().intern_de(expr);
                    expr = DataExpr::Facet {
                        base,
                        facet_iri: fr.f.as_ref().to_string(),
                        value: literal_lexical(&fr.l),
                    };
                }
                Some(expr)
            }
        }
    }

    fn build_literal(&mut self, lit: &Literal<RcStr>) -> Option<DataExpr> {
        let lexical = literal_lexical(lit);
        let dt_id = match lit {
            Literal::Simple { .. } => self.register_or_warn_entity(
                "http://www.w3.org/2001/XMLSchema#string",
                EntityKind::Datatype,
            )?,
            Literal::Language { .. } => self.register_or_warn_entity(
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString",
                EntityKind::Datatype,
            )?,
            Literal::Datatype { datatype_iri, .. } => {
                self.register_or_warn_entity(datatype_iri.as_ref(), EntityKind::Datatype)?
            }
        };
        Some(DataExpr::Literal {
            lexical,
            datatype: dt_id,
        })
    }

    fn map_data_range_from_ce(&mut self, ce: &ClassExpression<RcStr>) -> Option<DeId> {
        match ce {
            ClassExpression::DataSomeValuesFrom { dr, .. }
            | ClassExpression::DataAllValuesFrom { dr, .. }
            | ClassExpression::DataMinCardinality { dr, .. }
            | ClassExpression::DataMaxCardinality { dr, .. }
            | ClassExpression::DataExactCardinality { dr, .. } => self.map_data_range(dr),
            ClassExpression::DataHasValue { l, .. } => {
                let expr = self.build_literal(l)?;
                Some(self.ontology.dl_mut().intern_de(expr))
            }
            _ => None,
        }
    }

    fn named_object_property_id(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
    ) -> Option<EntityId> {
        match ope {
            ObjectPropertyExpression::ObjectProperty(prop) => self
                .register_or_warn_object_property(prop)
                .or_else(|| self.ontology.lookup_entity(iri_of(prop))),
            ObjectPropertyExpression::InverseObjectProperty(prop) => self
                .register_or_warn_object_property(prop)
                .or_else(|| self.ontology.lookup_entity(iri_of(prop))),
        }
    }

    pub(crate) fn map_dl_transitive_object_property(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
    ) -> bool {
        let Some(role) = self.map_role_expr(ope) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::TransitiveObjectProperty(role));
        true
    }

    pub(crate) fn map_dl_symmetric_object_property(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
    ) -> bool {
        let Some(role) = self.map_role_expr(ope) else {
            return false;
        };
        self.push_dl_axiom(DlAxiom::SymmetricObjectProperty(role));
        true
    }

    /// Map cardinality filler; `owl:Thing` means unqualified (no filler).
    fn universal_or_class_filler(&mut self, bce: &ClassExpression<RcStr>) -> Option<CeId> {
        if let ClassExpression::Class(class) = bce {
            if iri_of(&class.0) == "http://www.w3.org/2002/07/owl#Thing" {
                return None;
            }
        }
        self.map_class_expression(bce)
    }
}

fn iri_of<T: horned_owl::model::ForIRI>(entity: &horned_owl::model::IRI<T>) -> &str {
    entity.as_ref()
}

fn literal_lexical(lit: &Literal<RcStr>) -> String {
    match lit {
        Literal::Language { literal, lang } => format!("{literal}@{lang}"),
        _ => lit.literal().clone(),
    }
}

#[cfg(test)]
mod tests {
    use horned_owl::model::RcStr;
    use horned_owl::ontology::set::SetOntology;
    use ontologos_core::{CeId, DataExpr};
    use ontologos_dl::{is_datatype_consistent, LiteralIndex, LiteralValue};

    use crate::limits::ParseLimits;
    use crate::map::map_to_core;
    use crate::read::read_horned_owl_from_reader;
    use crate::Format;

    #[test]
    fn maps_datatype_union_intersection_axioms() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunionintersection1.ofn");
        let bytes = std::fs::read(&path).unwrap();
        let parsed: SetOntology<RcStr> =
            read_horned_owl_from_reader(&bytes[..], Format::Functional, ParseLimits::default())
                .unwrap();
        let (ont, report) = map_to_core(&parsed, ParseLimits::default()).unwrap();
        assert_eq!(
            report.meta.skipped_axiom_count, 0,
            "{:?}",
            report.meta.warnings
        );
        let store = ont.dl();
        let min_card = store
            .ce(CeId(0))
            .and_then(|ce| match ce {
                ontologos_core::ClassExpr::DataMinCardinality { range, .. } => *range,
                _ => None,
            })
            .expect("MinCard CE");
        let all_range = store
            .ce(CeId(2))
            .and_then(|ce| match ce {
                ontologos_core::ClassExpr::DataAll { range, .. } => Some(*range),
                _ => None,
            })
            .expect("All CE range");
        assert!(matches!(store.de(min_card), Some(DataExpr::Datatype(_))));
        assert!(matches!(store.de(all_range), Some(DataExpr::Not(_))));
    }

    #[test]
    fn maps_all_values_from_mixed1_axioms() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testallvaluesfrommixed1.ofn");
        let bytes = std::fs::read(&path).unwrap();
        let parsed: SetOntology<RcStr> =
            read_horned_owl_from_reader(&bytes[..], Format::Functional, ParseLimits::default())
                .unwrap();
        let (ont, report) = map_to_core(&parsed, ParseLimits::default()).unwrap();
        assert_eq!(
            report.meta.skipped_axiom_count, 0,
            "{:?}",
            report.meta.warnings
        );
        let store = ont.dl();
        let mut all_ranges = Vec::new();
        let mut some_ranges = Vec::new();
        for i in 0..store.ce_count() {
            if let Some(ce) = store.ce(CeId(i as u32)) {
                match ce {
                    ontologos_core::ClassExpr::DataAll { range, .. } => all_ranges.push(*range),
                    ontologos_core::ClassExpr::DataSome { range, .. } => some_ranges.push(*range),
                    _ => {}
                }
            }
        }
        assert_eq!(all_ranges.len(), 2);
        assert_eq!(some_ranges.len(), 1);
        assert!(all_ranges
            .iter()
            .any(|&r| matches!(store.de(r), Some(DataExpr::Or(_)))));
        assert!(all_ranges
            .iter()
            .any(|&r| matches!(store.de(r), Some(DataExpr::Literal { .. }))));
        assert!(matches!(
            store.de(some_ranges[0]),
            Some(DataExpr::Facet { .. })
        ));
    }

    #[test]
    fn maps_datatype_def6_before_property_range() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypedef6.ofn");
        let bytes = std::fs::read(&path).unwrap();
        let parsed: SetOntology<RcStr> =
            read_horned_owl_from_reader(&bytes[..], Format::Functional, ParseLimits::default())
                .unwrap();
        let (ont, report) = map_to_core(&parsed, ParseLimits::default()).unwrap();
        assert_eq!(
            report.meta.skipped_axiom_count, 0,
            "{:?}",
            report.meta.warnings
        );
        let store = ont.dl();
        let mut range_id = None;
        for ax in store.axioms() {
            if let ontologos_core::DlAxiom::DataPropertyRange { range, .. } = ax {
                range_id = Some(*range);
            }
        }
        let range_id = range_id.expect("range");
        let idx = LiteralIndex::from_store(store);
        let lit = LiteralValue {
            lexical: "16".into(),
            datatype: ont
                .lookup_entity("http://www.w3.org/2001/XMLSchema#integer")
                .unwrap(),
        };
        assert!(
            idx.satisfies_with_ontology(&lit, &ont, range_id),
            "de={:?}",
            store.de(range_id)
        );
        assert!(is_datatype_consistent(&ont));
    }

    #[test]
    fn maps_complement_subclass_into_dl_store() {
        let ofn = r#"
            Prefix(:=<http://example.org/>)
            Ontology(<http://example.org/test>
                SubClassOf(:A ObjectComplementOf(:B))
            )
        "#;
        let parsed: SetOntology<RcStr> =
            read_horned_owl_from_reader(ofn.as_bytes(), Format::Functional, ParseLimits::default())
                .unwrap();
        let (ont, report) = map_to_core(&parsed, ParseLimits::default()).unwrap();
        assert_eq!(
            report.meta.skipped_axiom_count, 0,
            "{:?}",
            report.meta.warnings
        );
        assert!(ont.dl().axiom_count() >= 1);
        assert!(ont.dl().ce_count() >= 2);
    }
}
