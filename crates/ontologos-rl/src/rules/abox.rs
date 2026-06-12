use ontologos_core::{Axiom, EntityId, EntityKind, Ontology};

use super::helpers::{
    expand_equivalent_classes, expand_same_as, infer_axiom, map_parallel, transitive_subproperties,
    transitive_superclasses, RuleContext,
};
use crate::report::RlRule;

fn domains_for_property(ontology: &Ontology, property: EntityId) -> Vec<EntityId> {
    let index = ontology.index();
    let mut domains: Vec<EntityId> = index.domains_of(property).to_vec();
    for sub in transitive_subproperties(ontology, property) {
        domains.extend(index.domains_of(sub).iter().copied());
    }
    domains.sort_unstable_by_key(|id| id.0);
    domains.dedup();
    domains
}

fn ranges_for_property(ontology: &Ontology, property: EntityId) -> Vec<EntityId> {
    let index = ontology.index();
    let mut ranges: Vec<EntityId> = index.ranges_of(property).to_vec();
    for sub in transitive_subproperties(ontology, property) {
        ranges.extend(index.ranges_of(sub).iter().copied());
    }
    ranges.sort_unstable_by_key(|id| id.0);
    ranges.dedup();
    ranges
}

pub(crate) fn apply_batch_b(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    apply_type_rules(ctx)?;
    apply_property_rules(ctx)?;
    apply_same_as_rules(ctx)?;
    detect_disjoint_clashes(ctx)?;
    Ok(())
}

fn apply_type_rules(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let property_assertions: Vec<(EntityId, EntityId, EntityId)> = ctx
        .ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => Some((*subject, *property, *object)),
            _ => None,
        })
        .collect();

    let domain_candidates: Vec<(EntityId, EntityId)> = {
        let ontology: &Ontology = ctx.ontology;
        map_parallel(
            ctx.parallelism,
            property_assertions.clone(),
            |(subject, property, _)| {
                domains_for_property(ontology, property)
                    .into_iter()
                    .map(|domain| (subject, domain))
                    .collect::<Vec<_>>()
            },
        )
        .into_iter()
        .flatten()
        .collect()
    };
    let range_candidates: Vec<(EntityId, EntityId)> = {
        let ontology: &Ontology = ctx.ontology;
        map_parallel(
            ctx.parallelism,
            property_assertions,
            |(_, property, object)| {
                ranges_for_property(ontology, property)
                    .into_iter()
                    .map(|range| (object, range))
                    .collect::<Vec<_>>()
            },
        )
        .into_iter()
        .flatten()
        .collect()
    };

    for (individual, class) in domain_candidates {
        infer_axiom(
            ctx,
            RlRule::TypeDomain,
            Axiom::ClassAssertion { individual, class },
            vec![],
        )?;
    }
    for (individual, class) in range_candidates {
        infer_axiom(
            ctx,
            RlRule::TypeRange,
            Axiom::ClassAssertion { individual, class },
            vec![],
        )?;
    }

    let class_assertions: Vec<(EntityId, EntityId)> = ctx
        .ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::ClassAssertion { individual, class } => Some((*individual, *class)),
            _ => None,
        })
        .collect();

    for (individual, class) in class_assertions {
        for super_class in transitive_superclasses(ctx.ontology, class) {
            infer_axiom(
                ctx,
                RlRule::TypeSubclass,
                Axiom::ClassAssertion {
                    individual,
                    class: super_class,
                },
                vec![],
            )?;
        }
        for equiv_class in expand_equivalent_classes(ctx.ontology, class) {
            if equiv_class == class {
                continue;
            }
            infer_axiom(
                ctx,
                RlRule::TypeEquivalent,
                Axiom::ClassAssertion {
                    individual,
                    class: equiv_class,
                },
                vec![],
            )?;
        }
    }
    Ok(())
}

fn apply_property_rules(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let assertions: Vec<(EntityId, EntityId, EntityId)> =
        ctx.index.property_assertions().iter().copied().collect();

    for (subject, property, object) in assertions {
        for sub_property in transitive_superproperties(ctx.ontology, property) {
            if sub_property != property {
                infer_axiom(
                    ctx,
                    RlRule::PropSub,
                    Axiom::ObjectPropertyAssertion {
                        subject,
                        property: sub_property,
                        object,
                    },
                    vec![],
                )?;
            }
        }

        if let Some(inverse) = ctx.ontology.inverse_of(property) {
            infer_axiom(
                ctx,
                RlRule::PropInverse,
                Axiom::ObjectPropertyAssertion {
                    subject: object,
                    property: inverse,
                    object: subject,
                },
                vec![],
            )?;
        }

        if ctx.index.symmetric_properties().contains(&property) {
            infer_axiom(
                ctx,
                RlRule::PropSymmetric,
                Axiom::ObjectPropertyAssertion {
                    subject: object,
                    property,
                    object: subject,
                },
                vec![],
            )?;
        }

        if ctx.index.reflexive_properties().contains(&property) {
            infer_axiom(
                ctx,
                RlRule::PropReflexive,
                Axiom::ObjectPropertyAssertion {
                    subject,
                    property,
                    object: subject,
                },
                vec![],
            )?;
        }

        if ctx.index.transitive_properties().contains(&property) {
            let chain: Vec<(EntityId, EntityId, EntityId)> =
                ctx.index.property_assertions().iter().copied().collect();
            for (chain_subject, chain_property, chain_object) in chain {
                if chain_property != property {
                    continue;
                }
                if chain_subject == object {
                    infer_axiom(
                        ctx,
                        RlRule::PropTransitive,
                        Axiom::ObjectPropertyAssertion {
                            subject,
                            property,
                            object: chain_object,
                        },
                        vec![],
                    )?;
                }
                if chain_object == subject {
                    infer_axiom(
                        ctx,
                        RlRule::PropTransitive,
                        Axiom::ObjectPropertyAssertion {
                            subject: chain_subject,
                            property,
                            object,
                        },
                        vec![],
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn transitive_superproperties(
    ontology: &ontologos_core::Ontology,
    property: EntityId,
) -> Vec<EntityId> {
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<EntityId> = ontology.direct_superproperties(property).to_vec();
    let mut out = vec![property];
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        out.push(current);
        stack.extend_from_slice(ontology.direct_superproperties(current));
    }
    out
}

fn apply_same_as_rules(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let individuals: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == EntityKind::Individual)
        .map(|(id, _)| id)
        .collect();

    for individual in individuals {
        let same_cluster = expand_same_as(ctx.ontology, individual);
        let classes: Vec<EntityId> = ctx.ontology.classes_of(individual).to_vec();
        let outgoing: Vec<(EntityId, EntityId)> =
            ctx.ontology.object_assertions_of(individual).to_vec();
        let incoming: Vec<(EntityId, EntityId)> =
            ctx.ontology.object_assertions_to(individual).to_vec();

        for same in same_cluster {
            if same == individual {
                continue;
            }
            for class in &classes {
                infer_axiom(
                    ctx,
                    RlRule::SameAsClass,
                    Axiom::ClassAssertion {
                        individual: same,
                        class: *class,
                    },
                    vec![],
                )?;
            }
            for &(property, object) in &outgoing {
                infer_axiom(
                    ctx,
                    RlRule::SameAsProp,
                    Axiom::ObjectPropertyAssertion {
                        subject: same,
                        property,
                        object,
                    },
                    vec![],
                )?;
            }
            for &(property, subject) in &incoming {
                infer_axiom(
                    ctx,
                    RlRule::SameAsProp,
                    Axiom::ObjectPropertyAssertion {
                        subject,
                        property,
                        object: same,
                    },
                    vec![],
                )?;
            }
        }
    }
    Ok(())
}

fn detect_disjoint_clashes(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let individuals: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == EntityKind::Individual)
        .map(|(id, _)| id)
        .collect();

    for individual in individuals {
        let classes: Vec<EntityId> = ctx.ontology.classes_of(individual).to_vec();
        for i in 0..classes.len() {
            for j in (i + 1)..classes.len() {
                if ctx
                    .ontology
                    .disjoint_with(classes[i])
                    .is_some_and(|set| set.contains(&classes[j]))
                {
                    ctx.report.clashes.push(format!(
                        "individual {:?} has types in disjoint classes {:?} and {:?}",
                        individual, classes[i], classes[j]
                    ));
                }
            }
        }

        let cluster = expand_same_as(ctx.ontology, individual);
        for i in 0..cluster.len() {
            for j in (i + 1)..cluster.len() {
                let a = cluster[i];
                let b = cluster[j];
                if ctx
                    .ontology
                    .different_from(a)
                    .is_some_and(|set| set.contains(&b))
                {
                    ctx.report.clashes.push(format!(
                        "individuals {:?} and {:?} are sameAs but also differentFrom",
                        a, b
                    ));
                }
            }
        }
    }
    Ok(())
}
