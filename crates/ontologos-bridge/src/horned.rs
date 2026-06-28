use horned_owl::model::{
    AsymmetricObjectProperty, Build, ClassAssertion, ClassExpression, DeclareClass,
    DeclareNamedIndividual, DeclareObjectProperty, DifferentIndividuals, DisjointClasses,
    EquivalentObjectProperties, FunctionalObjectProperty, Individual,
    InverseFunctionalObjectProperty, InverseObjectProperties, IrreflexiveObjectProperty,
    MutableOntology, ObjectPropertyAssertion, ObjectPropertyDomain, ObjectPropertyExpression,
    ObjectPropertyRange, RcStr, ReflexiveObjectProperty, SameIndividual, SubClassOf,
    SubObjectPropertyExpression, SubObjectPropertyOf, SymmetricObjectProperty,
    TransitiveObjectProperty,
};
use horned_owl::ontology::set::SetOntology;
use ontologos_core::{Axiom, EntityId, EntityKind, Ontology};

use crate::Result;

/// Convert a core ontology into a horned-owl set ontology.
pub fn core_to_horned(ontology: &Ontology) -> Result<SetOntology<RcStr>> {
    let b = Build::new_rc();
    let mut set = SetOntology::new();

    for (id, record) in ontology.entities().iter() {
        let iri = ontology.resolve_iri(record.iri)?;
        match record.kind {
            EntityKind::Class => {
                set.insert(DeclareClass(b.class(iri)));
            }
            EntityKind::Individual => {
                set.insert(DeclareNamedIndividual(b.named_individual(iri)));
            }
            EntityKind::ClassIndividual => {
                set.insert(DeclareClass(b.class(iri)));
                set.insert(DeclareNamedIndividual(b.named_individual(iri)));
            }
            EntityKind::ObjectProperty => {
                set.insert(DeclareObjectProperty(b.object_property(iri)));
            }
            EntityKind::DataProperty | EntityKind::AnnotationProperty | _ => {}
        }
        let _ = id;
    }

    for (_, axiom) in ontology.axioms().iter() {
        map_axiom(ontology, &b, &mut set, axiom)?;
    }

    Ok(set)
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Result<&str> {
    let record = ontology.entity(id)?;
    ontology.resolve_iri(record.iri).map_err(Into::into)
}

fn class_expr(
    ontology: &Ontology,
    b: &Build<RcStr>,
    id: EntityId,
) -> Result<ClassExpression<RcStr>> {
    Ok(ClassExpression::Class(b.class(entity_iri(ontology, id)?)))
}

fn named_individual(
    ontology: &Ontology,
    b: &Build<RcStr>,
    id: EntityId,
) -> Result<Individual<RcStr>> {
    Ok(Individual::Named(
        b.named_individual(entity_iri(ontology, id)?),
    ))
}

fn object_property(
    ontology: &Ontology,
    b: &Build<RcStr>,
    id: EntityId,
) -> Result<ObjectPropertyExpression<RcStr>> {
    Ok(ObjectPropertyExpression::ObjectProperty(
        b.object_property(entity_iri(ontology, id)?),
    ))
}

fn map_axiom(
    ontology: &Ontology,
    b: &Build<RcStr>,
    set: &mut SetOntology<RcStr>,
    axiom: &Axiom,
) -> Result<()> {
    match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => {
            set.insert(SubClassOf {
                sub: class_expr(ontology, b, *subclass)?,
                sup: class_expr(ontology, b, *superclass)?,
            });
        }
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => {
            set.insert(SubClassOf {
                sub: class_expr(ontology, b, *subclass)?,
                sup: ClassExpression::ObjectSomeValuesFrom {
                    ope: object_property(ontology, b, *property)?,
                    bce: Box::new(class_expr(ontology, b, *filler)?),
                },
            });
        }
        Axiom::EquivalentClasses(classes) => {
            for pair in classes.windows(2) {
                set.insert(SubClassOf {
                    sub: class_expr(ontology, b, pair[0])?,
                    sup: class_expr(ontology, b, pair[1])?,
                });
                set.insert(SubClassOf {
                    sub: class_expr(ontology, b, pair[1])?,
                    sup: class_expr(ontology, b, pair[0])?,
                });
            }
        }
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => {
            set.insert(SubObjectPropertyOf {
                sub: SubObjectPropertyExpression::ObjectPropertyExpression(object_property(
                    ontology,
                    b,
                    *sub_property,
                )?),
                sup: object_property(ontology, b, *super_property)?,
            });
        }
        Axiom::ObjectPropertyDomain { property, domain } => {
            set.insert(ObjectPropertyDomain {
                ope: object_property(ontology, b, *property)?,
                ce: class_expr(ontology, b, *domain)?,
            });
        }
        Axiom::ObjectPropertyRange { property, range } => {
            set.insert(ObjectPropertyRange {
                ope: object_property(ontology, b, *property)?,
                ce: class_expr(ontology, b, *range)?,
            });
        }
        Axiom::InverseObjectProperties { left, right } => {
            set.insert(InverseObjectProperties(
                b.object_property(entity_iri(ontology, *left)?),
                b.object_property(entity_iri(ontology, *right)?),
            ));
        }
        Axiom::TransitiveObjectProperty(property) => {
            set.insert(TransitiveObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::SymmetricObjectProperty(property) => {
            set.insert(SymmetricObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::ReflexiveObjectProperty(property) => {
            set.insert(ReflexiveObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::FunctionalObjectProperty(property) => {
            set.insert(FunctionalObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::InverseFunctionalObjectProperty(property) => {
            set.insert(InverseFunctionalObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::IrreflexiveObjectProperty(property) => {
            set.insert(IrreflexiveObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::AsymmetricObjectProperty(property) => {
            set.insert(AsymmetricObjectProperty(object_property(
                ontology, b, *property,
            )?));
        }
        Axiom::EquivalentObjectProperties(properties) => {
            for pair in properties.windows(2) {
                set.insert(EquivalentObjectProperties(vec![
                    object_property(ontology, b, pair[0])?,
                    object_property(ontology, b, pair[1])?,
                ]));
            }
        }
        Axiom::ClassAssertion { individual, class } => {
            set.insert(ClassAssertion {
                ce: class_expr(ontology, b, *class)?,
                i: named_individual(ontology, b, *individual)?,
            });
        }
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => {
            set.insert(ObjectPropertyAssertion {
                ope: object_property(ontology, b, *property)?,
                from: named_individual(ontology, b, *subject)?,
                to: named_individual(ontology, b, *object)?,
            });
        }
        Axiom::DataPropertyAssertion { .. }
        | Axiom::NegativeObjectPropertyAssertion { .. }
        | Axiom::NegativeDataPropertyAssertion { .. } => {}
        Axiom::SameIndividual(individuals) => {
            set.insert(SameIndividual(
                individuals
                    .iter()
                    .map(|id| named_individual(ontology, b, *id))
                    .collect::<Result<Vec<_>>>()?,
            ));
        }
        Axiom::DisjointClasses(classes) => {
            let expressions = classes
                .iter()
                .map(|id| class_expr(ontology, b, *id))
                .collect::<Result<Vec<_>>>()?;
            if expressions.len() >= 2 {
                set.insert(DisjointClasses(expressions));
            }
        }
        Axiom::DifferentIndividuals(individuals) => {
            if individuals.len() >= 2 {
                set.insert(DifferentIndividuals(
                    individuals
                        .iter()
                        .map(|id| named_individual(ontology, b, *id))
                        .collect::<Result<Vec<_>>>()?,
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    #[test]
    fn core_to_horned_exports_disjoint_classes() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::DisjointClasses(vec![a, b]))
            .unwrap();
        let horned = core_to_horned(&ontology).unwrap();
        assert!(horned.iter().any(|annotated| {
            matches!(
                &annotated.component,
                horned_owl::model::Component::DisjointClasses(_)
            )
        }));
    }

    #[test]
    fn core_to_horned_exports_different_individuals() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/a", EntityKind::Individual)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/b", EntityKind::Individual)
            .unwrap();
        ontology
            .add_axiom(Axiom::DifferentIndividuals(vec![a, b]))
            .unwrap();
        let horned = core_to_horned(&ontology).unwrap();
        assert!(horned.iter().any(|annotated| {
            matches!(
                &annotated.component,
                horned_owl::model::Component::DifferentIndividuals(_)
            )
        }));
    }

    #[test]
    fn core_to_horned_round_trip_subclass() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            })
            .unwrap();
        let horned = core_to_horned(&ontology).unwrap();
        assert!(horned.iter().count() >= 2);
    }
}
