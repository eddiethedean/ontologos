use ontologos_core::{Axiom, AxiomId, EntityId, Ontology};

use crate::report::{InferenceRecord, MaterializationReport, RdfsRule};

pub(crate) struct RuleContext<'a> {
    pub ontology: &'a mut Ontology,
    pub report: &'a mut MaterializationReport,
    pub record_traces: bool,
}

pub(crate) fn apply_sc_trans(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let classes: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == ontologos_core::EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    for subclass in classes {
        let supers: Vec<EntityId> = ctx.ontology.direct_superclasses(subclass).to_vec();
        for direct_super in supers {
            let indirect: Vec<EntityId> = ctx.ontology.direct_superclasses(direct_super).to_vec();
            for super_super in indirect {
                let premises = [
                    find_subclass_axiom(ctx.ontology, subclass, direct_super),
                    find_subclass_axiom(ctx.ontology, direct_super, super_super),
                ]
                .into_iter()
                .flatten()
                .collect();
                infer_subclass(ctx, RdfsRule::ScTrans, subclass, super_super, premises)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_sp_trans(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let properties: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == ontologos_core::EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    for sub_property in properties {
        let supers: Vec<EntityId> = ctx.ontology.direct_superproperties(sub_property).to_vec();
        for direct_super in supers {
            let indirect: Vec<EntityId> =
                ctx.ontology.direct_superproperties(direct_super).to_vec();
            for super_super in indirect {
                let premises = [
                    find_subproperty_axiom(ctx.ontology, sub_property, direct_super),
                    find_subproperty_axiom(ctx.ontology, direct_super, super_super),
                ]
                .into_iter()
                .flatten()
                .collect();
                infer_subproperty(ctx, RdfsRule::SpTrans, sub_property, super_super, premises)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_dom_inherit(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let properties: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == ontologos_core::EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    for sub_property in properties {
        let supers: Vec<EntityId> = ctx.ontology.direct_superproperties(sub_property).to_vec();
        for super_property in supers {
            let domains: Vec<EntityId> = ctx.ontology.index().domains_of(super_property).to_vec();
            for domain in domains {
                let premises = [
                    find_subproperty_axiom(ctx.ontology, sub_property, super_property),
                    find_domain_axiom(ctx.ontology, super_property, domain),
                ]
                .into_iter()
                .flatten()
                .collect();
                infer_domain(ctx, RdfsRule::DomInherit, sub_property, domain, premises)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn apply_rng_inherit(ctx: &mut RuleContext<'_>) -> ontologos_core::Result<()> {
    let properties: Vec<EntityId> = ctx
        .ontology
        .entities()
        .iter()
        .filter(|(_, record)| record.kind == ontologos_core::EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();

    for sub_property in properties {
        let supers: Vec<EntityId> = ctx.ontology.direct_superproperties(sub_property).to_vec();
        for super_property in supers {
            let ranges: Vec<EntityId> = ctx.ontology.index().ranges_of(super_property).to_vec();
            for range in ranges {
                let premises = [
                    find_subproperty_axiom(ctx.ontology, sub_property, super_property),
                    find_range_axiom(ctx.ontology, super_property, range),
                ]
                .into_iter()
                .flatten()
                .collect();
                infer_range(ctx, RdfsRule::RngInherit, sub_property, range, premises)?;
            }
        }
    }
    Ok(())
}

fn find_subclass_axiom(
    ontology: &Ontology,
    subclass: EntityId,
    superclass: EntityId,
) -> Option<AxiomId> {
    ontology.axioms().iter().find_map(|(id, axiom)| {
        matches!(
            axiom,
            Axiom::SubClassOf {
                subclass: s,
                superclass: sup,
            } if *s == subclass && *sup == superclass
        )
        .then_some(id)
    })
}

fn find_subproperty_axiom(
    ontology: &Ontology,
    sub_property: EntityId,
    super_property: EntityId,
) -> Option<AxiomId> {
    ontology.axioms().iter().find_map(|(id, axiom)| {
        matches!(
            axiom,
            Axiom::SubObjectPropertyOf {
                sub_property: sub,
                super_property: sup,
            } if *sub == sub_property && *sup == super_property
        )
        .then_some(id)
    })
}

fn find_domain_axiom(ontology: &Ontology, property: EntityId, domain: EntityId) -> Option<AxiomId> {
    ontology.axioms().iter().find_map(|(id, axiom)| {
        matches!(
            axiom,
            Axiom::ObjectPropertyDomain {
                property: p,
                domain: d,
            } if *p == property && *d == domain
        )
        .then_some(id)
    })
}

fn find_range_axiom(ontology: &Ontology, property: EntityId, range: EntityId) -> Option<AxiomId> {
    ontology.axioms().iter().find_map(|(id, axiom)| {
        matches!(
            axiom,
            Axiom::ObjectPropertyRange {
                property: p,
                range: r,
            } if *p == property && *r == range
        )
        .then_some(id)
    })
}

fn infer_subclass(
    ctx: &mut RuleContext<'_>,
    rule: RdfsRule,
    subclass: EntityId,
    superclass: EntityId,
    premises: Vec<AxiomId>,
) -> ontologos_core::Result<()> {
    if subclass == superclass {
        return Ok(());
    }
    let before = ctx.ontology.axiom_count();
    let conclusion = ctx.ontology.add_axiom(Axiom::SubClassOf {
        subclass,
        superclass,
    })?;
    if ctx.ontology.axiom_count() > before {
        *ctx.report.inferred_by_rule.entry(rule).or_default() += 1;
        if ctx.record_traces {
            ctx.report.traces.push(InferenceRecord {
                rule,
                premises,
                conclusion,
            });
        }
    }
    Ok(())
}

fn infer_subproperty(
    ctx: &mut RuleContext<'_>,
    rule: RdfsRule,
    sub_property: EntityId,
    super_property: EntityId,
    premises: Vec<AxiomId>,
) -> ontologos_core::Result<()> {
    if sub_property == super_property {
        return Ok(());
    }
    let before = ctx.ontology.axiom_count();
    let conclusion = ctx.ontology.add_axiom(Axiom::SubObjectPropertyOf {
        sub_property,
        super_property,
    })?;
    if ctx.ontology.axiom_count() > before {
        *ctx.report.inferred_by_rule.entry(rule).or_default() += 1;
        if ctx.record_traces {
            ctx.report.traces.push(InferenceRecord {
                rule,
                premises,
                conclusion,
            });
        }
    }
    Ok(())
}

fn infer_domain(
    ctx: &mut RuleContext<'_>,
    rule: RdfsRule,
    property: EntityId,
    domain: EntityId,
    premises: Vec<AxiomId>,
) -> ontologos_core::Result<()> {
    let before = ctx.ontology.axiom_count();
    let conclusion = ctx
        .ontology
        .add_axiom(Axiom::ObjectPropertyDomain { property, domain })?;
    if ctx.ontology.axiom_count() > before {
        *ctx.report.inferred_by_rule.entry(rule).or_default() += 1;
        if ctx.record_traces {
            ctx.report.traces.push(InferenceRecord {
                rule,
                premises,
                conclusion,
            });
        }
    }
    Ok(())
}

fn infer_range(
    ctx: &mut RuleContext<'_>,
    rule: RdfsRule,
    property: EntityId,
    range: EntityId,
    premises: Vec<AxiomId>,
) -> ontologos_core::Result<()> {
    let before = ctx.ontology.axiom_count();
    let conclusion = ctx
        .ontology
        .add_axiom(Axiom::ObjectPropertyRange { property, range })?;
    if ctx.ontology.axiom_count() > before {
        *ctx.report.inferred_by_rule.entry(rule).or_default() += 1;
        if ctx.record_traces {
            ctx.report.traces.push(InferenceRecord {
                rule,
                premises,
                conclusion,
            });
        }
    }
    Ok(())
}
