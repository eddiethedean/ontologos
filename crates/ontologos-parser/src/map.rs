use std::collections::BTreeMap;

use horned_owl::model::{
    AnnotatedComponent, Class, ClassExpression, Component, Individual, ObjectProperty,
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

    // Pass 1: explicit declarations only, so axiom mapping cannot pin entity kinds
    // before declared types are known (issue #3). Conflicting declarations for the same
    // IRI use last-wins with a warning.
    let mut declaration_kinds: BTreeMap<String, EntityKind> = BTreeMap::new();
    let mut declaration_warnings: Vec<String> = Vec::new();
    for annotated in source.iter() {
        if is_declaration(&annotated.component) {
            if let Some((iri, kind)) = declaration_component(&annotated.component) {
                if let Some(prev) = declaration_kinds.insert(iri.clone(), kind) {
                    if prev != kind {
                        declaration_warnings.push(format!(
                            "entity kind mismatch on declaration for {iri}: {prev:?} then {kind:?}; using last declaration"
                        ));
                    }
                }
            }
        }
    }
    for warning in declaration_warnings {
        report.meta.warn(warning);
    }

    let mut mapper = Mapper {
        ontology: &mut ontology,
        report: &mut report,
        limits,
    };
    for (iri, kind) in declaration_kinds {
        let _ = mapper.register_or_warn_entity(&iri, kind);
    }
    // Pass 2: logical axioms and metadata.
    for annotated in source.iter() {
        if !is_declaration(&annotated.component) {
            mapper.visit(annotated);
        }
    }

    report.meta.logical_axiom_count =
        report.meta.mapped_axiom_count + report.meta.skipped_axiom_count;
    Ok((ontology, report))
}

fn is_declaration(component: &Component<RcStr>) -> bool {
    matches!(
        component,
        Component::DeclareClass(_)
            | Component::DeclareObjectProperty(_)
            | Component::DeclareNamedIndividual(_)
            | Component::DeclareDataProperty(_)
            | Component::DeclareDatatype(_)
            | Component::DeclareAnnotationProperty(_)
    )
}

fn declaration_component(component: &Component<RcStr>) -> Option<(String, EntityKind)> {
    match component {
        Component::DeclareClass(decl) => Some((iri_of(&decl.0).to_owned(), EntityKind::Class)),
        Component::DeclareObjectProperty(decl) => {
            Some((iri_of(&decl.0).to_owned(), EntityKind::ObjectProperty))
        }
        Component::DeclareNamedIndividual(decl) => {
            Some((iri_of(&decl.0).to_owned(), EntityKind::Individual))
        }
        Component::DeclareDataProperty(decl) => {
            Some((iri_of(&decl.0).to_owned(), EntityKind::DataProperty))
        }
        Component::DeclareDatatype(_decl) => None,
        Component::DeclareAnnotationProperty(decl) => {
            Some((iri_of(&decl.0).to_owned(), EntityKind::AnnotationProperty))
        }
        _ => None,
    }
}

pub(crate) struct Mapper<'a> {
    pub(crate) ontology: &'a mut Ontology,
    pub(crate) report: &'a mut ParseReport,
    limits: ParseLimits,
}

#[derive(Copy, Clone)]
enum NamedLookup {
    Resolved(EntityId),
    RegistrationFailed,
    NotNamed,
}

impl NamedLookup {
    fn resolved_id(&self) -> Option<EntityId> {
        match self {
            Self::Resolved(id) => Some(*id),
            Self::RegistrationFailed | Self::NotNamed => None,
        }
    }

    #[allow(dead_code)]
    fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }
}

#[derive(Copy, Clone)]
struct ExistentialRestrictionLookup {
    prop: NamedLookup,
    filler: NamedLookup,
}

impl ExistentialRestrictionLookup {
    fn resolved(&self) -> Option<(EntityId, EntityId)> {
        Some((self.prop.resolved_id()?, self.filler.resolved_id()?))
    }

    fn lookups(&self) -> [NamedLookup; 2] {
        [self.prop, self.filler]
    }
}

fn collect_resolved(lookups: &[NamedLookup]) -> Option<Vec<EntityId>> {
    let mut ids = Vec::with_capacity(lookups.len());
    for lookup in lookups {
        ids.push(lookup.resolved_id()?);
    }
    Some(ids)
}

fn named_lookup_from_register(result: Option<EntityId>) -> NamedLookup {
    match result {
        Some(id) => NamedLookup::Resolved(id),
        None => NamedLookup::RegistrationFailed,
    }
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
                self.map_equivalent_object_properties(&axiom.0);
            }
            Component::DisjointObjectProperties(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DisjointObjectProperties);
                if !self.map_dl_disjoint_object_properties(&axiom.0) {
                    self.skip("DisjointObjectProperties not mapped in v0.2");
                }
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
                if !self.map_dl_inverse_functional(&axiom.0) {
                    self.skip("InverseFunctionalObjectProperty not mapped in v0.2");
                }
            }
            Component::ReflexiveObjectProperty(axiom) => {
                self.map_reflexive_property(&axiom.0);
            }
            Component::IrreflexiveObjectProperty(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::IrreflexiveObjectProperty);
                self.scan_object_property_expression(&axiom.0);
                if !self.map_dl_irreflexive(&axiom.0) {
                    self.skip("IrreflexiveObjectProperty not mapped in v0.2");
                }
            }
            Component::SymmetricObjectProperty(axiom) => {
                self.map_symmetric_property(&axiom.0);
            }
            Component::AsymmetricObjectProperty(axiom) => {
                self.map_asymmetric_property(&axiom.0);
            }
            Component::TransitiveObjectProperty(axiom) => {
                self.map_transitive_property(&axiom.0);
            }
            Component::HasKey(axiom) => {
                self.report.meta.note_construct(OwlConstruct::HasKey);
                self.scan_class_expression(&axiom.ce);
                if !self.map_dl_has_key(&axiom.ce, &axiom.vpe) {
                    self.skip("HasKey not mapped in v0.2");
                }
            }
            Component::ClassAssertion(axiom) => {
                self.map_class_assertion(&axiom.ce, &axiom.i);
            }
            Component::ObjectPropertyAssertion(axiom) => {
                self.map_object_property_assertion(&axiom.ope, &axiom.from, &axiom.to);
            }
            Component::DataPropertyAssertion(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAssertion);
                if !self.map_dl_data_property_assertion(&axiom.dp, &axiom.from, &axiom.to) {
                    self.skip("ABox assertion not mapped in v0.2");
                }
            }
            Component::NegativeObjectPropertyAssertion(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAssertion);
                if !self.map_dl_negative_object_property_assertion(
                    &axiom.ope,
                    &axiom.from,
                    &axiom.to,
                ) {
                    self.skip("ABox assertion not mapped in v0.2");
                }
            }
            Component::NegativeDataPropertyAssertion(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAssertion);
                if !self.map_dl_negative_data_property_assertion(&axiom.dp, &axiom.from, &axiom.to)
                {
                    self.skip("ABox assertion not mapped in v0.2");
                }
            }
            Component::SameIndividual(axiom) => self.map_same_individual(&axiom.0),
            Component::DifferentIndividuals(axiom) => self.map_different_individuals(&axiom.0),
            Component::SubDataPropertyOf(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                if !self.map_dl_sub_data_property(&axiom.sub, &axiom.sup) {
                    self.skip("data property axiom not mapped in v0.2");
                }
            }
            Component::EquivalentDataProperties(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                if !self.map_dl_equivalent_data_properties(&axiom.0) {
                    self.skip("data property axiom not mapped in v0.2");
                }
            }
            Component::DisjointDataProperties(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                if !self.map_dl_disjoint_data_properties(&axiom.0) {
                    self.skip("data property axiom not mapped in v0.2");
                }
            }
            Component::DataPropertyDomain(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                if !self.map_dl_data_property_domain(&axiom.dp, &axiom.ce) {
                    self.skip("data property axiom not mapped in v0.2");
                }
            }
            Component::DataPropertyRange(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                if !self.map_dl_data_property_range(&axiom.dp, &axiom.dr) {
                    self.skip("data property axiom not mapped in v0.2");
                }
            }
            Component::FunctionalDataProperty(axiom) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::DataPropertyAxiom);
                if !self.map_dl_functional_data_property(&axiom.0) {
                    self.skip("data property axiom not mapped in v0.2");
                }
            }
            Component::DatatypeDefinition(axiom) => {
                self.report.meta.note_construct(OwlConstruct::Datatype);
                if !self.map_dl_datatype_definition(&axiom.kind, &axiom.range) {
                    self.skip("DatatypeDefinition not mapped in v0.2");
                }
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
                if !self.map_dl_swrl_rule() {
                    self.skip("SWRL rule not mapped in v0.2");
                }
            }
        }
    }

    fn map_subclass_of(&mut self, sub: &ClassExpression<RcStr>, sup: &ClassExpression<RcStr>) {
        self.scan_class_expression(sub);
        self.scan_class_expression(sup);

        let sub_lookup = self.named_class(sub);
        let sup_lookup = self.named_class(sup);
        let existential = self.try_existential_restriction(sup);

        if let (Some(sub_id), Some(sup_id)) = (sub_lookup.resolved_id(), sup_lookup.resolved_id()) {
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
            (sub_lookup.resolved_id(), existential.resolved())
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

        if let Some(sub_id) = sub_lookup.resolved_id() {
            if self.map_intersection_superclass(sub_id, sup) {
                return;
            }
        }

        let mut lookups = vec![sub_lookup, sup_lookup];
        lookups.extend(existential.lookups());
        if self.map_dl_subclass_of(sub, sup) {
            return;
        }
        self.skip_if_unmapped(
            &lookups,
            "SubClassOf with complex class expression not mapped in v0.2",
        );
    }

    /// Decompose `SubClassOf(C, Intersection(...))` and nested EL operands into core axioms.
    fn map_intersection_superclass(
        &mut self,
        sub_id: EntityId,
        sup: &ClassExpression<RcStr>,
    ) -> bool {
        let ClassExpression::ObjectIntersectionOf(ops) = sup else {
            return false;
        };
        self.report
            .meta
            .note_construct(OwlConstruct::SubClassOfIntersection);
        let mut mapped_any = false;
        for op in ops {
            mapped_any |= self.map_el_superclass_operand(sub_id, op);
        }
        mapped_any
    }

    fn map_el_superclass_operand(
        &mut self,
        sub_id: EntityId,
        operand: &ClassExpression<RcStr>,
    ) -> bool {
        if self.map_intersection_superclass(sub_id, operand) {
            return true;
        }
        if let Some(sup_id) = self.named_class(operand).resolved_id() {
            self.report
                .meta
                .note_construct(OwlConstruct::SubClassOfNamed);
            self.push_axiom(Axiom::SubClassOf {
                subclass: sub_id,
                superclass: sup_id,
            });
            return true;
        }
        if let Some((prop_id, filler_id)) = self.try_existential_restriction(operand).resolved() {
            self.report
                .meta
                .note_construct(OwlConstruct::SubClassOfExistential);
            self.push_axiom(Axiom::SubClassOfExistential {
                subclass: sub_id,
                property: prop_id,
                filler: filler_id,
            });
            return true;
        }
        false
    }

    fn map_equivalent_classes(&mut self, classes: &[ClassExpression<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::EquivalentClasses);
        for ce in classes {
            self.scan_class_expression(ce);
        }
        let lookups: Vec<_> = classes.iter().map(|ce| self.named_class(ce)).collect();
        if let Some(ids) = collect_resolved(&lookups) {
            self.push_axiom(Axiom::EquivalentClasses(ids));
        } else if self.map_dl_equivalent_classes(classes) {
        } else {
            self.skip_if_unmapped(
                &lookups,
                "EquivalentClasses with complex operands not mapped in v0.2",
            );
        }
    }

    fn map_disjoint_classes(&mut self, classes: &[ClassExpression<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::DisjointClasses);
        for ce in classes {
            self.scan_class_expression(ce);
        }
        let lookups: Vec<_> = classes.iter().map(|ce| self.named_class(ce)).collect();
        if let Some(ids) = collect_resolved(&lookups) {
            self.push_axiom(Axiom::DisjointClasses(ids));
        } else if self.map_dl_disjoint_classes(classes) {
        } else {
            self.skip_if_unmapped(
                &lookups,
                "DisjointClasses with complex operands not mapped in v0.2",
            );
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
            SubObjectPropertyExpression::ObjectPropertyChain(chain) => {
                self.report
                    .meta
                    .note_construct(OwlConstruct::SubObjectPropertyChain);
                if !self.map_dl_sub_property_chain(chain, sup) {
                    self.skip("SubObjectPropertyChain not mapped in v0.2");
                }
            }
            SubObjectPropertyExpression::ObjectPropertyExpression(sub_ope) => {
                let sub_lookup = self.named_object_property(sub_ope);
                let sup_lookup = self.named_object_property(sup);
                if let (Some(sub_id), Some(sup_id)) =
                    (sub_lookup.resolved_id(), sup_lookup.resolved_id())
                {
                    self.push_axiom(Axiom::SubObjectPropertyOf {
                        sub_property: sub_id,
                        super_property: sup_id,
                    });
                } else if self.map_dl_sub_object_property_of(sub_ope, sup) {
                } else {
                    self.skip_if_unmapped(
                        &[sub_lookup, sup_lookup],
                        "SubObjectPropertyOf with non-named property not mapped in v0.2",
                    );
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
        let left_lookup = named_lookup_from_register(self.register_or_warn_object_property(left));
        let right_lookup = named_lookup_from_register(self.register_or_warn_object_property(right));
        if let (Some(left_id), Some(right_id)) =
            (left_lookup.resolved_id(), right_lookup.resolved_id())
        {
            self.push_axiom(Axiom::InverseObjectProperties {
                left: left_id,
                right: right_id,
            });
        } else {
            self.skip_if_unmapped(
                &[left_lookup, right_lookup],
                "InverseObjectProperties with unmapped operands not mapped in v0.2",
            );
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
        let prop_lookup = self.named_object_property(ope);
        let domain_lookup = self.named_class(ce);
        if let (Some(prop_id), Some(domain_id)) =
            (prop_lookup.resolved_id(), domain_lookup.resolved_id())
        {
            self.push_axiom(Axiom::ObjectPropertyDomain {
                property: prop_id,
                domain: domain_id,
            });
        } else if self.map_dl_property_domain(ope, ce) {
        } else {
            self.skip_if_unmapped(
                &[prop_lookup, domain_lookup],
                "ObjectPropertyDomain with complex operands not mapped in v0.2",
            );
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
        let prop_lookup = self.named_object_property(ope);
        let range_lookup = self.named_class(ce);
        if let (Some(prop_id), Some(range_id)) =
            (prop_lookup.resolved_id(), range_lookup.resolved_id())
        {
            self.push_axiom(Axiom::ObjectPropertyRange {
                property: prop_id,
                range: range_id,
            });
        } else if self.map_dl_property_range(ope, ce) {
        } else {
            self.skip_if_unmapped(
                &[prop_lookup, range_lookup],
                "ObjectPropertyRange with complex operands not mapped in v0.2",
            );
        }
    }

    fn map_transitive_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::TransitiveObjectProperty);
        let prop_lookup = self.named_object_property(ope);
        if let Some(prop_id) = prop_lookup.resolved_id() {
            self.push_axiom(Axiom::TransitiveObjectProperty(prop_id));
        } else if self.map_dl_transitive_object_property(ope) {
        } else {
            self.skip_if_unmapped(
                &[prop_lookup],
                "TransitiveObjectProperty with non-named property not mapped in v0.2",
            );
        }
    }

    fn map_symmetric_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::SymmetricObjectProperty);
        let prop_lookup = self.named_object_property(ope);
        if let Some(prop_id) = prop_lookup.resolved_id() {
            self.push_axiom(Axiom::SymmetricObjectProperty(prop_id));
        } else if self.map_dl_symmetric_object_property(ope) {
        } else {
            self.skip_if_unmapped(
                &[prop_lookup],
                "SymmetricObjectProperty with non-named property not mapped in v0.2",
            );
        }
    }

    fn map_reflexive_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::ReflexiveObjectProperty);
        let prop_lookup = self.named_object_property(ope);
        if let Some(prop_id) = prop_lookup.resolved_id() {
            self.push_axiom(Axiom::ReflexiveObjectProperty(prop_id));
        } else {
            self.skip_if_unmapped(
                &[prop_lookup],
                "ReflexiveObjectProperty with non-named property not mapped in v0.2",
            );
        }
    }

    fn map_functional_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::FunctionalObjectProperty);
        let prop_lookup = self.named_object_property(ope);
        if let Some(prop_id) = prop_lookup.resolved_id() {
            self.push_axiom(Axiom::FunctionalObjectProperty(prop_id));
        } else {
            self.skip_if_unmapped(
                &[prop_lookup],
                "FunctionalObjectProperty with non-named property not mapped in v0.2",
            );
        }
    }

    fn map_asymmetric_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::AsymmetricObjectProperty);
        let prop_lookup = self.named_object_property(ope);
        if let Some(prop_id) = prop_lookup.resolved_id() {
            self.push_axiom(Axiom::AsymmetricObjectProperty(prop_id));
        } else {
            self.skip_if_unmapped(
                &[prop_lookup],
                "AsymmetricObjectProperty with non-named property not mapped in v0.4",
            );
        }
    }

    fn map_equivalent_object_properties(&mut self, properties: &[ObjectPropertyExpression<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::EquivalentObjectProperties);
        for ope in properties {
            self.scan_object_property_expression(ope);
        }
        let lookups: Vec<_> = properties
            .iter()
            .map(|ope| self.named_object_property(ope))
            .collect();
        if let Some(ids) = collect_resolved(&lookups) {
            self.push_axiom(Axiom::EquivalentObjectProperties(ids));
        } else {
            self.skip_if_unmapped(
                &lookups,
                "EquivalentObjectProperties with non-named operands not mapped in v0.4",
            );
        }
    }

    fn map_class_assertion(&mut self, ce: &ClassExpression<RcStr>, individual: &Individual<RcStr>) {
        self.report
            .meta
            .note_construct(OwlConstruct::ClassAssertion);
        self.scan_class_expression(ce);
        let class_lookup = self.named_class(ce);
        let individual_lookup = self.named_individual(individual);
        if let (Some(class_id), Some(individual_id)) =
            (class_lookup.resolved_id(), individual_lookup.resolved_id())
        {
            self.push_axiom(Axiom::ClassAssertion {
                individual: individual_id,
                class: class_id,
            });
        } else if self.map_dl_class_assertion(ce, individual) {
        } else {
            self.skip_if_unmapped(
                &[class_lookup, individual_lookup],
                "ClassAssertion with complex operands not mapped in v0.4",
            );
        }
    }

    fn map_object_property_assertion(
        &mut self,
        ope: &ObjectPropertyExpression<RcStr>,
        from: &Individual<RcStr>,
        to: &Individual<RcStr>,
    ) {
        self.report
            .meta
            .note_construct(OwlConstruct::ObjectPropertyAssertion);
        self.scan_object_property_expression(ope);
        let property_lookup = self.named_object_property(ope);
        let from_lookup = self.named_individual(from);
        let to_lookup = self.named_individual(to);
        if let (Some(property_id), Some(subject_id), Some(object_id)) = (
            property_lookup.resolved_id(),
            from_lookup.resolved_id(),
            to_lookup.resolved_id(),
        ) {
            self.push_axiom(Axiom::ObjectPropertyAssertion {
                subject: subject_id,
                property: property_id,
                object: object_id,
            });
        } else if self.map_dl_object_property_assertion(ope, from, to) {
        } else {
            self.skip_if_unmapped(
                &[property_lookup, from_lookup, to_lookup],
                "ObjectPropertyAssertion with complex operands not mapped in v0.4",
            );
        }
    }

    fn map_same_individual(&mut self, individuals: &[Individual<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::IndividualEquality);
        let lookups: Vec<_> = individuals
            .iter()
            .map(|individual| self.named_individual(individual))
            .collect();
        if let Some(ids) = collect_resolved(&lookups) {
            self.push_axiom(Axiom::SameIndividual(ids));
        } else if self.map_dl_same_individual(individuals) {
        } else {
            self.skip_if_unmapped(
                &lookups,
                "SameIndividual with anonymous individuals not mapped in v0.4",
            );
        }
    }

    fn map_different_individuals(&mut self, individuals: &[Individual<RcStr>]) {
        self.report
            .meta
            .note_construct(OwlConstruct::IndividualEquality);
        let lookups: Vec<_> = individuals
            .iter()
            .map(|individual| self.named_individual(individual))
            .collect();
        if let Some(ids) = collect_resolved(&lookups) {
            self.push_axiom(Axiom::DifferentIndividuals(ids));
        } else if self.map_dl_different_individuals(individuals) {
        } else {
            self.skip_if_unmapped(
                &lookups,
                "DifferentIndividuals with anonymous individuals not mapped in v0.4",
            );
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

    fn skip_if_unmapped(&mut self, lookups: &[NamedLookup], message: &str) {
        if lookups
            .iter()
            .any(|lookup| matches!(lookup, NamedLookup::RegistrationFailed))
        {
            self.report.meta.skipped_axiom_count += 1;
        } else {
            self.skip(message);
        }
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
            Axiom::AsymmetricObjectProperty(_) => OwlConstruct::AsymmetricObjectProperty,
            Axiom::EquivalentObjectProperties(_) => OwlConstruct::EquivalentObjectProperties,
            Axiom::ClassAssertion { .. } => OwlConstruct::ClassAssertion,
            Axiom::ObjectPropertyAssertion { .. } => OwlConstruct::ObjectPropertyAssertion,
            Axiom::SameIndividual(_) | Axiom::DifferentIndividuals(_) => {
                OwlConstruct::IndividualEquality
            }
        };
        self.report.meta.note_profile_construct(construct);
        if matches!(axiom, Axiom::SubClassOfExistential { .. }) {
            self.report
                .meta
                .note_profile_construct(OwlConstruct::ObjectSomeValuesFrom);
        }
    }

    pub(crate) fn register_or_warn_class(&mut self, class: &Class<RcStr>) -> Option<EntityId> {
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

    pub(crate) fn register_or_warn_object_property(
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

    pub(crate) fn register_or_warn_entity(
        &mut self,
        iri: &str,
        kind: EntityKind,
    ) -> Option<EntityId> {
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
            ClassExpression::Class(class) => {
                let _ = self.register_or_warn_class(class);
            }
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
        if let ObjectPropertyExpression::ObjectProperty(prop) = ope {
            let _ = self.register_or_warn_object_property(prop);
        }
        if matches!(ope, ObjectPropertyExpression::InverseObjectProperty(_)) {
            self.report
                .meta
                .note_construct(OwlConstruct::InverseObjectProperties);
        }
    }

    fn named_class(&mut self, ce: &ClassExpression<RcStr>) -> NamedLookup {
        match ce {
            ClassExpression::Class(class) => {
                named_lookup_from_register(self.register_or_warn_class(class))
            }
            _ => NamedLookup::NotNamed,
        }
    }

    fn try_existential_restriction(
        &mut self,
        ce: &ClassExpression<RcStr>,
    ) -> ExistentialRestrictionLookup {
        match ce {
            ClassExpression::ObjectSomeValuesFrom { ope, bce } => ExistentialRestrictionLookup {
                prop: self.named_object_property(ope),
                filler: self.named_class(bce),
            },
            _ => ExistentialRestrictionLookup {
                prop: NamedLookup::NotNamed,
                filler: NamedLookup::NotNamed,
            },
        }
    }

    fn named_object_property(&mut self, ope: &ObjectPropertyExpression<RcStr>) -> NamedLookup {
        match ope {
            ObjectPropertyExpression::ObjectProperty(prop) => {
                named_lookup_from_register(self.register_or_warn_object_property(prop))
            }
            ObjectPropertyExpression::InverseObjectProperty(_) => NamedLookup::NotNamed,
        }
    }

    fn named_individual(&mut self, individual: &Individual<RcStr>) -> NamedLookup {
        named_lookup_from_register(self.map_individual_entity(individual))
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

    #[test]
    fn class_assertion_kind_mismatch_warns_without_complex_operands_skip() {
        use std::path::Path;

        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/class_individual_kind_clash.ttl");
        let ontology = crate::load_ontology(&path).expect("load");
        let meta = ontology.parse_meta().expect("parse_meta");

        assert!(
            meta.warnings
                .iter()
                .any(|w| w.contains("entity kind mismatch")),
            "expected kind mismatch warning, got: {:?}",
            meta.warnings
        );
        assert!(
            !meta.warnings.iter().any(|w| w.contains("complex operands")),
            "should not mislabel kind clash as unmapped complex expression"
        );
    }
}
