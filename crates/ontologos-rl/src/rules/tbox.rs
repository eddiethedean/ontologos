use ontologos_core::{Axiom, EntityId, EntityKind};

use super::helpers::{
    expand_equivalent_classes, expand_equivalent_properties, infer_axiom, is_subclass_of,
    transitive_subproperties, transitive_superclasses, transitive_superproperties, RuleContext,
};
use crate::report::RlRule;

pub(crate) fn apply_batch_a(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    apply_equivalent_classes(ctx)?;
    apply_equivalent_properties(ctx)?;
    apply_characteristic_propagation(ctx)?;
    apply_existential_subproperty(ctx)?;
    apply_existential_subsumption(ctx)?;
    Ok(())
}

fn apply_equivalent_classes(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let classes: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    for class in classes {
        let equiv: Vec<EntityId> = ctx
            .ontology
            .equivalents_of(class)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();
        for other in equiv {
            if class != other {
                infer_axiom(
                    ctx,
                    RlRule::EqClassSub,
                    Axiom::SubClassOf {
                        subclass: class,
                        superclass: other,
                    },
                    vec![],
                )?;
            }
        }
    }
    Ok(())
}

fn apply_equivalent_properties(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let properties: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    for property in properties {
        let equiv: Vec<EntityId> = ctx
            .ontology
            .equivalent_properties_of(property)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();
        for other in equiv {
            if property != other {
                infer_axiom(
                    ctx,
                    RlRule::EqPropSub,
                    Axiom::SubObjectPropertyOf {
                        sub_property: property,
                        super_property: other,
                    },
                    vec![],
                )?;
            }
        }
    }
    Ok(())
}

fn apply_characteristic_propagation(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let properties: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    for sub_property in properties {
        for super_property in transitive_superproperties(ctx.ontology, sub_property) {
            if super_property == sub_property {
                continue;
            }
            if ctx
                .ontology
                .index()
                .transitive_properties()
                .contains(&super_property)
            {
                infer_axiom(
                    ctx,
                    RlRule::CharPropagate,
                    Axiom::TransitiveObjectProperty(sub_property),
                    vec![],
                )?;
            }
            if ctx
                .ontology
                .index()
                .symmetric_properties()
                .contains(&super_property)
            {
                infer_axiom(
                    ctx,
                    RlRule::CharPropagate,
                    Axiom::SymmetricObjectProperty(sub_property),
                    vec![],
                )?;
            }
            if ctx
                .ontology
                .index()
                .reflexive_properties()
                .contains(&super_property)
            {
                infer_axiom(
                    ctx,
                    RlRule::CharPropagate,
                    Axiom::ReflexiveObjectProperty(sub_property),
                    vec![],
                )?;
            }
            if ctx
                .ontology
                .index()
                .functional_properties()
                .contains(&super_property)
            {
                infer_axiom(
                    ctx,
                    RlRule::CharPropagate,
                    Axiom::FunctionalObjectProperty(sub_property),
                    vec![],
                )?;
            }
            if ctx
                .ontology
                .index()
                .asymmetric_properties()
                .contains(&super_property)
            {
                infer_axiom(
                    ctx,
                    RlRule::CharPropagate,
                    Axiom::AsymmetricObjectProperty(sub_property),
                    vec![],
                )?;
            }
        }
    }
    Ok(())
}

fn apply_existential_subproperty(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let classes: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    for subclass in classes.clone() {
        let existentials = ctx.ontology.existentials_of(subclass).to_vec();
        for (property, filler) in existentials {
            for super_property in transitive_subproperties(ctx.ontology, property) {
                if super_property == property {
                    continue;
                }
                infer_axiom(
                    ctx,
                    RlRule::ExistentialSubProp,
                    Axiom::SubClassOfExistential {
                        subclass,
                        property: super_property,
                        filler,
                    },
                    vec![],
                )?;
            }
            for equiv_property in expand_equivalent_properties(ctx.ontology, property) {
                if equiv_property == property {
                    continue;
                }
                infer_axiom(
                    ctx,
                    RlRule::ExistentialSubProp,
                    Axiom::SubClassOfExistential {
                        subclass,
                        property: equiv_property,
                        filler,
                    },
                    vec![],
                )?;
            }
        }
    }

    for class in &classes {
        for equiv_class in expand_equivalent_classes(ctx.ontology, *class) {
            if equiv_class == *class {
                continue;
            }
            let existentials = ctx.ontology.existentials_of(*class).to_vec();
            for (property, filler) in existentials {
                infer_axiom(
                    ctx,
                    RlRule::ExistentialSubProp,
                    Axiom::SubClassOfExistential {
                        subclass: equiv_class,
                        property,
                        filler,
                    },
                    vec![],
                )?;
            }
        }
    }

    // scm-spo1: propagate existentials along subClassOf to subclasses.
    for subclass in classes.clone() {
        for super_class in transitive_superclasses(ctx.ontology, subclass) {
            let existentials = ctx.ontology.existentials_of(super_class).to_vec();
            for (property, filler) in existentials {
                infer_axiom(
                    ctx,
                    RlRule::ExistentialSubProp,
                    Axiom::SubClassOfExistential {
                        subclass,
                        property,
                        filler,
                    },
                    vec![],
                )?;
            }
        }
    }

    Ok(())
}

fn apply_existential_subsumption(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let mut existentials: Vec<(EntityId, EntityId, EntityId)> = Vec::new();
    for (id, axiom) in ctx.ontology.axioms().iter() {
        if id.0 as usize >= ctx.asserted_axiom_count {
            continue;
        }
        if let Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } = axiom
        {
            existentials.push((*subclass, *property, *filler));
        }
    }

    for (sub_x, prop_x, filler_x) in &existentials {
        let supers = transitive_superproperties(ctx.ontology, *prop_x);
        for (sub_y, prop_y, filler_y) in &existentials {
            if sub_x == sub_y {
                continue;
            }
            let fillers_compatible =
                filler_x == filler_y || is_subclass_of(ctx.ontology, *filler_x, *filler_y);
            if !fillers_compatible {
                continue;
            }
            let props_compatible = if prop_x == prop_y {
                // Same property: only filler subsumption (not reflexive same-filler pairs).
                filler_x != filler_y
            } else {
                supers.contains(prop_y)
                    || expand_equivalent_properties(ctx.ontology, *prop_x).contains(prop_y)
            };
            if props_compatible {
                infer_axiom(
                    ctx,
                    RlRule::ExistentialSubsumption,
                    Axiom::SubClassOf {
                        subclass: *sub_x,
                        superclass: *sub_y,
                    },
                    vec![],
                )?;
            }
        }
    }
    Ok(())
}
