use ontologos_core::{Axiom, EntityId, Ontology};

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
                infer_subclass(ctx, RdfsRule::ScTrans, subclass, super_super)?;
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
                infer_subproperty(ctx, RdfsRule::SpTrans, sub_property, super_super)?;
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
                infer_domain(ctx, RdfsRule::DomInherit, sub_property, domain)?;
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
                infer_range(ctx, RdfsRule::RngInherit, sub_property, range)?;
            }
        }
    }
    Ok(())
}

fn infer_subclass(
    ctx: &mut RuleContext<'_>,
    rule: RdfsRule,
    subclass: EntityId,
    superclass: EntityId,
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
                premises: Vec::new(),
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
                premises: Vec::new(),
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
                premises: Vec::new(),
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
                premises: Vec::new(),
                conclusion,
            });
        }
    }
    Ok(())
}
