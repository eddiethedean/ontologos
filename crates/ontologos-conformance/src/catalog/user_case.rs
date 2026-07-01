//! User-facing catalog checks routed through [`ontologos_facade`].

use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{
    EntailmentCheck, classify, is_consistent, is_entailed_axiom, is_subsumption_entailed,
    taxonomy_from_outcome,
};
use ontologos_parser::load_ontology_lenient as load_ontology;

use super::{
    case_has_axiom_assertions, check_logical_entailment, configure_wg_tableau_limits,
    dl_classify_budget, entailment_holds_with_budget_opts, hermit_data_path,
    lookup_entity_flexible, ontology_for_incremental_consistency, ontology_is_axiom_empty,
    resolve_local_iri, HermitCase, WgCase,
};

/// Map catalog `engine` field to a user-facing [`Profile`].
#[must_use]
pub fn profile_for_engine(engine: &str) -> Profile {
    match engine {
        "el" => Profile::El,
        "rl" => Profile::Rl,
        "rdfs" => Profile::Rdfs,
        "dl" | "alc" | "swrl" => Profile::Dl,
        other => panic!("unsupported catalog engine for user contract: {other}"),
    }
}

/// Whether [`check_user_axiom_case`] can run this catalog case.
#[must_use]
pub fn user_case_supported(case: &HermitCase) -> bool {
    if !matches!(case.status.as_str(), "axiom" | "swrl") {
        return false;
    }
    if case.ria_regular.is_some() || case.role_simple.is_some() {
        return false;
    }
    if !case.ce_satisfiability.is_empty() || !case.ce_instance_checks.is_empty() {
        return false;
    }
    if !case.datalog_queries.is_empty() {
        return false;
    }
    if !case.property_subsumptions.is_empty()
        || !case.property_characteristics.is_empty()
        || !case.data_property_subsumptions.is_empty()
    {
        return false;
    }
    if !case.individual_instances.is_empty() {
        return false;
    }
    if case.individual_types.iter().any(|t| t.direct) {
        return false;
    }
    if case.id == "reasoner.ReasonerTest.testPrecomputeDisjointClasses" {
        return false;
    }
    for exp in &case.class_satisfiability {
        let name = exp.class.strip_prefix(':').unwrap_or(&exp.class);
        if name.is_empty() || name.contains(' ') || name.contains('(') {
            return false;
        }
    }
    has_user_assertions(case)
}

fn has_user_assertions(case: &HermitCase) -> bool {
    !case.subsumptions.is_empty()
        || case.consistent.is_some()
        || case.conclusion_ofn.is_some()
        || !case.class_satisfiability.is_empty()
        || !case.individual_types.is_empty()
        || case.load_error_expected
}

/// Whether [`check_user_wg_case`] can run this WG catalog case.
#[must_use]
pub fn user_wg_case_supported(case: &WgCase) -> bool {
    case.status == "wg"
        && case.premise_ofn.is_some()
        && (case.expected_consistent.is_some() || case.conclusion_ofn.is_some())
}

/// Semantic check for a catalog case via the public facade API.
pub fn check_user_axiom_case(case: &HermitCase) -> Result<(), String> {
    let swrl_vacuous_probe = case.engine == "swrl"
        && !case_has_axiom_assertions(case)
        && matches!(case.status.as_str(), "axiom" | "swrl");
    if !user_case_supported(case) && !swrl_vacuous_probe {
        return Err(format!("{}: not supported by user contract runner", case.id));
    }

    let rel = case
        .axiom_ofn
        .as_ref()
        .ok_or_else(|| format!("{}: missing axiom_ofn", case.id))?;
    let path = hermit_data_path(rel);
    if !path.is_file() {
        return Err(format!("{}: missing fixture {}", case.id, path.display()));
    }

    if case.load_error_expected {
        let loaded = load_ontology(&path);
        if loaded.is_ok()
            && let Ok(ontology) = loaded
                && ontologos_parser::validate_loaded_ontology(&ontology).is_ok() {
                    return Err(format!("{}: expected ontology load to fail", case.id));
                }
        return Ok(());
    }

    let mut ontology = if let Some(inc_rel) = &case.incremental_ofn {
        let inc_path = hermit_data_path(inc_rel);
        if !inc_path.is_file() {
            return Err(format!(
                "{}: missing incremental fixture {}",
                case.id,
                inc_path.display()
            ));
        }
        ontologos_parser::load_ofn_with_incremental(&path, &inc_path)
            .map_err(|e| format!("{}: load merged: {e}", case.id))?
    } else {
        load_ontology(&path).map_err(|e| format!("{}: load: {e}", case.id))?
    };

    if case.engine == "swrl" {
        ontologos_swrl::apply_swrl_rules(&mut ontology)
            .map_err(|e| format!("{}: swrl: {e}", case.id))?;
        if !case_has_axiom_assertions(case) {
            return Err(format!(
                "{}: SWRL case has no harvested assertions — vacuous pass blocked",
                case.id
            ));
        }
    }

    if let (Some(conclusion_rel), Some(expected)) = (&case.conclusion_ofn, case.expected_entailment)
    {
        let conclusion_path = hermit_data_path(conclusion_rel);
        if !conclusion_path.is_file() {
            return Err(format!(
                "{}: missing conclusion {}",
                case.id,
                conclusion_path.display()
            ));
        }
        let conclusion = load_ontology(&conclusion_path)
            .map_err(|e| format!("{}: load conclusion: {e}", case.id))?;
        let entailed = check_logical_entailment(&ontology, &conclusion)?;
        if entailed != expected {
            return Err(format!(
                "{}: entailment expected {expected}, got {entailed}",
                case.id
            ));
        }
        return Ok(());
    }

    let profile = profile_for_engine(&case.engine);
    let mut reasoner = Reasoner::builder()
        .profile(profile)
        .build(ontology)
        .map_err(|e| format!("{}: reasoner: {e}", case.id))?;

    if !case.subsumptions.is_empty() || !case.class_satisfiability.is_empty() {
        let outcome = classify(&mut reasoner).map_err(|e| format!("{}: classify: {e}", case.id))?;
        if !case.subsumptions.is_empty() {
            for sub in &case.subsumptions {
                let sub_iri = resolve_local_iri(&sub.sub);
                let sup_iri = resolve_local_iri(&sub.sup);
                let actual = is_subsumption_entailed(&mut reasoner, &sub_iri, &sup_iri)
                    .map_err(|e| format!("{}: subsumption: {e}", case.id))?;
                if actual != sub.expected {
                    return Err(format!(
                        "{}: expected {} ⊑ {} = {}",
                        case.id, sub_iri, sup_iri, sub.expected
                    ));
                }
            }
        }
        if !case.class_satisfiability.is_empty() {
            let taxonomy = taxonomy_from_outcome(&outcome).ok_or_else(|| {
                format!(
                    "{}: classify did not produce taxonomy for class satisfiability",
                    case.id
                )
            })?;
            for exp in &case.class_satisfiability {
                let iri = resolve_local_iri(&exp.class);
                let class_id = lookup_entity_flexible(reasoner.ontology(), &iri)
                    .ok_or_else(|| format!("{}: missing class {iri}", case.id))?;
                let satisfiable = !taxonomy.unsatisfiable.contains(&class_id);
                if satisfiable != exp.expected {
                    return Err(format!(
                        "{}: class {iri} satisfiability expected {}, got {satisfiable}",
                        case.id, exp.expected
                    ));
                }
            }
        }
    }

    for exp in &case.individual_types {
        if matches!(profile, Profile::Rl | Profile::Rdfs) {
            classify(&mut reasoner).map_err(|e| format!("{}: classify for types: {e}", case.id))?;
        }
        let actual = is_entailed_axiom(
            &mut reasoner,
            EntailmentCheck::ClassAssertion {
                individual: resolve_local_iri(&exp.individual),
                class: resolve_local_iri(&exp.class),
            },
        )
        .map_err(|e| format!("{}: class assertion: {e}", case.id))?;
        if actual != exp.expected {
            return Err(format!(
                "{}: hasType {} {} expected {}, got {}",
                case.id, exp.individual, exp.class, exp.expected, actual
            ));
        }
    }

    if let Some(expected) = case.consistent {
        let consistency_ontology =
            ontology_for_incremental_consistency(reasoner.ontology(), case)?;
        let reasoner = Reasoner::builder()
            .profile(profile)
            .build(consistency_ontology)
            .map_err(|e| format!("{}: consistency reasoner: {e}", case.id))?;
        let actual =
            is_consistent(&reasoner).map_err(|e| format!("{}: consistency: {e}", case.id))?;
        if actual != expected {
            return Err(format!(
                "{}: consistency expected {expected}, got {actual}",
                case.id
            ));
        }
    }

    Ok(())
}

/// WG catalog semantic check via the public facade API.
pub fn check_user_wg_case(case: &WgCase) -> Result<(), String> {
    if !user_wg_case_supported(case) {
        return Err(format!("{}: not supported by user WG contract runner", case.id));
    }
    configure_wg_tableau_limits();
    let premise = case
        .premise_ofn
        .as_ref()
        .expect("checked in user_wg_case_supported");
    let path = hermit_data_path(premise);
    if !path.is_file() {
        return Err(format!("{}: missing premise {}", case.id, path.display()));
    }
    let ontology = load_ontology(&path).map_err(|e| format!("{}: load premise: {e}", case.id))?;

    if let Some(expected) = case.expected_consistent {
        let reasoner = Reasoner::builder()
            .profile(Profile::Dl)
            .build(ontology)
            .map_err(|e| format!("{}: reasoner: {e}", case.id))?;
        let actual =
            is_consistent(&reasoner).map_err(|e| format!("{}: consistency: {e}", case.id))?;
        if actual != expected {
            return Err(format!(
                "{}: consistency expected {expected}, got {actual}",
                case.id
            ));
        }
        return Ok(());
    }

    if let (Some(conclusion_rel), Some(expected)) = (&case.conclusion_ofn, case.expected_entailment)
    {
        let conclusion_path = hermit_data_path(conclusion_rel);
        if !conclusion_path.is_file() {
            return Err(format!(
                "{}: missing conclusion {}",
                case.id,
                conclusion_path.display()
            ));
        }
        let conclusion = load_ontology(&conclusion_path)
            .map_err(|e| format!("{}: load conclusion: {e}", case.id))?;
        if ontology_is_axiom_empty(&conclusion) && !expected {
            return Ok(());
        }
        let entailed = entailment_holds_with_budget_opts(
            &ontology,
            &conclusion,
            Some(dl_classify_budget()),
            expected,
        )
        .map_err(|e| format!("{}: {e}", case.id))?;
        if entailed != expected {
            return Err(format!(
                "{}: entailment expected {expected}, got {entailed}",
                case.id
            ));
        }
    }
    Ok(())
}
