use horned_owl::model::{
    AnnotatedComponent, Class, ClassExpression, Component, ObjectProperty,
    ObjectPropertyExpression, RcStr, SubObjectPropertyExpression,
};
use horned_owl::ontology::set::SetOntology;
use ontologos_core::{Axiom, EntityId, EntityKind, Error as CoreError, Ontology, OwlConstruct};

use crate::limits::ParseLimits;
use crate::report::ParseReport;
use crate::{Error, Result};

pub fn map_to_core(
    source: &SetOntology<RcStr>,
    limits: ParseLimits,
) -> Result<(Ontology, ParseReport)> {
    let mut ontology = Ontology::new();
    let mut report = ParseReport::new();
    let mut mapper = Mapper {
        ontology: &mut ontology,
        report: &mut report,
        limits,
    };

    for annotated in source.iter() {
        mapper.visit(annotated);
    }

    report.meta.logical_axiom_count =
        report.meta.mapped_axiom_count + report.meta.skipped_axiom_count;
    Ok((ontology, report))
}

struct Mapper<'a> {
    ontology: &'a mut Ontology,
    report: &'a mut ParseReport,
    limits: ParseLimits,
}

impl Mapper<'_> {
    fn visit(&mut self, annotated: &AnnotatedComponent<RcStr>) {
        match &annotated.component {
            Component::DeclareClass(decl) => {
                let _ = self.register_or_warn_class(&decl.0);
            }
            Component::DeclareObjectProperty(decl) => {
                let _ = self.register_or_warn_object_property(&decl.0);
            }
            Component::DeclareNamedIndividual(decl) => {
                let _ = self.register_or_warn_entity(iri_of(&decl.0), EntityKind::Individual);
            }
            Component::DeclareDataProperty(decl) => {
                let _ = self.register_or_warn_entity(iri_of(&decl.0), EntityKind::DataProperty);
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
            }
            Component::DeclareDatatype(_decl) => {
                self.report.meta.note_construct(OwlConstruct::Datatype);
            }
            Component::SubClassOf(axiom) => self.map_subclass_of(&axiom.sub, &axiom.sup),
            Component::EquivalentClasses(axiom) => self.map_equivalent_classes(&axiom.0),
            Component::DisjointClasses(axiom) => self.map_disjoint_classes(&axiom.0),
            Component::DisjointUnion(axiom) => {
                self.report.meta.note_construct(OwlConstruct::DisjointUnion);
                let _ = &axiom.0;
                self.skip("DisjointUnion not mapped in v0.2");
            }
            Component::SubObjectPropertyOf(axiom) => {
                self.map_sub_object_property(&axiom.sub, &axiom.sup);
            }
            Component::EquivalentObjectProperties(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::EquivalentObjectProperties);
                self.skip(&format!(
                    "EquivalentObjectProperties ({} operands) not mapped in v0.2",
                    axiom.0.len()
                ));
            }
            Component::DisjointObjectProperties(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DisjointObjectProperties);
                self.skip("DisjointObjectProperties not mapped in v0.2");
            }
            Component::InverseObjectProperties(axiom) => {
                self.map_inverse_properties(&axiom.0, &axiom.1);
            }
            Component::ObjectPropertyDomain(axiom) => {
                self.map_property_domain(&axiom.ope, &axiom.ce);
            }
            Component::ObjectPropertyRange(axiom) => {
                self.map_property_range(&axiom.ope, &axiom.ce);
            }
            Component::FunctionalObjectProperty(axiom) => {
                self.map_functional_property(&axiom.0);
            }
            Component::InverseFunctionalObjectProperty(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::InverseFunctionalObjectProperty);
                self.scan_object_property_expression(&axiom.0);
                self.skip("InverseFunctionalObjectProperty not mapped in v0.2");
            }
            Component::ReflexiveObjectProperty(axiom) => {
                self.map_reflexive_property(&axiom.0);
            }
            Component::IrreflexiveObjectProperty(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::IrreflexiveObjectProperty);
                self.scan_object_property_expression(&axiom.0);
                self.skip("IrreflexiveObjectProperty not mapped in v0.2");
            }
            Component::SymmetricObjectProperty(axiom) => {
                self.map_symmetric_property(&axiom.0);
            }
            Component::AsymmetricObjectProperty(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::AsymmetricObjectProperty);
                self.scan_object_property_expression(&axiom.0);
                self.skip("AsymmetricObjectProperty not mapped in v0.2");
            }
            Component::TransitiveObjectProperty(axiom) => {
                self.map_transitive_property(&axiom.0);
            }
            Component::HasKey(axiom) => {
                self.report.meta.note_construct(OwlConstruct::HasKey);
                self.scan_class_expression(&axiom.ce);
                let _ = &axiom.vpe;
                self.skip("HasKey not mapped in v0.2");
            }
            Component::ClassAssertion(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ClassAssertion);
                self.skip("ABox assertion not mapped in v0.2");
            }
            Component::ObjectPropertyAssertion(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectPropertyAssertion);
                self.skip("ABox assertion not mapped in v0.2");
            }
            Component::DataPropertyAssertion(_)
            | Component::NegativeObjectPropertyAssertion(_)
            | Component::NegativeDataPropertyAssertion(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAssertion);
                self.skip("ABox assertion not mapped in v0.2");
            }
            Component::SameIndividual(_) | Component::DifferentIndividuals(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::IndividualEquality);
                self.skip("individual equality axiom not mapped in v0.2");
            }
            Component::SubDataPropertyOf(_)
            | Component::EquivalentDataProperties(_)
            | Component::DisjointDataProperties(_)
            | Component::DataPropertyDomain(_)
            | Component::DataPropertyRange(_)
            | Component::FunctionalDataProperty(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                self.skip("data property axiom not mapped in v0.2");
            }
            Component::DatatypeDefinition(_) => {
                self.report.meta.note_construct(OwlConstruct::Datatype);
                self.skip("DatatypeDefinition not mapped in v0.2");
            }
            Component::Import(_) => {
                self.report.meta.note_construct(OwlConstruct::Import);
            }
            Component::OntologyAnnotation(_)
            | Component::AnnotationAssertion(_)
            | Component::SubAnnotationPropertyOf(_)
            | Component::AnnotationPropertyDomain(_)
            | Component::AnnotationPropertyRange(_) => {
                self.report.meta.note_construct(OwlConstruct::Annotation);
            }
            Component::DeclareAnnotationProperty(_) => {}
            Component::OntologyID(_) | Component::DocIRI(_) => {}
            Component::Rule(_) => {
                self.report.meta.note_construct(OwlConstruct::SwrlRule);
                self.skip("SWRL rule not mapped in v0.2");
            }
        }
    }

    fn map_subclass_of(&mut self, sub: &ClassExpression<RcStr>, sup: &ClassExpression<RcStr>) {
        self.scan_class_expression(sub);
        self.scan_class_expression(sup);

        if let (Some(sub_id), Some(sup_id)) = (self.named_class(sub), self.named_class(sup)) {
            self.report
                .meta
                .note_construct(OwlConstruct::SubClassOfNamed);
            self.push_axiom(Axiom::SubClassOf {
                subclass: sub_id,
                superclass: sup_id,
            });
            return;
        }

        if let (Some(sub_id), Some((prop_id, filler_id))) =
            (self.named_class(sub), self.existential_restriction(sup))
        {
            self.report
                .meta
                .note_construct(OwlConstruct::SubClassOfExistential);
            self.push_axiom(Axiom::SubClassOfExistential {
                subclass: sub_id,
                property: prop_id,
                filler: filler_id,
            });
            return;
        }

        if self.named_class(sub).is_some()
            && matches!(sup, ClassExpression::ObjectIntersectionOf(_))
        {
            self.report
                .meta
                .note_construct(OwlConstruct::SubClassOfIntersection);
        }

        self.skip("SubClassOf with complex class expression not mapped in v0.2");
    }

    fn map_equivalent_classes(&mut self, classes: &[ClassExpression<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::EquivalentClasses);
        for ce in classes {
            self.scan_class_expression(ce);
        }
        if let Some(ids) = self.all_named_classes(classes) {
            self.push_axiom(Axiom::EquivalentClasses(ids));
        } else {
            self.skip("EquivalentClasses with complex operands not mapped in v0.2");
        }
    }

    fn map_disjoint_classes(&mut self, classes: &[ClassExpression<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::DisjointClasses);
        for ce in classes {
            self.scan_class_expression(ce);
        }
        if let Some(ids) = self.all_named_classes(classes) {
            self.push_axiom(Axiom::DisjointClasses(ids));
        } else {
            self.skip("DisjointClasses with complex operands not mapped in v0.2");
        }
    }

    fn map_sub_object_property(
        &mut self,
        sub: &SubObjectPropertyExpression<RcStr>,
        sup: &ObjectPropertyExpression<RcStr>,
    ) {
        self.report
            .meta
            .note_construct(OwlConstruct::SubObjectPropertyOf);
        match sub {
            SubObjectPropertyExpression::ObjectPropertyChain(_) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::SubObjectPropertyChain);
                self.skip("SubObjectPropertyChain not mapped in v0.2");
            }
            SubObjectPropertyExpression::ObjectPropertyExpression(sub_ope) => {
                if let (Some(sub_id), Some(sup_id)) = (
                    self.named_object_property(sub_ope),
                    self.named_object_property(sup),
                ) {
                    self.push_axiom(Axiom::SubObjectPropertyOf {
                        sub_property: sub_id,
                        super_property: sup_id,
                    });
                } else {
                    self.skip("SubObjectPropertyOf with non-named property not mapped in v0.2");
                }
            }
        }
    }

    fn map_inverse_properties(
        &mut self,
        left: &ObjectProperty<RcStr>,
        right: &ObjectProperty<RcStr>,
    ) {
        self.report
            .meta
            .note_construct(OwlConstruct::InverseObjectProperties);
        match (
            self.register_object_property(left),
            self.register_object_property(right),
        ) {
            (Ok(left_id), Ok(right_id)) => {
                self.push_axiom(Axiom::InverseObjectProperties {
                    left: left_id,
                    right: right_id,
                });
            }
            (Err(err), _) | (_, Err(err)) => {
                self.skip(&format!(
                    "InverseObjectProperties registration failed: {err}"
                ));
            }
        }
    }

    fn map_property_domain(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        ce: &ClassExpression<RcStr>,
    ) {
        self.report
            .meta
            .note_construct(OwlConstruct::ObjectPropertyDomain);
        self.scan_object_property_expression(ope);
        self.scan_class_expression(ce);
        if let (Some(prop_id), Some(domain_id)) =
            (self.named_object_property(ope), self.named_class(ce))
        {
            self.push_axiom(Axiom::ObjectPropertyDomain {
                property: prop_id,
                domain: domain_id,
            });
        } else {
            self.skip("ObjectPropertyDomain with complex operands not mapped in v0.2");
        }
    }

    fn map_property_range(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        ce: &ClassExpression<RcStr>,
    ) {
        self.report
            .meta
            .note_construct(OwlConstruct::ObjectPropertyRange);
        self.scan_object_property_expression(ope);
        self.scan_class_expression(ce);
        if let (Some(prop_id), Some(range_id)) =
            (self.named_object_property(ope), self.named_class(ce))
        {
            self.push_axiom(Axiom::ObjectPropertyRange {
                property: prop_id,
                range: range_id,
            });
        } else {
            self.skip("ObjectPropertyRange with complex operands not mapped in v0.2");
        }
    }

    fn map_transitive_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::TransitiveObjectProperty);
        if let Some(prop_id) = self.named_object_property(ope) {
            self.push_axiom(Axiom::TransitiveObjectProperty(prop_id));
        } else {
            self.skip("TransitiveObjectProperty with non-named property not mapped in v0.2");
        }
    }

    fn map_symmetric_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::SymmetricObjectProperty);
        if let Some(prop_id) = self.named_object_property(ope) {
            self.push_axiom(Axiom::SymmetricObjectProperty(prop_id));
        } else {
            self.skip("SymmetricObjectProperty with non-named property not mapped in v0.2");
        }
    }

    fn map_reflexive_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::ReflexiveObjectProperty);
        if let Some(prop_id) = self.named_object_property(ope) {
            self.push_axiom(Axiom::ReflexiveObjectProperty(prop_id));
        } else {
            self.skip("ReflexiveObjectProperty with non-named property not mapped in v0.2");
        }
    }

    fn map_functional_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::FunctionalObjectProperty);
        if let Some(prop_id) = self.named_object_property(ope) {
            self.push_axiom(Axiom::FunctionalObjectProperty(prop_id));
        } else {
            self.skip("FunctionalObjectProperty with non-named property not mapped in v0.2");
        }
    }

    fn push_axiom(&mut self, axiom: Axiom) {
        if self.ontology.axiom_count() >= self.limits.max_axioms {
            self.report.meta.warn(format!(
                "axiom limit {} reached; skipping further axioms",
                self.limits.max_axioms
            ));
            self.report.meta.skipped_axiom_count += 1;
            return;
        }
        match self.ontology.add_axiom(axiom.clone()) {
            Ok(_) => {
                if self.ontology.axiom_count() > self.report.meta.mapped_axiom_count {
                    self.report.meta.mapped_axiom_count += 1;
                }
                self.note_profile_axiom(&axiom);
            }
            Err(err) => {
                self.report.meta.skipped_axiom_count += 1;
                self.report
                    .meta
                    .warn(format!("failed to store axiom: {err}"));
            }
        }
    }

    fn skip(&mut self, message: &str) {
        self.report.meta.skipped_axiom_count += 1;
        self.report.meta.warn(message);
    }

    fn note_profile_axiom(&mut self, axiom: &Axiom) {
        let construct = match axiom {
            Axiom::SubClassOf { .. } => OwlConstruct::SubClassOfNamed,
            Axiom::SubClassOfExistential { .. } => OwlConstruct::SubClassOfExistential,
            Axiom::EquivalentClasses(_) => OwlConstruct::EquivalentClasses,
            Axiom::DisjointClasses(_) => OwlConstruct::DisjointClasses,
            Axiom::ObjectPropertyDomain { .. } => OwlConstruct::ObjectPropertyDomain,
            Axiom::ObjectPropertyRange { .. } => OwlConstruct::ObjectPropertyRange,
            Axiom::SubObjectPropertyOf { .. } => OwlConstruct::SubObjectPropertyOf,
            Axiom::InverseObjectProperties { .. } => OwlConstruct::InverseObjectProperties,
            Axiom::TransitiveObjectProperty(_) => OwlConstruct::TransitiveObjectProperty,
            Axiom::SymmetricObjectProperty(_) => OwlConstruct::SymmetricObjectProperty,
            Axiom::ReflexiveObjectProperty(_) => OwlConstruct::ReflexiveObjectProperty,
            Axiom::FunctionalObjectProperty(_) => OwlConstruct::FunctionalObjectProperty,
        };
        self.report.meta.note_profile_construct(construct);
        if matches!(axiom, Axiom::SubClassOfExistential { .. }) {
            self.report
                .meta
                .note_profile_construct(OwlConstruct::ObjectSomeValuesFrom);
        }
    }

    fn register_or_warn_class(&mut self, class: &Class<RcStr>) -> Option<EntityId> {
        match self.register_class(class) {
            Ok(id) => Some(id),
            Err(err) => {
                self.report
                    .meta
                    .warn(format!("failed to register class {}: {err}", iri_of(class)));
                None
            }
        }
    }

    fn register_or_warn_object_property(
        &mut self,
        property: &ObjectProperty<RcStr>,
    ) -> Option<EntityId> {
        match self.register_object_property(property) {
            Ok(id) => Some(id),
            Err(err) => {
                self.report.meta.warn(format!(
                    "failed to register object property {}: {err}",
                    iri_of(property)
                ));
                None
            }
        }
    }

    fn register_or_warn_entity(&mut self, iri: &str, kind: EntityKind) -> Option<EntityId> {
        match self.register_entity(iri, kind) {
            Ok(id) => Some(id),
            Err(err) => {
                self.report
                    .meta
                    .warn(format!("failed to register entity {iri}: {err}"));
                None
            }
        }
    }

    fn register_class(&mut self, class: &Class<RcStr>) -> Result<EntityId> {
        self.register_entity(iri_of(class), EntityKind::Class)
    }

    fn register_object_property(&mut self, property: &ObjectProperty<RcStr>) -> Result<EntityId> {
        self.register_entity(iri_of(property), EntityKind::ObjectProperty)
    }

    fn register_entity(&mut self, iri: &str, kind: EntityKind) -> Result<EntityId> {
        if self.ontology.entity_count() >= self.limits.max_entities {
            return Err(Error::Parse(format!(
                "entity limit {} exceeded",
                self.limits.max_entities
            )));
        }
        self.ontology
            .entity_id(iri, kind)
            .map_err(|e: CoreError| Error::Parse(e.to_string()))
    }

    fn scan_class_expression(&mut self, ce: &ClassExpression<RcStr>) {
        match ce {
            ClassExpression::Class(_) => {}
            ClassExpression::ObjectIntersectionOf(ops) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectIntersectionOf);
                for op in ops {
                    self.scan_class_expression(op);
                }
            }
            ClassExpression::ObjectUnionOf(ops) => {
                self.report.meta.note_construct(OwlConstruct::ObjectUnionOf);
                for op in ops {
                    self.scan_class_expression(op);
                }
            }
            ClassExpression::ObjectComplementOf(inner) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectComplementOf);
                self.scan_class_expression(inner);
            }
            ClassExpression::ObjectOneOf(_) => {
                self.report.meta.note_construct(OwlConstruct::ObjectOneOf);
            }
            ClassExpression::ObjectSomeValuesFrom { ope, bce } => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectSomeValuesFrom);
                self.scan_object_property_expression(ope);
                self.scan_class_expression(bce);
            }
            ClassExpression::ObjectAllValuesFrom { ope, bce } => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectAllValuesFrom);
                self.scan_object_property_expression(ope);
                self.scan_class_expression(bce);
            }
            ClassExpression::ObjectHasValue { .. } => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectHasValue);
            }
            ClassExpression::ObjectHasSelf(_) => {
                self.report.meta.note_construct(OwlConstruct::ObjectHasSelf);
            }
            ClassExpression::ObjectMinCardinality { .. }
            | ClassExpression::ObjectMaxCardinality { .. }
            | ClassExpression::ObjectExactCardinality { .. } => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::ObjectCardinality);
            }
            ClassExpression::DataSomeValuesFrom { .. }
            | ClassExpression::DataAllValuesFrom { .. }
            | ClassExpression::DataHasValue { .. }
            | ClassExpression::DataMinCardinality { .. }
            | ClassExpression::DataMaxCardinality { .. }
            | ClassExpression::DataExactCardinality { .. } => {
                self.report.meta.note_construct(OwlConstruct::Datatype);
            }
        }
    }

    fn scan_object_property_expression(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        if matches!(ope, ObjectPropertyExpression::InverseObjectProperty(_)) {
            self.report
                .meta
                .note_construct(OwlConstruct::InverseObjectProperties);
        }
    }

    fn named_class(&mut self, ce: &ClassExpression<RcStr>) -> Option<EntityId> {
        match ce {
            ClassExpression::Class(class) => self.register_class(class).ok(),
            _ => None,
        }
    }

    fn all_named_classes(&mut self, classes: &[ClassExpression<RcStr>]) -> Option<Vec<EntityId>> {
        let mut ids = Vec::with_capacity(classes.len());
        for ce in classes {
            ids.push(self.named_class(ce)?);
        }
        Some(ids)
    }

    fn existential_restriction(
        &mut self,
        ce: &ClassExpression<RcStr>,
    ) -> Option<(EntityId, EntityId)> {
        match ce {
            ClassExpression::ObjectSomeValuesFrom { ope, bce } => {
                let prop_id = self.named_object_property(ope)?;
                let filler_id = self.named_class(bce)?;
                Some((prop_id, filler_id))
            }
            _ => None,
        }
    }

    fn named_object_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) -> Option<EntityId> {
        match ope {
            ObjectPropertyExpression::ObjectProperty(prop) => {
                self.register_object_property(prop).ok()
            }
            ObjectPropertyExpression::InverseObjectProperty(_) => None,
        }
    }
}

fn iri_of<T: horned_owl::model::ForIRI>(entity: &horned_owl::model::IRI<T>) -> &str {
    entity.as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::{EntityKind, Ontology};

    #[test]
    fn push_axiom_allows_axioms_at_entity_limit() {
        let mut ontology = Ontology::new();
        let mut report = ParseReport::new();
        let limits = ParseLimits {
            max_entities: 2,
            ..ParseLimits::default()
        };
        {
            let mut mapper = Mapper {
                ontology: &mut ontology,
                report: &mut report,
                limits,
            };
            let a = mapper
                .register_entity("http://ex.org/A", EntityKind::Class)
                .expect("A");
            let b = mapper
                .register_entity("http://ex.org/B", EntityKind::Class)
                .expect("B");
            mapper.push_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            });
        }
        assert_eq!(ontology.entity_count(), 2);
        assert_eq!(ontology.axiom_count(), 1);
    }
}
