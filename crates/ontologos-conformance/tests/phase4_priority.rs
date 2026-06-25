//! Phase 4 ROADMAP priority OWL WG cases — regression gate per thematic bucket.
use ontologos_conformance::{check_wg_case, read_wg_catalog_file};

fn case_by_suffix(suffix: &str) -> ontologos_conformance::WgCase {
    read_wg_catalog_file()
        .into_iter()
        .find(|c| c.id.contains(suffix))
        .unwrap_or_else(|| panic!("missing WG case with suffix {suffix}"))
}

fn assert_wg(suffix: &str) {
    check_wg_case(&case_by_suffix(suffix)).unwrap_or_else(|e| panic!("{suffix}: {e}"));
}

// Imports (harness + engine)
#[test]
fn wg_imports_007() {
    assert_wg("imports-2D007");
}

#[test]
fn wg_imports_008() {
    assert_wg("imports-2D008");
}

// Consistency — datatype / bottom / NPA
#[test]
fn wg_minus_inf_not_owlreal() {
    assert_wg("Minus-2Dinf-2Dnot-2Dowlreal");
}

#[test]
fn wg_bottom_object_property() {
    assert_wg("New-2DFeature-2DBottomObjectProperty-2D001");
}

#[test]
fn wg_rdfbased_npa_ind_fw() {
    assert_wg("Rdfbased-2Dsem-2Dnpa-2Dind-2Dfw");
}

#[test]
fn wg_rdfbased_maxcard_zero() {
    assert_wg("Rdfbased-2Dsem-2Drestrict-2Dmaxcard-2Dinst-2Dobj-2Dzero");
}

#[test]
fn wg_description_logic_040_inconsistent() {
    assert_wg("description-2Dlogic-2D040");
}

// QCR
#[test]
fn wg_object_qcr_001() {
    assert_wg("New-2DFeature-2DObjectQCR-2D001");
}

// Keys inconsistency
#[test]
fn wg_keys_002_inconsistent() {
    assert_wg("New-2DFeature-2DKeys-2D002");
}

// Entailment — description logic
#[test]
fn wg_description_logic_201() {
    assert_wg("description-2Dlogic-2D201");
}

#[test]
fn wg_description_logic_205() {
    assert_wg("description-2Dlogic-2D205");
}

// Rdfbased conditional entailment
#[test]
fn wg_rdfbased_rdfs_subclass_cond() {
    assert_wg("Rdfbased-2Dsem-2Drdfs-2Dsubclass-2Dcond");
}

#[test]
fn wg_rdfbased_restrict_allvalues_inst_obj() {
    assert_wg("Rdfbased-2Dsem-2Drestrict-2Dallvalues-2Dinst-2Dobj");
}

// Negative entailment (existing wg_phase4_check coverage extended)
#[test]
fn wg_i46_negative_entailment() {
    assert_wg("I4.6-2D004");
}

#[test]
fn wg_keys_004_negative_entailment() {
    assert_wg("New-2DFeature-2DKeys-2D004");
}
